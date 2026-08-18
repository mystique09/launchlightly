use std::env;

use eyre::{Result, eyre};
use launchlightly_infra_postgresql::{connect_pool, migrate};
use launchlightly_web::WebConfig;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let database_url = required_env("DATABASE_URL")?;
    let secret = required_env("BETTER_AUTH_SECRET")?;
    let public_url = required_env("BETTER_AUTH_URL")?;

    let pool = connect_pool(&database_url).await?;
    migrate(&pool).await?;
    let app = launchlightly_web::router(pool, WebConfig { secret, public_url }).await?;

    topcoat::start(app).await?;
    Ok(())
}

fn required_env(name: &str) -> Result<String> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => Err(eyre!("{name} must not be empty")),
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(eyre!("{name} must be set")),
        Err(env::VarError::NotUnicode(_)) => Err(eyre!("{name} must be valid Unicode")),
    }
}

#[cfg(test)]
mod tests {
    use super::required_env;

    #[test]
    fn missing_environment_error_names_the_variable() {
        let key = "LAUNCHLIGHTLY_TEST_VARIABLE_THAT_MUST_NOT_EXIST";
        let error = required_env(key).expect_err("test variable must be missing");

        assert!(error.to_string().contains(key));
    }
}
