use std::collections::HashSet;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{AppendHeaders, IntoResponse, Response};
use serde::Serialize;

use crate::adapter::compat::CompatClientError;
use crate::adapter::moodle::MoodleError;
use crate::adapter::smms::SmmsError;
use crate::auth::AuthError;
use crate::flight_plan::parser::ParserError;
use crate::flight_plan::validator::ValidatorError;

macro_rules! api_errors {
    (
        $(
            $name:ident
            $( { $( $( #[$meta:meta] )* $field:ident : $fty:ty ),* $(,)? } )?
            : $status:expr => $description:expr
        ),* $(,)?
    ) => {
        #[derive(std::fmt::Debug, thiserror::Error)]
        pub enum ApiError {
            $(
                #[error($description)]
                $name $( { $( $( #[$meta] )* $field : $fty ),* } )?,
            )*
        }

        impl ApiError {
            #[allow(unused_variables)]
            pub fn identifier(&self) -> &'static str {
                match self {
                    $(
                        ApiError::$name $( { $( $field: _ ),* } )? => stringify!($name),
                    )*
                }
            }

            #[allow(unused_variables)]
            pub fn status_code(&self) -> StatusCode {
                match self {
                    $(
                        ApiError::$name $( { $( $field: _ ),* } )? => $status,
                    )*
                }
            }
        }
    };
}

api_errors!(
    BadRequest { field: String, reason: String }: StatusCode::BAD_REQUEST => "bad request on field {field}: {reason}",
    NotFound { entity: String, id: String }: StatusCode::NOT_FOUND => "{entity} with id {id} not found",
    InvalidTokenClaims { field: String, reason: String }: StatusCode::INTERNAL_SERVER_ERROR => "invalid token claim {field}: {reason}",
    NotOwned { entity: String, id: String }: StatusCode::FORBIDDEN => "{entity} with {id} is not owned by current user",
    Forbidden { allowed_roles: HashSet<String> }: StatusCode::FORBIDDEN => "only user with roles {allowed_roles:?} can perform this action",
    Unauthorized: StatusCode::UNAUTHORIZED => "unauthorized",

    TrainingApplicationAlreadyAccepted: StatusCode::CONFLICT => "training application already accepted",
    ApplicationAlreadyExists: StatusCode::CONFLICT => "ATC application already exists",
    ApplicationCannotUpdate: StatusCode::CONFLICT => "ATC application cannot be updated at current status",
    CannotCreateForOtherTrainer: StatusCode::FORBIDDEN => "cannot create training for other trainers",
    CannotDeleteStartedTraining: StatusCode::FORBIDDEN => "cannot delete started training",
    CannotUpdateTrainerTrainee: StatusCode::FORBIDDEN => "cannot update training trainer or trainee",
    EventNotInBookingTime: StatusCode::CONFLICT => "event not in booking time",
    FlightNotFoundForCid: StatusCode::NOT_FOUND => "flight not found for cid",
    InsufficientAtcPermission: StatusCode::FORBIDDEN => "insufficient ATC permission",
    MissingSessionId: StatusCode::INTERNAL_SERVER_ERROR => "no sid in token",
    PositionBooked: StatusCode::CONFLICT => "event position booked",
    PositionBookedByAnotherUser: StatusCode::CONFLICT => "event position booked by another user",
    PositionNotBooked: StatusCode::CONFLICT => "event position not booked",
    RemoveStaffForbidden: StatusCode::FORBIDDEN => "only division director can remove staff role",
    SlotBooked: StatusCode::CONFLICT => "event slot already booked",
    SlotBookedByAnotherUser: StatusCode::CONFLICT => "event slot booked by another user",
    SlotNotBooked: StatusCode::CONFLICT => "event slot not booked",
    SoloExpirationNotProvided: StatusCode::BAD_REQUEST => "solo expiration not provided",

    RouteParser { #[from] source: ParserError }: StatusCode::INTERNAL_SERVER_ERROR => "internal error",
    RouteValidator { #[from] source: ValidatorError }: StatusCode::INTERNAL_SERVER_ERROR => "internal error",

    Compat { #[from] source: CompatClientError }: StatusCode::SERVICE_UNAVAILABLE => "transient error",
    Database { #[from] source: sqlx::Error}: StatusCode::SERVICE_UNAVAILABLE => "transient error",
    Moodle { #[from] source: MoodleError }: StatusCode::SERVICE_UNAVAILABLE => "transient error",
    Multipart { #[from] source: axum::extract::multipart::MultipartError }: StatusCode::SERVICE_UNAVAILABLE => "transient error",
    Smms { #[from] source: SmmsError }: StatusCode::SERVICE_UNAVAILABLE => "transient error",
);

impl ApiError {
    pub fn bad_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        ApiError::BadRequest {
            field: field.into(),
            reason: reason.into(),
        }
    }

    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        ApiError::NotFound {
            entity: entity.into(),
            id: id.into(),
        }
    }

    pub fn forbidden(allowed_roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        ApiError::Forbidden {
            allowed_roles: allowed_roles.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::MissingRole(role) => ApiError::forbidden([role]),
            AuthError::MissingAnyRole(roles) => ApiError::forbidden(roles),
            AuthError::Database(source) => ApiError::Database { source },
            AuthError::MissingBearerToken | AuthError::InvalidBearerToken | AuthError::Jwt(_) => {
                ApiError::Unauthorized
            }
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    type_: String,
    title: String,
    status: u16,
    detail: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status_code(),
            [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
            Json(ProblemDetails {
                type_: format!(
                    "urn:vatprc-uniapi-error:{}",
                    convert_case::ccase!(kebab, self.identifier())
                ),
                title: self.to_string(),
                status: self.status_code().as_u16(),
                detail: self.to_string(),
            }),
        )
            .into_response()
    }
}
