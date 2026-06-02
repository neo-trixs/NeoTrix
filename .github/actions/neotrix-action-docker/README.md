# NeoTrix Action (Docker)

Run [NeoTrix](https://neotrix.ai) inside its official Docker image — no host toolchain, fully hermetic.

## Usage

```yaml
jobs:
  neotrix-task:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: neotrix/neotrix/.github/actions/neotrix-action-docker@v1
        with:
          prompt: 'Add unit tests for src/lib.rs'
          api-key: ${{ secrets.OPENAI_API_KEY }}
```

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `prompt` | ✅ | — | The task prompt for NeoTrix |
| `api-key` | ✅ | — | LLM API key |
| `model` | ❌ | `''` | LLM model name |
| `provider` | ❌ | `openai` | LLM provider |
| `max-budget-usd` | ❌ | `''` | Budget cap |
| `working-directory` | ❌ | `${{ github.workspace }}` | Mounted into the container as `/workspace` |
| `image` | ❌ | `ghcr.io/neotrix/neotrix:latest` | Docker image reference |
| `version` | ❌ | `latest` | NeoTrix version (installed inside the image) |

## Variants

- **`neotrix-action`** — composite action that downloads the binary via `install.sh`. Smaller & faster, requires Linux x86_64 host.
- **`neotrix-action-docker`** — Docker-container action. Hermetic, works on any runner.

## How it works

1. Pulls the NeoTrix Docker image
2. Mounts `${{ inputs.working-directory }}` as `/workspace` inside the container
3. Runs `neotrix exec --json "<prompt>"` with the supplied API key

## Secrets

Store your API key as a repository / org secret (e.g. `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`).
