//! Advanced search example demonstrating pattern matching, keyword matching, and field-specific searches
//!
//! This example shows:
//! 1. Regex pattern matching
//! 2. Keyword matching (all keywords must be present)
//! 3. Field-specific searches (name, title, description, input_schema)
//! 4. Case-sensitive searches
//! 5. Word boundary matching

use toolsearch_rs::{
    search_tools, search_tools_with_keywords, search_tools_with_regex, SearchCriteria, SearchFields,
    SearchMode, ServerConfig, TransportConfig,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure multiple MCP servers with complex settings
    let servers = vec![
        ServerConfig {
            name: "file_operations_server".to_string(),
            transport: TransportConfig::Stdio {
                command: "mcp-file-server".to_string(),
                args: vec!["--verbose".to_string(), "--log-level=debug".to_string()],
                env: {
                    let mut env = HashMap::new();
                    env.insert("RUST_LOG".to_string(), "debug".to_string());
                    env.insert("MCP_SERVER_PORT".to_string(), "8080".to_string());
                    env
                },
            },
        },
        ServerConfig {
            name: "database_server".to_string(),
            transport: TransportConfig::Stdio {
                command: "mcp-db-server".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        },
        ServerConfig {
            name: "api_integration_server".to_string(),
            transport: TransportConfig::Stdio {
                command: "mcp-api-server".to_string(),
                args: vec!["--config".to_string(), "/etc/mcp/config.json".to_string()],
                env: {
                    let mut env = HashMap::new();
                    env.insert("API_KEY".to_string(), "secret-key".to_string());
                    env.insert("ENVIRONMENT".to_string(), "production".to_string());
                    env
                },
            },
        },
    ];

    println!("=== Example 1: Regex Pattern Matching ===\n");
    // Search for tools matching a regex pattern (e.g., tools starting with "read" or "write")
    match search_tools_with_regex(&servers, r"^(read|write|get|set)").await {
        Ok(results) => {
            println!("Found {} tool(s) matching pattern '^(read|write|get|set)':\n", results.len());
            for result in results {
                println!("Server: {}", result.server_name);
                println!("  Name: {}", result.tool.name);
                if let Some(desc) = &result.tool.description {
                    println!("  Description: {}", desc);
                }
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Example 2: Keyword Matching ===\n");
    // Search for tools that contain ALL keywords: "file" AND "read"
    match search_tools_with_keywords(&servers, vec!["file".to_string(), "read".to_string()]).await {
        Ok(results) => {
            println!("Found {} tool(s) with keywords ['file', 'read']:\n", results.len());
            for result in results {
                println!("Server: {}", result.server_name);
                println!("  Name: {}", result.tool.name);
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Example 3: Field-Specific Search ===\n");
    // Search only in tool names and titles (not descriptions)
    let criteria = SearchCriteria::with_query("query".to_string())
        .with_fields(SearchFields {
            name: true,
            title: true,
            description: false,
            input_schema: false,
        });
    match search_tools(&servers, &criteria).await {
        Ok(results) => {
            println!("Found {} tool(s) matching 'query' in name/title only:\n", results.len());
            for result in results {
                println!("Server: {}", result.server_name);
                println!("  Name: {}", result.tool.name);
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Example 4: Search in Input Schema ===\n");
    // Search in input schema properties (e.g., find tools that accept "path" parameter)
    let criteria = SearchCriteria::with_query("path".to_string())
        .with_fields(SearchFields {
            name: true,
            title: false,
            description: true,
            input_schema: true, // Enable schema search
        });
    match search_tools(&servers, &criteria).await {
        Ok(results) => {
            println!("Found {} tool(s) with 'path' in name, description, or schema:\n", results.len());
            for result in results {
                println!("Server: {}", result.server_name);
                println!("  Name: {}", result.tool.name);
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Example 5: Case-Sensitive Search ===\n");
    // Case-sensitive search for exact casing
    let criteria = SearchCriteria::with_query("Read".to_string())
        .case_sensitive(true);
    match search_tools(&servers, &criteria).await {
        Ok(results) => {
            println!("Found {} tool(s) with case-sensitive 'Read':\n", results.len());
            for result in results {
                println!("Server: {}", result.server_name);
                println!("  Name: {}", result.tool.name);
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Example 6: Word Boundary Matching ===\n");
    // Match whole words only (e.g., "read" matches "read_file" but not "bread")
    let criteria = SearchCriteria::with_query("read".to_string())
        .with_mode(SearchMode::WordBoundary);
    match search_tools(&servers, &criteria).await {
        Ok(results) => {
            println!("Found {} tool(s) with whole word 'read':\n", results.len());
            for result in results {
                println!("Server: {}", result.server_name);
                println!("  Name: {}", result.tool.name);
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Example 7: Complex Combined Criteria ===\n");
    // Combine multiple criteria: regex pattern + minimum description length
    let criteria = SearchCriteria::with_regex(r"file|directory|folder".to_string())
        .with_fields(SearchFields {
            name: true,
            title: true,
            description: true,
            input_schema: false,
        });
    match search_tools(&servers, &criteria).await {
        Ok(results) => {
            println!("Found {} tool(s) matching complex criteria:\n", results.len());
            for result in results {
                println!("Server: {}", result.server_name);
                println!("  Name: {}", result.tool.name);
                if let Some(desc) = &result.tool.description {
                    println!("  Description: {}", desc);
                }
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

