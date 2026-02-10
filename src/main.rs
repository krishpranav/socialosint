mod cli;
mod core;
mod detectors;
mod http;
mod instagram;
mod linkedin;
mod logger;
mod pwndb;
mod rate_limiter;
mod scraper;
mod telemetry;
mod tui;
mod twitter;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    telemetry::init();

    tui::display_banner();

    let args = cli::parse();

    core::run(args).await?;

    Ok(())
}
