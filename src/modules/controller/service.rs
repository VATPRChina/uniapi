use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Datelike, TimeZone, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::adapter::compat::{AtcConnection, CompatClient, CompatClientError};
use crate::modules::atc_position::repository::AtcPositionRepository;
use crate::modules::audit_log::models::AuditLogEntity;
use crate::modules::audit_log::service::{AuditLogService, AuditLogServiceError};
use crate::modules::user::service::user::{UserService, UserServiceError};

use super::models::{
    CompatFutureController, Controller, ControllerOnlineTime, ControllerOnlineTimeSummary,
    ControllerPermission, ControllerPositionKind, ControllerRating, ControllerSave,
    UserControllerState,
};
use super::repository::compat::CompatRepository;
use super::repository::controller::{AtcControllerPermissionRecord, ControllerRepository};
use super::repository::sector::SectorRepository;
use super::repository::user_atc_permission::{AtcPermissionRecord, UserAtcPermissionRepository};
use super::repository::user_atc_status::{
    AtcStatusRecord, UserAtcStatusRepository, UserAtcStatusTransactionRepository,
};

#[derive(Clone)]
pub struct ControllerService {
    db: PgPool,
    audit_log: AuditLogService,
    user: UserService,
    compat: CompatClient,
}

impl ControllerService {
    pub fn new(
        db: PgPool,
        audit_log: AuditLogService,
        user: UserService,
        compat: CompatClient,
    ) -> Self {
        Self {
            db,
            audit_log,
            user,
            compat,
        }
    }

    pub async fn list(&self) -> Result<Vec<Controller>, ControllerServiceError> {
        let rows = self.db.list_atc_controllers().await?;
        let mut controllers = BTreeMap::<Uuid, Controller>::new();

        for row in rows {
            let controller = controller(&row)?;
            let controller = controllers.entry(row.user_id).or_insert(controller);
            if let Some(permission) = controller_permission(&row)? {
                controller.permissions.push(permission);
            }
        }

        Ok(controllers.into_values().collect())
    }

    pub async fn find(&self, user_id: Uuid) -> Result<Controller, ControllerServiceError> {
        let status = self
            .db
            .find_user_atc_status_by_user_id(user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        let permissions = self.db.list_user_atc_permission_by_user_id(user_id).await?;
        controller_from_records(status, permissions)
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        status: ControllerSave,
        operated_by: Uuid,
    ) -> Result<Controller, ControllerServiceError> {
        let mut transaction = self.db.begin().await?;
        let before = controller_audit_snapshot(&mut transaction, user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        transaction.upsert_user_atc_status(user_id, &status).await?;
        let after = controller_audit_snapshot(&mut transaction, user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        transaction.commit().await?;

        self.audit_log
            .record(
                AuditLogEntity::UserAtcPermission(user_id, user_id),
                operated_by,
                Some(&before),
                Some(&after),
            )
            .await?;

        self.find(user_id).await
    }

    pub async fn has_any_permission(&self, user_id: Uuid) -> Result<bool, ControllerServiceError> {
        Ok(self
            .db
            .has_user_atc_permission_any_by_user_id(user_id)
            .await?)
    }

    pub async fn has_mentor_permission(
        &self,
        user_id: Uuid,
    ) -> Result<bool, ControllerServiceError> {
        Ok(self
            .db
            .has_user_atc_permission_mentor_by_user_id(user_id)
            .await?)
    }

    pub async fn has_sector_permission(
        &self,
        user_id: Uuid,
    ) -> Result<bool, ControllerServiceError> {
        let user = self
            .user
            .find_summary_by_id(user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        Ok(self.db.user_sector_can_online(user.id, &user.cid).await?)
    }

    pub async fn future_compat_controllers(
        &self,
    ) -> Result<Vec<CompatFutureController>, ControllerServiceError> {
        Ok(self
            .db
            .future_compat_controllers()
            .await?
            .into_iter()
            .map(|controller| CompatFutureController {
                callsign: controller.callsign,
                name: controller.name,
                start_at: controller.start_at,
                end_at: controller.end_at,
            })
            .collect())
    }

    pub async fn current_quarter_online_time(
        &self,
        user_id: Uuid,
    ) -> Result<ControllerOnlineTime, ControllerServiceError> {
        let user = self
            .user
            .find_summary_by_id(user_id)
            .await?
            .ok_or(ControllerServiceError::UserNotFound(user_id))?;
        let as_of = Utc::now();
        let period_start = current_quarter_start(as_of);
        let (sessions, online_data, member, tier_2_callsigns) = tokio::join!(
            self.member_atc_sessions(&user.cid),
            self.compat.get_online_data(),
            self.compat.get_member(&user.cid),
            self.db.list_tier_2_atc_position_callsigns(),
        );
        let sessions = sessions?;
        let rating = member?.rating;
        let tier_2_callsigns = tier_2_callsigns?
            .into_iter()
            .map(|callsign| callsign.to_ascii_uppercase())
            .collect::<HashSet<_>>();

        let mut quarter = ControllerOnlineTimeSummary::default();
        let mut lifetime = ControllerOnlineTimeSummary::default();
        for session in &sessions {
            if let (Some(start), Some(end)) = (session.start, session.end) {
                accumulate_session(
                    &mut quarter,
                    &mut lifetime,
                    &session.callsign,
                    start,
                    end,
                    period_start,
                    as_of,
                    rating,
                    &tier_2_callsigns,
                );
            }
        }

        match online_data {
            Ok(online_data) => {
                if let Ok(cid) = user.cid.parse::<i64>()
                    && let Some(controller) =
                        online_data.controllers.iter().find(|item| item.cid == cid)
                    && let Some(logon_time) = controller.logon_time
                {
                    accumulate_session(
                        &mut quarter,
                        &mut lifetime,
                        &controller.callsign,
                        logon_time,
                        as_of,
                        period_start,
                        as_of,
                        rating,
                        &tier_2_callsigns,
                    );
                }
            }
            Err(error) => {
                tracing::warn!(%error, "failed to include the current VATSIM session in online time");
            }
        }

        Ok(ControllerOnlineTime {
            period: format!("{}Q{}", as_of.year(), as_of.month0() / 3 + 1),
            period_start,
            as_of,
            total_seconds: quarter.total_seconds,
            by_position: quarter.by_position,
            lifetime,
        })
    }

    async fn member_atc_sessions(
        &self,
        cid: &str,
    ) -> Result<Vec<AtcConnection>, CompatClientError> {
        const PAGE_SIZE: usize = 1000;

        let mut sessions = Vec::new();
        loop {
            let page = self
                .compat
                .get_member_atc_sessions(cid, PAGE_SIZE, sessions.len())
                .await?;
            let count = page.count;
            let page_len = page.items.len();
            sessions.extend(page.items.into_iter().map(|item| item.connection_id));

            if page_len == 0 || sessions.len() >= count {
                return Ok(sessions);
            }
        }
    }
}

fn current_quarter_start(now: DateTime<Utc>) -> DateTime<Utc> {
    let first_month = now.month0() / 3 * 3 + 1;
    Utc.with_ymd_and_hms(now.year(), first_month, 1, 0, 0, 0)
        .single()
        .expect("the first day of a calendar quarter is valid")
}

fn accumulate_session(
    quarter: &mut ControllerOnlineTimeSummary,
    lifetime: &mut ControllerOnlineTimeSummary,
    callsign: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    period_start: DateTime<Utc>,
    as_of: DateTime<Utc>,
    rating: i32,
    tier_2_callsigns: &HashSet<String>,
) {
    let Some(position) = eligible_position(callsign, rating, tier_2_callsigns) else {
        return;
    };

    for (summary, since) in [(quarter, period_start), (lifetime, start)] {
        if let Some(seconds) = overlap_seconds(start, end, since, as_of) {
            summary.total_seconds += seconds;
            match position {
                OnlineTimePosition::S1 => summary.by_position.s1 += seconds,
                OnlineTimePosition::S2 => summary.by_position.s2 += seconds,
                OnlineTimePosition::S3 => summary.by_position.s3 += seconds,
                OnlineTimePosition::C1Plus => summary.by_position.c1_plus += seconds,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnlineTimePosition {
    S1,
    S2,
    S3,
    C1Plus,
}

#[cfg(test)]
fn session_seconds(
    callsign: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    period_start: DateTime<Utc>,
    as_of: DateTime<Utc>,
    rating: i32,
    tier_2_callsigns: &HashSet<String>,
) -> Option<u64> {
    eligible_position(callsign, rating, tier_2_callsigns)?;
    overlap_seconds(start, end, period_start, as_of)
}

fn overlap_seconds(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    period_start: DateTime<Utc>,
    as_of: DateTime<Utc>,
) -> Option<u64> {
    let overlap_start = start.max(period_start);
    let overlap_end = end.min(as_of);
    (overlap_end > overlap_start).then(|| (overlap_end - overlap_start).num_seconds() as u64)
}

fn eligible_position(
    callsign: &str,
    rating: i32,
    tier_2_callsigns: &HashSet<String>,
) -> Option<OnlineTimePosition> {
    let callsign = callsign.to_ascii_uppercase();
    if !is_vatprc_position(&callsign)
        || is_training_position(&callsign)
        || is_military_position(&callsign)
        || is_tier_2_position(&callsign, tier_2_callsigns)
    {
        return None;
    }

    let position = match callsign.rsplit('_').next()? {
        "GND" | "DEL" | "RMP" => OnlineTimePosition::S1,
        "TWR" => OnlineTimePosition::S2,
        "APP" => OnlineTimePosition::S3,
        "CTR" => OnlineTimePosition::C1Plus,
        _ => return None,
    };

    rating_allows_position(rating, position).then_some(position)
}

fn rating_allows_position(rating: i32, position: OnlineTimePosition) -> bool {
    match rating {
        2 => matches!(position, OnlineTimePosition::S1 | OnlineTimePosition::S2),
        3 => matches!(
            position,
            OnlineTimePosition::S1 | OnlineTimePosition::S2 | OnlineTimePosition::S3
        ),
        4 => matches!(
            position,
            OnlineTimePosition::S2 | OnlineTimePosition::S3 | OnlineTimePosition::C1Plus
        ),
        5..=12 => matches!(
            position,
            OnlineTimePosition::S3 | OnlineTimePosition::C1Plus
        ),
        _ => false,
    }
}

fn is_training_position(callsign: &str) -> bool {
    let parts = callsign.split('_').collect::<Vec<_>>();
    parts.len() > 2
        && parts[1..parts.len() - 1].iter().any(|part| {
            let Some(suffix) = part.strip_prefix('X').or_else(|| part.strip_prefix('I')) else {
                return false;
            };
            suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn is_military_position(callsign: &str) -> bool {
    let parts = callsign.split('_').collect::<Vec<_>>();
    parts.len() > 2 && parts[1..parts.len() - 1].contains(&"MIL")
}

fn is_tier_2_position(callsign: &str, tier_2_callsigns: &HashSet<String>) -> bool {
    tier_2_callsigns.iter().any(|tier_2_callsign| {
        tier_2_callsign
            .strip_prefix('*')
            .map_or(callsign == tier_2_callsign, |suffix| {
                callsign.ends_with(suffix)
            })
    })
}

fn is_vatprc_position(callsign: &str) -> bool {
    let callsign = callsign.to_ascii_uppercase();
    let parts = callsign.split('_').collect::<Vec<_>>();
    let Some(prefix) = parts.first() else {
        return false;
    };
    let Some(position) = parts.last() else {
        return false;
    };
    let prefix = prefix.as_bytes();

    prefix.len() == 4
        && prefix[0] == b'Z'
        && b"BGHJLPSUWY".contains(&prefix[1])
        && prefix[2..].iter().all(u8::is_ascii_alphabetic)
        && matches!(*position, "DEL" | "GND" | "RMP" | "TWR" | "APP" | "CTR")
}

fn controller(row: &AtcControllerPermissionRecord) -> Result<Controller, ControllerServiceError> {
    Ok(Controller {
        user_id: row.user_id,
        is_visiting: row.is_visiting,
        is_absent: row.is_absent,
        rating: parse_rating(Some(&row.rating))?,
        permissions: Vec::new(),
    })
}

fn controller_permission(
    row: &AtcControllerPermissionRecord,
) -> Result<Option<ControllerPermission>, ControllerServiceError> {
    let (Some(position_kind_id), Some(state)) = (&row.position_kind_id, &row.state) else {
        return Ok(None);
    };

    Ok(Some(ControllerPermission {
        position_kind: parse_position_kind(position_kind_id)?,
        state: parse_controller_state(state)?,
        solo_expires_at: row.solo_expires_at,
    }))
}

fn controller_from_records(
    status: AtcStatusRecord,
    permissions: Vec<AtcPermissionRecord>,
) -> Result<Controller, ControllerServiceError> {
    Ok(Controller {
        user_id: status.user_id,
        is_visiting: status.is_visiting.unwrap_or(false),
        is_absent: status.is_absent.unwrap_or(false),
        rating: parse_rating(status.rating.as_deref())?,
        permissions: permissions
            .into_iter()
            .map(|permission| {
                Ok(ControllerPermission {
                    position_kind: parse_position_kind(&permission.position_kind_id)?,
                    state: parse_controller_state(&permission.state)?,
                    solo_expires_at: permission.solo_expires_at,
                })
            })
            .collect::<Result<_, ControllerServiceError>>()?,
    })
}

fn parse_rating(rating: Option<&str>) -> Result<ControllerRating, ControllerServiceError> {
    let rating = rating.unwrap_or("OBS");
    rating
        .parse()
        .map_err(|_| ControllerServiceError::InvalidControllerRating(rating.to_owned()))
}

fn parse_position_kind(
    position_kind: &str,
) -> Result<ControllerPositionKind, ControllerServiceError> {
    position_kind.parse().map_err(|_| {
        ControllerServiceError::InvalidControllerPositionKind(position_kind.to_owned())
    })
}

fn parse_controller_state(state: &str) -> Result<UserControllerState, ControllerServiceError> {
    state
        .parse()
        .map_err(|_| ControllerServiceError::InvalidControllerState(state.to_owned()))
}

#[derive(serde::Serialize)]
struct ControllerAuditSnapshot {
    status: AtcStatusRecord,
    permissions: Vec<AtcPermissionRecord>,
}

async fn controller_audit_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<Option<ControllerAuditSnapshot>, sqlx::Error> {
    let Some(status) = (&mut **transaction)
        .find_user_atc_status_by_user_id_for_update(user_id)
        .await?
    else {
        return Ok(None);
    };
    let permissions = (&mut **transaction)
        .list_user_atc_permission_by_user_id(user_id)
        .await?;

    Ok(Some(ControllerAuditSnapshot {
        status,
        permissions,
    }))
}

#[derive(Debug, thiserror::Error)]
pub enum ControllerServiceError {
    #[error("user {0} not found")]
    UserNotFound(Uuid),
    #[error("failed to query controller information: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid controller state {0}")]
    InvalidControllerState(String),
    #[error("invalid controller rating {0}")]
    InvalidControllerRating(String),
    #[error("invalid controller position kind {0}")]
    InvalidControllerPositionKind(String),
    #[error("failed to access controller user: {0}")]
    User(#[from] UserServiceError),
    #[error("failed to record controller audit log: {0}")]
    AuditLog(#[from] AuditLogServiceError),
    #[error("failed to access VATSIM controller sessions: {0}")]
    Compat(#[from] CompatClientError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(value: &str) -> DateTime<Utc> {
        value.parse().unwrap()
    }

    #[test]
    fn rejects_invalid_controller_state() {
        assert!(matches!(
            parse_controller_state("student"),
            Err(ControllerServiceError::InvalidControllerState(value)) if value == "student"
        ));
    }

    #[test]
    fn finds_the_start_of_the_current_calendar_quarter() {
        assert_eq!(
            current_quarter_start(time("2026-08-23T12:34:56Z")),
            time("2026-07-01T00:00:00Z")
        );
        assert_eq!(
            current_quarter_start(time("2027-01-01T00:00:00Z")),
            time("2027-01-01T00:00:00Z")
        );
    }

    #[test]
    fn recognizes_vatprc_controlling_positions() {
        for callsign in [
            "ZBAA_TWR",
            "ZSAA_TWR",
            "ZGAA_TWR",
            "ZPAA_TWR",
            "ZUAA_TWR",
            "ZLAA_TWR",
            "ZWAA_TWR",
            "ZHAA_TWR",
            "ZJAA_TWR",
            "ZYAA_TWR",
            "ZSHA_E_CTR",
            "zgzu_app",
            "ZUUU_2_GND",
        ] {
            assert!(is_vatprc_position(callsign), "{callsign}");
        }
        for callsign in [
            "VHHH_TWR",
            "ZKPY_CTR",
            "ZMAA_TWR",
            "ZBAA_OBS",
            "ZBAA_ATIS",
            "ZBAA_DEP",
            "ZSHA_FSS",
        ] {
            assert!(!is_vatprc_position(callsign), "{callsign}");
        }
    }

    #[test]
    fn allows_only_positions_within_one_rating_level() {
        let tier_2_callsigns = HashSet::new();
        for (rating, allowed) in [
            (2, [true, true, false, false]),
            (3, [true, true, true, false]),
            (4, [false, true, true, true]),
            (5, [false, false, true, true]),
            (7, [false, false, true, true]),
            (8, [false, false, true, true]),
            (10, [false, false, true, true]),
            (11, [false, false, true, true]),
            (12, [false, false, true, true]),
        ] {
            for (callsign, expected) in ["ZBAA_GND", "ZBAA_TWR", "ZBAA_APP", "ZBAA_CTR"]
                .into_iter()
                .zip(allowed)
            {
                assert_eq!(
                    eligible_position(callsign, rating, &tier_2_callsigns).is_some(),
                    expected,
                    "rating {rating}, callsign {callsign}"
                );
            }
        }

        for rating in [-1, 0, 1, 13] {
            assert!(eligible_position("ZBAA_TWR", rating, &tier_2_callsigns).is_none());
        }
    }

    #[test]
    fn excludes_training_tier_2_and_non_vatprc_positions() {
        let tier_2_callsigns = ["ZBAL_TWR", "*_MIL_TWR"]
            .into_iter()
            .map(str::to_owned)
            .collect();

        for callsign in [
            "ZBAA_X_CTR",
            "ZBAA_X1_CTR",
            "ZBAA__X2__CTR",
            "ZBAA__I_CTR",
            "ZBAA_I__CTR",
            "ZBAA__I1__CTR",
            "ZBAA_I2_CTR",
            "zbaa_i2_ctr",
        ] {
            assert!(
                eligible_position(callsign, 5, &tier_2_callsigns).is_none(),
                "{callsign}"
            );
        }
        for callsign in ["ZBAA_W_CTR", "ZBAA_2_CTR", "ZBAA_IX_CTR"] {
            assert!(
                eligible_position(callsign, 5, &tier_2_callsigns).is_some(),
                "{callsign}"
            );
        }
        for callsign in ["ZBAL_TWR", "ZBAA_MIL_TWR"] {
            assert!(
                eligible_position(callsign, 3, &tier_2_callsigns).is_none(),
                "{callsign}"
            );
        }
        for callsign in [
            "ZBAA_MIL_TWR",
            "ZBAA__MIL__APP",
            "ZBAA_MIL_CTR",
            "zbaa_mil_gnd",
        ] {
            assert!(
                eligible_position(callsign, 5, &HashSet::new()).is_none(),
                "{callsign}"
            );
        }
        for callsign in ["VHHH_TWR", "ZKPY_CTR", "ZBAA_DEP", "ZSHA_FSS"] {
            assert!(
                eligible_position(callsign, 5, &tier_2_callsigns).is_none(),
                "{callsign}"
            );
        }
    }

    #[test]
    fn groups_lifetime_and_quarter_seconds_by_position() {
        let period_start = time("2026-07-01T00:00:00Z");
        let as_of = time("2026-07-01T01:00:00Z");
        let mut quarter = ControllerOnlineTimeSummary::default();
        let mut lifetime = ControllerOnlineTimeSummary::default();
        let tier_2_callsigns = HashSet::new();
        for callsign in [
            "ZBAA_GND",
            "ZBAA_DEL",
            "zbaa_2_rmp",
            "ZBAA_TWR",
            "ZBAA_APP",
            "ZSHA_E_CTR",
            "ZBAA_DEP",
            "ZSHA_FSS",
            "RJTT_TWR",
            "ZBAA_OBS",
        ] {
            accumulate_session(
                &mut quarter,
                &mut lifetime,
                callsign,
                time("2026-06-30T23:00:00Z"),
                time("2026-07-01T02:00:00Z"),
                period_start,
                as_of,
                3,
                &tier_2_callsigns,
            );
        }
        assert_eq!(quarter.total_seconds, 5 * 3600);
        assert_eq!(lifetime.total_seconds, 5 * 7200);
        assert_eq!(quarter.by_position.s1, 3 * 3600);
        assert_eq!(quarter.by_position.s2, 3600);
        assert_eq!(quarter.by_position.s3, 3600);
        assert_eq!(quarter.by_position.c1_plus, 0);
        assert_eq!(lifetime.by_position.s1, 3 * 7200);
        assert_eq!(lifetime.by_position.s2, 7200);
        assert_eq!(lifetime.by_position.s3, 7200);
        assert_eq!(lifetime.by_position.c1_plus, 0);
    }

    #[test]
    fn includes_historical_and_live_sessions_without_counting_future_or_invalid_time() {
        let period_start = time("2026-07-01T00:00:00Z");
        let as_of = time("2026-07-01T01:00:00Z");
        let mut quarter = ControllerOnlineTimeSummary::default();
        let mut lifetime = ControllerOnlineTimeSummary::default();
        let tier_2_callsigns = HashSet::new();
        for (start, end) in [
            ("2026-06-01T00:00:00Z", "2026-06-01T01:00:00Z"),
            ("2026-07-01T00:30:00Z", "2026-07-01T01:00:00Z"),
            ("2026-07-01T02:00:00Z", "2026-07-01T03:00:00Z"),
            ("2026-07-01T00:30:00Z", "2026-07-01T00:00:00Z"),
        ] {
            accumulate_session(
                &mut quarter,
                &mut lifetime,
                "ZBAA_TWR",
                time(start),
                time(end),
                period_start,
                as_of,
                3,
                &tier_2_callsigns,
            );
        }
        assert_eq!(quarter.total_seconds, 1800);
        assert_eq!(quarter.by_position.s2, 1800);
        assert_eq!(lifetime.total_seconds, 5400);
        assert_eq!(lifetime.by_position.s2, 5400);
    }

    #[test]
    fn counts_only_the_part_of_a_session_inside_the_quarter() {
        let period_start = time("2026-07-01T00:00:00Z");
        let as_of = time("2026-08-23T12:00:00Z");
        let tier_2_callsigns = HashSet::new();

        assert_eq!(
            session_seconds(
                "ZBAA_TWR",
                time("2026-06-30T23:00:00Z"),
                time("2026-07-01T02:30:00Z"),
                period_start,
                as_of,
                3,
                &tier_2_callsigns,
            ),
            Some(9_000)
        );
        assert_eq!(
            session_seconds(
                "RJTT_TWR",
                time("2026-08-23T10:00:00Z"),
                as_of,
                period_start,
                as_of,
                3,
                &tier_2_callsigns,
            ),
            None
        );
    }
}
