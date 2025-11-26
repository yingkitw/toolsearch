//! # toolsearch-rs
//!
//! A Rust library for searching tools across multiple MCP (Model Context Protocol) servers.
//!
//! This library provides functionality to:
//! - Connect to multiple MCP servers
//! - List and search tools across servers
//! - Filter tools by name, description, or other criteria
//!
//! ## Simple Example
//!
//! ```no_run
//! use toolsearch::{load_servers, simple_search};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load servers from config file (validates automatically)
//! let servers = load_servers("servers.json")?;
//!
//! // Simple search - auto-detects search mode
//! let results = simple_search(&servers, "read file").await?;
//! for result in results {
//!     println!("Found tool: {} on server: {}", result.tool_name(), result.server_name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Example with Builder
//!
//! ```no_run
//! use toolsearch::{load_servers, SearchBuilder};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let servers = load_servers("servers.json")?;
//!
//! let results = SearchBuilder::new(servers)
//!     .query("read,file")  // Comma-separated = keyword matching
//!     .limit(10)          // Limit results
//!     .sort_by_tool()     // Sort by tool name
//!     .search()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use anyhow::Context;
use futures::future::join_all;
use rmcp::model::Tool;
use rmcp::ServiceExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub mod error;
pub mod search;
pub use error::ToolSearchError;
pub use search::{load_servers, simple_search, SearchBuilder};

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Name identifier for the server
    pub name: String,
    /// Transport configuration
    pub transport: TransportConfig,
}

impl ServerConfig {
    /// Validate the server configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Server name cannot be empty".to_string());
        }

        match &self.transport {
            TransportConfig::Stdio { command, .. } => {
                if command.is_empty() {
                    return Err(format!("Command cannot be empty for server: {}", self.name));
                }
            }
            TransportConfig::Sse { url, .. } => {
                if url.is_empty() {
                    return Err(format!("URL cannot be empty for server: {}", self.name));
                }
                // Basic URL validation
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(format!("Invalid URL format for server {}: {}", self.name, url));
                }
            }
        }

        Ok(())
    }
}

/// Transport configuration for connecting to MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransportConfig {
    /// Standard I/O transport (spawns a process)
    #[serde(rename = "stdio")]
    Stdio {
        /// Command to execute
        command: String,
        /// Command arguments
        args: Vec<String>,
        /// Environment variables (optional)
        #[serde(default)]
        env: HashMap<String, String>,
    },
    /// SSE (Server-Sent Events) transport
    #[serde(rename = "sse")]
    Sse {
        /// URL endpoint
        url: String,
        /// Headers (optional)
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

/// Result of a tool search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchMatch {
    /// Name of the server where the tool was found
    pub server_name: String,
    /// The tool that matched the search
    pub tool: Tool,
}

impl ToolSearchMatch {
    /// Get the tool name as a string
    pub fn tool_name(&self) -> &str {
        self.tool.name.as_ref()
    }
}

/// Sort order for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort by server name, then tool name
    ServerThenTool,
    /// Sort by tool name, then server name
    ToolThenServer,
    /// No sorting (keep original order)
    None,
}

/// Options for search operations
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Timeout for server connections and queries
    pub timeout: Option<Duration>,
    /// Sort order for results
    pub sort_order: SortOrder,
    /// Continue searching other servers if one fails
    pub continue_on_error: bool,
    /// Maximum number of results to return
    pub max_results: Option<usize>,
}

/// Search mode for pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Simple substring matching (case-insensitive by default)
    Substring,
    /// Regular expression pattern matching
    Regex,
    /// Keyword matching (all keywords must be present)
    Keywords,
    /// Word boundary matching (whole words only)
    WordBoundary,
}

/// Fields to search in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchFields {
    /// Search in tool name
    pub name: bool,
    /// Search in tool title
    pub title: bool,
    /// Search in tool description
    pub description: bool,
    /// Search in input schema (property names and descriptions)
    pub input_schema: bool,
}

impl Default for SearchFields {
    fn default() -> Self {
        Self {
            name: true,
            title: true,
            description: true,
            input_schema: false,
        }
    }
}

/// Search criteria for filtering tools
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    /// Search query string
    pub query: Option<String>,
    /// Exact name match
    pub name: Option<String>,
    /// Search mode
    pub mode: SearchMode,
    /// Fields to search in
    pub fields: SearchFields,
    /// Case sensitive search
    pub case_sensitive: bool,
    /// Minimum description length
    pub min_description_length: Option<usize>,
    /// Keywords for keyword matching mode
    pub keywords: Vec<String>,
    /// Compiled regex pattern (cached for performance)
    #[allow(clippy::type_complexity)]
    regex: Option<Result<Regex, regex::Error>>,
}

impl SearchCriteria {
    /// Create a new search criteria with a query string
    pub fn with_query(query: String) -> Self {
        Self {
            query: Some(query),
            name: None,
            mode: SearchMode::Substring,
            fields: SearchFields::default(),
            case_sensitive: false,
            min_description_length: None,
            keywords: vec![],
            regex: None,
        }
    }

    /// Create a search criteria for exact name match
    pub fn with_name(name: String) -> Self {
        Self {
            query: None,
            name: Some(name),
            mode: SearchMode::Substring,
            fields: SearchFields::default(),
            case_sensitive: false,
            min_description_length: None,
            keywords: vec![],
            regex: None,
        }
    }

    /// Create a search criteria with regex pattern
    pub fn with_regex(pattern: String) -> Self {
        let regex = Regex::new(&pattern);
        Self {
            query: Some(pattern),
            name: None,
            mode: SearchMode::Regex,
            fields: SearchFields::default(),
            case_sensitive: false,
            min_description_length: None,
            keywords: vec![],
            regex: Some(regex),
        }
    }

    /// Create a search criteria with keywords (all must match)
    pub fn with_keywords(keywords: Vec<String>) -> Self {
        Self {
            query: None,
            name: None,
            mode: SearchMode::Keywords,
            fields: SearchFields::default(),
            case_sensitive: false,
            min_description_length: None,
            keywords,
            regex: None,
        }
    }

    /// Create an empty search criteria that matches all tools
    pub fn match_all() -> Self {
        Self {
            query: None,
            name: None,
            mode: SearchMode::Substring,
            fields: SearchFields::default(),
            case_sensitive: false,
            min_description_length: None,
            keywords: vec![],
            regex: None,
        }
    }

    /// Set search mode
    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        // Recompile regex if needed
        if mode == SearchMode::Regex {
            if let Some(ref query) = self.query {
                self.regex = Some(Regex::new(query));
            }
        }
        self
    }

    /// Set fields to search in
    pub fn with_fields(mut self, fields: SearchFields) -> Self {
        self.fields = fields;
        self
    }

    /// Set case sensitivity
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Extract text from input schema for searching
    fn extract_schema_text(schema: &Value) -> String {
        let mut text = String::new();
        
        if let Some(obj) = schema.as_object() {
            // Extract property names
            if let Some(properties) = obj.get("properties") {
                if let Some(props_obj) = properties.as_object() {
                    for key in props_obj.keys() {
                        text.push_str(key);
                        text.push(' ');
                    }
                }
            }
            
            // Extract descriptions from schema
            if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
                text.push_str(desc);
                text.push(' ');
            }
            
            // Recursively extract from nested objects
            for value in obj.values() {
                if value.is_object() {
                    text.push_str(&Self::extract_schema_text(value));
                } else if let Some(s) = value.as_str() {
                    text.push_str(s);
                    text.push(' ');
                }
            }
        }
        
        text
    }

    /// Check if text matches the query based on search mode
    fn text_matches(&self, text: &str) -> bool {
        let search_text = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        match self.mode {
            SearchMode::Substring => {
                let query = if self.case_sensitive {
                    self.query.as_ref().unwrap().clone()
                } else {
                    self.query.as_ref().unwrap().to_lowercase()
                };
                search_text.contains(&query)
            }
            SearchMode::Regex => {
                if let Some(ref regex_result) = self.regex {
                    match regex_result {
                        Ok(regex) => regex.is_match(text),
                        Err(_) => false,
                    }
                } else if let Some(ref query) = self.query {
                    // Fallback: compile regex on the fly
                    match Regex::new(query) {
                        Ok(regex) => regex.is_match(text),
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
            SearchMode::Keywords => {
                let keywords = if self.case_sensitive {
                    self.keywords.clone()
                } else {
                    self.keywords.iter().map(|k| k.to_lowercase()).collect()
                };
                keywords.iter().all(|keyword| search_text.contains(keyword))
            }
            SearchMode::WordBoundary => {
                let query = if self.case_sensitive {
                    self.query.as_ref().unwrap().clone()
                } else {
                    self.query.as_ref().unwrap().to_lowercase()
                };
                // Simple word boundary check
                let pattern = format!(r"\b{}\b", regex::escape(&query));
                match Regex::new(&pattern) {
                    Ok(regex) => {
                        if self.case_sensitive {
                            regex.is_match(text)
                        } else {
                            regex.is_match(&search_text)
                        }
                    }
                    Err(_) => search_text.contains(&query),
                }
            }
        }
    }

    /// Check if a tool matches the search criteria
    pub fn matches(&self, tool: &Tool) -> bool {
        // Exact name match takes precedence
        if let Some(ref name) = self.name {
            let tool_name: &str = tool.name.as_ref();
            return if self.case_sensitive {
                tool_name == name
            } else {
                tool_name.eq_ignore_ascii_case(name)
            };
        }

        // Check minimum description length
        if let Some(min_len) = self.min_description_length {
            if tool
                .description
                .as_ref()
                .map(|d| d.len() < min_len)
                .unwrap_or(true)
            {
                return false;
            }
        }

        // If no query or keywords, match all (unless we have other filters)
        if self.query.is_none() && self.keywords.is_empty() {
            return true;
        }

        // Collect all searchable text from different fields
        let mut searchable_texts = Vec::new();

        if self.fields.name {
            searchable_texts.push(("name", tool.name.as_ref().to_string()));
        }

        if self.fields.title {
            if let Some(ref title) = tool.title {
                searchable_texts.push(("title", title.to_string()));
            }
        }

        if self.fields.description {
            if let Some(ref desc) = tool.description {
                searchable_texts.push(("description", desc.as_ref().to_string()));
            }
        }

        if self.fields.input_schema {
            // Convert Arc<Map> to Value for extraction
            let schema_value: Value = serde_json::to_value(&*tool.input_schema)
                .unwrap_or(Value::Object(serde_json::Map::new()));
            let schema_text = Self::extract_schema_text(&schema_value);
            if !schema_text.is_empty() {
                searchable_texts.push(("input_schema", schema_text));
            }
        }

        // Check if any field matches
        for (_field_name, text) in searchable_texts {
            if self.text_matches(&text) {
                return true;
            }
        }

        false
    }
}

/// Connect to an MCP server using the provided transport configuration
/// Returns a RunningService that can be used to interact with the server
async fn connect_to_server(
    config: &ServerConfig,
) -> Result<rmcp::service::RunningService<rmcp::RoleClient, ()>, ToolSearchError> {
    match &config.transport {
        TransportConfig::Stdio { command, args, env } => {
            let mut cmd = Command::new(command);
            cmd.args(args);
            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            cmd.envs(env);

            let mut child = cmd
                .spawn()
                .with_context(|| format!("Failed to spawn command: {}", command))?;

            // Get stdin/stdout from child process
            // Note: tuple order is (read, write) = (stdout, stdin)
            let stdin = child.stdin.take().ok_or_else(|| {
                ToolSearchError::Connection("Failed to get stdin from child process".to_string())
            })?;
            let stdout = child.stdout.take().ok_or_else(|| {
                ToolSearchError::Connection("Failed to get stdout from child process".to_string())
            })?;

            // Create a basic client service and serve it with stdio transport
            // The unit type () implements Service<RoleClient> as a basic client
            // Tuple order: (read, write) = (stdout, stdin)
            let service = ().serve((stdout, stdin))
                .await
                .map_err(|e| ToolSearchError::Connection(format!("Failed to initialize client: {}", e)))?;
            Ok(service)
        }
        TransportConfig::Sse { url, headers: _ } => {
            // SSE transport implementation would go here
            // For now, return an error as SSE support may need additional setup
            Err(ToolSearchError::UnsupportedTransport(
                format!("SSE transport not yet implemented for URL: {}", url),
            ))
        }
    }
}

/// List all tools from a single MCP server
pub async fn list_tools_from_server(
    config: &ServerConfig,
) -> Result<Vec<Tool>, ToolSearchError> {
    list_tools_from_server_with_timeout(config, None).await
}

/// List all tools from a single MCP server with timeout
pub async fn list_tools_from_server_with_timeout(
    config: &ServerConfig,
    timeout_duration: Option<Duration>,
) -> Result<Vec<Tool>, ToolSearchError> {
    let connect_future = connect_to_server(config);
    
    let service = if let Some(timeout_dur) = timeout_duration {
        timeout(timeout_dur, connect_future)
            .await
            .map_err(|_| ToolSearchError::Connection(format!(
                "Connection timeout after {:?} for server: {}",
                timeout_dur, config.name
            )))?
    } else {
        connect_future.await
    }?;
    
    let peer = service.peer();

    // List all tools (handling pagination)
    let mut tools = Vec::new();
    let mut cursor = None;

    loop {
        let list_future = peer.list_tools(Some(rmcp::model::PaginatedRequestParam { cursor }));
        
        let result = if let Some(timeout_dur) = timeout_duration {
            timeout(timeout_dur, list_future)
                .await
                .map_err(|_| ToolSearchError::Connection(format!(
                    "List tools timeout after {:?} for server: {}",
                    timeout_dur, config.name
                )))?
        } else {
            list_future.await
        }?;

        tools.extend(result.tools);

        if result.next_cursor.is_some() {
            cursor = result.next_cursor;
        } else {
            break;
        }
    }

    Ok(tools)
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            sort_order: SortOrder::ServerThenTool,
            continue_on_error: true,
            max_results: None,
        }
    }
}

/// Search for tools across multiple MCP servers (sequential)
pub async fn search_tools(
    servers: &[ServerConfig],
    criteria: &SearchCriteria,
) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
    search_tools_with_options(servers, criteria, &SearchOptions::default()).await
}

/// Search for tools across multiple MCP servers with options
pub async fn search_tools_with_options(
    servers: &[ServerConfig],
    criteria: &SearchCriteria,
    options: &SearchOptions,
) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
    
    // Validate all server configurations first
    for server in servers {
        if let Err(e) = server.validate() {
            if !options.continue_on_error {
                return Err(ToolSearchError::Connection(e));
            }
            eprintln!("Warning: Invalid server configuration {}: {}", server.name, e);
        }
    }
    
    // Query all servers in parallel
    let server_futures: Vec<_> = servers
        .iter()
        .filter_map(|server_config| {
            // Skip invalid configurations if continuing on error
            if server_config.validate().is_err() && options.continue_on_error {
                return None;
            }
            let config = server_config.clone();
            let timeout_dur = options.timeout;
            Some(async move {
                let result = list_tools_from_server_with_timeout(&config, timeout_dur).await;
                (config.name.clone(), result)
            })
        })
        .collect();

    let server_results = join_all(server_futures).await;
    
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for (server_name, server_result) in server_results {
        match server_result {
            Ok(tools) => {
                for tool in tools {
                    if criteria.matches(&tool) {
                        results.push(ToolSearchMatch {
                            server_name: server_name.clone(),
                            tool,
                        });
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Error connecting to server {}: {}", server_name, e);
                if options.continue_on_error {
                    errors.push(error_msg);
                } else {
                    return Err(e);
                }
            }
        }
    }

    // Log errors if continuing on error
    if !errors.is_empty() && options.continue_on_error {
        for error in &errors {
            eprintln!("{}", error);
        }
    }

    // Sort results
    match options.sort_order {
        SortOrder::ServerThenTool => {
            results.sort_by(|a, b| {
                a.server_name
                    .cmp(&b.server_name)
                    .then_with(|| a.tool_name().cmp(b.tool_name()))
            });
        }
        SortOrder::ToolThenServer => {
            results.sort_by(|a, b| {
                a.tool_name()
                    .cmp(b.tool_name())
                    .then_with(|| a.server_name.cmp(&b.server_name))
            });
        }
        SortOrder::None => {
            // Keep original order
        }
    }

    // Limit results if specified
    if let Some(max) = options.max_results {
        results.truncate(max);
    }

    Ok(results)
}

/// Convenience function to search tools with a query string
pub async fn search_tools_with_query(
    servers: &[ServerConfig],
    query: &str,
) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
    let criteria = SearchCriteria::with_query(query.to_string());
    search_tools(servers, &criteria).await
}

/// Search tools using regex pattern
pub async fn search_tools_with_regex(
    servers: &[ServerConfig],
    pattern: &str,
) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
    let criteria = SearchCriteria::with_regex(pattern.to_string());
    search_tools(servers, &criteria).await
}

/// Search tools using keywords (all must match)
pub async fn search_tools_with_keywords(
    servers: &[ServerConfig],
    keywords: Vec<String>,
) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
    let criteria = SearchCriteria::with_keywords(keywords);
    search_tools(servers, &criteria).await
}

/// List all tools from all servers without filtering
pub async fn list_all_tools(
    servers: &[ServerConfig],
) -> Result<Vec<ToolSearchMatch>, ToolSearchError> {
    let criteria = SearchCriteria {
        query: None,
        name: None,
        mode: SearchMode::Substring,
        fields: SearchFields::default(),
        case_sensitive: false,
        min_description_length: None,
        keywords: vec![],
        regex: None,
    };
    search_tools(servers, &criteria).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_criteria_matches() {
        use std::sync::Arc;
        use serde_json::Map;
        
        let tool = Tool {
            name: "test_tool".to_string().into(),
            title: None,
            description: Some("A test tool for testing".to_string().into()),
            input_schema: Arc::new(Map::new()),
            annotations: None,
            icons: None,
            output_schema: None,
        };

        // Test query matching
        let criteria = SearchCriteria::with_query("test".to_string());
        assert!(criteria.matches(&tool));

        let criteria = SearchCriteria::with_query("nonexistent".to_string());
        assert!(!criteria.matches(&tool));

        // Test exact name matching
        let criteria = SearchCriteria::with_name("test_tool".to_string());
        assert!(criteria.matches(&tool));

        let criteria = SearchCriteria::with_name("other_tool".to_string());
        assert!(!criteria.matches(&tool));

        // Test regex matching
        let criteria = SearchCriteria::with_regex(r"test.*tool".to_string());
        assert!(criteria.matches(&tool));

        // Test keyword matching
        let criteria = SearchCriteria::with_keywords(vec!["test".to_string(), "tool".to_string()]);
        assert!(criteria.matches(&tool));

        let criteria = SearchCriteria::with_keywords(vec!["test".to_string(), "nonexistent".to_string()]);
        assert!(!criteria.matches(&tool));

        // Test word boundary matching
        let criteria = SearchCriteria::with_query("test".to_string())
            .with_mode(SearchMode::WordBoundary);
        assert!(criteria.matches(&tool));
    }
}

