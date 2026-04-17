# Technology Stack

## Core Language

- **Rust** (Edition 2024)
  - Memory-safe systems programming
  - Zero-cost abstractions
  - Cross-platform compilation
  - Latest 2024 edition features

## Runtime & Concurrency

- **tokio** `1.0` (features: `full`)
  - Async runtime for I/O-bound operations
  - Multi-threaded executor
  - Channel primitives for inter-agent communication

## Serialization & Data

- **serde** `1.0` (features: `derive`)
  - Type-safe serialization framework
- **serde_json** `1.0`
  - JSON parsing/generation for MCP protocol
- **toml** `0.8`
  - Configuration file parsing
- **schemars** `1.0`
  - JSON Schema generation (feature: `schema-generation`)

## Protocol Layer

- **pmcp** `1.8` (features: `schema-generation`)
  - MCP (Model Context Protocol) implementation
- **agent-client-protocol** `0.10` (latest: `0.10.4`)
  - ACP (Agent Communication Protocol) client
  - Bidirectional communication with editors/AI agents

## Web Framework

- **rocket** `0.5.1`
  - Web framework with focus on usability, security, extensibility, speed
  - HTTP server for remote agent support
  - Webhook endpoints for external integrations

## Search & Tokenization

- **bm25** (latest)
  - In-memory keyword/search engine
  - BM25 scoring algorithm
- **tokenizers** (latest)
  - HuggingFace-style tokenizers
  - BPE, WordPiece, Unigram support
  - High-performance text processing

## Modal.com Integration

- **modal-rs** (latest)
  - Rust SDK for Modal cloud platform
  - Sandbox creation and management
  - Webhook handling for serverless execution
  - Remote GPU/Compute resource access

## Rate Limiting

- **governor** (latest)
  - Generic Cell Rate Algorithm (GCRA)
  - Per-agent RPM/RPD throttling
  - Thread-safe, low-memory footprint
- **tower_governor** `0.8` (for HTTP middleware)
  - Tower-compatible rate limiting

## CLI

- **clap** `4.5` (features: `derive`)
  - Command-line argument parsing
  - Subcommand routing
  - Auto-generated help text

## Logging & Observability

- **tracing** `0.1`
  - Structured event logging
  - Span-based context tracking
- **tracing-subscriber** `0.3` (features: `env-filter`, `json`)
  - JSON output formatter
  - Dynamic log level filtering

## Error Handling

- **anyhow** `1.0`
  - Application-level error handling
  - Context enrichment
- **thiserror** `1.0`
  - Custom error type derivation

## Database & Persistence

- **sqlx** (latest)
  - Compile-time checked SQL
  - Async database driver
  - Support: PostgreSQL, MySQL, SQLite
- **sqlite** (via sqlx)
  - Lightweight local storage for usage tracking

## Utilities

- **chrono** `0.4` (features: `serde`)
  - Date/time handling with serialization
- **regex** `1.0`
  - Pattern matching for routing keywords
- **ignore** `0.4`
  - File tree traversal with gitignore support
- **base64** `0.22`
  - Encoding/decoding for binary data
- **url** `2.5`
  - URL parsing and validation
- **sha2** `0.10`
  - Cryptographic hashing for content verification
- **uuid** `1.0` (features: `v4`, `serde`)
  - Unique identifier generation
- **textwrap** `0.16.2`
  - Terminal text formatting

## Testing

- **tokio-test** `0.4`
  - Async test utilities
- **tempfile** `3.0`
  - Temporary file/directory creation
- **mockall** `0.13`
  - Mock object generation
- **serial_test** `3.0`
  - Sequential test execution
- **proptest** `1.0` (optional)
  - Property-based testing
- **once_cell** `1.19`
  - Lazy initialization for test fixtures

## Platform Support

- **Primary:** Linux, macOS, Windows
- **Target:** Android (Termux)
- **Cross-compilation:** `cross` or `cargo-zigbuild`
