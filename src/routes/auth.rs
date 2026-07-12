use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Duration, Utc};
use rand::RngExt;
use serde::Serialize;
use std::collections::BTreeSet;
use tracing::instrument;
use ulid::Ulid;
use uuid::Uuid;

use crate::adapter::vatsim_auth::{VatsimAuthError, generate_pkce};
use crate::dto::*;
use crate::jwt::JwtError;
use crate::repository::auth::device_authorization::DeviceAuthorizationRepositoryExt;
use crate::repository::auth::device_authorization::NewDeviceAuthorization;
use crate::repository::auth::session::{RefreshSessionIssue, RefreshSessionRow};
use crate::repository::auth::session::{SessionRepositoryExt, SessionTransactionExt};
use crate::repository::auth::user::UserLoginRow;
use crate::repository::auth::user::UserRepositoryExt;
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(authorize, device_authorization, token, unsafe_assume_user))]
pub(crate) struct ApiDoc;

const USER_CODE_ALPHABET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";

pub fn build_auth_routes() -> Router<Services> {
    Router::new()
        .route("/authorize", get(authorize))
        .route("/device_authorization", post(device_authorization))
        .route("/device", get(device_confirm))
        .route("/login", get(login))
        .route("/callback/vatsim", get(vatsim_callback))
        .route("/token", post(token))
        .route("/__unsafe_assume_user", post(unsafe_assume_user))
}

#[utoipa::path(
    get,
    path = "auth/authorize",
    tag = "Auth",
    params(
        ("response_type" = String, Query, description = "OAuth response type. Must be code."),
        ("client_id" = String, Query, description = "OAuth client identifier"),
        ("redirect_uri" = String, Query, description = "Client redirect URI"),
        ("state" = Option<String>, Query, description = "Opaque client state")
    ),
    responses((status = 302, description = "Redirects to authorization destination"))
)]
#[instrument(skip(services, query), fields(client_id = %query.client_id))]
async fn authorize(
    State(services): State<Services>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response, AuthUserError> {
    if query.response_type != "code" {
        return Err(AuthUserError::invalid_request(
            "Invalid request",
            "Invalid response type",
            "The response_type parameter is invalid.",
            None,
        ));
    }
    if !services
        .jwt()
        .check_client_redirect(&query.client_id, &query.redirect_uri)
    {
        return Err(AuthUserError::invalid_request(
            "Invalid client",
            "Client validation failed",
            "The client or redirect URI is invalid.",
            None,
        ));
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
        &serde_json::to_string(&auth_state).map_err(AuthUserError::internal)?,
    );
    let cookie = format!(
        "auth-{state_id}={auth_state_cookie}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=600"
    );

    let mut response =
        Redirect::temporary(&format!("/auth/login?state={state_id}")).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(AuthUserError::internal)?,
    );
    Ok(response)
}

#[utoipa::path(
    post,
    path = "auth/device_authorization",
    tag = "Auth",
    request_body(content = DeviceAuthorizationRequest, content_type = "application/x-www-form-urlencoded"),
    responses(
        (status = 200, description = "Successful response", body = DeviceAuthorizationResponse),
        (status = 400, description = "OAuth error response", body = AuthApiErrorBody),
        (status = 401, description = "OAuth client authentication error", body = AuthApiErrorBody)
    )
)]
#[instrument(skip(services, headers, request), fields(client_id = %request.client_id))]
async fn device_authorization(
    State(services): State<Services>,
    headers: HeaderMap,
    Form(request): Form<DeviceAuthorizationRequest>,
) -> Result<Json<DeviceAuthorizationResponse>, AuthApiError> {
    tracing::info!(client_id = %request.client_id, "creating device authorization");
    if !services.jwt().check_client_exists(&request.client_id) {
        return Err(AuthApiError::invalid_client("client_id not found"));
    }

    let device_code = Ulid::new();
    let user_code = random_user_code();
    let expires_at = Utc::now() + Duration::seconds(services.jwt().device_authz_expires_seconds());

    services
        .db()
        .create_device_authorization(NewDeviceAuthorization {
            device_code,
            user_code: &user_code,
            expires_at,
            client_id: &request.client_id,
        })
        .await
        .map_err(AuthApiError::from)?;

    let verification_uri = format!("{}/auth/device", request_origin(&headers));
    let formatted_user_code = format!("{}-{}", &user_code[..4], &user_code[4..]);

    tracing::info!(%device_code, client_id = %request.client_id, "device authorization issued");

    Ok(Json(DeviceAuthorizationResponse {
        device_code: device_code.to_string(),
        user_code: formatted_user_code,
        verification_uri: verification_uri.clone(),
        verification_uri_complete: Some(format!("{verification_uri}?user_code={user_code}")),
        expires_in: (expires_at - Utc::now()).num_seconds().max(0) as u32,
        interval: None,
    }))
}

#[instrument(skip(services, query))]
async fn device_confirm(
    State(services): State<Services>,
    Query(query): Query<DeviceConfirmQuery>,
) -> Result<Response, AuthUserError> {
    if !query.confirm.unwrap_or(false) {
        return Ok(render_device_code_ui(query.user_code.as_deref()).into_response());
    }

    let code = normalize_user_code(query.user_code.as_deref());
    let Some(device_authz) = services
        .db()
        .find_device_authorization_by_user_code(&code)
        .await
        .map_err(AuthUserError::from)?
    else {
        return Ok(render_callback_ui(
            "Error",
            "Invalid code",
            "The code provided is not found in our records.",
            Some("/auth/device"),
        ));
    };

    if device_authz.user_id.is_some() {
        delete_device_authorization(&services, Ulid::from(device_authz.device_code)).await?;
        return Ok(render_callback_ui(
            "Error",
            "Invalid code",
            "The code provided has already been used.",
            Some("/auth/device"),
        ));
    }
    if device_authz.expires_at < Utc::now() {
        delete_device_authorization(&services, Ulid::from(device_authz.device_code)).await?;
        return Ok(render_callback_ui(
            "Error",
            "Invalid code",
            "The code provided is expired.",
            Some("/auth/device"),
        ));
    }

    let state = Ulid::new().to_string();
    let auth_state = AuthenticationState {
        auth_type: AuthenticationStateType::Device,
        client_id: None,
        redirect_uri: None,
        user_code: Some(device_authz.user_code),
        state: None,
    };
    let cookie = auth_state_cookie(&state, &auth_state)?;

    let mut response = Redirect::temporary(&format!("/auth/login?state={state}")).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(AuthUserError::internal)?,
    );
    Ok(response)
}

#[instrument(skip(services, query))]
async fn login(
    State(services): State<Services>,
    Query(query): Query<LoginQuery>,
) -> Result<Response, AuthUserError> {
    let (challenge, verifier) = generate_pkce();
    let url = services
        .vatsim_auth()
        .authorization_url(&query.state, &challenge)?;
    let cookie_value = percent_encode_cookie_value(&verifier);
    let cookie = format!(
        "auth-{}-code_verifier={cookie_value}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=600",
        query.state
    );

    let mut response = Redirect::temporary(&url).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(AuthUserError::internal)?,
    );
    Ok(response)
}

#[instrument(skip(services, headers, query))]
async fn vatsim_callback(
    State(services): State<Services>,
    headers: HeaderMap,
    Query(query): Query<VatsimCallbackQuery>,
) -> Result<Response, AuthUserError> {
    let Some(code) = query.code else {
        if query.error.as_deref() == Some("access_denied") {
            return Ok(render_callback_ui(
                "Error",
                "Access denied",
                "You have denied the request.",
                Some("/auth/login"),
            ));
        }
        return Ok(render_callback_ui(
            "Error",
            "Missing code",
            "Are you coming from VATSIM Connect?",
            None,
        ));
    };
    let Some(state) = query.state else {
        return Ok(render_callback_ui(
            "Invalid state",
            "Authentication state not found.",
            "Please check if cookie is enabled for your browser and try again.",
            None,
        ));
    };

    let cookies = cookies_from_headers(&headers);
    let verifier_cookie_name = format!("auth-{state}-code_verifier");
    let verifier = cookies
        .iter()
        .find_map(|(name, value)| (name == &verifier_cookie_name).then(|| value.clone()))
        .unwrap_or_default();

    let token = services.vatsim_auth().get_token(&code, &verifier).await?;
    let vatsim_user = services.vatsim_auth().get_user(&token.access_token).await?;
    let user = upsert_user(
        &services,
        &vatsim_user.data.cid,
        &vatsim_user.data.personal.full_name,
        &vatsim_user.data.personal.email,
    )
    .await?;

    let auth_state_cookie_name = format!("auth-{state}");
    let Some(auth_state_cookie) = cookies
        .iter()
        .find_map(|(name, value)| (name == &auth_state_cookie_name).then(|| value.clone()))
    else {
        return Ok(render_callback_ui(
            "Invalid state",
            &format!("Hello {}", vatsim_user.data.cid),
            "Authentication state not found. Please check if cookie is enabled for your browser and try again.",
            None,
        ));
    };
    let auth_state: AuthenticationState =
        serde_json::from_str(&auth_state_cookie).map_err(AuthUserError::internal)?;

    let mut response = match auth_state.auth_type {
        AuthenticationStateType::Code => {
            let Some(client_id) = auth_state.client_id else {
                return Ok(render_callback_ui(
                    "Error",
                    "Internal error",
                    "Please try again later.",
                    Some("/auth/login"),
                ));
            };
            let Some(redirect_uri) = auth_state.redirect_uri else {
                return Ok(render_callback_ui(
                    "Error",
                    "Internal error",
                    "Please try again later.",
                    Some("/auth/login"),
                ));
            };
            let refresh =
                issue_refresh_token(&services, user.id, user.updated_at, &client_id, None, true)
                    .await?;
            let Some(authz_code) = refresh.code else {
                return Ok(render_callback_ui(
                    "Error",
                    "Internal error",
                    "Please try again later.",
                    Some("/auth/login"),
                ));
            };
            let auth_code =
                services
                    .jwt()
                    .issue_auth_code(authz_code, &client_id, &redirect_uri)?;
            let mut redirect = url::Url::parse(&redirect_uri).map_err(AuthUserError::internal)?;
            redirect.query_pairs_mut().append_pair("code", &auth_code);
            if let Some(state) = auth_state.state {
                redirect.query_pairs_mut().append_pair("state", &state);
            }
            Redirect::temporary(redirect.as_str()).into_response()
        }
        AuthenticationStateType::Device => {
            let Some(user_code) = auth_state.user_code else {
                return Ok(render_callback_ui(
                    "Error",
                    "Internal error",
                    "Please try again later.",
                    Some("/auth/device"),
                ));
            };
            tracing::info!(%user_code, user_id = %user.id, "associating device authorization with user");
            services
                .db()
                .associate_device_authorization_user(&user_code, user.id)
                .await
                .map_err(AuthUserError::from)?;
            render_callback_ui(
                "Welcome",
                &format!("Hello {}", vatsim_user.data.cid),
                "Login successful, please return to your device.",
                None,
            )
        }
    };

    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&delete_cookie_header(&verifier_cookie_name))
            .map_err(AuthUserError::internal)?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&delete_cookie_header(&auth_state_cookie_name))
            .map_err(AuthUserError::internal)?,
    );
    Ok(response)
}

#[utoipa::path(
    post,
    path = "auth/token",
    tag = "Auth",
    request_body(content = AccessTokenRequest, content_type = "application/x-www-form-urlencoded"),
    responses(
        (status = 200, description = "Successful response", body = TokenResponse),
        (status = 400, description = "OAuth error response", body = AuthApiErrorBody),
        (status = 401, description = "OAuth client authentication error", body = AuthApiErrorBody)
    )
)]
#[instrument(skip(services, request), fields(grant_type = %request.grant_type, client_id = %request.client_id))]
async fn token(
    State(services): State<Services>,
    Form(request): Form<AccessTokenRequest>,
) -> Result<Json<TokenResponse>, AuthApiError> {
    match request.grant_type.as_str() {
        "urn:ietf:params:oauth:grant-type:device_code" => {
            device_code_grant(services, request).await
        }
        "refresh_token" => refresh_token_grant(services, request).await,
        "authorization_code" => authorization_code_grant(services, request).await,
        "client_credentials" => client_credentials_grant(services, request),
        _ => Err(AuthApiError::new(
            OAuthErrorCode::UnsupportedGrantType,
            "The authorization grant type is not supported by the authorization server.",
        )),
    }
}

#[utoipa::path(
    post,
    path = "auth/__unsafe_assume_user",
    tag = "Auth",
    security(("oauth2" = [])),
    request_body = UnsafeAssumeUserRequest,
    responses(
        (status = 200, description = "Successful response", body = TokenResponse),
        (status = 400, description = "OAuth error response", body = AuthApiErrorBody),
        (status = 401, description = "OAuth client authentication error", body = AuthApiErrorBody)
    )
)]
#[instrument(skip(services, headers, request), fields(cid = %request.cid))]
async fn unsafe_assume_user(
    State(services): State<Services>,
    headers: HeaderMap,
    Json(request): Json<UnsafeAssumeUserRequest>,
) -> Result<Json<TokenResponse>, AuthApiError> {
    let client_id = authenticated_api_client(&services, &headers)?;
    if !services
        .jwt()
        .check_client_can_unsafe_assume_user(&client_id)
    {
        return Err(AuthApiError::unauthorized_client(
            "client is not allowed to assume users",
        ));
    }

    let cid = request.cid.trim();
    if cid.is_empty() {
        return Err(AuthApiError::invalid_request("cid is required"));
    }

    let id = match request.id {
        Some(id) => id
            .parse::<Ulid>()
            .map(Uuid::from)
            .map_err(|_| AuthApiError::invalid_request("id must be a ULID"))?,
        None => Uuid::from(Ulid::new()),
    };
    let full_name = request
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(cid);
    let roles = request
        .roles
        .unwrap_or_default()
        .into_iter()
        .map(|role| role.trim().to_string())
        .filter(|role| !role.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let user = services
        .db()
        .upsert_user_assumed_user(id, cid, full_name, request.email.as_deref(), roles)
        .await
        .map_err(AuthApiError::from)?;

    tracing::info!(assumed_user_id = %user.id, client_id = %client_id, "issuing unsafe assumed user token");
    let refresh =
        issue_refresh_token(&services, user.id, user.updated_at, &client_id, None, false).await?;
    let access_token =
        services
            .jwt()
            .issue_access_token(user.id, user.updated_at, refresh.token, &client_id)?;

    Ok(Json(TokenResponse {
        access_token: access_token.token,
        token_type: "Bearer",
        expires_in: access_token.expires_in,
        refresh_token: Some(refresh.token.to_string()),
        scope: access_token.scope,
    }))
}

#[instrument(skip(services, request), fields(client_id = %request.client_id))]
async fn device_code_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Json<TokenResponse>, AuthApiError> {
    let Some(device_code) = parse_required_ulid(&request.device_code, "Missing device code")?
    else {
        return Err(AuthApiError::invalid_grant("Device code not found"));
    };

    let Some(device_authz) = services
        .db()
        .find_device_authorization_for_grant(device_code)
        .await
        .map_err(AuthApiError::from)?
    else {
        return Err(AuthApiError::invalid_grant("Device code not found"));
    };

    if device_authz.expires_at < Utc::now() {
        delete_device_authorization(&services, device_code).await?;
        return Err(AuthApiError::expired_token("Device code expired"));
    }
    let Some(user_id) = device_authz.user_id else {
        return Err(AuthApiError::authorization_pending(
            "User has not yet authorized this device",
        ));
    };
    if device_authz.client_id != request.client_id {
        return Err(AuthApiError::invalid_client("Client ID mismatch"));
    }
    let Some(user_updated_at) = device_authz.user_updated_at else {
        return Err(AuthApiError::invalid_grant("Device code not found"));
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
    tracing::info!(%user_id, client_id = %request.client_id, "issuing tokens from device code grant");
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
    }))
}

#[instrument(skip(services, request), fields(client_id = %request.client_id))]
async fn refresh_token_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Json<TokenResponse>, AuthApiError> {
    let Some(refresh_token) = parse_required_ulid(&request.refresh_token, "Missing refresh token")?
    else {
        return Err(AuthApiError::invalid_grant("Refresh token not found"));
    };

    let Some(refresh) = find_session(&services, refresh_token).await? else {
        return Err(AuthApiError::invalid_grant("Refresh token not found"));
    };
    if refresh.user_updated_at != refresh.updated_at {
        return Err(AuthApiError::invalid_grant(
            "Refresh token has been revoked",
        ));
    }
    if refresh.expires_in < Utc::now() {
        return Err(AuthApiError::invalid_grant("Refresh token expired"));
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
    tracing::info!(user_id = %refresh.user_id, client_id = %refresh.client_id, "issuing tokens from refresh token grant");
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
    }))
}

#[instrument(skip(services, request), fields(client_id = %request.client_id))]
async fn authorization_code_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Json<TokenResponse>, AuthApiError> {
    if request.client_id.is_empty() || request.code.is_empty() {
        return Err(AuthApiError::invalid_request("Missing client_id or code"));
    }

    let Some(validated_code) = services
        .jwt()
        .validate_auth_code(&request.code, &request.client_id)?
    else {
        return Err(AuthApiError::invalid_grant("Invalid authorization code"));
    };

    let Some(session) = find_session_by_code(&services, validated_code.code).await? else {
        return Err(AuthApiError::invalid_grant("Invalid authorization code"));
    };
    clear_code(&services, validated_code.code).await?;

    tracing::info!(user_id = %session.user_id, client_id = %validated_code.client_id, "issuing tokens from authorization code grant");
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
    }))
}

#[instrument(skip(services, request), fields(client_id = %request.client_id))]
fn client_credentials_grant(
    services: Services,
    request: AccessTokenRequest,
) -> Result<Json<TokenResponse>, AuthApiError> {
    if request.client_id.is_empty() || request.client_secret.is_empty() {
        return Err(AuthApiError::invalid_request(
            "Missing client_id or client_secret",
        ));
    }
    if !services
        .jwt()
        .check_client_secret(&request.client_id, &request.client_secret)
    {
        return Err(AuthApiError::invalid_client(
            "client_id or client_secret is invalid",
        ));
    }

    tracing::info!(client_id = %request.client_id, "issuing client credentials access token");
    let access_token = services
        .jwt()
        .issue_client_access_token(&request.client_id)?;
    Ok(Json(TokenResponse {
        access_token: access_token.token,
        token_type: "Bearer",
        expires_in: access_token.expires_in,
        refresh_token: None,
        scope: access_token.scope,
    }))
}

#[instrument(skip(services, user_updated_at), fields(user_id = %user_id, client_id = %client_id, create_code = create_code))]
async fn issue_refresh_token(
    services: &Services,
    user_id: uuid::Uuid,
    user_updated_at: chrono::DateTime<Utc>,
    client_id: &str,
    old_token: Option<Ulid>,
    create_code: bool,
) -> Result<RefreshSessionIssue, sqlx::Error> {
    let expires_in = Utc::now() + Duration::days(services.jwt().refresh_expires_days());

    tracing::info!(%user_id, %client_id, old_token = ?old_token, create_code, "issuing refresh token");
    let mut transaction = services.db().begin().await?;
    let issue = transaction
        .issue_session_refresh_token(
            user_id,
            user_updated_at,
            expires_in,
            client_id,
            old_token,
            create_code,
        )
        .await?;
    transaction.commit().await?;
    Ok(issue)
}

#[instrument(skip(services), fields(token = %token))]
async fn find_session(
    services: &Services,
    token: Ulid,
) -> Result<Option<RefreshSessionRow>, AuthApiError> {
    services
        .db()
        .find_session(token)
        .await
        .map_err(AuthApiError::from)
}

#[instrument(skip(services), fields(code = %code))]
async fn find_session_by_code(
    services: &Services,
    code: Ulid,
) -> Result<Option<RefreshSessionRow>, AuthApiError> {
    services
        .db()
        .find_session_by_code(code)
        .await
        .map_err(AuthApiError::from)
}

#[instrument(skip(services), fields(code = %code))]
async fn clear_code(services: &Services, code: Ulid) -> Result<(), AuthApiError> {
    services
        .db()
        .clear_session_code(code)
        .await
        .map_err(AuthApiError::from)
}

#[instrument(skip(services), fields(device_code = %device_code))]
async fn delete_device_authorization(
    services: &Services,
    device_code: Ulid,
) -> Result<(), sqlx::Error> {
    tracing::info!(%device_code, "deleting device authorization");
    services.db().delete_device_authorization(device_code).await
}

fn parse_required_ulid(value: &str, missing: &'static str) -> Result<Option<Ulid>, AuthApiError> {
    if value.trim().is_empty() {
        return Err(AuthApiError::invalid_request(missing));
    }

    Ok(value.parse::<Ulid>().ok())
}

#[instrument(skip(services, headers))]
fn authenticated_api_client(
    services: &Services,
    headers: &HeaderMap,
) -> Result<String, AuthApiError> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| AuthApiError::invalid_client("missing bearer token"))?;
    let token = services.jwt().validate_access_token_claims(token)?;
    if token.subject.parse::<Ulid>() != token.client_id.parse::<Ulid>() {
        return Err(AuthApiError::unauthorized_client(
            "user tokens are not allowed for this endpoint",
        ));
    }

    Ok(token.subject)
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

fn auth_state_cookie(
    state_id: &str,
    auth_state: &AuthenticationState,
) -> Result<String, AuthUserError> {
    let auth_state_cookie = percent_encode_cookie_value(
        &serde_json::to_string(auth_state).map_err(AuthUserError::internal)?,
    );
    Ok(format!(
        "auth-{state_id}={auth_state_cookie}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=600"
    ))
}

fn delete_cookie_header(name: &str) -> String {
    format!("{name}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0")
}

fn cookies_from_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|cookies| cookies.split(';'))
        .filter_map(|cookie| {
            let (name, value) = cookie.trim().split_once('=')?;
            Some((name.to_string(), percent_decode_cookie_value(value)))
        })
        .collect()
}

fn percent_decode_cookie_value(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16)
        {
            decoded.push(hex);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn normalize_user_code(user_code: Option<&str>) -> String {
    user_code
        .unwrap_or_default()
        .to_uppercase()
        .chars()
        .filter(|char| USER_CODE_ALPHABET.contains(&(*char as u8)))
        .collect()
}

fn render_device_code_ui(user_code: Option<&str>) -> Response {
    let code = normalize_user_code(user_code);
    if code.len() != 8 {
        let error = user_code
            .map(|value| {
                format!(
                    r#"<h2 class="text-xl text-red-700">The provided code <span class="font-mono">{}</span> is invalid.</h2>"#,
                    html_escape(&value.to_uppercase())
                )
            })
            .unwrap_or_default();
        return html_response(format!(
            r#"<!doctype html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><link href="/style.css" rel="stylesheet"></head>
<body class="grid h-screen place-items-center bg-slate-100">
<form class="container max-w-2xl bg-white shadow-2xl rounded-xl p-6 flex flex-col gap-y-2">
<h1 class="text-4xl font-bold">Device Code Login</h1>
{error}
<h2 class="text-2xl">Please type your code as on your device.</h2>
<input class="my-4 border-2 rounded-md text-3xl font-bold text-center uppercase" type="text" name="user_code" required placeholder="BCDF-GHJK">
<button type="submit" class="font-bold bg-sky-700 text-white px-2 py-1 rounded-md shadow-md hover:bg-sky-500">Proceed</button>
</form></body></html>"#
        ));
    }

    html_response(format!(
        r#"<!doctype html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><link href="/style.css" rel="stylesheet"></head>
<body class="grid h-screen place-items-center bg-slate-100">
<form class="container max-w-2xl bg-white shadow-2xl rounded-xl p-6 flex flex-col gap-y-2">
<h1 class="text-4xl font-bold">Device Code Login</h1>
<h2 class="text-2xl">Please check if the following code matches your device.</h2>
<div><div class="text-3xl font-bold w-fit mx-auto my-4">{}-{}</div></div>
<input type="hidden" name="user_code" value="{}">
<input type="hidden" name="confirm" value="true">
<button type="submit" class="font-bold bg-sky-700 text-white px-2 py-1 rounded-md shadow-md hover:bg-sky-500">Proceed</button>
</form></body></html>"#,
        &code[..4],
        &code[4..],
        html_escape(user_code.unwrap_or_default())
    ))
}

fn render_callback_ui(
    title: &str,
    message: &str,
    description: &str,
    redirect: Option<&str>,
) -> Response {
    let action = redirect
        .map(|href| {
            format!(
                r#"<a href="{}" class="font-bold bg-sky-700 text-white px-2 py-1 rounded-md shadow-md hover:bg-sky-500">Retry</a>"#,
                html_escape(href)
            )
        })
        .unwrap_or_default();
    html_response(format!(
        r#"<!doctype html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><link href="/style.css" rel="stylesheet"></head>
<body class="grid h-screen place-items-center bg-slate-100">
<div class="container max-w-2xl bg-white shadow-2xl rounded-xl p-6 space-y-2">
<h1 class="text-4xl font-bold">{}</h1>
<h2 class="text-2xl">{}</h2>
<p>{}</p>
<div>{action}</div>
</div></body></html>"#,
        html_escape(title),
        html_escape(message),
        html_escape(description)
    ))
}

fn html_response(content: String) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        content,
    )
        .into_response()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[instrument(skip(services, full_name, email), fields(cid = %cid))]
async fn upsert_user(
    services: &Services,
    cid: &str,
    full_name: &str,
    email: &str,
) -> Result<UserLoginRow, AuthUserError> {
    tracing::info!(%cid, "upserting authenticated user login");
    services
        .db()
        .upsert_user_login(cid, full_name, email)
        .await
        .map_err(AuthUserError::from)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum OAuthErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    InvalidScope,
    UnauthorizedClient,
    UnsupportedGrantType,
    AuthorizationPending,
    ExpiredToken,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
struct AuthApiErrorBody {
    error: OAuthErrorCode,
    error_description: String,
}

#[derive(Debug)]
struct AuthApiError {
    status: StatusCode,
    body: AuthApiErrorBody,
}

impl AuthApiError {
    fn new(error: OAuthErrorCode, error_description: impl Into<String>) -> Self {
        let status = match error {
            OAuthErrorCode::InvalidClient => StatusCode::UNAUTHORIZED,
            _ => StatusCode::BAD_REQUEST,
        };

        Self {
            status,
            body: AuthApiErrorBody {
                error,
                error_description: error_description.into(),
            },
        }
    }

    fn invalid_request(error_description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidRequest, error_description)
    }

    fn invalid_client(error_description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidClient, error_description)
    }

    fn invalid_grant(error_description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidGrant, error_description)
    }

    fn unauthorized_client(error_description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::UnauthorizedClient, error_description)
    }

    fn authorization_pending(error_description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::AuthorizationPending, error_description)
    }

    fn expired_token(error_description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::ExpiredToken, error_description)
    }
}

impl From<JwtError> for AuthApiError {
    fn from(_: JwtError) -> Self {
        Self::invalid_grant("Invalid token")
    }
}

impl From<sqlx::Error> for AuthApiError {
    fn from(error: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: AuthApiErrorBody {
                error: OAuthErrorCode::InvalidRequest,
                error_description: format!("The request could not be processed: {error}"),
            },
        }
    }
}

impl IntoResponse for AuthApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            [
                (header::CONTENT_TYPE, "application/json"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Json(self.body),
        )
            .into_response()
    }
}

#[derive(Debug)]
struct AuthUserError {
    title: String,
    message: String,
    description: String,
    redirect: Option<&'static str>,
}

impl AuthUserError {
    fn invalid_request(
        title: impl Into<String>,
        message: impl Into<String>,
        description: impl Into<String>,
        redirect: Option<&'static str>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            description: description.into(),
            redirect,
        }
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(error = %error, "auth user-facing route failed");
        Self {
            title: "Error".to_string(),
            message: "Internal error".to_string(),
            description: "Please try again later.".to_string(),
            redirect: None,
        }
    }
}

impl From<JwtError> for AuthUserError {
    fn from(error: JwtError) -> Self {
        Self::internal(error)
    }
}

impl From<VatsimAuthError> for AuthUserError {
    fn from(error: VatsimAuthError) -> Self {
        tracing::error!(error = %error, "VATSIM authentication failed");
        Self {
            title: "Error".to_string(),
            message: "Authentication failed".to_string(),
            description: "Please try again later.".to_string(),
            redirect: Some("/auth/login"),
        }
    }
}

impl From<sqlx::Error> for AuthUserError {
    fn from(error: sqlx::Error) -> Self {
        Self::internal(error)
    }
}

impl IntoResponse for AuthUserError {
    fn into_response(self) -> Response {
        render_callback_ui(&self.title, &self.message, &self.description, self.redirect)
    }
}
