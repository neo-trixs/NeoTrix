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
**仓库根 `deny.toml`**（2026-09-29 澄清）。

⛔ `config/deny.toml` 曾是同名但内容完全不同的 cargo-deny **官方样板**（含
`[graph]` 等未启用节），已删除。**根目录那份才是真门配置** —— CI
`.github/workflows/deny.yml` 消费它，含全仓多版本台账注释与精确 skip。
两个同名文件并存是「最容易改错」的那类歧义，故只留一份。

## Sections
- `[advisories]` - 安全 advisory 检查
- `[licenses]` - 许可证检查
- `[bans]` - 依赖禁止列表（含多版本台账）
- `[sources]` - 来源检查

## Usage
```bash
cargo deny --config deny.toml check
```
