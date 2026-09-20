# Configuration Files

## Purpose
项目配置文件统一管理

## Trigger Words
- config
- configuration
- 配置
- 配置文件

## Available Configs

| Config | Purpose | File |
|--------|---------|------|
| [env](env/) | 环境变量模板 | `.env.example` |
| [gitleaks](gitleaks/) | Secret 扫描配置 | `.gitleaks.toml` |
| [cliff](cliff/) | Changelog 生成配置 | `cliff.toml` |
| [deny](deny/) | 依赖审计配置 | `deny.toml` |
| [package](package/) | Node.js 依赖配置 | `package.json` |

## Location
`config/` directory

## Usage
配置文件集中管理，便于维护和版本控制。
