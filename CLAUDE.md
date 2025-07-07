# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sentio is a modular Rust application for intelligent email processing. It consists of six main services that work together to provide an AI-powered email workflow system.

## Development Commands

### Build
```bash
cargo build              # Build all services
cargo build -p <service> # Build specific service (e.g., cargo build -p sentio-core)
```

### Testing
```bash
cargo test               # Run all tests
cargo test -p <service>  # Run tests for specific service
```

### Running
```bash
cargo run -p sentio-core # Run the core service
```

### Linting and Formatting
```bash
cargo fmt                # Format code
cargo clippy             # Run linter
```

## Project Architecture

### Service Structure
The project is organized as a Cargo workspace with the following services:

- **`services/core`**: Central orchestrator and workflow engine
- **`services/email`**: SMTP client and email handling
- **`services/llm`**: Large Language Model integration (DeepSeek)
- **`services/memory`**: Persistent data storage and retrieval
- **`services/shared_logic`**: Common utilities, types, and database interactions
- **`services/telemetry`**: Logging and metrics collection

### Key Architecture Patterns

1. **Service Composition**: The core service orchestrates other services through dependency injection
2. **Trait-Based Design**: Services implement common traits for testability (e.g., `EmailClient`, `LlmClient`)
3. **Error Handling**: Uses `anyhow` for error propagation and `thiserror` for custom error types
4. **Configuration**: Centralized configuration management through `shared_logic/config.rs`
5. **Async/Await**: All services use async Rust with tokio runtime

### Service Dependencies

- **Core** depends on: email, llm, memory, shared_logic, telemetry
- **Email** depends on: shared_logic
- **LLM** depends on: shared_logic
- **Memory** depends on: shared_logic
- **Telemetry** is standalone
- **Shared Logic** is the foundation for all services

## Key Files and Locations

### Entry Points
- `services/core/src/main.rs` - Main application entry point
- `services/core/src/lib.rs` - Core service public API
- `services/core/src/workflow.rs` - Email processing workflow logic

### Configuration
- `services/shared_logic/src/config.rs` - Configuration management
- `config/prompts.yaml` - LLM prompts configuration

### Testing
- `services/core/src/test_utils.rs` - Mock implementations for testing
- `services/memory/tests/` - Integration tests for memory service

## Development Workflow

1. **Adding New Features**: Start by updating the relevant service's `lib.rs` and implementing the trait
2. **Testing**: Use mock implementations in `test_utils.rs` for unit tests
3. **Integration**: Wire services together in `services/core/src/workflow.rs`
4. **Configuration**: Add new config options to `shared_logic/src/config.rs`

## Important Notes

- All services use structured logging with `tracing`
- Database operations are abstracted through repository pattern in memory service
- Email operations support both real SMTP and mock clients for testing
- LLM integration is currently configured for DeepSeek API
- The project uses file-based persistence for the memory service