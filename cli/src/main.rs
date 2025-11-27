use clap::{Parser, Subcommand};
use colored::Colorize;
use nexus_core::{AppState, NexusConfig, Server};
use nexus_event_fabric::NatsClient;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Query events from the API
async fn query_events(limit: u32) -> anyhow::Result<serde_json::Value> {
    let url = format!("http://localhost:8080/events?limit={}", limit);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Server returned status: {}", response.status());
    }
    
    let data = response.json().await?;
    Ok(data)
}

/// Get a specific event by ID
async fn get_event_by_id(event_id: &str) -> anyhow::Result<serde_json::Value> {
    let url = format!("http://localhost:8080/events/{}", event_id);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Server returned status: {}", response.status());
    }
    
    let data = response.json().await?;
    Ok(data)
}

/// Get system metrics
async fn get_metrics() -> anyhow::Result<serde_json::Value> {
    let url = "http://localhost:8080/metrics";
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Server returned status: {}", response.status());
    }
    
    let data = response.json().await?;
    Ok(data)
}

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
        /// Event ID to get (optional)
        event_id: Option<String>,
        
        /// Number of events to show when listing
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },
    
    /// View system metrics
    Metrics,
    
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
        
        Commands::Events { event_id, limit } => {
            // If event ID is provided, get that specific event
            if let Some(id) = event_id {
                println!("{} Fetching event {}...", "📋".cyan(), id);
                
                match get_event_by_id(&id).await {
                    Ok(event) => {
                        println!();
                        println!("{} Event Details", "━".repeat(40).bright_cyan());
                        println!();
                        println!("{} {}", "ID:".bright_white().bold(), event["id"].as_str().unwrap_or("unknown").bright_cyan());
                        println!("{} {}", "Type:".bright_white(), event["type"].as_str().unwrap_or("unknown"));
                        println!("{} {}", "Source:".bright_white(), event["source"].as_str().unwrap_or("unknown"));
                        println!("{} {}", "Time:".bright_white(), event["time"].as_str().unwrap_or("unknown"));
                        println!();
                        println!("{}", "Data:".bright_white().bold());
                        if let Some(data) = event.get("data") {
                            println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
                        }
                        println!();
                    }
                    Err(e) => {
                        eprintln!("{} Failed to fetch event: {}", "✗".red(), e);
                        eprintln!("{} Make sure the server is running and the event ID is correct", "💡".yellow());
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }
            
            // Otherwise, list recent events
            println!("{} Fetching last {} events...", "📋".cyan(), limit);
            
            match query_events(limit).await {
                Ok(events_data) => {
                    let total = events_data["total"].as_u64().unwrap_or(0);
                    let empty_vec = vec![];
                    let events = events_data["events"].as_array().unwrap_or(&empty_vec);
                    
                    println!();
                    println!("{} {} total events, showing {}", "ℹ".bright_blue(), total, events.len());
                    println!("{}", "─".repeat(80).bright_black());
                    
                    if events.is_empty() {
                        println!("{} No events found", "ℹ".yellow());
                    } else {
                        for event in events {
                            let id = event["id"].as_str().unwrap_or("unknown");
                            let event_type = event["type"].as_str().unwrap_or("unknown");
                            let time = event["time"].as_str().unwrap_or("unknown");
                            
                            println!();
                            println!("{} {}", "ID:".bright_white().bold(), id.bright_cyan());
                            println!("{} {}", "Type:".bright_white(), event_type);
                            println!("{} {}", "Time:".bright_white(), time);
                            
                            if let Some(data) = event.get("data") {
                                println!("{} {}", "Data:".bright_white(), serde_json::to_string_pretty(data).unwrap_or_default());
                            }
                            println!("{}", "─".repeat(80).bright_black());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Failed to fetch events: {}", "✗".red(), e);
                    eprintln!("{} Make sure the server is running on http://localhost:8080", "💡".yellow());
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Metrics => {
            println!("{} Fetching system metrics...", "📊".cyan());
            
            match get_metrics().await {
                Ok(metrics) => {
                    println!();
                    println!("{} System Metrics", "━".repeat(40).bright_cyan());
                    println!();
                    
                    // Events metrics
                    if let Some(events) = metrics.get("events") {
                        println!("{}", "Events:".bright_white().bold());
                        println!("  Published:    {}", events["published"].as_u64().unwrap_or(0).to_string().bright_green());
                        println!("  Replayed:     {}", events["replayed"].as_u64().unwrap_or(0));
                        println!("  Failed:       {}", events["failed"].as_u64().unwrap_or(0).to_string().bright_red());
                        println!("  Success Rate: {}%", format!("{:.2}", events["success_rate"].as_f64().unwrap_or(0.0)).bright_green());
                        println!();
                    }
                    
                    // Functions metrics
                    if let Some(functions) = metrics.get("functions") {
                        println!("{}", "Functions:".bright_white().bold());
                        println!("  Executed:     {}", functions["executed"].as_u64().unwrap_or(0).to_string().bright_green());
                        println!("  Succeeded:    {}", functions["succeeded"].as_u64().unwrap_or(0));
                        println!("  Failed:       {}", functions["failed"].as_u64().unwrap_or(0).to_string().bright_red());
                        println!("  Success Rate: {}%", format!("{:.2}", functions["success_rate"].as_f64().unwrap_or(0.0)).bright_green());
                        println!("  Avg Time:     {}ms", format!("{:.2}", functions["avg_execution_time_ms"].as_f64().unwrap_or(0.0)));
                        println!();
                    }
                    
                    // System metrics
                    if let Some(system) = metrics.get("system") {
                        println!("{}", "System:".bright_white().bold());
                        println!("  Uptime:       {}s", system["uptime_seconds"].as_u64().unwrap_or(0));
                        let nats_status = if system["nats_connected"].as_bool().unwrap_or(false) {
                            "Connected".bright_green()
                        } else {
                            "Disconnected".bright_red()
                        };
                        println!("  NATS:         {}", nats_status);
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("{} Failed to fetch metrics: {}", "✗".red(), e);
                    eprintln!("{} Make sure the server is running on http://localhost:8080", "💡".yellow());
                    std::process::exit(1);
                }
            }
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
