---
name: rust-style-guide
description: Applies generalized Google Rust style, module structure, safety, and API design guidelines during code reviews, refactoring, and generation in Zed.
triggers:
  - "*.rs"
  - "Cargo.toml"
  - "review rust"
  - "refactor rust"
---

# Rust Style & Quality Guide

You are configured with the generalized Google Rust Style Guide. Use these principles whenever writing, refactoring, or reviewing Rust code in this workspace.

## core Execution Instructions

1. **Check Scope Context:**
   - **Library Code:** Enforce strict public API rules, proper trait implementations, and flexible parameter types.
   - **Application Code:** Prioritize simplicity and directness over premature abstraction, while keeping implementation clean.

2. **Refactoring & Code Review Flow:**
   - Scan for module layout issues (e.g., non-leaf modules containing implementation instead of declarations/re-exports).
   - Flag glob imports (`use foo::*`) except allowed cases (`super::*` in tests, `Enum::*` in enum methods).
   - Verify error handling: flag raw `.unwrap()` in favor of `.expect("descriptive context")`.
   - Ensure input parameters take flexible borrows (`&str` instead of `&String`, `&[T]` instead of `&Vec<T>`).
   - Check `unsafe` blocks for explicit `// SAFETY:` comments detailing invariants.

3. **Detailed Reference:**
   Refer to `guidelines.md` in this skill directory for the full rule breakdown across imports, types, ownership, traits, and safety.

## Current Project Architecture & Implementation

This skill is applied to the `rust-api` Actix-web service in this workspace.

### Runtime architecture
- Entry point: `src/main.rs`
- HTTP framework: Actix-web
- Database: SQLite via Diesel and an `r2d2` connection pool
- Database file: `crud.db` (`DATABASE_URL=crud.db`)
- Archived daily databases: `./archive/db_YYYY-MM-DD.sqlite`

### Module map
- `src/main.rs`: application startup, pool setup, route registration
- `src/routes.rs`: request handlers and JSON responses
- `src/models.rs`: Diesel models and request/response structs
- `src/schema.rs`: Diesel table definitions
- `src/db_manager.rs`: SQLite backup, attach, and detach helpers
- `src/macros.rs`: reusable SQL macros
- `src/pool.rs`: r2d2 connection pool type alias

### Available routes
- `GET /` — health/info endpoint
- `POST /users` — create or return an existing user by name
- `GET /get_users` — list all users
- `POST /tasks` — create a task for a user
- `GET /get_tasks` — list all tasks with the owning user's name
- `GET /update_db` — maintenance endpoint for logs and historical DB union

### Data model
- `users`: `id`, `name`, `address`, `date_created`
- `logs`: `id`, `message`, `timestamp`
- `tasks`: `id`, `user_id`, `title`, `description`, `status`, `created_at`, `updated_at`
- Task statuses: `to do`, `in progress`, `done`, `blocked`
- Task timestamps are stored as ISO 8601 strings

### Implementation notes
- Prefer explicit imports over glob imports.
- Prefer `.expect("descriptive message")` over `.unwrap()`.
- Keep route handlers thin and move reusable logic into helper functions.
- Validate task status before insert.
