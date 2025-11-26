//! Example demonstrating all search modes and their differences
//!
//! This example shows the difference between:
//! - Substring matching
//! - Regex pattern matching
//! - Keyword matching
//! - Word boundary matching

use toolsearch::{search_tools, SearchCriteria, SearchFields, SearchMode, ServerConfig, TransportConfig};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let servers = vec![
        ServerConfig {
            name: "example_server".to_string(),
            transport: TransportConfig::Stdio {
                command: "mcp-server".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        },
    ];

    let search_term = "read";

    println!("=== Search Mode Comparison ===\n");
    println!("Searching for: '{}'\n", search_term);

    // 1. Substring matching (default)
    println!("1. Substring Matching:");
    println!("   Matches: 'read', 'bread', 'read_file', 'reader'");
    let criteria = SearchCriteria::with_query(search_term.to_string())
        .with_mode(SearchMode::Substring);
    match search_tools(&servers, &criteria).await {
        Ok(results) => println!("   Found {} result(s)\n", results.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 2. Word boundary matching
    println!("2. Word Boundary Matching:");
    println!("   Matches: 'read', 'read_file', 'read_data'");
    println!("   Does NOT match: 'bread', 'reader'");
    let criteria = SearchCriteria::with_query(search_term.to_string())
        .with_mode(SearchMode::WordBoundary);
    match search_tools(&servers, &criteria).await {
        Ok(results) => println!("   Found {} result(s)\n", results.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 3. Regex pattern matching
    println!("3. Regex Pattern Matching:");
    println!("   Pattern: '^read|write'");
    println!("   Matches tools starting with 'read' or 'write'");
    let criteria = SearchCriteria::with_regex(r"^read|^write".to_string());
    match search_tools(&servers, &criteria).await {
        Ok(results) => println!("   Found {} result(s)\n", results.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 4. Keyword matching
    println!("4. Keyword Matching:");
    println!("   Keywords: ['file', 'read']");
    println!("   Matches tools containing BOTH 'file' AND 'read'");
    let criteria = SearchCriteria::with_keywords(vec!["file".to_string(), "read".to_string()]);
    match search_tools(&servers, &criteria).await {
        Ok(results) => println!("   Found {} result(s)\n", results.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 5. Field-specific search
    println!("5. Field-Specific Search:");
    println!("   Searching only in tool names (not descriptions)");
    let criteria = SearchCriteria::with_query(search_term.to_string())
        .with_fields(SearchFields {
            name: true,
            title: false,
            description: false,
            input_schema: false,
        });
    match search_tools(&servers, &criteria).await {
        Ok(results) => println!("   Found {} result(s)\n", results.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 6. Case-sensitive search
    println!("6. Case-Sensitive Search:");
    println!("   Query: 'Read' (capital R)");
    println!("   Matches: 'Read', 'ReadFile'");
    println!("   Does NOT match: 'read', 'read_file'");
    let criteria = SearchCriteria::with_query("Read".to_string())
        .case_sensitive(true);
    match search_tools(&servers, &criteria).await {
        Ok(results) => println!("   Found {} result(s)\n", results.len()),
        Err(e) => println!("   Error: {}\n", e),
    }

    Ok(())
}

