//! Example of listing all tools from MCP servers
//!
//! This example demonstrates the simplest way to list all tools:
//! 1. Load servers from config file
//! 2. Use SearchBuilder to list all tools
//! 3. Display results grouped by server

use toolsearch_rs::{load_servers, SearchBuilder};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load servers from configuration file
    let servers = load_servers("examples/config_example.json")?;

    println!("Listing all tools from {} server(s)...\n", servers.len());

    // Use SearchBuilder - no query means match all
    match SearchBuilder::new(servers).search().await {
        Ok(results) => {
            println!("Found {} tool(s) total:\n", results.len());
            
            // Group by server
            let mut by_server: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
            for result in &results {
                by_server
                    .entry(result.server_name.clone())
                    .or_insert_with(Vec::new)
                    .push(result);
            }

            for (server_name, tools) in by_server {
                println!("Server: {} ({} tools)", server_name, tools.len());
                for tool_result in tools {
                    println!("  - {}", tool_result.tool_name());
                    if let Some(desc) = &tool_result.tool.description {
                        println!("    {}", desc.as_ref());
                    }
                }
                println!();
            }

            // Also output as JSON
            println!("JSON output:");
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        Err(e) => {
            eprintln!("Error listing tools: {}", e);
        }
    }

    Ok(())
}

