use std::collections::{BTreeMap, BTreeSet};

use sqlx::FromRow;

use super::{InvalidNavdataError, NavdataResult, NavdataService};
use crate::modules::navdata::models::MainlandFirBoundaries;

const MAINLAND_FIRS: [&str; 9] = [
    "ZBPE", "ZGZU", "ZHWH", "ZJSA", "ZLHW", "ZPKM", "ZSHA", "ZWUQ", "ZYSH",
];

impl NavdataService {
    /// Read ARINC 424 UF boundaries from the configured DFD v2 database.
    /// F = FIR, B = combined FIR/UIR. C (delegated airspace) and UIR-only
    /// records are deliberately excluded from horizontal FIR coverage.
    pub async fn mainland_fir_boundaries(&self) -> NavdataResult<MainlandFirBoundaries> {
        let records = sqlx::query_as::<_, FirRecord>(
            "SELECT area_code, fir_uir_identifier, fir_uir_indicator, seqno,
                    boundary_via, fir_uir_latitude AS latitude, fir_uir_longitude AS longitude
             FROM tbl_uf_fir_uir
             WHERE fir_uir_identifier IN ('ZBPE', 'ZGZU', 'ZHWH', 'ZJSA', 'ZLHW',
                                         'ZPKM', 'ZSHA', 'ZWUQ', 'ZYSH')
               AND fir_uir_indicator IN ('F', 'B')
             ORDER BY area_code, fir_uir_identifier, fir_uir_indicator, seqno",
        )
        .fetch_all(&self.db)
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

fn build_boundaries(records: Vec<FirRecord>) -> NavdataResult<MainlandFirBoundaries> {
    let found: BTreeSet<_> = records
        .iter()
        .map(|r| r.fir_uir_identifier.as_str())
        .collect();
    let missing: Vec<_> = MAINLAND_FIRS
        .into_iter()
        .filter(|id| !found.contains(id))
        .collect();
    if !missing.is_empty() {
        return Err(invalid(format!(
            "missing mainland FIRs: {}",
            missing.join(", ")
        )));
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
    Ok(MainlandFirBoundaries { rings })
}

// DFD v2 boundary_via describes the edge FROM this record TO the next.
// G = great circle, H = rhumb line, E = close to the first vertex.
// https://developers.navigraph.com/docs/navigation-data/dfd-data-format-v2
// Densify curved edges at <= 0.05 degree intervals before point-in-ring testing.
fn append_edge(ring: &mut Vec<[f64; 2]>, from: &FirRecord, to: &FirRecord) {
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
            ring.push([from.longitude, from.latitude]);
        } else if from.boundary_via.starts_with('G') {
            let a = ((1. - fraction) * angle).sin() / angle.sin();
            let b = (fraction * angle).sin() / angle.sin();
            let x = a * lat1.cos() * lon1.cos() + b * lat2.cos() * lon2.cos();
            let y = a * lat1.cos() * lon1.sin() + b * lat2.cos() * lon2.sin();
            let z = a * lat1.sin() + b * lat2.sin();
            ring.push([y.atan2(x).to_degrees(), z.atan2(x.hypot(y)).to_degrees()]);
        } else {
            let mercator = |latitude: f64| (std::f64::consts::FRAC_PI_4 + latitude / 2.).tan().ln();
            let y = mercator(lat1) * (1. - fraction) + mercator(lat2) * fraction;
            ring.push([
                (lon1 + delta_lon * fraction).to_degrees(),
                (2. * y.exp().atan() - std::f64::consts::FRAC_PI_2).to_degrees(),
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::navdata::repository::PreferredRouteRepository;

    fn records() -> Vec<FirRecord> {
        MAINLAND_FIRS
            .into_iter()
            .flat_map(|identifier| {
                [
                    (20., 100., "H "),
                    (20., 120., "H "),
                    (40., 120., "H "),
                    (40., 100., "HE"),
                ]
                .into_iter()
                .enumerate()
                .map(move |(index, (latitude, longitude, via))| FirRecord {
                    area_code: "EEU".into(),
                    fir_uir_identifier: identifier.into(),
                    fir_uir_indicator: "F".into(),
                    seqno: (index as i64 + 1) * 10,
                    boundary_via: via.into(),
                    latitude,
                    longitude,
                })
            })
            .collect()
    }

    async fn navdata() -> NavdataService {
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE tbl_uf_fir_uir (area_code TEXT, fir_uir_identifier TEXT, fir_uir_indicator TEXT,
            seqno INTEGER, boundary_via TEXT, fir_uir_latitude REAL, fir_uir_longitude REAL)").execute(&db).await.unwrap();
        NavdataService {
            db,
            preferred_routes: PreferredRouteRepository::from_csv_path("assets/test/routes.csv")
                .unwrap(),
        }
    }

    async fn insert(navdata: &NavdataService, record: FirRecord) {
        sqlx::query("INSERT INTO tbl_uf_fir_uir VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(record.area_code)
            .bind(record.fir_uir_identifier)
            .bind(record.fir_uir_indicator)
            .bind(record.seqno)
            .bind(record.boundary_via)
            .bind(record.latitude)
            .bind(record.longitude)
            .execute(&navdata.db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn queries_ordered_mainland_fir_records_and_excludes_delegated_and_foreign_airspace() {
        let navdata = navdata().await;
        let mut records = records();
        // Exercise combined FIR/UIR records as well as FIR-only records.
        for record in &mut records {
            if record.fir_uir_identifier == "ZBPE" {
                record.fir_uir_indicator = "B".into();
            }
        }
        // Insert out of order to verify SQL ordering.
        for record in records.into_iter().rev() {
            insert(&navdata, record).await;
        }
        for (identifier, indicator) in [("ZSHA", "C"), ("ZSHA", "U"), ("VHHK", "F"), ("ZMUB", "F")]
        {
            for mut record in self::records().into_iter().take(4) {
                record.fir_uir_identifier = identifier.into();
                record.fir_uir_indicator = indicator.into();
                record.latitude -= 40.;
                insert(&navdata, record).await;
            }
        }
        let boundaries = navdata.mainland_fir_boundaries().await.unwrap();
        assert_eq!(boundaries.rings.len(), 9);
        assert!(boundaries.contains((30., 110.)));
        assert!(boundaries.contains((20., 110.)));
        assert!(!boundaries.contains((-10., 110.)));
        assert!(!boundaries.contains((30., 140.)));
        let selector = crate::modules::flight::flight_plan::preferred_route_segment::PreferredRouteSegmentService::new(&navdata, "ZBAA", "RJTT").await;
        assert!(selector.is_ok());
    }

    #[tokio::test]
    async fn incomplete_data_is_an_error_but_domestic_selection_needs_no_fir_records() {
        let navdata = navdata().await;
        let error = navdata
            .mainland_fir_boundaries()
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("missing mainland FIRs"));
        assert!(error.contains("ZSHA"));
        let selector = crate::modules::flight::flight_plan::preferred_route_segment::PreferredRouteSegmentService::new(&navdata, "ZBAA", "ZSPD").await.unwrap();
        assert!(selector.select(&[]).unwrap().is_empty());
    }

    #[test]
    fn validates_closure_path_types_coordinates_and_multiple_rings() {
        let mut missing_end = records();
        missing_end[3].boundary_via = "H ".into();
        assert!(
            build_boundaries(missing_end)
                .unwrap_err()
                .to_string()
                .contains("missing end")
        );
        let mut invalid_path = records();
        invalid_path[0].boundary_via = "R ".into();
        assert!(
            build_boundaries(invalid_path)
                .unwrap_err()
                .to_string()
                .contains("unsupported boundary")
        );
        let mut invalid_coordinate = records();
        invalid_coordinate[0].latitude = f64::NAN;
        assert!(build_boundaries(invalid_coordinate).is_err());
        let mut multiple = records();
        let mut extra: Vec<_> = records().into_iter().take(4).collect();
        for r in &mut extra {
            r.seqno += 40;
            r.latitude -= 40.;
        }
        multiple.splice(4..4, extra);
        let boundaries = build_boundaries(multiple).unwrap();
        assert_eq!(boundaries.rings.len(), 10);
        assert!(boundaries.contains((-10., 110.)));
    }

    #[test]
    fn great_circle_and_rhumb_line_edges_use_the_published_path_type() {
        let mut records = records();
        let mut from = records.remove(0);
        let mut to = records.remove(0);
        from.latitude = 60.;
        from.longitude = -10.;
        to.latitude = 60.;
        to.longitude = 10.;
        from.boundary_via = "G ".into();
        let mut great_circle = Vec::new();
        append_edge(&mut great_circle, &from, &to);
        assert!(great_circle[great_circle.len() / 2][1] > 60.3);
        from.boundary_via = "H ".into();
        let mut rhumb = Vec::new();
        append_edge(&mut rhumb, &from, &to);
        assert!((rhumb[rhumb.len() / 2][1] - 60.).abs() < 1e-10);
    }
}
