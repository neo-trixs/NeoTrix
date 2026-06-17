# CLI Reference

NeoTrix ships with a single `neotrix` binary. Run `neotrix --help` to see all available commands.

## Global Flags

| Flag | Description |
|------|-------------|
| `--provider` | API provider (anthropic, openai, google) |
| `--model` | Model identifier override |
| `--serve` | Start HTTP server instead of interactive session |
| `--addr` | Server address (default `127.0.0.1:3000`) |
| `--config` | Path to config file |
| `--verbose` | Enable verbose logging |
| `--version` | Print version and exit |

## Core Commands

| Command | Description |
|---------|-------------|
| `neotrix` | Start interactive REPL session |
| `neotrix exec "<prompt>"` | Run a single prompt and exit |
| `neotrix tui` | Launch terminal UI (desktop mode) |
| `neotrix --serve` | Start OpenAI-compatible HTTP server |

## Interactive Slash Commands

### Core

| Command | Aliases | Description |
|---------|---------|-------------|
| `/help` | `/h` | Show available commands |
| `/config` | | View or edit current configuration |
| `/model` | | Switch the active model |
| `/stats` | | Display session statistics |
| `/exit` | `/q`, `/quit` | End the session |
| `/clear` | | Clear the screen |
| `/version` | `/v` | Show version info |

### Session Management

| Command | Aliases | Description |
|---------|---------|-------------|
| `/session` | `/s` | Manage sessions (list, switch) |
| `/save` | | Save current session to disk |
| `/load` | | Load a previous session |

### Search & Knowledge

| Command | Aliases | Description |
|---------|---------|-------------|
| `/search` | | Search the knowledge base |
| `/web` | | Perform a web search via configured engine |

### Brain & Evolution

| Command | Aliases | Description |
|---------|---------|-------------|
| `/brain` | | Inspect brain state and metrics |
| `/evolve` | | Trigger a SEAL self-iteration cycle |
| `/absorb` | | Absorb content into the knowledge store |

### Diagnostics

| Command | Aliases | Description |
|---------|---------|-------------|
| `/cost` | | Show token usage and cost estimate |
| `/status` | | Show system status and subsystem health |

## Configuration File

NeoTrix reads configuration from `~/.neotrix/config.toml`. See the [Configuration](/guide/configuration) guide for details.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NEOTRIX_PROVIDER` | Default API provider |
| `NEOTRIX_MODEL` | Default model |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GOOGLE_API_KEY` | Google API key |
| `NEOTRIX_CONFIG` | Config file path override |
| `NEOTRIX_LOG` | Log level (trace, debug, info, warn, error) |
