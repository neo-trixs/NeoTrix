# Development

## Prerequisites

- **Rust**: 1.79 or later (install via [rustup](https://rustup.rs/))
- **System**: macOS or Linux (Windows with WSL2)
- **Node.js**: 18+ (for Tauri desktop builds)
- **pnpm**: Recommended for frontend dependencies

## Building from Source

```bash
git clone https://github.com/neotrix/neotrix.git
cd neotrix

# Build the CLI
cargo build --release

# Run tests
cargo test --lib

# Run all tests including integration
cargo test
```

## Workspace Structure

The NeoTrix monorepo is organized into several crates:

```
core/                   # Core engine: E8, VSA, HyperCube
  nt_core_consciousness/   # Self-model, attention, metrics
  nt_core_e8/              # E8 reasoning engine
  nt_core_hcube/           # VSA HyperCube knowledge store
  nt_core_knowledge/       # Knowledge base and versioning
  nt_core_meta/            # Meta-cognition KPI

neotrix/                # Binary entrypoint (CLI + server)
  nt_core_negentropy/      # Negentropy metric sensors
  nt_memory_crawl/         # Web crawling pipeline

nt_mind/                # Cognitive subsystems
  nt_mind_ingestion/       # Knowledge ingestion pipeline

nt_memory/              # Memory and storage

nt_act_goal/            # Goal management

nt_shield/              # Safety and constraints

nt_world/               # World interaction (search, crawl)

self_iterating/         # SEAL evolution pipeline
```

### Key Crates

| Crate | Purpose |
|-------|---------|
| `core/*` | Foundation: E8 reasoning, VSA, consciousness, metrics |
| `neotrix` | Binary: CLI parsing, server, session management |
| `nt_mind` | High-level cognition: curiosity, theory of mind |
| `self_iterating` | SEAL pipeline: self-modification, meta-learning |

## Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p neotrix --lib

# Run tests with output
cargo test -- --nocapture
```

NeoTrix has **3240+ tests** — all passing. Tests are organized alongside source modules. Integration tests live in `tests/` at the workspace root.

## Code Style

- **Rust edition**: 2021
- **Unsafe code**: Forbidden in core crates (`#![forbid(unsafe_code)]`)
- **Formatting**: `cargo fmt` (rustfmt)
- **Linting**: `cargo clippy -- -D warnings`

## Contributing

1. Check [open issues](https://github.com/neotrix/neotrix/issues) for tasks
2. Fork the repository and create a feature branch
3. Make changes with tests
4. Run `cargo test` and `cargo clippy` locally
5. Submit a pull request

### Pull Request Guidelines

- Keep PRs focused on a single concern
- Include test coverage for new functionality
- Update documentation if behavior changes
- Ensure no new `unsafe` code in core crates
- Run `cargo fmt` before committing

## Documentation

This VitePress site lives in `docs/`. To run it locally:

```bash
cd docs
pnpm install
pnpm run dev
```

## CI/CD

The project uses GitHub Actions:
- **CI**: Build + test on every PR
- **Docs**: Deploys to GitHub Pages on push to main
- **Release**: Builds binaries on tag push
