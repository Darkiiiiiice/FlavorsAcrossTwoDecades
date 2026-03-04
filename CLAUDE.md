# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**味延廿载 (Flavors Across Two Decades)** is a narrative simulation game where a player on Mars remotely manages a restaurant on Earth through an AI robot named "盼盼 (Panpan)". Key features include:
- Interplanetary communication delay simulation (4-24 minutes)
- AI-driven robot with personality and autonomy
- Recipe collection through travel and experimentation
- Memory fragment recovery system

## Build Commands

```bash
# Build all crates
cargo build

# Build backend only
cargo build --package flavors-backend

# Build release
cargo build --release

# Run backend with config
./target/debug/flavors-backend -c crates/backend/config/default.toml

# Run with custom database path
./target/debug/flavors-backend -c crates/backend/config/default.toml -d "sqlite:path/to/game.db"
```

## Development Commands

```bash
# Run clippy (linting)
cargo clippy --package flavors-backend

# Run tests
cargo test

# Format code
cargo fmt
```

## Architecture

### Workspace Structure
```
crates/
├── backend/    # Game server (Axum + SQLite + Ollama)
└── frontend/   # TUI client (ratatui) - placeholder
```

### Backend Modules
- **api/**: HTTP endpoints (health, saves, websocket)
- **config/**: Settings loaded from TOML files with env override
- **db/**: SQLite database with embedded migrations and seed data
- **game/**: Core game state and logic
- **error.rs**: Unified error handling with thiserror

### Key Design Decisions

1. **Embedded SQL Migrations**: SQL files are embedded at compile time via `include_str!`. No external migrations directory needed at runtime.

2. **Auto-create Database**: If the SQLite database file doesn't exist, it's automatically created using the `sqlite3` command. Parent directories are also created.

3. **LLM Integration**: Uses Ollama for AI responses. Configure model via `[llm]` section in config file.

4. **Configuration Priority**: Config file < Environment variables (prefix: `FLAVORS__`)

## Configuration

Default config at `crates/backend/config/default.toml`:
- Server: host/port
- Database: SQLite URL (supports `sqlite:path.db`, `sqlite:///absolute/path.db`, `sqlite::memory:`)
- LLM: Ollama provider, model name, base URL
- Game: communication delay range, auto-save interval

## Database

- Tables: `saves`, `player_configs`, `event_logs`, `game_config`
- Migrations: `crates/backend/migrations/`
- Seed data: 11 default game config entries initialized on first run

## Error Handling

Uses `thiserror` with a hierarchical error system:
- `GameError`: Top-level error with HTTP status mapping
- `DatabaseError`: Database-specific errors
- Subsystem errors: `GardenError`, `KitchenError`, `TravelError`, etc.

All errors return structured JSON with: `code`, `message`, `request_id`, `timestamp`
