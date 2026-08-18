use launchlightly_infra_postgresql::{SuperAdminSeed, migrate, seed_super_admin};
use launchlightly_web::{WebConfig, router};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use topcoat::router::{Body, Method, Router, StatusCode, request::Request, to_bytes};

fn test_config() -> WebConfig {
    WebConfig {
        secret: "test-secret-key-that-is-at-least-32-characters-long".to_owned(),
        public_url: "http://localhost:3000".to_owned(),
    }
}

async fn get_html(app: &Router, path: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .expect("page request");
    let response = app.handle(request).await;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read page response");

    (
        status,
        String::from_utf8(body.to_vec()).expect("page response is UTF-8"),
    )
}

#[tokio::test]
async fn health_is_served_by_topcoat() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");
    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())
        .expect("health request");

    assert_eq!(app.handle(request).await.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn sign_in_page_exposes_the_email_password_flow() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");

    let (status, html) = get_html(&app, "/sign-in").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("rel=\"icon\""));
    assert!(html.contains("<title>Sign in · LaunchLightly</title>"));
    assert!(html.contains("<h1>Sign in to LaunchLightly</h1>"));
    assert!(html.contains("id=\"sign-in-form\""));
    assert!(html.contains("autocomplete=\"email\""));
    assert!(html.contains("autocomplete=\"current-password\""));
    assert!(html.contains("href=\"/sign-up\""));
}

#[tokio::test]
async fn sign_up_page_exposes_the_supported_registration_fields() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");

    let (status, html) = get_html(&app, "/sign-up").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<h1>Create your account</h1>"));
    assert!(html.contains("id=\"sign-up-form\""));
    assert!(html.contains("autocomplete=\"name\""));
    assert!(html.contains("autocomplete=\"new-password\""));
    assert!(html.contains("id=\"confirm-password\""));
    assert!(html.contains("minlength=\"8\""));
    assert!(html.contains("maxlength=\"128\""));
    assert!(html.contains("href=\"/sign-in\""));
}

#[tokio::test]
async fn security_page_exposes_session_and_password_controls() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");

    let (status, html) = get_html(&app, "/account/security").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<title>Account security · LaunchLightly</title>"));
    assert!(html.contains("<h1>Account security</h1>"));
    assert!(html.contains("id=\"change-password-form\""));
    assert!(html.contains("id=\"account-username\""));
    assert!(html.contains("autocomplete=\"username\""));
    assert!(html.contains("id=\"sessions-list\""));
    assert!(html.contains("id=\"revoke-other-sessions\""));
    assert!(html.contains("id=\"sign-out\""));
    assert!(html.contains("id=\"sign-out-status\""));
    assert!(html.contains("sign out every other session"));
}

#[tokio::test]
async fn auth_ui_does_not_offer_unconfigured_authentication_methods() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");

    let (sign_in_status, sign_in) = get_html(&app, "/sign-in").await;
    let (sign_up_status, sign_up) = get_html(&app, "/sign-up").await;
    assert_eq!(sign_in_status, StatusCode::OK);
    assert_eq!(sign_up_status, StatusCode::OK);
    let entry_pages = format!("{sign_in}{sign_up}").to_ascii_lowercase();

    for unsupported in ["google", "github", "magic link", "forgot password"] {
        assert!(
            !entry_pages.contains(unsupported),
            "unconfigured auth method must not be rendered: {unsupported}"
        );
    }
}

#[tokio::test]
async fn unsupported_authentication_api_routes_are_not_mounted() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");

    for (method, path) in [
        (Method::POST, "/api/auth/sign-in/username"),
        (Method::GET, "/api/auth/list-accounts"),
        (Method::POST, "/api/auth/update-user"),
        (Method::POST, "/api/auth/change-email"),
        (Method::POST, "/api/auth/delete-user"),
        (Method::GET, "/api/auth/delete-user/callback"),
    ] {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .header("origin", "http://localhost:3000")
            .header("host", "localhost:3000")
            .body(Body::from("{}"))
            .expect("unsupported authentication request");

        assert_eq!(app.handle(request).await.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn password_recovery_routes_are_not_mounted_without_delivery() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");

    for (method, path) in [
        (Method::POST, "/api/auth/forget-password"),
        (Method::POST, "/api/auth/reset-password"),
        (Method::GET, "/api/auth/reset-password/token"),
    ] {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .header("origin", "http://localhost:3000")
            .header("host", "localhost:3000")
            .body(Body::from("{}"))
            .expect("password recovery request");

        assert_eq!(app.handle(request).await.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn change_password_enforces_the_signup_password_byte_limit() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");
    let password = "🐎".repeat(64);
    assert!(password.chars().count() <= 128);
    assert!(password.len() > 128);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/change-password")
        .header("content-type", "application/json")
        .header("origin", "http://localhost:3000")
        .header("host", "localhost:3000")
        .body(Body::from(
            json!({
                "currentPassword": "correct-horse-battery-staple",
                "newPassword": password,
                "revokeOtherSessions": true
            })
            .to_string(),
        ))
        .expect("change-password request");

    let response = app.handle(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn root_page_routes_visitors_by_their_session() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");

    let (status, html) = get_html(&app, "/").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("data-session-gate"));
    assert!(html.contains("/api/auth/get-session"));
    assert!(html.contains("/account/security"));
    assert!(html.contains("/sign-in"));
    assert!(html.contains("id=\"session-gate-retry\""));
}

#[tokio::test]
async fn malformed_public_url_is_rejected() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let result = router(
        pool,
        WebConfig {
            secret: "test-secret-key-that-is-at-least-32-characters-long".to_owned(),
            public_url: "not-a-url".to_owned(),
        },
    )
    .await;

    let error = match result {
        Ok(_) => panic!("malformed public URL must fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("public URL"));
}

#[tokio::test]
async fn plain_http_is_rejected_outside_loopback_development() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let result = router(
        pool,
        WebConfig {
            secret: "test-secret-key-that-is-at-least-32-characters-long".to_owned(),
            public_url: "http://launchlightly.example".to_owned(),
        },
    )
    .await;

    let error = match result {
        Ok(_) => panic!("plain HTTP on a remote host must fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("HTTPS"));
}

#[tokio::test]
async fn invalid_auth_config_preserves_the_safe_validation_reason() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .expect("lazy pool");
    let result = router(
        pool,
        WebConfig {
            secret: "too-short".to_owned(),
            public_url: "http://localhost:3000".to_owned(),
        },
    )
    .await;

    let error = match result {
        Ok(_) => panic!("invalid Better Auth configuration must fail"),
        Err(error) => error,
    };
    let message = error.to_string();
    let message = message.to_ascii_lowercase();
    assert!(message.contains("secret"));
    assert!(message.contains("32"));
}

#[sqlx::test]
#[ignore = "requires PostgreSQL"]
async fn auth_routes_mount_and_seeded_password_signs_in(pool: PgPool) {
    migrate(&pool).await.expect("migrate");
    let seed = SuperAdminSeed::new(
        "owner@example.com",
        "correct-horse-battery-staple",
        Some("LaunchLightly Owner".to_owned()),
    )
    .expect("valid seed");
    seed_super_admin(&pool, &seed).await.expect("seed");

    let app = router(pool.clone(), test_config())
        .await
        .expect("build router");

    let auth_root = Request::builder()
        .method(Method::GET)
        .uri("/api/auth")
        .body(Body::empty())
        .expect("auth root request");
    assert_eq!(app.handle(auth_root).await.status(), StatusCode::NO_CONTENT);

    let ok = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/ok")
        .body(Body::empty())
        .expect("auth ok request");
    assert_eq!(app.handle(ok).await.status(), StatusCode::OK);

    let sign_in = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/sign-in/email")
        .header("content-type", "application/json")
        .header("origin", "http://localhost:3000")
        .header("host", "localhost:3000")
        .body(Body::from(
            json!({
                "email": "Owner@Example.COM",
                "password": "correct-horse-battery-staple"
            })
            .to_string(),
        ))
        .expect("sign-in request");
    let response = app.handle(sign_in).await;
    assert_eq!(response.status(), StatusCode::OK);

    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .expect("session cookie")
        .to_owned();

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read sign-in response");
    let payload: Value = serde_json::from_slice(&body).expect("valid sign-in response JSON");
    assert!(
        payload["token"]
            .as_str()
            .is_some_and(|token| !token.is_empty())
    );

    let current_session = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/get-session")
        .header("cookie", &cookie)
        .body(Body::empty())
        .expect("current-session request");
    let response = app.handle(current_session).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read current-session response");
    let current_session: Value =
        serde_json::from_slice(&body).expect("valid current-session response JSON");
    assert!(current_session["session"]["id"].is_string());
    assert!(current_session["session"].get("token").is_none());

    let user_id: String = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind("owner@example.com")
        .fetch_one(&pool)
        .await
        .expect("seeded user id");
    sqlx::query(
        "INSERT INTO sessions (id, expires_at, token, user_id, active) \
         VALUES ('expired-test-session', NOW() - INTERVAL '1 hour', \
         'expired-test-token', $1, TRUE)",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("insert expired active session");

    let list_sessions = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/list-sessions")
        .header("cookie", cookie)
        .body(Body::empty())
        .expect("list-sessions request");
    let response = app.handle(list_sessions).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read list-sessions response");
    let sessions: Value = serde_json::from_slice(&body).expect("valid list-sessions response JSON");
    let sessions = sessions.as_array().expect("session response is an array");
    assert!(!sessions.is_empty());
    assert!(
        sessions
            .iter()
            .all(|session| session.get("token").is_none())
    );
    assert!(
        sessions
            .iter()
            .all(|session| session["id"] != "expired-test-session")
    );
}
