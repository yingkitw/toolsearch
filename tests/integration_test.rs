use toolsearch_rs::{list_all_tools, search_tools_with_query, SearchCriteria, ServerConfig, TransportConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_search_criteria() {
    use toolsearch_rs::SearchCriteria;
    use rmcp::model::Tool;

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

    // Test description matching
    let criteria = SearchCriteria::with_query("testing".to_string());
    assert!(criteria.matches(&tool));
}

#[tokio::test]
async fn test_server_config_serialization() {
    let config = ServerConfig {
        name: "test_server".to_string(),
        transport: TransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: HashMap::new(),
        },
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.name, deserialized.name);
    match (&config.transport, &deserialized.transport) {
        (TransportConfig::Stdio { command: c1, args: a1, .. },
         TransportConfig::Stdio { command: c2, args: a2, .. }) => {
            assert_eq!(c1, c2);
            assert_eq!(a1, a2);
        }
        _ => panic!("Transport config mismatch"),
    }
}

// Note: Integration tests that actually connect to MCP servers would require
// running MCP servers, which is beyond the scope of unit tests.
// These would be better suited as example programs or manual tests.

