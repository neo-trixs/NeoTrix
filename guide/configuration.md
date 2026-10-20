# Configuration

NeoTrix reads configuration from `~/.neotrix/config.toml`. This file is created automatically on first run with default values.

## File Location

| Platform | Path |
|----------|------|
| macOS | `~/.neotrix/config.toml` |
| Linux | `~/.neotrix/config.toml` |
| Windows | `%USERPROFILE%\.neotrix\config.toml` |

Override with the `--config` flag or `NEOTRIX_CONFIG` environment variable.

## Example Config

```toml
[provider]
name = "anthropic"
model = "claude-sonnet-4-20260514"

[provider.keys]
anthropic = "sk-ant-..."
openai = "sk-..."
google = "..."

[budget]
max_cost_per_session = 0.50        # USD
max_tokens_per_response = 4096
max_tokens_per_session = 100000

[behavior]
verbose = false
auto_save = true
auto_evolve = true
evolve_interval_minutes = 30

[search]
engine = "duckduckgo"
max_results = 8

[server]
host = "127.0.0.1"
port = 3000

[theme]
mode = "dark"                       # light, dark, auto
```

## Setting Reference

### `[provider]`

| Key | Default | Description |
|-----|---------|-------------|
| `name` | `anthropic` | API provider: `anthropic`, `openai`, `google` |
| `model` | — | Model identifier; overrides provider default |

### `[provider.keys]`

Store API keys here as TOML key-value pairs. Keys can also be set via environment variables (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY`).

### `[budget]`

| Key | Default | Description |
|-----|---------|-------------|
| `max_cost_per_session` | `0.50` | Cost cap per session (USD) |
| `max_tokens_per_response` | `4096` | Max tokens per single response |
| `max_tokens_per_session` | `100000` | Max cumulative tokens per session |

### `[behavior]`

| Key | Default | Description |
|-----|---------|-------------|
| `verbose` | `false` | Enable verbose logging |
| `auto_save` | `true` | Auto-save session on exit |
| `auto_evolve` | `true` | Run SEAL evolution periodically |
| `evolve_interval_minutes` | `30` | Minutes between evolution cycles |

### `[search]`

| Key | Default | Description |
|-----|---------|-------------|
| `engine` | `duckduckgo` | Search engine for `/web` command |
| `max_results` | `8` | Max results per search |

### `[server]`

| Key | Default | Description |
|-----|---------|-------------|
| `host` | `127.0.0.1` | HTTP server bind address |
| `port` | `3000` | HTTP server port |

### `[theme]`

| Key | Default | Description |
|-----|---------|-------------|
| `mode` | `dark` | UI theme: `light`, `dark`, `auto` |

## Environment Variables

Environment variables take precedence over config file values:

- `NEOTRIX_PROVIDER` — overrides `[provider].name`
- `NEOTRIX_MODEL` — overrides `[provider].model`
- `ANTHROPIC_API_KEY` — overrides `[provider.keys].anthropic`
- `OPENAI_API_KEY` — overrides `[provider.keys].openai`
- `GOOGLE_API_KEY` — overrides `[provider.keys].google`
- `NEOTRIX_CONFIG` — overrides config file path
