// src/main.rs
use anyhow::Result;
use bl1nk_agents_manager::*;
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
    /// Search for keywords in the registry
    Search {
        /// The keyword query
        #[arg(short, long)]
        query: String,
        /// Perform fuzzy search
        #[arg(short, long)]
        fuzzy: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level)))
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("🚀 Starting BL1NK Agents Manager (v{})", env!("CARGO_PKG_VERSION"));

    // 1. Load configuration
    let config = if let Some(path) = args.config {
        config::Config::load(path)?
    } else {
        config::Config::load_default()?
    };

    tracing::info!("✅ Loaded {} agents", config.agents.len());

    // 2. Perform system discovery
    let report = system::discovery::DiscoveryEngine::scan().await.ok();

    // 3. Initialize the orchestrator
    let orchestrator = mcp::Orchestrator::new(config, report).await?;

    // 4. Handle Subcommands
    if let Some(cmd) = args.command {
        match cmd {
            Commands::Delegate { task_type, prompt } => {
                run_interactive_delegate(orchestrator, task_type, prompt).await?;
            }
            Commands::Search { query, fuzzy } => {
                tracing::info!("🔍 Searching agents for: '{}'", query);
                let results = orchestrator.registry_service.search_agents(&query, fuzzy);
                if results.is_empty() {
                    println!("No results found.");
                } else {
                    for res in results {
                        println!("- Agent: {}, Score: {:.2}", res.id, res.score);
                    }
                }
            }
        }
    } else {
        // Run the MCP server
        tracing::info!("🎧 Starting MCP server on stdio");
        orchestrator.run_stdio().await?;
    }

    Ok(())
}

async fn run_interactive_delegate(orchestrator: mcp::Orchestrator, task_type: String, prompt: String) -> Result<()> {
    use crate::mcp::DelegateTaskArgs;
    use std::io::{self, Write};

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
        println!("\n🤖 PROPOSED PLAN:");
        println!("   Agent: {}", proposal.agent_name);
        println!("   Reason: {}", proposal.reasoning);

        print!("\nProceed? [Y/n]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "n" {
            let res = orchestrator.approve_task_internal(output.task_id, None).await?;
            if let Some(r) = res.result {
                println!("\n✅ Result:\n{}", r);
            }
        }
    } else if let Some(r) = output.result {
        println!("\n✅ Result:\n{}", r);
    }

    Ok(())
}
