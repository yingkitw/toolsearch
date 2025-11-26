//! Simplified high-level search API
//!
//! This module provides a simple, intuitive interface for searching tools.
//! It automatically handles complexity like search mode detection, error handling,
//! and result formatting.

use crate::{SearchCriteria, SearchOptions, ServerConfig, SortOrder, ToolSearchMatch, ToolSearchError};
use std::time::Duration;

/// Simple search builder for intuitive tool searching
pub struct SearchBuilder {
    servers: Vec<ServerConfig>,
    query: Option<String>,
    keywords: Option<Vec<String>>,
    options: SearchOptions,
}

impl SearchBuilder {
    /// Create a new search builder with server configurations
    pub fn new(servers: Vec<ServerConfig>) -> Self {
        Self {
            servers,
            query: None,
            keywords: None,
            options: SearchOptions::default(),
        }
    }

    /// Set the search query (auto-detects search mode)
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Set keywords for keyword matching (all must be present)
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = Some(keywords);
        self.query = None; // Clear query when using keywords
        self
    }

    /// Set maximum number of results
    pub fn limit(mut self, max: usize) -> Self {
        self.options.max_results = Some(max);
        self
    }

    /// Set timeout in seconds
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.options.timeout = Some(Duration::from_secs(seconds));
        self
    }

    /// Sort results by tool name first, then server
    pub fn sort_by_tool(mut self) -> Self {
        self.options.sort_order = SortOrder::ToolThenServer;
        self
    }

    /// Sort results by server first, then tool (default)
    pub fn sort_by_server(mut self) -> Self {
        self.options.sort_order = SortOrder::ServerThenTool;
        self
    }

    /// Execute the search
    pub async fn search(self) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
        use crate::search_tools_with_options;

        // Auto-detect search mode based on query
        let criteria = if let Some(ref keywords) = self.keywords {
            // Use keyword matching if keywords are explicitly set
            SearchCriteria::with_keywords(keywords.clone())
        } else if let Some(ref query) = self.query {
            // Auto-detect: if query looks like regex, use regex mode
            // Otherwise use substring matching
            if is_likely_regex(query) {
                SearchCriteria::with_regex(query.clone())
            } else if query.contains(',') {
                // Comma-separated values -> keyword matching
                let keywords: Vec<String> = query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                SearchCriteria::with_keywords(keywords)
            } else {
                SearchCriteria::with_query(query.clone())
            }
        } else {
            // No query -> match all
            SearchCriteria::match_all()
        };

        search_tools_with_options(&self.servers, &criteria, &self.options).await
    }
}

/// Check if a query string looks like a regex pattern
fn is_likely_regex(query: &str) -> bool {
    // Simple heuristic: if it contains regex-like characters, treat as regex
    query.contains('^') || query.contains('$') || query.contains('*') || 
    query.contains('+') || query.contains('?') || query.contains('|') ||
    query.contains('[') || query.contains('(')
}

/// Simple function to search tools - handles most common cases automatically
///
/// # Example
/// ```no_run
/// use toolsearch::simple_search;
/// use toolsearch::ServerConfig;
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let servers = vec![
///     ServerConfig {
///         name: "server1".to_string(),
///         transport: toolsearch::TransportConfig::Stdio {
///             command: "mcp-server".to_string(),
///             args: vec![],
///             env: HashMap::new(),
///         },
///     },
/// ];
///
/// // Simple search - auto-detects mode
/// let results = simple_search(&servers, "read file").await?;
/// # Ok(())
/// # }
/// ```
pub async fn simple_search(
    servers: &[ServerConfig],
    query: &str,
) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
    SearchBuilder::new(servers.to_vec())
        .query(query)
        .search()
        .await
}

/// Load servers from a JSON configuration file
pub fn load_servers(config_path: &str) -> Result<Vec<ServerConfig>, Box<dyn std::error::Error>> {
    use std::fs;
    use serde_json;
    
    let config_data = fs::read_to_string(config_path)?;
    let servers: Vec<ServerConfig> = serde_json::from_str(&config_data)?;
    
    // Validate all servers
    for server in &servers {
        server.validate()
            .map_err(|e| format!("Invalid server configuration '{}': {}", server.name, e))?;
    }
    
    Ok(servers)
}

