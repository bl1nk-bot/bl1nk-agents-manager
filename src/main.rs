mod config;
mod mcp;
mod agents;
mod rate_limit;
mod system;
mod persistence;

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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Delegate a task to an agent (CLI interactive mode)
    Delegate {
        /// Type of task
        #[arg(short, long)]
        task_type: String,

        /// Prompt for the agent
        #[arg(short, long)]
        prompt: String,
    },
}

/// Application entry point that parses CLI arguments, initializes logging, loads configuration,
/// performs system discovery, initializes the MCP orchestrator, and runs the MCP server on stdio.
///
/// On success the function completes with `Ok(())`; on failure it returns an error from startup
/// operations (configuration loading, orchestrator initialization, or server runtime).
///
/// # Examples
///
/// ```no_run
/// // Run the compiled binary:
/// // $ bl1nk-agents-manager --config /path/to/config.toml
/// ```
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
    let report = match system::discovery::DiscoveryEngine::scan().await {
        Ok(report) => {
            if let Err(e) = system::discovery::DiscoveryEngine::save(&report).await {
                tracing::error!("❌ Failed to save discovery report: {}", e);
            }
            Some(report)
        }
        Err(e) => {
            tracing::error!("❌ System discovery failed: {}", e);
            None
        }
    };

    // Initialize the orchestrator
    let orchestrator = mcp::Orchestrator::new(config, report).await?;

    if let Some(cmd) = args.command {
        match cmd {
            Commands::Delegate { task_type, prompt } => {
                run_interactive_delegate(orchestrator, task_type, prompt).await?;
            }
        }
    } else {
        // Run the MCP server
        tracing::info!("🎧 Starting MCP server on stdio");
        tracing::info!("Host: {} | Port: {}", args.host, args.port);

        orchestrator.run_stdio().await?;
    }

    Ok(())
}

async fn run_interactive_delegate(
    orchestrator: mcp::Orchestrator,
    task_type: String,
    prompt: String,
) -> Result<()> {
    use std::io::{self, Write};
    use crate::mcp::DelegateTaskArgs;

    println!("🧠 Routing task: '{}'", task_type);
    
    let args = DelegateTaskArgs {
        task_type,
        prompt,
        agent_id: None,
        interactive: true,
        background: false,
        context: None,
    };

    let output = orchestrator.delegate_task_internal(args).await?;

    if let Some(proposal) = output.proposal {
        println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
        println!("║ 🤖 PROPOSED PLAN                                                          ║");
        println!("╠═══════════════════════════════════════════════════════════════════════════╣");
        println!("║ Agent:  {:<65} ║", proposal.agent_id);
        println!("║ Name:   {:<65} ║", proposal.agent_name);
        println!("║ Cost:   {:<65?} ║", proposal.cost_category);
        println!("║ Status: {:<65} ║", proposal.availability);
        println!("╠═══════════════════════════════════════════════════════════════════════════╣");
        println!("║ REASONING:                                                                ║");
        for line in textwrap::wrap(&proposal.reasoning, 73) {
            println!("║ {:<73} ║", line);
        }
        println!("╚═══════════════════════════════════════════════════════════════════════════╝");

        print!("\nProceed? [Y/n/modify]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        if choice == "n" || choice == "no" {
            println!("❌ Task cancelled.");
            return Ok(());
        } else if choice == "modify" || choice == "m" {
            println!("\nAvailable Capable Agents:");
            for (i, agent) in proposal.capable_agents.iter().enumerate() {
                println!("  {}. {}", i + 1, agent);
            }
            print!("\nEnter agent ID or number: ");
            io::stdout().flush()?;

            let mut agent_input = String::new();
            io::stdin().read_line(&mut agent_input)?;
            let agent_id = agent_input.trim();

            let confirmed_id = if let Ok(idx) = agent_id.parse::<usize>() {
                            idx.checked_sub(1)
                                .and_then(|i| proposal.capable_agents.get(i).cloned())
                                .unwrap_or_else(|| agent_id.to_string())
            } else {
                agent_id.to_string()
            };

            println!("🚀 Executing with agent: {}", confirmed_id);
            let final_output = orchestrator.approve_task_internal(output.task_id, Some(confirmed_id)).await?;
            if let Some(res) = final_output.result {
                println!("\n✅ Result:\n{}", res);
            }
        } else {
            println!("🚀 Executing plan...");
            let final_output = orchestrator.approve_task_internal(output.task_id, None).await?;
            if let Some(res) = final_output.result {
                println!("\n✅ Result:\n{}", res);
            }
        }
    } else {
        println!("✅ Task executed directly.");
        if let Some(res) = output.result {
            println!("\nResult:\n{}", res);
        }
    }

    Ok(())
}
