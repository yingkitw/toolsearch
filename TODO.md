# TODO

This file tracks planned improvements, features, and tasks for toolsearch-rs.

## High Priority

### Core Features
- [ ] Add SSE (Server-Sent Events) transport support
  - Currently only stdio transport is implemented
  - SSE transport is defined but not implemented
  - Needed for HTTP-based MCP servers

- [ ] Add tool caching mechanism
  - Cache tool lists per server to avoid repeated queries
  - Configurable cache TTL
  - Invalidate cache on server errors

- [ ] Add relevance scoring for search results
  - Score tools based on query match quality
  - Consider multiple factors: name match, description match, schema match
  - Sort by relevance score by default

### Performance
- [ ] Add connection pooling for MCP servers
  - Reuse connections when querying same server multiple times
  - Reduce connection overhead

- [ ] Implement incremental search
  - Stream results as they come in
  - Useful for large result sets

- [ ] Add result pagination support
  - For CLI and library API
  - Handle large result sets efficiently

## Medium Priority

### Search Enhancements
- [ ] Add fuzzy matching for tool names
  - Handle typos in search queries
  - Use edit distance algorithms

- [ ] Add semantic search support
  - Use embeddings to find semantically similar tools
  - Integrate with embedding models

- [ ] Add tool category/tag support
  - Allow tools to be categorized
  - Search by category

- [ ] Add tool usage statistics
  - Track which tools are most commonly used
  - Use for relevance ranking

### CLI Improvements
- [ ] Add interactive mode
  - Interactive search with autocomplete
  - Browse tools interactively

- [ ] Add progress bars for long-running queries
  - Show progress when querying multiple servers
  - Use indicatif or similar crate

- [ ] Add colored output
  - Better visual distinction in terminal
  - Highlight matches in search results

- [ ] Add export formats
  - Export to CSV, YAML, TOML
  - Useful for documentation generation

### Configuration
- [ ] Add configuration file validation schema
  - JSON schema for config files
  - Better error messages for invalid configs

- [ ] Add default configuration discovery
  - Look for config in standard locations
  - Support environment variable overrides

- [ ] Add configuration profiles
  - Multiple server configurations
  - Switch between profiles easily

## Low Priority

### Documentation
- [ ] Add more comprehensive API documentation
  - More examples for each function
  - Better doc comments

- [ ] Add architecture decision records (ADRs)
  - Document design decisions
  - Explain trade-offs

- [ ] Add troubleshooting guide
  - Common issues and solutions
  - Debug tips

### Testing
- [ ] Add integration tests with mock MCP servers
  - Test against real MCP protocol
  - Better test coverage

- [ ] Add performance benchmarks
  - Benchmark parallel queries
  - Compare with sequential queries

- [ ] Add fuzzing tests
  - Fuzz search queries
  - Fuzz configuration parsing

### Developer Experience
- [ ] Add logging support
  - Configurable log levels
  - Structured logging

- [ ] Add metrics/telemetry
  - Track search performance
  - Track error rates

- [ ] Add tracing support
  - Distributed tracing for debugging
  - Integration with tracing ecosystem

## Future Considerations

### Advanced Features
- [ ] Add tool dependency resolution
  - Some tools depend on others
  - Include dependencies in results

- [ ] Add tool conflict detection
  - Detect duplicate tool names across servers
  - Warn about conflicts

- [ ] Add tool versioning support
  - Handle multiple versions of same tool
  - Version-aware search

- [ ] Add tool recommendation system
  - Suggest related tools
  - Learn from usage patterns

### Integration
- [ ] Add WASM support
  - Compile to WebAssembly
  - Use in browser environments

- [ ] Add Python bindings
  - PyO3 bindings for Python integration
  - Make accessible to Python ecosystem

- [ ] Add Node.js bindings
  - napi-rs bindings
  - Make accessible to Node.js ecosystem

### Ecosystem
- [ ] Publish to crates.io
  - Make library available publicly
  - Version releases

- [ ] Add GitHub Actions CI/CD
  - Automated testing
  - Automated releases

- [ ] Add Docker image
  - Pre-built CLI tool
  - Easy deployment

## Completed ✅

- [x] Basic tool search functionality
- [x] Multiple search modes (substring, regex, keywords, word boundary)
- [x] Field-specific search (name, title, description, schema)
- [x] Parallel server queries
- [x] Result sorting
- [x] Timeout support
- [x] Configuration validation
- [x] Multiple output formats (text, JSON, table)
- [x] Simplified API with SearchBuilder
- [x] Auto-detection of search modes
- [x] Comprehensive examples
- [x] CLI tool with simplified interface
- [x] Error handling and recovery
- [x] Documentation and README

## Notes

- Focus on simplicity and ease of use
- Maintain backward compatibility
- Performance is critical for agentic AI use cases
- Token efficiency is the primary goal

