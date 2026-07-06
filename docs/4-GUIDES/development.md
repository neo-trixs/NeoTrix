# Development

> This page is under development.

## Build

```sh
cargo build -p neotrix              # CLI
cargo build -p neotrix-tauri        # Desktop
cargo check --features full --lib -p neotrix  # Full check
```

## Test

```sh
cargo test -p neotrix --lib
npm test                     # from src-tauri/frontend
scripts/test-all.sh          # full suite
```

## Code Conventions

See `AGENTS.md` at the project root for full conventions.
