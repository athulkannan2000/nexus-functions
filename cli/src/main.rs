use clap::{Parser, Subcommand};
use colored::Colorize;
use nexus_core::{AppState, NexusConfig, Server};
use nexus_event_fabric::NatsClient;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Parser)]
#[command(name = "nexus")]
#[command(author, version, about = "Nexus Functions - Event-Driven Serverless Platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start local development server
    Dev {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        /// Path to nexus.yaml configuration
        #[arg(short, long, default_value = "nexus.yaml")]
        config: String,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Replay an event by ID
    Replay {
        /// Event ID to replay
        event_id: String,
    },
    
    /// Create a new function from template
    New {
        /// Function name
        name: String,
        
        /// Programming language (rust, assemblyscript)
        #[arg(short, long, default_value = "rust")]
        lang: String,
    },
    
    /// View recent events
    Events {
        /// Number of events to show
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },
    
    /// View function logs
    Logs {
        /// Function name
        function: String,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev { port, config, verbose } => {
            println!("{}", "🚀 Nexus Functions - Development Server".bright_cyan().bold());
            println!();
            
            if verbose {
                std::env::set_var("RUST_LOG", "debug");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }
            
            // Initialize observability
            nexus_observability::init_tracing()?;
            
            // Load configuration
            let config_path = Path::new(&config);
            let nexus_config = if config_path.exists() {
                println!("{} Loading configuration from {}...", "✓".green(), config);
                match NexusConfig::from_file(config_path) {
                    Ok(cfg) => {
                        println!("{} Loaded {} function(s)", "✓".green(), cfg.functions.len());
                        for func in &cfg.functions {
                            println!("  {} {}", "→".cyan(), func.name);
                        }
                        cfg
                    }
                    Err(e) => {
                        eprintln!("{} Failed to load config: {}", "✗".red(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{} No nexus.yaml found, using default configuration", "⚠".yellow());
                NexusConfig {
                    version: "v1".to_string(),
                    functions: vec![],
                }
            };
            
            // Initialize NATS client
            println!("{} Starting embedded NATS JetStream...", "✓".green());
            let nats_client = Arc::new(RwLock::new(NatsClient::new()));
            
            // Connect to NATS (embedded mode - will connect to local NATS if available)
            let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
            
            {
                let mut client = nats_client.write().await;
                match client.connect_with_retry(&nats_url, 5).await {
                    Ok(_) => {
                        println!("{} Connected to NATS at {}", "✓".green(), nats_url);
                        
                        // Create default stream
                        if let Err(e) = client.create_stream("events").await {
                            println!("{} Warning: Failed to create stream: {}", "⚠".yellow(), e);
                            println!("{} Event replay may not be available", "⚠".yellow());
                        } else {
                            println!("{} JetStream stream 'events' ready", "✓".green());
                        }
                    }
                    Err(e) => {
                        println!("{} Could not connect to NATS: {}", "⚠".yellow(), e);
                        println!("{} Continuing without event persistence (events will not be replayed)", "⚠".yellow());
                    }
                }
            }
            
            println!("{} Serving HTTP on http://localhost:{}...", "✓".green(), port);
            println!();
            println!("{}", "Ready to receive events! 🎉".bright_green());
            println!("Press Ctrl+C to stop");
            println!();
            
            // Create application state
            let app_state = match AppState::new(nexus_config, nats_client) {
                Ok(state) => state,
                Err(e) => {
                    eprintln!("{} Failed to create application state: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            };
            
            // Start the server
            let server = Server::new(port, app_state);
            
            tokio::select! {
                result = server.run() => {
                    if let Err(e) = result {
                        eprintln!("{} Server error: {}", "✗".red(), e);
                        std::process::exit(1);
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!();
                    println!("{} Shutting down gracefully...", "✓".yellow());
                }
            }
        }
        
        Commands::Replay { event_id } => {
            println!("{} Replaying event {}...", "⟳".cyan(), event_id);
            // TODO: Implement replay logic
            println!("{} Replay not yet implemented", "⚠".yellow());
        }
        
        Commands::New { name, lang } => {
            println!("{} Creating new {} function: {}...", "✨".bright_cyan(), lang, name);
            // TODO: Implement template generation
            println!("{} Template generation not yet implemented", "⚠".yellow());
        }
        
        Commands::Events { limit } => {
            println!("{} Showing last {} events...", "📋".cyan(), limit);
            // TODO: Implement event listing
            println!("{} Event listing not yet implemented", "⚠".yellow());
        }
        
        Commands::Logs { function, follow } => {
            println!("{} Viewing logs for function: {}...", "📜".cyan(), function);
            if follow {
                println!("{} Following logs (Ctrl+C to stop)...", "👀".cyan());
            }
            // TODO: Implement log viewing
            println!("{} Log viewing not yet implemented", "⚠".yellow());
        }
    }

    Ok(())
}
