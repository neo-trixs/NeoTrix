# OpenGhost 吸收来源与许可边界

本目录（`apps/neobot-desktop/neobot-ui/src/vendor/openghost/`）的三个文件
**逐字取自** `/Users/neo/Downloads/OpenGhost-1.2.0`（上游 `ANDRETRIPOL/OpenGhost` v1.2.0）：

| 文件 | 上游 | md5（入库时） | 用途 |
|---|---|---|---|
| `markdown.js` | 根目录同名 | `0b7576b6a31d5de48dffd1c99290d98d` | 消息体 markdown → HTML（自转义） |
| `highlight.js` | 根目录同名 | `1648471e71b9618c729ad89541541122` | 代码块关键字高亮（无依赖） |
| `tex.js` | 根目录同名 | `8e82a48c2a07a7358ac0bedd7f635934` | `$…$` / `$$…$$` 公式转 unicode（markdown.js 内部调用，无 MathJax 依赖） |

## 许可（必读，错一条即侵权）

上游 LICENSE 是**双条款**：

- ✅ **MIT（可取）**：源代码 —— agent 引擎、工具、渲染引擎。
  本目录三文件属「渲染引擎」，在 MIT 内。
- ⛔ **非商用保留（禁取）**：名字 OpenGhost、ghost 徽标/图标/动画、
  **视觉设计（布局、颜色、界面）**，且明文含 rebrand。
  ⇒ **只取渲染结构，不取任何样式**：气泡配色、排版、动效一律本仓自写
  （`neobot-root.tsx` 同目录 `nb-markdown.css`），不得打开上游 `styles.css`
  「参考配色」，不得复制动画曲线（`approval-card.js` 的 LEAVE/REVEAL 等）。
- 义务：保留上游版权声明（`Copyright (c) 2026 Andrew`，见各文件头注释
  —— 上游文件本身无头注，此文件即声明载体），不得自称为官方 OpenGhost。

## 取了什么、没取什么

| 取 | 没取（及原因） |
|---|---|
| markdown/highlight/tex 三文件（逐字，md5 锁） | `styles.css`（视觉设计，禁取） |
| 用法：`Markdown.render(text)` 最终消息；`Markdown.blocks` 流式（暂不用） | `diagram.js`（241KB 渲染引擎，先读架构再定） |
| `I18n` 最小垫片（本目录 `shim.ts`，仅 `code.copy` 中英） | 上游 `i18n.js`（18KB 全量词条，暂不需要） |
| | `approval-card.js`（要 Glyphs/I18n/CSS 全套 + 自持区无面板 UI，先欠着） |
| | `liquid-glass.js`、动画、splash、ghost 素材（禁取类） |
| | `agent-tools.js` 风险表（本仓 `nt_policy` 已有同类覆盖，增量再议） |
| | `desktop/keys.js`（Electron safeStorage；本仓等 keyring 插件） |

## 升级/校验

- 上游文件**永不手改**：改即分叉，md5 对不上时先问「是我改的还是上游变的」。
- 复核：`md5 src/vendor/openghost/*.js` 对上表。

## ⛔ 这些文件**必须在版本控制里**（曾被 gitignore 静默排除）

`neobot-root.tsx` 直接 import 本目录（`tex → markdown → highlight` + `shim`），
`.gitignore` 的一条 blanket `vendor/` 曾把整个目录连文件一起排除
⇒ **干净克隆构建不出 `neobot-ui`**（无 prebuild 拷贝步骤）。
实测 `git ls-files apps/neobot-desktop/neobot-ui/src/vendor/` = 0。

根 `.gitignore` 已加 `!apps/neobot-desktop/neobot-ui/src/vendor/` 反忽略。
ⓘ **「视觉设计禁取」不是把代码排除出版本控制的理由** —— 禁取的是上游
`styles.css`/动画/徽标，那些**不在**本目录；本目录只有 MIT 的渲染引擎。
守门：`node scripts/ops/neobot-check-selfcontained.mjs`（断言所有相对 import 都已被 git 跟踪）。
- `markdown.js` 依赖 `window.Tex`（行内公式）与 `window.I18n`（复制按钮 aria）——
  加载顺序必须是 tex → markdown；I18n 由 `shim.ts` 提供（加载顺序任意，早于渲染即可）。
