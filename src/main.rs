use clap::{Parser, Subcommand};
use toolsearch::{load_servers, SearchBuilder};

#[derive(Parser)]
#[command(name = "toolsearch")]
#[command(about = "Search tools across MCP servers", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for tools matching a query (auto-detects search mode)
    Search {
        /// Path to JSON configuration file with server configurations
        #[arg(short, long)]
        config: String,
        /// Search query (auto-detects: regex if contains ^$|*, keywords if comma-separated)
        query: String,
        /// Output format: json, text, or table
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Maximum number of results to return
        #[arg(short, long)]
        limit: Option<usize>,
        /// Sort by tool name instead of server name
        #[arg(long)]
        sort_by_tool: bool,
    },
    /// List all tools from all servers
    List {
        /// Path to JSON configuration file with server configurations
        #[arg(short, long)]
        config: String,
        /// Output format: json, text, or table
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Maximum number of results to return
        #[arg(short, long)]
        limit: Option<usize>,
        /// Sort by tool name instead of server name
        #[arg(long)]
        sort_by_tool: bool,
    },
    /// Validate server configuration file
    Validate {
        /// Path to JSON configuration file with server configurations
        #[arg(short, long)]
        config: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            config,
            query,
            format,
            limit,
            sort_by_tool,
        } => {
            // Load and validate servers
            let servers = load_servers(&config)?;

            // Build search with simple API
            let mut builder = SearchBuilder::new(servers).query(&query);
            
            if let Some(max) = limit {
                builder = builder.limit(max);
            }
            
            if sort_by_tool {
                builder = builder.sort_by_tool();
            }

            let results = builder.search().await?;
            print_results(&results, &format, &format!("Found {} tool(s) matching '{}'", results.len(), query))?;
        }
        Commands::List {
            config,
            format,
            limit,
            sort_by_tool,
        } => {
            // Load and validate servers
            let servers = load_servers(&config)?;

            // Build search to list all tools
            let mut builder = SearchBuilder::new(servers);
            
            if let Some(max) = limit {
                builder = builder.limit(max);
            }
            
            if sort_by_tool {
                builder = builder.sort_by_tool();
            }

            let results = builder.search().await?;
            print_results(&results, &format, &format!("Found {} tool(s) across all servers", results.len()))?;
        }
        Commands::Validate { config } => {
            match load_servers(&config) {
                Ok(servers) => {
                    println!("✓ Configuration file is valid!");
                    println!("✓ Found {} server(s)", servers.len());
                    for server in &servers {
                        println!("  - {}", server.name);
                    }
                }
                Err(e) => {
                    eprintln!("✗ Configuration error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

/// Print search results in the specified format
fn print_results(
    results: &[toolsearch::ToolSearchMatch],
    format: &str,
    header: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(results)?);
        }
        "table" => {
            if results.is_empty() {
                println!("No results found");
            } else {
                println!("{}\n", header);
                println!("{:<30} {:<40} {}", "SERVER", "TOOL NAME", "DESCRIPTION");
                println!("{}", "-".repeat(100));
                for result in results {
                    let desc = result
                        .tool
                        .description
                        .as_ref()
                        .map(|d| {
                            let desc_str: &str = d.as_ref();
                            if desc_str.len() > 50 {
                                format!("{}...", &desc_str[..47])
                            } else {
                                desc_str.to_string()
                            }
                        })
                        .unwrap_or_else(|| "N/A".to_string());
                    println!(
                        "{:<30} {:<40} {}",
                        result.server_name,
                        result.tool_name(),
                        desc
                    );
                }
            }
        }
        _ => {
            if results.is_empty() {
                println!("No results found");
            } else {
                println!("{}\n", header);
                for result in results {
                    println!("Server: {}", result.server_name);
                    println!("  Name: {}", result.tool_name());
                    if let Some(desc) = &result.tool.description {
                        println!("  Description: {}", desc.as_ref());
                    }
                    if let Some(title) = &result.tool.title {
                        let title_str: &str = title.as_ref();
                        println!("  Title: {}", title_str);
                    }
                    println!();
                }
            }
        }
    }
    Ok(())
}

