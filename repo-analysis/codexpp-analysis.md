# codex-plusplus 架构分析 → NeoTrix Desktop 设计纲领

> 2026-05-28 | 源: https://github.com/b-nnett/codex-plusplus

## 三层架构

```
Loader (injected into app.asar, ~1KB)
  ↓ require()
Runtime (user-dir, hot-reloadable)
  ↓ discover & load
Tweaks (user-dir/tweaks/*/manifest.json + index.js)
```

## 核心设计决策

| 决策 | codex++ 方案 | 为什么 |
|------|-------------|--------|
| 注入点 | 替换 `app.asar` 的 `package.json#main` | 比复制整个 asar (~115MB) 快，只加 ~1KB |
| 预加载 | `session.setPreloads()` 非替换 | 避免覆盖 Codex 自己的 preload |
| UI 注入 | DOM MutationObserver → 检测 Radix `[role="dialog"]` | 比正则替换 minified JS 更健壮 |
| 更新检测 | advisory only，手动审核后更新 | 不给恶意 tweak 自动传播路径 |
| 修补恢复 | 备份原始 asar/plist → `repair` 幂等恢复 | 自愈：检测 hash 漂移后自动重打补丁 |
| 持久化 | 全在 user-dir 内 | 可热重载，不污染 app bundle |

## 对 NeoTrix 的启示

1. **Shell 是薄层** — 桌面端只负责窗口、菜单、托盘；核心逻辑全在 neotrix-core
2. **插件式面板** — 每个 UI 面板是独立模块（manifest + mount/unmount），类似 tweak
3. **外部热重载** — `~/.config/neotrix/panels/*/` 可热插拔
4. **自愈架构** — Brain engine 崩溃自动重启、配置损坏自动回滚
5. **观察而非修补** — UI 层通过 DOM/Event 观察与 core 交互，不修改 core 代码
