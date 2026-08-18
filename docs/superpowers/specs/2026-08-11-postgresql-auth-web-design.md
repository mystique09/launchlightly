# PostgreSQL, Better Auth, and Topcoat Design

## Goal

Add the smallest production-shaped persistence and web foundation for LaunchLightly:

- one PostgreSQL infrastructure crate that owns connection setup, migrations, and an explicit super-admin seed;
- one web crate that keeps Topcoat as the application router and mounts Better Auth's Axum routes;
- one server binary that connects to PostgreSQL, runs migrations, builds the web application, and starts serving requests.

This work does not add flag persistence, demo data, organization tables, OAuth providers, or a product UI.

## Workspace layout

```text
crates/
  infra-postgresql/
    Cargo.toml
    migrations/
    src/
      lib.rs
      bin/
        migrate.rs
        seed.rs
  web/
    Cargo.toml
    src/lib.rs
bin/
  launchlight-server/
    Cargo.toml
    src/main.rs
```

The migration directory stays inside `infra-postgresql` because that crate owns the schema and embeds the migrations at compile time. This avoids a separate migrations-only crate and makes the migration path independent of the process working directory.

## Dependency alignment

Better Auth 0.10 uses SQLx 0.8. The workspace SQLx dependency will be aligned to SQLx 0.8 so the infrastructure crate and Better Auth can share the same `PgPool` type.

Topcoat remains the primary router. Its optional `tower` feature will be enabled so it can host the Better Auth Axum service. Better Auth will use only its Axum, PostgreSQL, and Rustls features; Axum 0.8 and Tower 0.5 are compatible with this bridge.

## PostgreSQL infrastructure crate

`launchlightly-infra-postgresql` will expose:

```rust
pub async fn connect_pool(database_url: &str) -> Result<PgPool, Error>;
pub async fn migrate(pool: &PgPool) -> Result<(), Error>;
pub async fn seed_super_admin(
    pool: &PgPool,
    config: SuperAdminSeed,
) -> Result<SeedOutcome, Error>;
```

`connect_pool` creates and verifies a bounded SQLx PostgreSQL pool. `migrate` runs migrations embedded with `sqlx::migrate!`, so callers do not need to locate migration files at runtime.

The initial migration creates Better Auth's core `users`, `accounts`, `sessions`, and `verifications` tables and their required indexes and constraints. Column names and types will follow Better Auth 0.10's PostgreSQL schema so its `SqlxAdapter` can use them directly.

The crate also provides small `migrate` and `seed` binaries. They read `DATABASE_URL` and return actionable errors without printing secrets. The server calls `migrate` during startup but never calls the seed automatically.

## Super-admin seed

The seed creates only one bootstrap user and no demo product data. Configuration comes from:

- `LAUNCHLIGHTLY_SUPER_ADMIN_EMAIL` (required)
- `LAUNCHLIGHTLY_SUPER_ADMIN_PASSWORD` (required)
- `LAUNCHLIGHTLY_SUPER_ADMIN_NAME` (optional, defaults to `Super Admin`)

Creation validates the email and the same 8–128 character password limits used by Better Auth, then uses Better Auth's public password-hashing primitive. Better Auth owns the credential format while the two database inserts remain in one transaction. The resulting user is email-verified and has role `super_admin`. The seed stores the same Better Auth-generated hash in the user's metadata and matching `accounts` credential record. It does not implement or repeat password hashing itself.

The operation is idempotent for the configured canonical email:

- if the seeded super admin does not exist, it is created once;
- if the matching seeded super admin already exists, the seed reports `AlreadyExists` and changes nothing;
- if a bootstrap super admin already exists under another email, the seed fails instead of creating a second one;
- if the email belongs to another kind of account, the seed fails instead of silently promoting or taking it over;
- rerunning the seed never resets the password or creates another user/account pair.

This guarantees the seed itself produces one super admin. It deliberately does not add a database rule forbidding administrators from assigning the role to other users later.

## Web crate

`launchlightly-web` owns application routing and Better Auth configuration. It builds a Better Auth instance from the shared `PgPool` and configures the initial plugins for:

- email and password sign-up/sign-in;
- session management;
- password management;
- account management;
- admin authorization using the `super_admin` role.

The crate exposes a router-building function that accepts the pool and web/auth configuration. A native Topcoat health endpoint lives at `/health`.

Better Auth's Axum router is first nested under `/api/auth` in an outer Axum router and supplied with its required Better Auth state. That complete Axum service is then mounted into Topcoat through `TowerRoute` at both:

- `/api/auth`
- `/api/auth/{*rest}`

Both routes are required because Topcoat's catch-all does not match the bare prefix. The Axum nesting is required because `TowerRoute` preserves the incoming URI while Better Auth expects the mount prefix to be stripped before matching its child routes.

This mounts Better Auth endpoints without replacing Topcoat. Authentication helpers for future native Topcoat handlers are outside this slice because Better Auth's Axum session extractor only works inside Axum handlers.

## Server startup

The server binary performs this sequence:

1. read and validate `DATABASE_URL`, Better Auth secret, and public base URL;
2. create the shared PostgreSQL pool;
3. run embedded migrations;
4. build the Topcoat router with the mounted Better Auth service;
5. start Topcoat.

Missing or invalid configuration stops startup with a clear error. The server does not seed data implicitly.

## Verification

Tests will cover the behavior at the boundaries instead of mocking SQLx:

1. run the embedded migrations twice against PostgreSQL and confirm both runs succeed;
2. run the super-admin seed twice and confirm there is exactly one matching user and one credential account;
3. authenticate the seeded credentials through Better Auth;
4. confirm the native Topcoat health endpoint responds;
5. confirm requests under both the bare and nested Better Auth paths reach the mounted Axum router.

Final verification will format, compile, lint, run the workspace tests, and exercise the database tests against a disposable PostgreSQL instance.
