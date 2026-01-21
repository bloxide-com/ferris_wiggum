# Ralph - Core Engine

The core Ralph autonomous agent system, implementing Geoffrey Huntley's Ralph technique with Rust.

## Features

- Persistent session management
- Token-aware context rotation
- Git operations and commit management
- cursor-agent CLI integration
- Guardrails/Signs learning system
- Stream JSON parsing and signal detection

## Architecture

This crate is server-only and provides the core engine for:
- Managing Ralph sessions
- Spawning and monitoring cursor-agent processes
- Tracking token usage and context health
- Handling git operations
- Learning from failures via guardrails
