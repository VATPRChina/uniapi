use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::adapter::vatsim_auth::{VatsimAuthError, generate_pkce};
use crate::jwt::JwtError;
use crate::repository::{
    device_authorization::{self as device_authorization_repository, NewDeviceAuthorization},
    session::{self as session_repository, RefreshSessionIssue, RefreshSessionRow},
    user::{self as user_repository, UserLoginRow},
};
use crate::services::Services;

#[derive(utoipa::OpenApi)]
#[openapi(paths(authorize, device_authorization, token))]
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
}

#[utoipa::path(get, path = "auth/authorize", tag = "Auth", responses((status = 302, description = "Redirects to authorization destination")))]
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

#[utoipa::path(post, path = "auth/device_authorization", tag = "Auth", responses((status = 200, description = "Successful response", body = serde_json::Value)))]
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

    device_authorization_repository::create(
        services.db(),
        NewDeviceAuthorization {
            device_code,
            user_code: &user_code,
            expires_at,
            client_id: &request.client_id,
        },
    )
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

async fn device_confirm(
    State(services): State<Services>,
    Query(query): Query<DeviceConfirmQuery>,
) -> Result<Response, AuthEndpointError> {
    if !query.confirm.unwrap_or(false) {
        return Ok(render_device_code_ui(query.user_code.as_deref()).into_response());
    }

    let code = normalize_user_code(query.user_code.as_deref());
    let Some(device_authz) =
        device_authorization_repository::find_by_user_code(services.db(), &code)
            .await
            .map_err(AuthEndpointError::Database)?
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
        HeaderValue::from_str(&cookie).map_err(AuthEndpointError::InvalidHeader)?,
    );
    Ok(response)
}

async fn login(
    State(services): State<Services>,
    Query(query): Query<LoginQuery>,
) -> Result<Response, AuthEndpointError> {
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
        HeaderValue::from_str(&cookie).map_err(AuthEndpointError::InvalidHeader)?,
    );
    Ok(response)
}

async fn vatsim_callback(
    State(services): State<Services>,
    headers: HeaderMap,
    Query(query): Query<VatsimCallbackQuery>,
) -> Result<Response, AuthEndpointError> {
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
    let auth_state: AuthenticationState = serde_json::from_str(&auth_state_cookie)
        .map_err(AuthEndpointError::StateDeserialization)?;

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
            let mut redirect = url::Url::parse(&redirect_uri)
                .map_err(|error| AuthEndpointError::InvalidRedirectUri(error.to_string()))?;
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
            device_authorization_repository::associate_user(services.db(), &user_code, user.id)
                .await
                .map_err(AuthEndpointError::Database)?;
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
            .map_err(AuthEndpointError::InvalidHeader)?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&delete_cookie_header(&auth_state_cookie_name))
            .map_err(AuthEndpointError::InvalidHeader)?,
    );
    Ok(response)
}

#[utoipa::path(post, path = "auth/token", tag = "Auth", responses((status = 200, description = "Successful response", body = serde_json::Value)))]
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

    let Some(device_authz) =
        device_authorization_repository::find_for_grant(services.db(), device_code)
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

    let Some(refresh) = find_session(&services, refresh_token).await? else {
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

    let Some(session) = find_session_by_code(&services, validated_code.code).await? else {
        return Ok(token_error("invalid_grant", "Invalid authorization code"));
    };
    clear_code(&services, validated_code.code).await?;

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
    user_id: uuid::Uuid,
    user_updated_at: chrono::DateTime<Utc>,
    client_id: &str,
    old_token: Option<Ulid>,
    create_code: bool,
) -> Result<RefreshSessionIssue, AuthEndpointError> {
    let expires_in = Utc::now() + Duration::days(services.jwt().refresh_expires_days());

    session_repository::issue_refresh_token(
        services.db(),
        user_id,
        user_updated_at,
        expires_in,
        client_id,
        old_token,
        create_code,
    )
    .await
    .map_err(AuthEndpointError::Database)
}

async fn find_session(
    services: &Services,
    token: Ulid,
) -> Result<Option<RefreshSessionRow>, AuthEndpointError> {
    session_repository::find(services.db(), token)
        .await
        .map_err(AuthEndpointError::Database)
}

async fn find_session_by_code(
    services: &Services,
    code: Ulid,
) -> Result<Option<RefreshSessionRow>, AuthEndpointError> {
    session_repository::find_by_code(services.db(), code)
        .await
        .map_err(AuthEndpointError::Database)
}

async fn clear_code(services: &Services, code: Ulid) -> Result<(), AuthEndpointError> {
    session_repository::clear_code(services.db(), code)
        .await
        .map_err(AuthEndpointError::Database)
}

async fn delete_device_authorization(
    services: &Services,
    device_code: Ulid,
) -> Result<(), AuthEndpointError> {
    device_authorization_repository::delete(services.db(), device_code)
        .await
        .map_err(AuthEndpointError::Database)
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

fn auth_state_cookie(
    state_id: &str,
    auth_state: &AuthenticationState,
) -> Result<String, AuthEndpointError> {
    let auth_state_cookie = percent_encode_cookie_value(
        &serde_json::to_string(auth_state).map_err(AuthEndpointError::StateSerialization)?,
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

async fn upsert_user(
    services: &Services,
    cid: &str,
    full_name: &str,
    email: &str,
) -> Result<UserLoginRow, AuthEndpointError> {
    user_repository::upsert_login(services.db(), cid, full_name, email)
        .await
        .map_err(AuthEndpointError::Database)
}

#[derive(Deserialize, utoipa::ToSchema)]
struct AuthorizeQuery {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: Option<String>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
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

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
enum AuthenticationStateType {
    #[serde(rename = "CODE")]
    Code,
    #[serde(rename = "DEVICE")]
    Device,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct DeviceConfirmQuery {
    user_code: Option<String>,
    confirm: Option<bool>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct LoginQuery {
    state: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct VatsimCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct DeviceAuthorizationRequest {
    client_id: String,
    #[allow(dead_code)]
    scope: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
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

#[derive(Deserialize, utoipa::ToSchema)]
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

#[derive(Serialize, utoipa::ToSchema)]
struct TokenResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    scope: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct TokenErrorResponse {
    error: &'static str,
    error_description: &'static str,
}

#[derive(Debug)]
enum AuthEndpointError {
    Database(sqlx::Error),
    InvalidRedirectUri(String),
    InvalidHeader(header::InvalidHeaderValue),
    Jwt(JwtError),
    StateDeserialization(serde_json::Error),
    StateSerialization(serde_json::Error),
    TokenError {
        error: &'static str,
        error_description: &'static str,
    },
    VatsimAuth(VatsimAuthError),
}

impl From<JwtError> for AuthEndpointError {
    fn from(error: JwtError) -> Self {
        Self::Jwt(error)
    }
}

impl From<VatsimAuthError> for AuthEndpointError {
    fn from(error: VatsimAuthError) -> Self {
        Self::VatsimAuth(error)
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
            AuthEndpointError::InvalidRedirectUri(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InternalErrorResponse {
                    message: format!("Invalid redirect URI: {error}"),
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
            AuthEndpointError::StateDeserialization(error) => (
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
            AuthEndpointError::VatsimAuth(error) => (
                StatusCode::BAD_GATEWAY,
                Json(InternalErrorResponse {
                    message: format!("VATSIM authentication failed: {error}"),
                }),
            )
                .into_response(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct InternalErrorResponse {
    message: String,
}
