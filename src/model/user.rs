use std::collections::HashSet;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::model::user_role::UserRole;

#[derive(Debug, Clone)]
pub struct UserSummary {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub email: Option<String>,
    pub direct_roles: HashSet<UserRole>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub cid: String,
    pub full_name: String,
    pub email: Option<String>,
    pub direct_roles: HashSet<UserRole>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub moodle_user: Option<MoodleUser>,
}

#[derive(Debug, Clone, Copy)]
pub struct MoodleUser {
    pub id: i64,
}
