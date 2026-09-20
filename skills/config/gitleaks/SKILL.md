# Gitleaks Configuration

## Purpose
Secret 扫描配置

## Trigger Words
- gitleaks
- secret scanning
- 密钥扫描
- 安全扫描

## File
`config/.gitleaks.toml`

## Configuration
- 使用默认规则作为基础
- 白名单特定文件/路径
- 自定义检测规则

## Usage
```bash
gitleaks detect --config .gitleaks.toml
```
