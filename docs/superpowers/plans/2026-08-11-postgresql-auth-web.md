# PostgreSQL, Better Auth, and Topcoat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add embedded SQLx PostgreSQL migrations, an explicit idempotent super-admin seed, a shared pool interface, and a Topcoat web application that mounts Better Auth's Axum routes.

**Architecture:** `launchlightly-infra-postgresql` owns PostgreSQL connection setup, schema migrations, and bootstrap seeding behind three small functions. `launchlightly-web` owns Better Auth construction and mounts its completed Axum service into the primary Topcoat router. The server composes the two crates, migrates on startup, and never seeds implicitly.

**Tech Stack:** Rust 2024, Tokio, SQLx 0.8.6, PostgreSQL, Better Auth 0.10.0, Axum 0.8, Topcoat 0.6.0, Tower 0.5.

## Global Constraints

- Preserve all pre-existing user changes in `Cargo.toml`, `Cargo.lock`, `crates/launchlightly-domain/`, `.DS_Store`, and `CONTEXT.md`.
- Keep Topcoat 0.6.0 as the primary application router; enable its `tower` feature and do not remove it.
- Pin Better Auth to `=0.10.0` with `default-features = false` and features `axum`, `sqlx-postgres`, and `rustls`.
- Align the shared SQLx version to `0.8.6`; Better Auth 0.10 cannot consume a SQLx 0.9 `PgPool`.
- Keep migrations under `crates/infra-postgresql/migrations/` and embed them with `sqlx::migrate!`.
- The seed creates only one bootstrap super admin and no demo flags, organizations, projects, or environments.
- Seeding is explicit; server startup runs migrations but never runs the seed.
- Do not log or hard-code the super-admin password or Better Auth secret.
- Do not add a database constraint that prevents assigning `super_admin` to additional users later.
- Do not commit, stage, push, or rewrite unrelated files; the user retains git control.

---

### Task 1: Align dependencies and add the PostgreSQL pool and migrations

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/infra-postgresql/Cargo.toml`
- Create: `crates/infra-postgresql/migrations/20260811000000_create_better_auth_core.sql`
- Create: `crates/infra-postgresql/tests/postgresql.rs`
- Create: `crates/infra-postgresql/src/lib.rs`
- Create: `crates/infra-postgresql/build.rs`

**Interfaces:**
- Produces: `pub async fn connect_pool(database_url: &str) -> Result<PgPool, Error>`
- Produces: `pub async fn migrate(pool: &PgPool) -> Result<(), Error>`
- Produces: re-exported `sqlx::PgPool`
- Consumes: `DATABASE_URL` only in callers, never inside the library functions

- [ ] **Step 1: Add the minimal crate manifest and write the failing database test**

Add workspace dependencies for Axum 0.8, Chrono 0.4, SQLx 0.8.6 with `runtime-tokio-rustls`, `postgres`, `chrono`, `time`, `uuid`, `json`, `macros`, and `migrate`, Better Auth `=0.10.0` with its exact required features, and Topcoat 0.6.0 with `tower` while preserving its defaults.

Create the infrastructure manifest and an ignored real-PostgreSQL test whose production-breaking mutation is removing migration idempotency:

```rust
use launchlightly_infra_postgresql::migrate;
use sqlx::PgPool;

#[sqlx::test]
#[ignore = "requires PostgreSQL"]
async fn embedded_migrations_can_run_twice(pool: PgPool) {
    migrate(&pool).await.expect("first migration run");
    migrate(&pool).await.expect("second migration run");

    let tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' \
         AND table_name IN ('users', 'accounts', 'sessions', 'verifications')",
    )
    .fetch_one(&pool)
    .await
    .expect("count Better Auth tables");

    assert_eq!(tables, 4);
}
```

- [ ] **Step 2: Run the test target and verify RED**

Run:

```bash
cargo test -p launchlightly-infra-postgresql --test postgresql --no-run
```

Expected: compilation fails because `launchlightly_infra_postgresql::migrate` does not exist yet.

- [ ] **Step 3: Add the Better Auth 0.10 core schema and minimal pool implementation**

Copy the released Better Auth 0.10 core PostgreSQL table shapes for `users`, `sessions`, `accounts`, and `verifications`, including their foreign keys, unique constraints, timestamp defaults, and indexes. Do not add organization or optional-plugin tables.

Implement one embedded migrator and the two functions:

```rust
pub use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn connect_pool(database_url: &str) -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(Error::Connect)
}

pub async fn migrate(pool: &PgPool) -> Result<(), Error> {
    MIGRATOR.run(pool).await.map_err(Error::Migrate)
}
```

Use a focused `thiserror` enum that preserves SQLx and migration errors without exposing credentials. Add `build.rs` with `cargo:rerun-if-changed=migrations` so migration changes rebuild the embedded bundle.

- [ ] **Step 4: Verify GREEN at compile time**

Run:

```bash
cargo test -p launchlightly-infra-postgresql --test postgresql --no-run
```

Expected: the integration test compiles successfully. Its real execution is performed against disposable PostgreSQL in Task 5.

---

### Task 2: Add transactional, idempotent super-admin seeding and commands

**Files:**
- Modify: `crates/infra-postgresql/Cargo.toml`
- Modify: `crates/infra-postgresql/tests/postgresql.rs`
- Modify: `crates/infra-postgresql/src/lib.rs`
- Create: `crates/infra-postgresql/src/bin/migrate.rs`
- Create: `crates/infra-postgresql/src/bin/seed.rs`
- Modify: `justfile`

**Interfaces:**
- Produces: `pub struct SuperAdminSeed { email: String, password: String, name: String }`
- Produces: `pub fn SuperAdminSeed::new(email: impl Into<String>, password: impl Into<String>, name: Option<String>) -> Result<Self, Error>`
- Produces: `pub enum SeedOutcome { Created { user_id: String }, AlreadyExists { user_id: String } }`
- Produces: `pub async fn seed_super_admin(pool: &PgPool, seed: &SuperAdminSeed) -> Result<SeedOutcome, Error>`
- Consumes: Task 1's migrated Better Auth core schema

- [ ] **Step 1: Write failing seed behavior tests**

Add a second ignored SQLx test. The production-breaking mutations are removing the existing-email branch, changing the role/email verification values, dropping the credential account insert, or resetting the password on a second run.

```rust
#[sqlx::test]
#[ignore = "requires PostgreSQL"]
async fn super_admin_seed_creates_one_authenticatable_identity(pool: PgPool) {
    migrate(&pool).await.expect("migrate");
    let seed = SuperAdminSeed::new(
        "owner@example.com",
        "correct-horse-battery-staple",
        Some("LaunchLightly Owner".to_owned()),
    )
    .expect("valid seed");

    let first = seed_super_admin(&pool, &seed).await.expect("first seed");
    let second = seed_super_admin(&pool, &seed).await.expect("second seed");

    assert!(matches!(first, SeedOutcome::Created { .. }));
    assert!(matches!(second, SeedOutcome::AlreadyExists { .. }));

    let user_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email = $1 AND role = 'super_admin' AND email_verified",
    )
    .bind("owner@example.com")
    .fetch_one(&pool)
    .await
    .expect("count seeded users");
    let account_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM accounts a JOIN users u ON u.id = a.user_id \
         WHERE u.email = $1 AND a.provider_id = 'credential'",
    )
    .bind("owner@example.com")
    .fetch_one(&pool)
    .await
    .expect("count credential accounts");

    assert_eq!(user_count, 1);
    assert_eq!(account_count, 1);
}
```

Add a non-database unit test that rejects empty email/password inputs and defaults an omitted name to `Super Admin`.

- [ ] **Step 2: Run the focused tests and verify RED**

Run:

```bash
cargo test -p launchlightly-infra-postgresql --test postgresql --no-run
cargo test -p launchlightly-infra-postgresql --lib
```

Expected: compilation fails because `SuperAdminSeed`, `SeedOutcome`, and `seed_super_admin` do not exist.

- [ ] **Step 3: Implement the minimal seed transaction**

Canonicalize the email with `trim().to_ascii_lowercase()`. Validate it with the same Validator 0.19 email rule Better Auth uses, enforce Better Auth's 8–128 character password limits, and default a blank/missing name to `Super Admin`.

Inside one SQLx transaction:

1. acquire `pg_advisory_xact_lock(hashtext('launchlightly:seed-super-admin'))` so concurrent seed commands serialize;
2. select the configured email and existing super-admin rows `FOR UPDATE`;
3. return `AlreadyExists` without changing the password when the configured super admin already exists;
4. fail when a bootstrap super admin already exists under another email;
5. fail when the configured email is occupied by another user type;
6. generate Better Auth's password hash using `better_auth::types_mod::hash_password(None, password)`;
7. insert one UUID-v7-string user with `email_verified = TRUE`, role `super_admin`, and `metadata = {"password_hash": hash}`;
8. insert one matching credential account with the same hash in `accounts.password`;
9. commit and return `Created`.

Do not impose a global one-super-admin database index and do not modify an existing account.

- [ ] **Step 4: Add explicit migrate and seed commands**

`migrate.rs` reads required `DATABASE_URL`, connects, migrates, and exits. `seed.rs` reads required `DATABASE_URL`, `LAUNCHLIGHTLY_SUPER_ADMIN_EMAIL`, and `LAUNCHLIGHTLY_SUPER_ADMIN_PASSWORD`, plus optional `LAUNCHLIGHTLY_SUPER_ADMIN_NAME`, then connects, migrates, and seeds. Both install color-eyre and return errors; neither prints secrets.

Add these recipes:

```just
db-migrate:
    cargo run -p launchlightly-infra-postgresql --bin migrate

db-seed:
    cargo run -p launchlightly-infra-postgresql --bin seed
```

- [ ] **Step 5: Verify GREEN at compile and unit-test time**

Run:

```bash
cargo test -p launchlightly-infra-postgresql --lib
cargo test -p launchlightly-infra-postgresql --test postgresql --no-run
```

Expected: unit tests pass and the ignored real-PostgreSQL test compiles.

---

### Task 3: Build the Better Auth application behind Topcoat

**Files:**
- Create: `crates/web/Cargo.toml`
- Create: `crates/web/src/lib.rs`
- Create: `crates/web/tests/web.rs`

**Interfaces:**
- Produces: `pub struct WebConfig { pub secret: String, pub public_url: String }`
- Produces: `pub async fn router(pool: PgPool, config: WebConfig) -> Result<topcoat::router::Router, Error>`
- Consumes: Task 1's shared SQLx 0.8 `PgPool`
- Consumes: Task 2's Better Auth-compatible seeded user

- [ ] **Step 1: Write failing router boundary tests**

Create the manifest and write one in-process Topcoat test for native health plus one ignored SQLx test for the full authentication bridge. The mutations caught are dropping the native route, dropping either TowerRoute mount, failing to strip `/api/auth`, or configuring Better Auth without email/password support.

```rust
#[tokio::test]
async fn health_is_served_by_topcoat() {
    let pool = PgPoolOptions::new().connect_lazy("postgres://unused").expect("lazy pool");
    let app = router(pool, test_config()).await.expect("build router");
    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())
        .expect("health request");

    assert_eq!(app.handle(request).await.status(), StatusCode::NO_CONTENT);
}
```

The database test calls `migrate`, seeds `owner@example.com`, builds the Topcoat router, checks `GET /api/auth/ok` returns 200, then sends JSON to `POST /api/auth/sign-in/email` and asserts 200 plus a session token in the response. It uses `Router::handle` directly, real Topcoat routing, the real Axum bridge, the real Better Auth SQLx adapter, and real PostgreSQL.

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```bash
cargo test -p launchlightly-web --test web --no-run
```

Expected: compilation fails because `launchlightly_web::router` and `WebConfig` do not exist.

- [ ] **Step 3: Construct Better Auth with the approved plugins**

Create `AuthConfig` with the supplied secret, `base_url` equal to `<public_url-without-trailing-slash>/api/auth`, `base_path("/api/auth")`, and password minimum length 8. Build `BetterAuth<SqlxAdapter>` using the shared pool and:

```rust
EmailPasswordPlugin::new().enable_signup(true)
SessionManagementPlugin::new()
PasswordManagementPlugin::new().require_current_password(true)
AccountManagementPlugin::new()
AdminPlugin::new()
    .admin_role("super_admin")
    .default_user_role("user")
```

Wrap the result in `Arc` and keep construction private to the web module.

- [ ] **Step 4: Mount the state-bound Axum service into Topcoat**

Create the Axum service before Topcoat mounting so it has no missing state and strips the prefix:

```rust
let auth_service: axum::Router = axum::Router::new()
    .nest("/api/auth", auth.clone().axum_router())
    .with_state(auth);
```

Register native health and both bridge paths:

```rust
Router::builder()
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
    .build()
```

Return focused web construction errors without leaking the secret.

- [ ] **Step 5: Verify GREEN at compile and non-database test time**

Run:

```bash
cargo test -p launchlightly-web --test web
```

Expected: health passes and the PostgreSQL-dependent authentication test remains explicitly ignored until Task 5.

---

### Task 4: Wire the server startup path

**Files:**
- Modify: `bin/launchlight-server/Cargo.toml`
- Modify: `bin/launchlight-server/src/main.rs`

**Interfaces:**
- Consumes: `connect_pool`, `migrate`, `WebConfig`, and `router`
- Consumes environment: required `DATABASE_URL`, `BETTER_AUTH_SECRET`, and `BETTER_AUTH_URL`; Topcoat consumes optional `HOST` and `PORT`

- [ ] **Step 1: Add the server dependencies and minimal startup composition**

Replace Hello World with:

```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let database_url = required_env("DATABASE_URL")?;
    let secret = required_env("BETTER_AUTH_SECRET")?;
    let public_url = required_env("BETTER_AUTH_URL")?;

    let pool = connect_pool(&database_url).await?;
    migrate(&pool).await?;
    let app = launchlightly_web::router(pool, WebConfig { secret, public_url }).await?;

    topcoat::start(app).await?;
    Ok(())
}
```

Implement `required_env` locally so missing and non-Unicode values name the variable without printing any secret value. Do not seed during startup.

- [ ] **Step 2: Compile the server**

Run:

```bash
cargo check -p launchlight-server
```

Expected: successful server compilation with the Topcoat start path and no Hello World output.

---

### Task 5: Run real PostgreSQL verification and whole-workspace checks

**Files:**
- Modify only files required to fix verified defects found by these commands

**Interfaces:**
- Verifies all interfaces produced in Tasks 1-4

- [ ] **Step 1: Start one disposable PostgreSQL container**

Use a unique explicit container name, database, user, password, and host port. Wait for `pg_isready` inside the container, then export an admin `DATABASE_URL` that SQLx tests can use to create/drop isolated test databases. Do not touch unrelated containers.

- [ ] **Step 2: Run the ignored real-database tests**

Run:

```bash
cargo test -p launchlightly-infra-postgresql --test postgresql -- --ignored
cargo test -p launchlightly-web --test web -- --ignored
```

Expected: migrations run twice, seeding runs twice with one user and credential account, Better Auth signs in with the seeded password, and both Topcoat mount shapes respond.

- [ ] **Step 3: Exercise the migrate and seed commands twice**

Against a clean disposable database, run `just db-migrate` twice and `just db-seed` twice with non-production environment values. Query the database from inside the container to confirm one `super_admin` user and one credential account for the configured email.

- [ ] **Step 4: Run complete local verification**

Run fresh:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo tree -d
```

Confirm there is one SQLx 0.8 line in the dependency graph and no SQLx 0.9 line. Treat duplicate versions unrelated to this change as observations, not automatic scope expansion.

- [ ] **Step 5: Stop and remove only the disposable PostgreSQL container**

Remove the exact container created in Step 1. Report verification evidence and any remaining limitations. Do not stage, commit, or push.
