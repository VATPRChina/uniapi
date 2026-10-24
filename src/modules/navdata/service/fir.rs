use std::collections::{BTreeMap, BTreeSet};

use sqlx::{FromRow, SqlitePool};

use super::{InvalidNavdataError, NavdataResult, NavdataService};
use crate::modules::navdata::models::ChinaMainlandBoundary;

const MAINLAND_FIRS: [&str; 9] = [
    "ZBPE", "ZGZU", "ZHWH", "ZJSA", "ZLHW", "ZPKM", "ZSHA", "ZWUQ", "ZYSH",
];

impl NavdataService {
    /// Read ARINC 424 UF boundaries from the configured DFD v2 database.
    /// F = FIR, B = combined FIR/UIR. C (delegated airspace) and UIR-only
    /// records are deliberately excluded from horizontal FIR coverage.
    /// Missing tables or incomplete mainland coverage disable geographic
    /// classification, while malformed data and database failures remain errors.
    pub async fn mainland_fir_boundaries(db: &SqlitePool) -> NavdataResult<ChinaMainlandBoundary> {
        let has_table: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master
             WHERE name = 'tbl_uf_fir_uir' COLLATE NOCASE AND type IN ('table', 'view'))",
        )
        .fetch_one(db)
        .await?;
        if !has_table {
            tracing::warn!(
                "FIR boundary table is missing; continuing without geographic China classification"
            );
            return Ok(ChinaMainlandBoundary::default());
        }
        let records = sqlx::query_as::<_, FirRecord>(
            "SELECT area_code, fir_uir_identifier, fir_uir_indicator, seqno,
                    boundary_via, fir_uir_latitude AS latitude, fir_uir_longitude AS longitude
             FROM tbl_uf_fir_uir
             WHERE fir_uir_identifier IN ('ZBPE', 'ZGZU', 'ZHWH', 'ZJSA', 'ZLHW',
                                         'ZPKM', 'ZSHA', 'ZWUQ', 'ZYSH')
               AND fir_uir_indicator IN ('F', 'B')
             ORDER BY area_code, fir_uir_identifier, fir_uir_indicator, seqno",
        )
        .fetch_all(db)
        .await?;
        build_boundaries(records)
    }
}

#[derive(Debug, FromRow)]
struct FirRecord {
    area_code: String,
    fir_uir_identifier: String,
    fir_uir_indicator: String,
    seqno: i64,
    boundary_via: String,
    latitude: f64,
    longitude: f64,
}

fn invalid(message: impl Into<String>) -> InvalidNavdataError {
    InvalidNavdataError::InvalidFirBoundary(message.into())
}

fn build_boundaries(records: Vec<FirRecord>) -> NavdataResult<ChinaMainlandBoundary> {
    let found: BTreeSet<_> = records
        .iter()
        .map(|r| r.fir_uir_identifier.as_str())
        .collect();
    let missing: Vec<_> = MAINLAND_FIRS
        .into_iter()
        .filter(|id| !found.contains(id))
        .collect();
    if !missing.is_empty() {
        tracing::warn!(
            missing_firs = %missing.join(", "),
            "mainland FIR boundaries are incomplete; continuing without geographic China classification"
        );
        return Ok(ChinaMainlandBoundary::default());
    }
    let mut groups = BTreeMap::<_, Vec<FirRecord>>::new();
    for record in records {
        groups
            .entry((
                record.area_code.clone(),
                record.fir_uir_identifier.clone(),
                record.fir_uir_indicator.clone(),
            ))
            .or_default()
            .push(record);
    }
    let mut rings = Vec::new();
    for ((_, identifier, _), records) in groups {
        let mut pending = Vec::new();
        let mut previous = None;
        for record in records {
            if previous.is_some_and(|seqno| seqno >= record.seqno) {
                return Err(invalid(format!("{identifier}: non-increasing sequence")));
            }
            previous = Some(record.seqno);
            if !record.latitude.is_finite()
                || !record.longitude.is_finite()
                || record.latitude.abs() >= 90.
                || record.longitude.abs() > 180.
            {
                return Err(invalid(format!(
                    "{identifier}: invalid boundary coordinates"
                )));
            }
            let via = record.boundary_via.as_bytes();
            if via.len() != 2 || !matches!(via[0], b'G' | b'H') || !matches!(via[1], b' ' | b'E') {
                return Err(invalid(format!(
                    "{identifier}: unsupported boundary via {:?}",
                    record.boundary_via
                )));
            }
            let closed = via[1] == b'E';
            pending.push(record);
            if closed {
                if pending.len() < 3 {
                    return Err(invalid(format!(
                        "{identifier}: boundary has fewer than three vertices"
                    )));
                }
                let mut ring = Vec::new();
                for (from, to) in pending
                    .iter()
                    .zip(pending.iter().cycle().skip(1))
                    .take(pending.len())
                {
                    append_edge(&mut ring, from, to);
                }
                rings.push(ring);
                pending.clear();
            }
        }
        if !pending.is_empty() {
            return Err(invalid(format!(
                "{identifier}: missing end-of-boundary marker"
            )));
        }
    }
    ChinaMainlandBoundary::from_rings(rings).map_err(invalid)
}

// DFD v2 boundary_via describes the edge FROM this record TO the next.
// G = great circle, H = rhumb line, E = close to the first vertex.
// https://developers.navigraph.com/docs/navigation-data/dfd-data-format-v2
// Densify curved edges at <= 0.05 degree intervals before point-in-ring testing.
fn append_edge(ring: &mut Vec<(f64, f64)>, from: &FirRecord, to: &FirRecord) {
    let (lat1, lon1) = (from.latitude.to_radians(), from.longitude.to_radians());
    let (lat2, lon2) = (to.latitude.to_radians(), to.longitude.to_radians());
    let delta_lon = (lon2 - lon1 + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
        - std::f64::consts::PI;
    let angle = ((lat1.sin() * lat2.sin()) + lat1.cos() * lat2.cos() * delta_lon.cos())
        .clamp(-1., 1.)
        .acos();
    let steps = ((from.latitude - to.latitude)
        .abs()
        .max(delta_lon.to_degrees().abs())
        / 0.05)
        .ceil()
        .max(1.) as usize;
    for step in 0..steps {
        let fraction = step as f64 / steps as f64;
        if step == 0 || angle < 1e-12 {
            ring.push((from.longitude, from.latitude));
        } else if from.boundary_via.starts_with('G') {
            let a = ((1. - fraction) * angle).sin() / angle.sin();
            let b = (fraction * angle).sin() / angle.sin();
            let x = a * lat1.cos() * lon1.cos() + b * lat2.cos() * lon2.cos();
            let y = a * lat1.cos() * lon1.sin() + b * lat2.cos() * lon2.sin();
            let z = a * lat1.sin() + b * lat2.sin();
            ring.push((y.atan2(x).to_degrees(), z.atan2(x.hypot(y)).to_degrees()));
        } else {
            let mercator = |latitude: f64| (std::f64::consts::FRAC_PI_4 + latitude / 2.).tan().ln();
            let y = mercator(lat1) * (1. - fraction) + mercator(lat2) * fraction;
            ring.push((
                (lon1 + delta_lon * fraction).to_degrees(),
                (2. * y.exp().atan() - std::f64::consts::FRAC_PI_2).to_degrees(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::navdata::models::{Airport, AnyFix, GeoPoint};

    async fn database() -> SqlitePool {
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    async fn create_table(db: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE tbl_uf_fir_uir (
                area_code TEXT, fir_uir_identifier TEXT, fir_uir_indicator TEXT,
                seqno INTEGER, boundary_via TEXT, fir_uir_latitude REAL,
                fir_uir_longitude REAL)",
        )
        .execute(db)
        .await
        .unwrap();
    }

    fn records(identifiers: &[&str]) -> Vec<FirRecord> {
        identifiers
            .iter()
            .flat_map(|identifier| {
                [(100., 20., "H "), (120., 20., "H "), (110., 40., "HE")]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (longitude, latitude, via))| FirRecord {
                        area_code: "PAC".into(),
                        fir_uir_identifier: (*identifier).into(),
                        fir_uir_indicator: "F".into(),
                        seqno: index as i64,
                        boundary_via: via.into(),
                        latitude,
                        longitude,
                    })
            })
            .collect()
    }

    #[tokio::test]
    async fn missing_table_disables_geographic_coverage_but_keeps_icao_classification() {
        let boundary = NavdataService::mainland_fir_boundaries(&database().await)
            .await
            .unwrap();
        assert!(!boundary.contains((30., 110.)));
        assert!(!boundary.contains((0., 0.)));
        let airport = AnyFix::Airport(Airport {
            identifier: "ZBAA".try_into().unwrap(),
            latitude: 40.0.try_into().unwrap(),
            longitude: 116.0.try_into().unwrap(),
        });
        assert!(airport.is_china_or_unknown(&boundary));
        for identifier in ["ZMUB", "ZKPY", "RJAA"] {
            let foreign = AnyFix::Airport(Airport {
                identifier: identifier.try_into().unwrap(),
                latitude: 30.0.try_into().unwrap(),
                longitude: 110.0.try_into().unwrap(),
            });
            assert!(!foreign.is_china_or_unknown(&boundary));
        }
        assert!(!AnyFix::from(GeoPoint::new(30., 110.).unwrap()).is_china_or_unknown(&boundary));
        assert!(AnyFix::Unknown("UNKNOWN".into()).is_china_or_unknown(&boundary));
    }

    #[tokio::test]
    async fn empty_or_partial_mainland_tables_disable_geographic_coverage() {
        let db = database().await;
        create_table(&db).await;
        let empty = NavdataService::mainland_fir_boundaries(&db).await.unwrap();
        assert!(!empty.contains((30., 110.)));
        for record in records(&["ZBPE"]) {
            sqlx::query("INSERT INTO tbl_uf_fir_uir VALUES (?, ?, ?, ?, ?, ?, ?)")
                .bind(record.area_code)
                .bind(record.fir_uir_identifier)
                .bind(record.fir_uir_indicator)
                .bind(record.seqno)
                .bind(record.boundary_via)
                .bind(record.latitude)
                .bind(record.longitude)
                .execute(&db)
                .await
                .unwrap();
        }
        let partial = NavdataService::mainland_fir_boundaries(&db).await.unwrap();
        assert!(!partial.contains((30., 110.)));
    }

    #[test]
    fn complete_boundaries_still_classify_positions() {
        let boundary = build_boundaries(records(&MAINLAND_FIRS)).unwrap();
        assert!(boundary.contains((30., 110.)));
        assert!(!boundary.contains((30., 130.)));
    }

    #[test]
    fn malformed_complete_boundaries_remain_errors() {
        let mut records = records(&MAINLAND_FIRS);
        records[0].boundary_via = "C ".into();
        assert!(matches!(
            build_boundaries(records),
            Err(InvalidNavdataError::InvalidFirBoundary(_))
        ));
    }

    #[tokio::test]
    async fn schema_and_connection_errors_are_not_treated_as_missing_boundaries() {
        let db = database().await;
        sqlx::query("CREATE TABLE tbl_uf_fir_uir (wrong_column TEXT)")
            .execute(&db)
            .await
            .unwrap();
        assert!(matches!(
            NavdataService::mainland_fir_boundaries(&db).await,
            Err(InvalidNavdataError::DatabaseError(_))
        ));
        db.close().await;
        assert!(matches!(
            NavdataService::mainland_fir_boundaries(&db).await,
            Err(InvalidNavdataError::DatabaseError(_))
        ));
    }

    #[tokio::test]
    async fn navdata_service_initializes_without_a_fir_table() {
        let path = std::env::temp_dir().join(format!("vatprc-no-firs-{}.db", ulid::Ulid::new()));
        let db = SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
        db.close().await;
        let service = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
            .await
            .unwrap();
        assert!(!service.china_boundaries().contains((30., 110.)));
        drop(service);
        std::fs::remove_file(path).unwrap();
    }
}
