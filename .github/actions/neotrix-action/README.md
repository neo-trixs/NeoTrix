# NeoTrix Action

Run [NeoTrix](https://neotrix.ai) — the AI-native developer toolkit — in your GitHub Actions CI/CD pipelines.

## Usage

```yaml
jobs:
  neotrix-task:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: neotrix/neotrix/.github/actions/neotrix-action@v1
        with:
          prompt: 'Refactor the error handling to use a custom Result type'
          api-key: ${{ secrets.OPENAI_API_KEY }}
```

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `prompt` | ✅ | — | The task prompt for NeoTrix |
| `api-key` | ✅ | — | LLM API key (e.g. OpenAI, Anthropic) |
| `model` | ❌ | `''` | LLM model to use |
| `provider` | ❌ | `openai` | LLM provider (`openai`, `anthropic`, etc.) |
| `max-budget-usd` | ❌ | `''` | Maximum budget in USD |
| `working-directory` | ❌ | `${{ github.workspace }}` | Working directory |
| `version` | ❌ | `latest` | NeoTrix version to use |

## Examples

### Basic — execute a task

```yaml
- uses: neotrix/neotrix/.github/actions/neotrix-action@v1
  with:
    prompt: 'Add input validation to all public API endpoints'
    api-key: ${{ secrets.OPENAI_API_KEY }}
```

### With a specific model and budget cap

```yaml
- uses: neotrix/neotrix/.github/actions/neotrix-action@v1
  with:
    prompt: 'Write unit tests for the auth module'
    api-key: ${{ secrets.ANTHROPIC_API_KEY }}
    provider: anthropic
    model: claude-sonnet-4-20250514
    max-budget-usd: '2.00'
```

### On a PR — auto-fix lint issues

```yaml
name: NeoTrix Auto-Fix
on:
  pull_request:
    paths: ['src/**']

jobs:
  auto-fix:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: neotrix/neotrix/.github/actions/neotrix-action@v1
        with:
          prompt: 'Fix all clippy warnings in src/'
          api-key: ${{ secrets.OPENAI_API_KEY }}
      - uses: peter-evans/create-pull-request@v6
        with:
          commit-message: 'chore: auto-fix clippy warnings'
          branch: neotrix-fix
```

## Secrets

Store your API key as a [GitHub Actions secret](https://docs.github.com/en/actions/security-guides/using-secrets-in-github-actions) (e.g. `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`) and reference it with `${{ secrets.XXX }}`.

## How it works

1. Downloads the `neotrix` CLI binary via the official install script
2. Writes a minimal `~/.config/neotrix/config.toml` with your provider
3. Runs `neotrix exec --json "<prompt>"` in the checked-out repository
