# Environment Variables

## Purpose
环境变量模板和配置

## Trigger Words
- env
- environment variables
- 环境变量
- .env

## File
`config/.env.example`

## Variables
| Variable | Description | Default |
|----------|-------------|---------|
| NEOTRIX_PROVIDER | LLM provider | openai |
| NEOTRIX_API_KEY | API key | - |
| NEOTRIX_BASE_URL | Base URL | - |
| NEOTRIX_MODEL | Model name | gpt-4o |
| NEOTRIX_EMBEDDING_API_KEY | Embedding API key | - |

## Usage
```bash
cp config/.env.example .env
# Edit .env with your values
```
