# Node.js Package Configuration

## Purpose
Node.js 依赖管理

## Trigger Words
- package
- npm
- node.js
- 依赖管理
- playwright
- e2e

## File
**`skills/e2e/package.json`**（2026-09-29 重新指向）。

⛔ `config/package.json` 曾存在但**零引用**（无 scripts、仅 1 个依赖
`@ai-sdk/openai-compatible`、无任何 CI/Makefile/scripts 消费），已于
2026-09-29 连同其 lockfile 删除。真实的 Node 依赖清单在 `skills/e2e/package.json`，
配套 `playwright.config.ts`，带 `test` / `test:headed` 脚本。

## Dependencies
- `@playwright/test`: 端到端测试

## Usage
```bash
cd skills/e2e && npm install
cd skills/e2e && npm test
```

## Integration
端到端测试套件。Playwright 驱动，spec 与配置同目录：`skills/e2e/cli-smoke.spec.ts`、
`diag.spec.ts`、`playwright.config.ts`。
