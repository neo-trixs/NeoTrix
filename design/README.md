# NeoTrix 前端设计目录 (Design Workspace)

此目录用于管理 `preview-ui-v2.html` 的迭代开发和版本管理。

## 快速开始

```bash
# 修改前备份
cp ../../preview-ui-v2.html backups/preview-ui-v2.html.bak.$(date +%Y%m%d_%H%M%S)

# 在迭代分支中实验
cp ../../preview-ui-v2.html iterations/my-feature.html
# ... 修改 iterations/my-feature.html ...

# 验证通过后同步回主模板
cp iterations/my-feature.html ../../preview-ui-v2.html
```

## 目录用途

| 目录 | 用途 |
|------|------|
| `template/` | 指向项目根 `preview-ui-v2.html` 的符号链接 |
| `backups/` | 每次修改前的完整备份 |
| `iterations/` | 实验性功能分支，先测试再合入 |
| `components/` | 可复用 UI 组件片段 |
| `assets/` | 设计资源 (图标、配色等) |
