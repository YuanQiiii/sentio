# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- **Build all services**: `cargo build --workspace`
- **Run all tests**: `cargo test --workspace`
- **Run tests for a specific service**: `cargo test -p <service-name>` (e.g., `cargo test -p sentio_memory`)
- **Run the main application**: `cargo run -p sentio_core`
- **Run in watch mode for development**: `cargo watch -x "run -p sentio_core"`
- **Check for linting issues**: `cargo clippy --workspace -- -D warnings`
- **Format code**: `cargo fmt --workspace`

## Code Architecture

This project is a Rust-based AI email assistant with a microservices architecture. The services are organized as a Cargo workspace.

- **`sentio_core`**: The main service that orchestrates the other services.
- **`sentio_memory`**: Manages user memory, interaction history, and profiles using MongoDB.
- **`sentio_llm`**: Integrates with Large Language Models (LLMs) like DeepSeek for generating responses.
- **`sentio_email`**: Handles sending emails via SMTP.
- **`sentio_telemetry`**: Provides logging and tracing for observability.
- **`shared_logic`**: Contains shared code for configuration, database connections, and data types used across multiple services.

## Configuration

- **Environment Variables**: Sensitive information like API keys and database URLs are configured through a `.env` file. Refer to `.env.example` for the required variables. Environment variables override configuration files.
- **Prompts**: LLM prompts are managed in `config/prompts.yaml`. This allows for easy modification of prompts without changing the code.
