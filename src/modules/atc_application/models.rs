use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct AtcApplication {
    pub id: Uuid,
    pub user_id: Uuid,
    pub application_filing_id: Uuid,
    pub review_filing_id: Option<Uuid>,
    pub applied_at: DateTime<Utc>,
    pub status: AtcApplicationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AtcApplicationStatus {
    Submitted,
    InWaitlist,
    Approved,
    Rejected,
    Aborted,
}

impl AtcApplicationStatus {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::Submitted => "Submitted",
            Self::InWaitlist => "InWaitlist",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
            Self::Aborted => "Aborted",
        }
    }
}

impl TryFrom<&str> for AtcApplicationStatus {
    type Error = InvalidAtcApplicationStatus;

    fn try_from(status: &str) -> Result<Self, Self::Error> {
        match status {
            "Submitted" => Ok(Self::Submitted),
            "InWaitlist" => Ok(Self::InWaitlist),
            "Approved" => Ok(Self::Approved),
            "Rejected" => Ok(Self::Rejected),
            "Aborted" => Ok(Self::Aborted),
            _ => Err(InvalidAtcApplicationStatus(status.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid ATC application status {0}")]
pub struct InvalidAtcApplicationStatus(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_database_status() {
        assert_eq!(
            AtcApplicationStatus::try_from("submitted").unwrap_err(),
            InvalidAtcApplicationStatus("submitted".to_owned())
        );
    }
}
