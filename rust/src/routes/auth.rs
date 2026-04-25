use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ulid::Ulid;
use uuid::Uuid;

use crate::{jwt::JwtError, services::Services};

const USER_CODE_ALPHABET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";

pub fn build_auth_routes() -> Router<Services> {
    Router::new()
        .route("/authorize", get(authorize))
        .route("/device_authorization", post(device_authorization))
        .route("/token", post(token))
}

async fn authorize(
    State(services): State<Services>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response, AuthEndpointError> {
    if query.response_type != "code" {
        return Ok((StatusCode::BAD_REQUEST, "invalid response_type").into_response());
    }
    if !services
        .jwt()
        .check_client_redirect(&query.client_id, &query.redirect_uri)
    {
        return Ok((StatusCode::UNAUTHORIZED, "client is invalid").into_response());
    }

    let state_id = Ulid::new().to_string();
    let auth_state = AuthenticationState {
        auth_type: AuthenticationStateType::Code,
        client_id: Some(query.client_id),
        redirect_uri: Some(query.redirect_uri),
        user_code: None,
        state: query.state,
    };
    let auth_state_cookie = percent_encode_cookie_value(
        &serde_json::to_string(&auth_state).map_err(AuthEndpointError::StateSerialization)?,
    );
    let cookie = format!(
        "auth-{state_id}={auth_state_cookie}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=600"
    );

    let mut response =
        Redirect::temporary(&format!("/auth/login?state={state_id}")).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(AuthEndpointError::InvalidHeader)?,
    );
    Ok(response)
}

async fn device_authorization(
    State(services): State<Services>,
    headers: HeaderMap,
    Form(request): Form<DeviceAuthorizationRequest>,
) -> Result<Response, AuthEndpointError> {
    if !services.jwt().check_client_exists(&request.client_id) {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(TokenErrorResponse {
                error: "invalid_client",
                error_description: "client_id not found",
            }),
        )
            .into_response());
    }

    let device_code = Ulid::new();
    let user_code = random_user_code();
    let expires_at = Utc::now() + Duration::seconds(services.jwt().device_authz_expires_seconds());

    sqlx::query(
        r#"
        INSERT INTO device_authorization (device_code, user_code, expires_at, client_id)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(Uuid::from(device_code))
    .bind(&user_code)
    .bind(expires_at)
    .bind(&request.client_id)
    .execute(services.db())
    .await
    .map_err(AuthEndpointError::Database)?;

    let verification_uri = format!("{}/auth/device", request_origin(&headers));
    let formatted_user_code = format!("{}-{}", &user_code[..4], &user_code[4..]);

    Ok(Json(DeviceAuthorizationResponse {
        device_code: device_code.to_string(),
        user_code: formatted_user_code,
        verification_uri: verification_uri.clone(),
        verification_uri_complete: Some(format!("{verification_uri}?user_code={user_code}")),
        expires_in: (expires_at - Utc::now()).num_seconds().max(0) as u32,
        interval: None,
    })
    .into_response())
}

async fn token(
    State(services): State<Services>,
    Form(request): Form<AccessTokenRequest>,
) -> Result<Response, AuthEndpointError> {
    match request.grant_type.as_str() {
        "urn:ietf:params:oauth:grant-type:device_code" => {
            device_code_grant(services, request).await
        }
        "refresh_token" => refresh_token_grant(services, request).await,
        "authorization_code" => authorization_code_grant(services, request).await,
        "client_credentials" => client_credentials_grant(services, request),
        _ => Ok(token_error(
            "unsupported_grant_type",
            "The authorization grant type is not supported by the authorization server.",
        )),
    }
}

async fn device_code_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Response, AuthEndpointError> {
    let Some(device_code) = parse_required_ulid(&request.device_code, "Missing device code")?
    else {
        return Ok(token_error("invalid_grant", "Device code not found"));
    };

    let Some(device_authz) = sqlx::query_as::<_, DeviceAuthorizationRow>(
        r#"
        SELECT device_authorization.device_code, device_authorization.user_code,
               device_authorization.expires_at, device_authorization.client_id,
               device_authorization.user_id, "user".updated_at AS user_updated_at
        FROM device_authorization
        LEFT JOIN "user" ON "user".id = device_authorization.user_id
        WHERE device_authorization.device_code = $1
        "#,
    )
    .bind(Uuid::from(device_code))
    .fetch_optional(services.db())
    .await
    .map_err(AuthEndpointError::Database)?
    else {
        return Ok(token_error("invalid_grant", "Device code not found"));
    };

    if device_authz.expires_at < Utc::now() {
        delete_device_authorization(&services, device_code).await?;
        return Ok(token_error("expired_token", "Device code expired"));
    }
    let Some(user_id) = device_authz.user_id else {
        return Ok(token_error(
            "authorization_pending",
            "User has not yet authorized this device",
        ));
    };
    if device_authz.client_id != request.client_id {
        return Ok(token_error("invalid_client", "Client ID mismatch"));
    }
    let Some(user_updated_at) = device_authz.user_updated_at else {
        return Ok(token_error("invalid_grant", "Device code not found"));
    };

    let refresh = issue_refresh_token(
        &services,
        user_id,
        user_updated_at,
        &request.client_id,
        None,
        false,
    )
    .await?;
    let access_token = services.jwt().issue_access_token(
        user_id,
        user_updated_at,
        refresh.token,
        &request.client_id,
    )?;
    delete_device_authorization(&services, device_code).await?;

    Ok(Json(TokenResponse {
        access_token: access_token.token,
        token_type: "Bearer",
        expires_in: access_token.expires_in,
        refresh_token: Some(refresh.token.to_string()),
        scope: access_token.scope,
    })
    .into_response())
}

async fn refresh_token_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Response, AuthEndpointError> {
    let Some(refresh_token) = parse_required_ulid(&request.refresh_token, "Missing refresh token")?
    else {
        return Ok(token_error("invalid_grant", "Refresh token not found"));
    };

    let Some(refresh) = find_refresh_session(&services, refresh_token).await? else {
        return Ok(token_error("invalid_grant", "Refresh token not found"));
    };
    if refresh.user_updated_at != refresh.updated_at {
        return Ok(token_error(
            "invalid_grant",
            "Refresh token has been revoked",
        ));
    }
    if refresh.expires_in < Utc::now() {
        return Ok(token_error("invalid_grant", "Refresh token expired"));
    }

    let new_refresh = issue_refresh_token(
        &services,
        refresh.user_id,
        refresh.updated_at,
        &refresh.client_id,
        Some(refresh_token),
        false,
    )
    .await?;
    let access_token = services.jwt().issue_access_token(
        refresh.user_id,
        refresh.updated_at,
        new_refresh.token,
        &refresh.client_id,
    )?;

    Ok(Json(TokenResponse {
        access_token: access_token.token,
        token_type: "Bearer",
        expires_in: access_token.expires_in,
        refresh_token: Some(new_refresh.token.to_string()),
        scope: access_token.scope,
    })
    .into_response())
}

async fn authorization_code_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Response, AuthEndpointError> {
    if request.client_id.is_empty() || request.code.is_empty() {
        return Ok(token_error("invalid_grant", "Missing client_id or code"));
    }

    let Some(validated_code) = services
        .jwt()
        .validate_auth_code(&request.code, &request.client_id)?
    else {
        return Ok(token_error("invalid_grant", "Invalid authorization code"));
    };

    let Some(session) = find_refresh_session_by_code(&services, validated_code.code).await? else {
        return Ok(token_error("invalid_grant", "Invalid authorization code"));
    };
    clear_session_code(&services, validated_code.code).await?;

    let access_token = services.jwt().issue_access_token(
        session.user_id,
        session.updated_at,
        session.token,
        &validated_code.client_id,
    )?;

    Ok(Json(TokenResponse {
        access_token: access_token.token,
        token_type: "Bearer",
        expires_in: access_token.expires_in,
        refresh_token: Some(session.token.to_string()),
        scope: access_token.scope,
    })
    .into_response())
}

fn client_credentials_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Response, AuthEndpointError> {
    if request.client_id.is_empty() || request.client_secret.is_empty() {
        return Ok(token_error(
            "invalid_grant",
            "Missing client_id or client_secret",
        ));
    }
    if !services
        .jwt()
        .check_client_secret(&request.client_id, &request.client_secret)
    {
        return Ok(token_error(
            "invalid_grant",
            "client_id or client_secret is invalid",
        ));
    }

    let access_token = services
        .jwt()
        .issue_client_access_token(&request.client_id)?;
    Ok(Json(TokenResponse {
        access_token: access_token.token,
        token_type: "Bearer",
        expires_in: access_token.expires_in,
        refresh_token: None,
        scope: access_token.scope,
    })
    .into_response())
}

async fn issue_refresh_token(
    services: &Services,
    user_id: Uuid,
    user_updated_at: DateTime<Utc>,
    client_id: &str,
    old_token: Option<Ulid>,
    create_code: bool,
) -> Result<RefreshSessionIssue, AuthEndpointError> {
    let token = Ulid::new();
    let code = create_code.then(Ulid::new);
    let expires_in = Utc::now() + Duration::days(services.jwt().refresh_expires_days());

    let mut transaction = services
        .db()
        .begin()
        .await
        .map_err(AuthEndpointError::Database)?;
    sqlx::query(
        r#"
        INSERT INTO session (token, user_id, user_updated_at, expires_in, code, client_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::from(token))
    .bind(user_id)
    .bind(user_updated_at)
    .bind(expires_in)
    .bind(code.map(Uuid::from))
    .bind(client_id)
    .execute(&mut *transaction)
    .await
    .map_err(AuthEndpointError::Database)?;

    if let Some(old_token) = old_token {
        sqlx::query("DELETE FROM session WHERE token = $1")
            .bind(Uuid::from(old_token))
            .execute(&mut *transaction)
            .await
            .map_err(AuthEndpointError::Database)?;
    }

    sqlx::query("DELETE FROM session WHERE user_id = $1 AND expires_in < now()")
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(AuthEndpointError::Database)?;
    sqlx::query(
        r#"
        DELETE FROM session
        WHERE user_id = $1
          AND user_updated_at <> (SELECT updated_at FROM "user" WHERE id = $1)
        "#,
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(AuthEndpointError::Database)?;

    transaction
        .commit()
        .await
        .map_err(AuthEndpointError::Database)?;

    Ok(RefreshSessionIssue { token, code })
}

async fn find_refresh_session(
    services: &Services,
    token: Ulid,
) -> Result<Option<RefreshSessionRow>, AuthEndpointError> {
    sqlx::query_as::<_, RefreshSessionRow>(
        r#"
        SELECT session.token, session.user_id, session.user_updated_at, session.expires_in,
               session.code, session.client_id, "user".updated_at
        FROM session
        JOIN "user" ON "user".id = session.user_id
        WHERE session.token = $1
        "#,
    )
    .bind(Uuid::from(token))
    .fetch_optional(services.db())
    .await
    .map_err(AuthEndpointError::Database)
}

async fn find_refresh_session_by_code(
    services: &Services,
    code: Ulid,
) -> Result<Option<RefreshSessionRow>, AuthEndpointError> {
    sqlx::query_as::<_, RefreshSessionRow>(
        r#"
        SELECT session.token, session.user_id, session.user_updated_at, session.expires_in,
               session.code, session.client_id, "user".updated_at
        FROM session
        JOIN "user" ON "user".id = session.user_id
        WHERE session.code = $1
        "#,
    )
    .bind(Uuid::from(code))
    .fetch_optional(services.db())
    .await
    .map_err(AuthEndpointError::Database)
}

async fn clear_session_code(services: &Services, code: Ulid) -> Result<(), AuthEndpointError> {
    sqlx::query("UPDATE session SET code = NULL WHERE code = $1")
        .bind(Uuid::from(code))
        .execute(services.db())
        .await
        .map_err(AuthEndpointError::Database)?;
    Ok(())
}

async fn delete_device_authorization(
    services: &Services,
    device_code: Ulid,
) -> Result<(), AuthEndpointError> {
    sqlx::query("DELETE FROM device_authorization WHERE device_code = $1")
        .bind(Uuid::from(device_code))
        .execute(services.db())
        .await
        .map_err(AuthEndpointError::Database)?;
    Ok(())
}

fn parse_required_ulid(
    value: &str,
    missing: &'static str,
) -> Result<Option<Ulid>, AuthEndpointError> {
    if value.trim().is_empty() {
        return Err(AuthEndpointError::TokenError {
            error: "invalid_request",
            error_description: missing,
        });
    }

    Ok(value.parse::<Ulid>().ok())
}

fn token_error(error: &'static str, error_description: &'static str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(TokenErrorResponse {
            error,
            error_description,
        }),
    )
        .into_response()
}

fn random_user_code() -> String {
    let mut rng = rand::rng();
    (0..8)
        .map(|_| {
            let index = rng.random_range(0..USER_CODE_ALPHABET.len());
            USER_CODE_ALPHABET[index] as char
        })
        .collect()
}

fn request_origin(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|value| value.to_str().ok())
        .unwrap_or("localhost");

    format!("{scheme}://{host}")
}

fn percent_encode_cookie_value(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

#[derive(Deserialize)]
struct AuthorizeQuery {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
}

#[derive(Serialize)]
struct AuthenticationState {
    #[serde(rename = "Type")]
    auth_type: AuthenticationStateType,
    #[serde(rename = "ClientId")]
    client_id: Option<String>,
    #[serde(rename = "RedirectUri")]
    redirect_uri: Option<String>,
    #[serde(rename = "UserCode")]
    user_code: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
}

#[derive(Serialize)]
enum AuthenticationStateType {
    #[serde(rename = "CODE")]
    Code,
}

#[derive(Deserialize)]
struct DeviceAuthorizationRequest {
    client_id: String,
    #[allow(dead_code)]
    scope: Option<String>,
}

#[derive(Serialize)]
struct DeviceAuthorizationResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_uri_complete: Option<String>,
    expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<u32>,
}

#[derive(Deserialize)]
struct AccessTokenRequest {
    #[serde(default)]
    grant_type: String,
    #[serde(default)]
    client_id: String,
    #[serde(default)]
    device_code: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    #[allow(dead_code)]
    code_verifier: String,
    #[serde(default)]
    client_secret: String,
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    scope: String,
}

#[derive(Serialize)]
struct TokenErrorResponse {
    error: &'static str,
    error_description: &'static str,
}

struct RefreshSessionIssue {
    token: Ulid,
    #[allow(dead_code)]
    code: Option<Ulid>,
}

#[derive(FromRow)]
struct RefreshSessionRow {
    #[sqlx(try_from = "Uuid")]
    token: Ulid,
    user_id: Uuid,
    user_updated_at: DateTime<Utc>,
    expires_in: DateTime<Utc>,
    #[allow(dead_code)]
    code: Option<Uuid>,
    client_id: String,
    updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct DeviceAuthorizationRow {
    #[allow(dead_code)]
    #[sqlx(try_from = "Uuid")]
    device_code: Ulid,
    #[allow(dead_code)]
    user_code: String,
    expires_at: DateTime<Utc>,
    client_id: String,
    user_id: Option<Uuid>,
    user_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
enum AuthEndpointError {
    Database(sqlx::Error),
    InvalidHeader(header::InvalidHeaderValue),
    Jwt(JwtError),
    StateSerialization(serde_json::Error),
    TokenError {
        error: &'static str,
        error_description: &'static str,
    },
}

impl From<JwtError> for AuthEndpointError {
    fn from(error: JwtError) -> Self {
        Self::Jwt(error)
    }
}

impl IntoResponse for AuthEndpointError {
    fn into_response(self) -> Response {
        match self {
            AuthEndpointError::TokenError {
                error,
                error_description,
            } => token_error(error, error_description),
            AuthEndpointError::Database(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InternalErrorResponse {
                    message: format!("Database query failed: {error}"),
                }),
            )
                .into_response(),
            AuthEndpointError::InvalidHeader(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InternalErrorResponse {
                    message: error.to_string(),
                }),
            )
                .into_response(),
            AuthEndpointError::Jwt(error) => (
                StatusCode::BAD_REQUEST,
                Json(InternalErrorResponse {
                    message: error.to_string(),
                }),
            )
                .into_response(),
            AuthEndpointError::StateSerialization(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InternalErrorResponse {
                    message: error.to_string(),
                }),
            )
                .into_response(),
        }
    }
}

#[derive(Serialize)]
struct InternalErrorResponse {
    message: String,
}
