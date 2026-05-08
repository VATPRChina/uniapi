use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::{
    adapter::{compat::CompatClientError, moodle::MoodleError, smms::SmmsError},
    flight_plan::{parser::ParserError, validator::ValidatorError},
};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("training application already accepted")]
    AlreadyAccepted,

    #[error("ATC application already exists")]
    ApplicationAlreadyExists,

    #[error("ATC application cannot update")]
    ApplicationCannotUpdate,

    #[error("ATC application not found")]
    ApplicationNotFound,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("callsign not found")]
    CallsignNotFound,

    #[error("cannot create training for other trainers")]
    CannotCreateForOtherTrainer,

    #[error("cannot delete started training")]
    CannotDeleteStartedTraining,

    #[error("cannot update training trainer or trainee")]
    CannotUpdateTrainerTrainee,

    #[error(transparent)]
    Compat(CompatClientError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("event not found")]
    EventNotFound,

    #[error("event not in booking time")]
    EventNotInBookingTime,

    #[error("flight not found for cid")]
    FlightNotFoundForCid,

    #[error("forbidden")]
    Forbidden,

    #[error("insufficient ATC permission")]
    InsufficientAtcPermission,

    #[error("invalid airspace id")]
    InvalidAirspaceId,

    #[error("invalid ATC rating")]
    InvalidAtcRating,

    #[error("invalid event id")]
    InvalidEventId,

    #[error("invalid id")]
    InvalidId,

    #[error("invalid position id")]
    InvalidPositionId,

    #[error("invalid ATC position kind")]
    InvalidPositionKind,

    #[error("invalid sid")]
    InvalidSessionId,

    #[error("invalid slot id")]
    InvalidSlotId,

    #[error("invalid token claims")]
    InvalidTokenClaims,

    #[error("invalid user id")]
    InvalidUserId,

    #[error("no sid in token")]
    MissingSessionId,

    #[error(transparent)]
    Moodle(MoodleError),

    #[error(transparent)]
    Multipart(axum::extract::multipart::MultipartError),

    #[error("not found")]
    NotFound,

    #[error("not implemented")]
    NotImplemented,

    #[error("not owned")]
    NotOwned,

    #[error("event position booked")]
    PositionBooked,

    #[error("event position booked by another user")]
    PositionBookedByAnotherUser,

    #[error("event position not booked")]
    PositionNotBooked,

    #[error("event ATC position not found")]
    PositionNotFound,

    #[error("refresh token {0} not found")]
    RefreshTokenNotFound(String),

    #[error("only division director can remove staff role")]
    RemoveStaffForbidden,

    #[error(transparent)]
    RouteParser(ParserError),

    #[error(transparent)]
    RouteValidator(ValidatorError),

    #[error("sheet not found")]
    SheetNotFound,

    #[error("event slot already booked")]
    SlotBooked,

    #[error("event slot booked by another user")]
    SlotBookedByAnotherUser,

    #[error("event slot not booked")]
    SlotNotBooked,

    #[error("event slot not found")]
    SlotNotFound,

    #[error("solo expiration not provided")]
    SoloExpirationNotProvided,

    #[error("Image upload to SM.MS failed: {0}")]
    Smms(SmmsError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("user not found")]
    UserNotFound,
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_)
            | Self::CannotUpdateTrainerTrainee
            | Self::InvalidAirspaceId
            | Self::InvalidAtcRating
            | Self::InvalidEventId
            | Self::InvalidId
            | Self::InvalidPositionId
            | Self::InvalidPositionKind
            | Self::InvalidSlotId
            | Self::InvalidUserId
            | Self::Multipart(_)
            | Self::SoloExpirationNotProvided => StatusCode::BAD_REQUEST,

            Self::ApplicationAlreadyExists
            | Self::ApplicationCannotUpdate
            | Self::AlreadyAccepted
            | Self::CannotDeleteStartedTraining
            | Self::PositionBooked
            | Self::SlotBooked => StatusCode::CONFLICT,

            Self::CannotCreateForOtherTrainer
            | Self::EventNotInBookingTime
            | Self::Forbidden
            | Self::InsufficientAtcPermission
            | Self::NotOwned
            | Self::PositionBookedByAnotherUser
            | Self::RemoveStaffForbidden
            | Self::SlotBookedByAnotherUser => StatusCode::FORBIDDEN,

            Self::ApplicationNotFound
            | Self::CallsignNotFound
            | Self::EventNotFound
            | Self::FlightNotFoundForCid
            | Self::NotFound
            | Self::PositionNotBooked
            | Self::PositionNotFound
            | Self::SheetNotFound
            | Self::SlotNotBooked
            | Self::SlotNotFound
            | Self::UserNotFound => StatusCode::NOT_FOUND,

            Self::InvalidSessionId
            | Self::InvalidTokenClaims
            | Self::MissingSessionId
            | Self::RefreshTokenNotFound(_)
            | Self::Unauthorized => StatusCode::UNAUTHORIZED,

            Self::Compat(_) | Self::Moodle(_) => StatusCode::BAD_GATEWAY,

            Self::Database(_) | Self::RouteParser(_) | Self::RouteValidator(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }

            Self::NotImplemented => StatusCode::NOT_IMPLEMENTED,

            Self::Smms(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn detail(&self) -> String {
        match self {
            Self::BadRequest(message) => message.clone(),
            Self::Moodle(MoodleError::Request(inner)) => format!("{:?}", inner),
            _ => self.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        crate::problem::problem_response(self.status(), self.detail())
    }
}
