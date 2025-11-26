use toolsearch::{SearchCriteria, SearchFields, SearchMode, SearchOptions, ServerConfig, SortOrder, TransportConfig};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_server_config_validation() {
    // Valid config
    let valid_config = ServerConfig {
        name: "test_server".to_string(),
        transport: TransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec![],
            env: HashMap::new(),
        },
    };
    assert!(valid_config.validate().is_ok());

    // Invalid: empty name
    let invalid_config = ServerConfig {
        name: "".to_string(),
        transport: TransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec![],
            env: HashMap::new(),
        },
    };
    assert!(invalid_config.validate().is_err());

    // Invalid: empty command
    let invalid_config2 = ServerConfig {
        name: "test".to_string(),
        transport: TransportConfig::Stdio {
            command: "".to_string(),
            args: vec![],
            env: HashMap::new(),
        },
    };
    assert!(invalid_config2.validate().is_err());

    // Invalid: bad SSE URL
    let invalid_config3 = ServerConfig {
        name: "test".to_string(),
        transport: TransportConfig::Sse {
            url: "not-a-url".to_string(),
            headers: HashMap::new(),
        },
    };
    assert!(invalid_config3.validate().is_err());

    // Valid: good SSE URL
    let valid_config2 = ServerConfig {
        name: "test".to_string(),
        transport: TransportConfig::Sse {
            url: "https://example.com/sse".to_string(),
            headers: HashMap::new(),
        },
    };
    assert!(valid_config2.validate().is_ok());
}

#[test]
fn test_search_options_default() {
    let options = SearchOptions::default();
    assert_eq!(options.timeout, Some(Duration::from_secs(30)));
    assert_eq!(options.sort_order, SortOrder::ServerThenTool);
    assert!(options.continue_on_error);
    assert_eq!(options.max_results, None);
}

#[test]
fn test_search_criteria_match_all() {
    use std::sync::Arc;
    use serde_json::Map;
    use rmcp::model::Tool;

    let criteria = SearchCriteria::match_all();
    
    let tool = Tool {
        name: "any_tool".to_string().into(),
        title: None,
        description: Some("Any description".to_string().into()),
        input_schema: Arc::new(Map::new()),
        annotations: None,
        icons: None,
        output_schema: None,
    };

    // Should match all tools
    assert!(criteria.matches(&tool));
}

#[test]
fn test_search_fields_default() {
    let fields = SearchFields::default();
    assert!(fields.name);
    assert!(fields.title);
    assert!(fields.description);
    assert!(!fields.input_schema);
}

#[test]
fn test_tool_search_match_tool_name() {
    use std::sync::Arc;
    use serde_json::Map;
    use rmcp::model::Tool;
    use toolsearch::ToolSearchMatch;

    let tool = Tool {
        name: "test_tool".to_string().into(),
        title: None,
        description: None,
        input_schema: Arc::new(Map::new()),
        annotations: None,
        icons: None,
        output_schema: None,
    };

    let match_result = ToolSearchMatch {
        server_name: "test_server".to_string(),
        tool,
    };

    assert_eq!(match_result.tool_name(), "test_tool");
}

