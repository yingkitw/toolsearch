//! Simple usage examples showing the intuitive API
//!
//! This example demonstrates how easy it is to use toolsearch-rs
//! with the simplified API that handles complexity automatically.

use toolsearch::{load_servers, simple_search, SearchBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load servers from config file (validates automatically)
    let servers = load_servers("examples/config_example.json")?;

    println!("=== Example 1: Simple Search ===\n");
    // Just search - auto-detects mode, handles everything
    let results = simple_search(&servers, "read").await?;
    println!("Found {} tools matching 'read'\n", results.len());

    println!("=== Example 2: Search with Builder Pattern ===\n");
    // More control with builder pattern
    let results = SearchBuilder::new(servers.clone())
        .query("file,read")  // Comma-separated = keyword matching
        .limit(10)           // Limit to 10 results
        .sort_by_tool()      // Sort by tool name
        .search()
        .await?;
    println!("Found {} tools (limited to 10, sorted by tool name)\n", results.len());

    println!("=== Example 3: Regex Auto-Detection ===\n");
    // Query looks like regex? Auto-detects and uses regex mode
    let results = SearchBuilder::new(servers.clone())
        .query("^read|^write")  // Auto-detected as regex
        .search()
        .await?;
    println!("Found {} tools matching regex pattern\n", results.len());

    println!("=== Example 4: List All Tools ===\n");
    // No query = list all tools
    let results = SearchBuilder::new(servers)
        .limit(20)
        .search()
        .await?;
    println!("Found {} total tools (limited to 20)\n", results.len());

    Ok(())
}

