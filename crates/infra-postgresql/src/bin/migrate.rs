use launchlightly_infra_postgresql::{connect_pool, migrate};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let database_url = required_env("DATABASE_URL")?;
    let pool = connect_pool(&database_url).await?;
    migrate(&pool).await?;

    Ok(())
}

fn required_env(name: &str) -> eyre::Result<String> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(eyre::eyre!("{name} must be set")),
    }
}
