mod config;
mod mcp;
mod agents;
mod rate_limit;
mod system;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

/// BL1NK Agents Manager - Intelligent MCP/ACP Orchestrator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Server host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Run in daemon mode (background)
    #[arg(short, long)]
    daemon: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&args.log_level))
        )
        .with_writer(std::io::stderr) // Force logs to stderr
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("🚀 Starting BL1NK Agents Manager");
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = if let Some(config_path) = args.config {
        tracing::info!("Loading config from: {:?}", config_path);
        config::Config::load(config_path)?
    } else {
        tracing::info!("Loading config from default locations");
        config::Config::load_default()?
    };

    tracing::info!("✅ Loaded {} agents", config.agents.len());
    tracing::info!("✅ Loaded {} routing rules", config.routing.rules.len());

    // Log routing tier
    tracing::info!("📊 Routing tier: {:?}", config.routing.tier);

    // Perform system discovery
    tracing::info!("🔍 Scanning system resources...");
    match system::discovery::DiscoveryEngine::scan().await {
        Ok(report) => {
            if let Err(e) = system::discovery::DiscoveryEngine::save(&report).await {
                tracing::error!("❌ Failed to save discovery report: {}", e);
            }
        }
        Err(e) => tracing::error!("❌ System discovery failed: {}", e),
    }

    // Initialize the orchestrator
    let orchestrator = mcp::Orchestrator::new(config).await?;

    // Run the MCP server
    tracing::info!("🎧 Starting MCP server on stdio");
    tracing::info!("Host: {} | Port: {}", args.host, args.port);
    
    orchestrator.run_stdio().await?;

    Ok(())
}
