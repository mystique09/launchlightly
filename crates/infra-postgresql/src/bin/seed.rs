use launchlightly_infra_postgresql::{SuperAdminSeed, connect_pool, migrate, seed_super_admin};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let database_url = required_env("DATABASE_URL")?;
    let email = required_env("LAUNCHLIGHTLY_SUPER_ADMIN_EMAIL")?;
    let password = required_env("LAUNCHLIGHTLY_SUPER_ADMIN_PASSWORD")?;
    let name = std::env::var("LAUNCHLIGHTLY_SUPER_ADMIN_NAME").ok();
    let seed = SuperAdminSeed::new(email, password, name)?;

    let pool = connect_pool(&database_url).await?;
    migrate(&pool).await?;
    seed_super_admin(&pool, &seed).await?;

    Ok(())
}

fn required_env(name: &str) -> eyre::Result<String> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(eyre::eyre!("{name} must be set")),
    }
}
