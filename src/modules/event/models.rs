use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub title_en: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub start_booking_at: Option<DateTime<Utc>>,
    pub end_booking_at: Option<DateTime<Utc>>,
    pub start_atc_booking_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub community_link: Option<String>,
    pub vatsim_link: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct EventSave {
    pub title: String,
    pub title_en: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub start_booking_at: Option<DateTime<Utc>>,
    pub end_booking_at: Option<DateTime<Utc>>,
    pub start_atc_booking_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub community_link: Option<String>,
    pub vatsim_link: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct EventAirspace {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub icao_codes: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct EventAirspaceSave {
    pub name: String,
    pub icao_codes: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EventAtcPosition {
    pub id: Uuid,
    pub event_id: Uuid,
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub position_kind_id: String,
    pub minimum_controller_state: i32,
    pub booking_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct EventAtcPositionSave {
    pub callsign: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub remarks: Option<String>,
    pub position_kind_id: String,
    pub minimum_controller_state: i32,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EventSlot {
    pub id: Uuid,
    pub airspace_id: Uuid,
    pub enter_at: DateTime<Utc>,
    pub leave_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub callsign: Option<String>,
    pub aircraft_type_icao: Option<String>,
    pub booking_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct EventSlotSave {
    pub airspace_id: Uuid,
    pub enter_at: DateTime<Utc>,
    pub leave_at: Option<DateTime<Utc>>,
    pub callsign: Option<String>,
    pub aircraft_type_icao: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct EventBooking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct EventAtcBooking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub booked_at: DateTime<Utc>,
}
