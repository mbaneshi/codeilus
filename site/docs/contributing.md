# Contributing

## Development Setup

```bash
git clone https://github.com/codeilus/codeilus.git
cd codeilus
cargo build
cargo test
cargo clippy  # must be zero warnings
```

## Project Structure

```
codeilus/
+-- Cargo.toml              # workspace root
+-- crates/                  # 16 Rust crates
+-- frontend/                # SvelteKit 5 + TailwindCSS 4
+-- export-template/         # Vanilla HTML/JS for static export
+-- migrations/              # SQLite schema files
+-- site/                    # This documentation (mkdocs-material)
```

## Code Style

- Zero `cargo clippy` warnings, zero compiler warnings
- `thiserror` for error types, `tracing` for logging
- Async with `tokio`, CPU-parallel with `rayon`
- All DB operations through repository structs
- Events flow through `EventBus` (tokio broadcast)
- Tests use in-memory SQLite (`DbPool::in_memory()`)

## Architecture Rules

- `codeilus-core` is **read-only** &mdash; never modify after Sprint 0
- No cross-dependencies between sibling crates
- All shared types go through `core`
- IDs are i64 newtype wrappers (never raw i64 or UUID)

## Running Tests

```bash
# All tests
cargo test

# Single crate
cargo test -p codeilus-parse

# With logging
RUST_LOG=debug cargo test
```

## Documentation Site

```bash
cd site
uv run mkdocs serve
# Open http://localhost:8000
```

## Pull Requests

- Keep PRs focused on a single change
- Ensure `cargo clippy` and `cargo test` pass
- Update documentation if adding new features
