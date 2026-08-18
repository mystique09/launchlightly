use std::{env, time::Duration};

use eyre::{Result, eyre};
use launchlightly_infra_postgresql::{connect_pool, migrate};
use launchlightly_web::WebConfig;
use tokio::{net::TcpListener, signal};
use topcoat::router::RouterService;
use tracing::info;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();
    setup_tracing()?;

    let database_url = required_env("DATABASE_URL")?;
    let secret = required_env("BETTER_AUTH_SECRET")?;
    let public_url = required_env("BETTER_AUTH_URL")?;

    let pool = connect_pool(&database_url).await?;
    info!("connected to PostgreSQL");
    migrate(&pool).await?;
    info!("database migrations are up to date");

    let app = launchlightly_web::router(pool.clone(), WebConfig { secret, public_url }).await?;
    let host = server_host()?;
    let port = server_port()?;
    let listener = TcpListener::bind((host.as_str(), port)).await?;
    let address = listener.local_addr()?;
    let service = RouterService::new(app).shutdown_timeout(SHUTDOWN_TIMEOUT);

    info!(%address, "server listening");
    let server_result = topcoat::serve_until(listener, service, shutdown_signal()).await;

    info!("closing database pool");
    pool.close().await;
    info!("database pool closed");
    server_result?;
    info!("server shutdown complete");
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

fn server_host() -> Result<String> {
    match env::var("HOST") {
        Ok(value) if value.trim().is_empty() => Err(eyre!("HOST must not be empty")),
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_HOST.to_owned()),
        Err(env::VarError::NotUnicode(_)) => Err(eyre!("HOST must be valid Unicode")),
    }
}

fn server_port() -> Result<u16> {
    match env::var("PORT") {
        Ok(value) => value
            .parse()
            .map_err(|error| eyre!("PORT must be a valid port number: {error}")),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_PORT),
        Err(env::VarError::NotUnicode(_)) => Err(eyre!("PORT must be valid Unicode")),
    }
}

fn setup_tracing() -> Result<()> {
    let crate_name = env!("CARGO_CRATE_NAME");
    let crate_version = env!("CARGO_PKG_VERSION");
    let default_filter = format!(
        "info,{crate_name}=debug,launchlightly_web=debug,launchlightly_infra_postgresql=debug"
    );
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    let subscriber =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_filter(filter));

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|error| eyre!("failed to install tracing subscriber: {error}"))?;
    LogTracer::init().map_err(|error| eyre!("failed to install log tracer: {error}"))?;

    tracing_log::log::info!("[LAUNCHLIGHTLY] {crate_name} v{crate_version}");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install the Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install the SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }

    info!("shutdown signal received; gracefully stopping server");
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
