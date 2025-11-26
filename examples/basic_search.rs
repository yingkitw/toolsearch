//! Basic example of searching tools across MCP servers
//!
//! This example demonstrates the simplest way to search tools:
//! 1. Load servers from config file
//! 2. Search with a simple query (auto-detects mode)
//! 3. Display the results

use toolsearch_rs::{load_servers, simple_search};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load servers from configuration file
    let servers = load_servers("examples/config_example.json")?;

    // Simple search - auto-detects search mode based on query
    let query = "search";
    println!("Searching for tools matching '{}'...", query);

    match simple_search(&servers, query).await {
        Ok(results) => {
            if results.is_empty() {
                println!("No tools found matching '{}'", query);
            } else {
                println!("Found {} tool(s):\n", results.len());
                for result in results {
                    println!("Server: {}", result.server_name);
                    println!("  Name: {}", result.tool_name());
                    if let Some(desc) = &result.tool.description {
                        println!("  Description: {}", desc.as_ref());
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("Error searching tools: {}", e);
        }
    }

    Ok(())
}

