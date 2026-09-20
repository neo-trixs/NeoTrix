# Cargo-Deny Configuration

## Purpose
依赖审计和许可证检查

## Trigger Words
- deny
- cargo-deny
- dependency audit
- 依赖审计
- 许可证检查

## File
`config/deny.toml`

## Sections
- `[graph]` - 依赖图配置
- `[advisories]` - 安全 advisory 检查
- `[licenses]` - 许可证检查
- `[bans]` - 依赖禁止列表
- `[sources]` - 来源检查

## Usage
```bash
cargo deny --config deny.toml check
```
