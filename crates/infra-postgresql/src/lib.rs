use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPoolOptions;
use thiserror::Error;
use uuid::Uuid;
use validator::ValidateEmail;

pub use sqlx::PgPool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to connect to PostgreSQL")]
    Connect(#[source] sqlx::Error),

    #[error("failed to run PostgreSQL migrations")]
    Migrate(#[source] MigrateError),

    #[error("super-admin email must not be empty")]
    EmptySuperAdminEmail,

    #[error("super-admin password must not be empty")]
    EmptySuperAdminPassword,

    #[error("super-admin email must be a valid email address")]
    InvalidSuperAdminEmail,

    #[error("super-admin password must be between 8 and 128 characters")]
    InvalidSuperAdminPassword,

    #[error("a non-super-admin user already uses the configured email")]
    SuperAdminEmailOccupied,

    #[error("a bootstrap super-admin already exists with a different email")]
    SuperAdminAlreadyExists,

    #[error("failed to hash super-admin password")]
    PasswordHash(#[source] better_auth::AuthError),

    #[error("failed to seed super-admin")]
    Seed(#[source] sqlx::Error),
}

#[derive(Clone, PartialEq, Eq)]
pub struct SuperAdminSeed {
    email: String,
    password: String,
    name: String,
}

impl std::fmt::Debug for SuperAdminSeed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SuperAdminSeed")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .field("name", &self.name)
            .finish()
    }
}

impl SuperAdminSeed {
    pub fn new(
        email: impl Into<String>,
        password: impl Into<String>,
        name: Option<String>,
    ) -> Result<Self, Error> {
        let email = email.into().trim().to_ascii_lowercase();
        let password = password.into();
        let name = name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "Super Admin".to_owned());

        if email.is_empty() {
            return Err(Error::EmptySuperAdminEmail);
        }
        if !email.validate_email() {
            return Err(Error::InvalidSuperAdminEmail);
        }
        if password.is_empty() {
            return Err(Error::EmptySuperAdminPassword);
        }
        if !(8..=128).contains(&password.len()) {
            return Err(Error::InvalidSuperAdminPassword);
        }

        Ok(Self {
            email,
            password,
            name,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedOutcome {
    Created { user_id: String },
    AlreadyExists { user_id: String },
}

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

pub async fn seed_super_admin(pool: &PgPool, seed: &SuperAdminSeed) -> Result<SeedOutcome, Error> {
    let mut transaction = pool.begin().await.map_err(Error::Seed)?;

    sqlx::query("SELECT pg_advisory_xact_lock(hashtext('launchlightly:seed-super-admin'))")
        .execute(&mut *transaction)
        .await
        .map_err(Error::Seed)?;

    let existing_seed = sqlx::query_scalar::<_, String>(
        "SELECT id FROM users \
         WHERE role = 'super_admin' AND LOWER(email) = $1 \
         FOR UPDATE",
    )
    .bind(&seed.email)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(Error::Seed)?;

    if let Some(user_id) = existing_seed {
        transaction.commit().await.map_err(Error::Seed)?;
        return Ok(SeedOutcome::AlreadyExists { user_id });
    }

    let another_super_admin = sqlx::query_scalar::<_, String>(
        "SELECT id FROM users WHERE role = 'super_admin' LIMIT 1 FOR UPDATE",
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(Error::Seed)?;

    if another_super_admin.is_some() {
        return Err(Error::SuperAdminAlreadyExists);
    }

    let occupied_email =
        sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE LOWER(email) = $1 FOR UPDATE")
            .bind(&seed.email)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(Error::Seed)?;

    if occupied_email.is_some() {
        return Err(Error::SuperAdminEmailOccupied);
    }

    let password_hash = better_auth::types_mod::hash_password(None, &seed.password)
        .await
        .map_err(Error::PasswordHash)?;
    let user_id = Uuid::now_v7().to_string();
    let metadata = serde_json::json!({ "password_hash": password_hash });

    sqlx::query(
        "INSERT INTO users (id, name, email, email_verified, role, metadata) \
         VALUES ($1, $2, $3, TRUE, 'super_admin', $4)",
    )
    .bind(&user_id)
    .bind(&seed.name)
    .bind(&seed.email)
    .bind(metadata)
    .execute(&mut *transaction)
    .await
    .map_err(Error::Seed)?;

    sqlx::query(
        "INSERT INTO accounts (id, account_id, provider_id, user_id, password) \
         VALUES ($1, $2, 'credential', $3, $4)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(&user_id)
    .bind(&user_id)
    .bind(&password_hash)
    .execute(&mut *transaction)
    .await
    .map_err(Error::Seed)?;

    transaction.commit().await.map_err(Error::Seed)?;

    Ok(SeedOutcome::Created { user_id })
}

#[cfg(test)]
mod tests {
    use super::SuperAdminSeed;

    #[test]
    fn super_admin_seed_defaults_blank_or_missing_name() {
        let missing =
            SuperAdminSeed::new("owner@example.com", "password", None).expect("valid seed");
        let blank = SuperAdminSeed::new("owner@example.com", "password", Some(" ".to_owned()))
            .expect("valid seed");

        assert_eq!(missing.name, "Super Admin");
        assert_eq!(blank.name, "Super Admin");
    }

    #[test]
    fn super_admin_seed_debug_output_redacts_password() {
        let seed = SuperAdminSeed::new("owner@example.com", "correct-horse-battery-staple", None)
            .expect("valid seed");

        assert!(!format!("{seed:?}").contains("correct-horse-battery-staple"));
    }
}
