# Getting Started

## Installation

### macOS / Linux (supported by `install.sh`)

One-liner (canonical endpoint):

```bash
curl -fsSL https://neotrix.ai/install | bash
```

Raw mirror, with options (`--version`, `--no-modify-path`):

```bash
curl -fsSL https://raw.githubusercontent.com/neo-trixs/NeoTrix/main/install.sh | bash -s -- --version 0.18.0
```

`install.sh` auto-detects OS (macOS/Linux) and architecture (x86_64 / aarch64),
then installs via the first available method, in order: Homebrew → cargo →
pre-built binary → build from source. The binary lands in `~/.neotrix/bin` and
PATH is updated for `bash`/`zsh`/`fish`.

- **macOS** (Apple Silicon `arm64`, Intel `x86_64`): supported. Alternative:
  `brew install neotrix-desktop` for the Desktop app (see below).
- **Linux** (`x86_64`, `aarch64`): supported.
- **Windows**: `install.sh` requires bash — use the MSI/NSIS installer from
  the [releases page](https://github.com/neo-trixs/NeoTrix/releases) instead.

### Windows

Download and run the `neotrix-...-x64.msi` (or `.exe` NSIS) from the
[releases page](https://github.com/neo-trixs/NeoTrix/releases).

### Homebrew (macOS / Linux)

```bash
brew tap neo-trixs/neotrix
brew install neotrix            # CLI
brew install --cask neotrix-desktop   # Desktop app (once cask is published)
```

### Cargo (from source)

```bash
cargo install neotrix
```

### Pre-built Binaries

Download the latest release for your platform from the [releases page](https://github.com/neo-trixs/NeoTrix/releases).

Supported platforms:
- **macOS**: Apple Silicon (arm64) and Intel (x86_64)
- **Linux**: x86_64, aarch64
- **Windows**: x86_64

### Build from Source

```bash
git clone https://github.com/neo-trixs/NeoTrix.git
cd neotrix
cargo build --release
./target/release/neotrix --version
```

## Quickstart

### Set up an API Provider

NeoTrix is model agnostic. Set your preferred provider:

```bash
export NEOTRIX_PROVIDER=anthropic
export ANTHROPIC_API_KEY=sk-ant-...

# Or use OpenAI
export NEOTRIX_PROVIDER=openai
export OPENAI_API_KEY=sk-...

# Or use any OpenAI-compatible provider
export NEOTRIX_PROVIDER=openai
export OPENAI_BASE_URL=https://your-provider.com/v1
export OPENAI_API_KEY=your-key
```

### Run the CLI

```bash
# Start an interactive session
neotrix

# Execute a one-off command
neotrix exec "Explain what E8 reasoning is"

# Start the desktop TUI
neotrix tui
```

### Run a Benchmark

```bash
neotrix bench clawbench
```

Outputs per-trajectory classification (NormalDiffusion / LimitCycle / StrangeAttractor) with confidence scores, action switch rates, and reward variance.

### Start the HTTP Server

NeoTrix exposes an OpenAI-compatible API endpoint, so you can use it as a drop-in provider for any tool that supports custom OpenAI endpoints:

```bash
neotrix --serve --addr 0.0.0.0:3000

# Configure OpenCode to route through NeoTrix
opencode config set provider openai
opencode config set openai-base-url http://localhost:3000/v1

# Or with Aider
aider --openai-api-base http://localhost:3000/v1
```

This means NeoTrix handles the reasoning and knowledge layers, while OpenCode/Aider handle file I/O and diff application.

### Desktop App

```bash
# Install the desktop app
brew install neotrix-desktop

# Launch
neotrix-desktop
```

Or build from source:

```bash
cd src-tauri/frontend && npm install && npm run build
cargo build -p neotrix-tauri
./target/release/neotrix-desktop
```

## Next Steps

- Read the [CLI Reference](/guide/cli) for all available commands
- Learn about the [Desktop App](/guide/desktop) features
- Configure NeoTrix to your needs with [Configuration](/guide/configuration)
- Contribute to development with the [Development Guide](/guide/development)
- Explore the [API Reference](/api/overview) for integration
