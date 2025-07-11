# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sentio is a simplified Rust application for intelligent email processing. It has been restructured from a complex microservices architecture to a single, well-organized crate following the YAGNI principle.

## Development Commands

### Build
```bash
cargo build              # Build the application
```

### Testing
```bash
cargo test               # Run all tests
```

### Running
```bash
cargo run                # Run the application
```

### Linting and Formatting
```bash
cargo fmt                # Format code
cargo clippy             # Run linter
```

## Project Architecture

### Simplified Structure
The project is now organized as a single crate with the following modules:

- **`src/main.rs`**: Application entry point
- **`src/config.rs`**: Configuration management
- **`src/error.rs`**: Error types and handling
- **`src/telemetry.rs`**: Logging initialization
- **`src/workflow.rs`**: Core email processing workflow
- **`src/email/`**: Email client and SMTP handling
- **`src/llm/`**: LLM integration (DeepSeek)
- **`src/memory/`**: Persistent memory storage

### Key Architecture Principles

1. **Single Crate**: Start with one well-organized codebase instead of premature microservices
2. **Module Organization**: Use Rust's module system for logical separation
3. **YAGNI**: No abstractions for "imagined future needs"
4. **Organic Growth**: Let architecture emerge from actual requirements

### Core Components

- **EmailWorkflow**: Orchestrates email analysis and response generation
- **LlmClient**: Interface for LLM interactions (DeepSeek implementation)
- **EmailClient**: SMTP client for sending emails
- **MemoryStore**: Unified memory storage with file persistence

## Key Files and Locations

### Entry Points
- `src/main.rs` - Main application entry point
- `src/workflow.rs` - Email processing logic

### Configuration
- `src/config.rs` - Configuration management
- Environment variables with prefix `SENTIO_`

### Core Modules
- `src/email/client.rs` - Email sending functionality
- `src/llm/client.rs` - LLM integration
- `src/memory/store.rs` - Memory persistence

## Development Workflow

1. **Configuration**: Set environment variables (see config.rs for list)
2. **Testing**: Write tests alongside implementation
3. **Running**: Use `cargo run` to start the application

## Environment Variables

Key environment variables:
- `SENTIO_EMAIL_SMTP_HOST` - SMTP server host
- `SENTIO_EMAIL_SMTP_USERNAME` - SMTP username
- `SENTIO_EMAIL_SMTP_PASSWORD` - SMTP password
- `SENTIO_LLM_API_KEY` - DeepSeek API key
- `SENTIO_LLM_MODEL` - LLM model name

## Important Notes

- The application uses file-based persistence for memory storage
- All modules are designed to be testable with clear interfaces
- Configuration is loaded from environment variables for simplicity
- The project favors clarity over premature optimization