# NeoTrix 前端设计规范 (Frontend Design Conventions)

## 核心规则: 改前必备份

每次修改 `preview-ui-v2.html` 前，必须执行：

```bash
cp preview-ui-v2.html design/backups/preview-ui-v2.html.bak.$(date +%Y%m%d_%H%M%S)
```

备份文件统一存放于 `design/backups/`。

## 设计迭代工作流

```
1. 备份当前模板  →  design/backups/
2. 在 iterations/ 中创建实验性分支文件进行测试
3. 验证通过后合并到 design/template/preview-ui-v2.html
4. 最后同步到项目根 preview-ui-v2.html
5. 更新 design/CHANGELOG.md
```

## 目录结构

```
design/
├── FRONTEND_CONVENTIONS.md   ← 本文件: 规范与规则
├── CHANGELOG.md              ← 变更日志
├── README.md                 ← 设计概览
├── template/                 ← 主模板 (link to ../preview-ui-v2.html)
│   └── preview-ui-v2.html -> ../../preview-ui-v2.html
├── backups/                  ← 历史备份 (改前必存)
│   └── preview-ui-v2.html.bak.*
├── iterations/               ← 实验性功能分支 (先测试再合入)
│   └── *.html
├── components/               ← 可复用的 UI 组件片段 (HTML + CSS + JS)
│   └── *.html
└── assets/                   ← 设计资源 (字体、图标、配色参考)
    └── *.{svg,png,json}
```

## 设计原则

1. **单文件优先**: 原型阶段保持单 HTML 文件，方便快速迭代
2. **组件化思维**: 每个独立 UI 单元提取到 `components/`，在主模板中内联
3. **CSS 自定义属性**: 所有颜色、间距、圆角使用 `var(--nt-*)` 变量
4. **降级友好**: 后端离线时模版须优雅降级，不白屏
5. **修改前先备份**: 见上方核心规则

## 备份命名规则

```
preview-ui-v2.html.bak.<YYYYMMDD_HHMMSS>
```

## 变更日志格式

每个 `CHANGELOG.md` 条目:

```
### YYYY-MM-DD HH:MM — 简短描述

- 变更: 具体改动
- 文件: preview-ui-v2.html
- 原因: 为什么改
- 备份: preview-ui-v2.html.bak.<timestamp>
```
