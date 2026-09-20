# Git-Cliff Configuration

## Purpose
Changelog 自动生成配置

## Trigger Words
- cliff
- changelog generation
- 变更日志生成
- release notes

## File
`config/cliff.toml`

## Configuration
- Header: "# Changelog\n\n"
- Body: 按 commit group 分组
- Footer: 自动添加贡献者

## Usage
```bash
git-cliff --config cliff.toml > CHANGELOG.md
```

## Integration
GitHub Actions workflow `.github/workflows/release.yml` 使用此配置。
