use kanva_server::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kanva_server=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env()?;
    let addr = kanva_server::start_server(config).await?;

    tracing::info!("Server running at {}", addr);

    // Keep the main task alive
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");

    Ok(())
}
