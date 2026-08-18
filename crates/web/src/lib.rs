use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{Method, header::CONTENT_LENGTH},
    middleware::Next,
    response::{IntoResponse, Response},
};
use better_auth::{
    AuthConfig, AuthSession, AuthUser, BetterAuth, CurrentSession, SessionOps,
    adapters::SqlxAdapter,
    handlers::AxumIntegration,
    plugins::{
        AdminPlugin, EmailPasswordPlugin, PasswordManagementPlugin, SessionManagementPlugin,
    },
};
use launchlightly_infra_postgresql::PgPool;
use thiserror::Error;
use topcoat::router::{Methods, Path, Router, StatusCode, route, tower::TowerRoute};
use url::{Host, Url};

mod auth_ui;

const MAX_PASSWORD_BYTES: usize = 128;
const MAX_AUTH_REQUEST_BYTES: usize = 16 * 1024;

pub struct WebConfig {
    pub secret: String,
    pub public_url: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "public URL must be an absolute HTTPS origin (plain HTTP is allowed only for localhost or a loopback address)"
    )]
    InvalidPublicUrl,

    #[error("could not construct Better Auth: {0}")]
    Build(#[source] better_auth::AuthError),
}

#[route(GET "/health")]
async fn health() -> topcoat::Result<StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}

async fn enforce_change_password_limit(request: Request, next: Next) -> Response {
    if request.method() != Method::POST || request.uri().path() != "/api/auth/change-password" {
        return next.run(request).await;
    }

    let (parts, body) = request.into_parts();
    let body = match to_bytes(body, MAX_AUTH_REQUEST_BYTES).await {
        Ok(body) => body,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let password_is_too_long = serde_json::from_slice::<serde_json::Value>(&body)
        .ok()
        .and_then(|value| value.get("newPassword")?.as_str().map(str::len))
        .is_some_and(|length| length > MAX_PASSWORD_BYTES);

    if password_is_too_long {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "message": "Password must be at most 128 bytes"
            })),
        )
            .into_response();
    }

    next.run(Request::from_parts(parts, Body::from(body))).await
}

async fn canonicalize_email(request: Request, next: Next) -> Response {
    const EMAIL_PATHS: [&str; 3] = [
        "/api/auth/sign-up/email",
        "/api/auth/sign-in/email",
        "/api/auth/admin/create-user",
    ];

    if request.method() != Method::POST || !EMAIL_PATHS.contains(&request.uri().path()) {
        return next.run(request).await;
    }

    let (mut parts, body) = request.into_parts();
    let body = match to_bytes(body, MAX_AUTH_REQUEST_BYTES).await {
        Ok(body) => body,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let mut payload = match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(payload) => payload,
        Err(_) => return next.run(Request::from_parts(parts, Body::from(body))).await,
    };
    if let Some(email) = payload.get_mut("email")
        && let Some(value) = email.as_str()
    {
        *email = serde_json::Value::String(value.trim().to_ascii_lowercase());
    }
    let body = serde_json::to_vec(&payload).expect("JSON value is serializable");
    parts.headers.remove(CONTENT_LENGTH);

    next.run(Request::from_parts(parts, Body::from(body))).await
}

async fn current_session(session: CurrentSession<SqlxAdapter>) -> Response {
    (
        [("cache-control", "no-store")],
        axum::Json(serde_json::json!({
            "user": {
                "name": session.user.name(),
                "email": session.user.email(),
            },
            "session": {
                "id": session.session.id(),
            },
        })),
    )
        .into_response()
}

async fn list_sessions(
    State(auth): State<Arc<BetterAuth<SqlxAdapter>>>,
    current: CurrentSession<SqlxAdapter>,
) -> Response {
    let sessions = match auth.database().get_user_sessions(current.user.id()).await {
        Ok(sessions) => sessions,
        Err(error) => return IntoResponse::into_response(error),
    };
    let sessions = sessions
        .into_iter()
        .filter(|session| session.active() && session.expires_at() > chrono::Utc::now())
        .map(|session| {
            serde_json::json!({
                "id": session.id(),
                "createdAt": session.created_at(),
                "expiresAt": session.expires_at(),
            })
        })
        .collect::<Vec<_>>();

    ([("cache-control", "no-store")], axum::Json(sessions)).into_response()
}

pub async fn router(pool: PgPool, config: WebConfig) -> Result<Router, Error> {
    let public_url = Url::parse(config.public_url.trim()).map_err(|_| Error::InvalidPublicUrl)?;
    let is_loopback = match public_url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    };
    if !matches!(public_url.scheme(), "http" | "https")
        || public_url.host_str().is_none()
        || (public_url.scheme() == "http" && !is_loopback)
        || !matches!(public_url.path(), "" | "/")
        || public_url.query().is_some()
        || public_url.fragment().is_some()
    {
        return Err(Error::InvalidPublicUrl);
    }
    let base_url = format!("{}/api/auth", public_url.origin().ascii_serialization());
    let adapter: SqlxAdapter = SqlxAdapter::from_pool(pool);
    let auth = Arc::new(
        BetterAuth::<SqlxAdapter>::new(
            AuthConfig::new(config.secret)
                .base_url(base_url)
                .base_path("/api/auth")
                .password_min_length(8)
                .disabled_path("/sign-in/username")
                .disabled_path("/get-session")
                .disabled_path("/list-sessions")
                .disabled_path("/update-user")
                .disabled_path("/change-email")
                .disabled_path("/delete-user")
                .disabled_path("/delete-user/callback")
                .disabled_path("/forget-password")
                .disabled_path("/reset-password")
                .disabled_path("/reset-password/{token}")
                .disabled_path("/set-password"),
        )
        .database(adapter)
        .plugin(EmailPasswordPlugin::new().enable_signup(true))
        .plugin(SessionManagementPlugin::new())
        .plugin(PasswordManagementPlugin::new().require_current_password(true))
        .plugin(
            AdminPlugin::new()
                .admin_role("super_admin")
                .default_user_role("user"),
        )
        .build()
        .await
        .map_err(Error::Build)?,
    );

    let auth_child = axum::Router::new()
        .route(
            "/",
            axum::routing::get(|| async { axum::http::StatusCode::NO_CONTENT }),
        )
        .route("/get-session", axum::routing::get(current_session))
        .route("/list-sessions", axum::routing::get(list_sessions))
        .merge(auth.clone().axum_router());
    let auth_service: axum::Router = axum::Router::new()
        .nest("/api/auth", auth_child)
        .with_state(auth)
        .layer(axum::middleware::from_fn(canonicalize_email))
        .layer(axum::middleware::from_fn(enforce_change_password_limit));

    Ok(Router::builder()
        .layout(auth_ui::app_layout)
        .page(auth_ui::index_page)
        .page(auth_ui::sign_in_page)
        .page(auth_ui::sign_up_page)
        .page(auth_ui::security_page)
        .route(health)
        .route(TowerRoute::new(
            Methods::Any,
            Path::new("/api/auth"),
            auth_service.clone(),
        ))
        .route(TowerRoute::new(
            Methods::Any,
            Path::new("/api/auth/{*rest}"),
            auth_service,
        ))
        .build())
}
