# 前端来源与许可

本目录（`apps/neobot-desktop/frontend/`）的代码 **1:1 取自**：

| 项 | 值 |
|---|---|
| 项目 | [dsh-tauri/deepseek-harness-desktop](https://github.com/dsh-tauri/deepseek-harness-desktop) |
| 版本 | **0.19.1**（用户提供的本地件 `/Users/neo/Downloads/deepseek-harness-desktop-0.19.1`） |
| 许可 | **MIT License + 附加条款「No Commercial Secondary Development」（非纯 MIT，见下）** |
| 版权 | Copyright (c) 2026 deepseek-harness-desktop contributors |
| 引入日期 | 2026-09-30 |
| 引入理由 | 用户指示：「把这个代码吃透，使用这个原代码来完善构建，移除之前的脏乱代码」 |

## ⛔ 许可：不是纯 MIT（2026-09-30 更正）

上游在 `LICENSE`（MIT 正文）之外另附 `LICENSE.details`，构成**附加条款**：

> **Additional Terms — No Commercial Secondary Development**
> 1. 不得将本软件用于**二次开发**（含修改、改编、衍生）以获取商业利益、
>    金钱报酬，或作为付费商业产品/服务的一部分。**直接使用**本软件从事
>    商业目的仍被允许。
> 2. **本附加条款与 MIT 冲突时，以本附加条款为准。**

⇒ **三点必须让所有后来者知道：**

1. 本目录**正在被修改**（见下方增量表），按该条款字面即落入
   「secondary development」的定义 ⇒ 若 NeoTrix 走商用，构成**未授权的
   商用二次开发**。
2. 「直接使用允许商用」与「修改后不得商用」的分界**在本仓已经跨过**：
   我们不是原样分发，而是持续改动。
3. 本条**不构成法律意见**，是需要你（项目所有者）做商业判断的**待决项**。
   可选处置（按侵入性从低到高）：
   - 确认本项目/下游分发**非商业** ⇒ 现状可接受，仅需保留本节记录；
   - 商用 ⇒ ①向上游取得书面授权；或 ②移除本 vendored 树，只保留「读懂后的
     自研实现」（注意：本仓多处逻辑派生自它，移除是**架构级**决定，不是删目录）；
   - ⛔ **不可接受的做法**：改门脚本让 `check-license.sh` 变绿。

门：`bash scripts/check-license.sh`（G6，校验本节与实际条款一致，不判法律）。

## 保留了哪些（上游原件，逐字）

`src/`（133 文件 / 15,047 行）· `packages/`（14 个包 / 694 个 ts 源码，
另有 `dist/` 构建产物）· `public/` ·
`assets/` · `patches/` · `test/` · `types/` · `scripts/` · 全部根配置文件
（含 `tailwind.config.js` / `tailwind.plugins.config.js` / `pnpm-workspace.yaml` /
`pnpm-lock.yaml` / `vite.config.ts` / 4 个 `vitest.*.config.ts`）·
`LICENSE` · `LICENSE.details` · `THIRD_PARTY_NOTICES.md` · `AGENTS.md`

## NeoBot 增量（与上游的 diff，改完即记）

| 文件 | 改动 |
|---|---|
| `src/neobot-root.tsx` + `src/api-panel.ts` + `src/api-panel.css` + `src/dom.ts`（新增） | 自持对话区 + API 契约面板 |
| `src/layout/components/webview.tsx` | `selfHosted` 时渲染自持根而非 iframe |
| `src/store/modules/harness/store.ts` | `selfHosted` 状态；自持时跳过 DSH 整段 |
| `src/ui/dialog/config.tsx` | 加 `api` 页签 |
| `index.html` / `pet.html` | 标题 → NeoBot / NeoBot Pet |
| `src/ui/dialog/about.tsx` | 兜底文案 → NeoBot |
| `src/i18n/locales/{zh-CN,en-US}.json` | 4 处 `DeepSeek Harness` → NeoBot（`app.wordmark` 不动，测试锁了它） |
| `public/favicon.svg` | 黑豚剪影（派生，注释内已注来源；与 dock 彩豚分工：chrome 用单色） |

⛔ 除上表外未动。`dsh://` 协议名、`DSH_HOME`、`dsh-tauri` 桥名是标识符，不是品牌，一律不动。
包名保持 `deepseek-harness-desktop`（改名会与 lockfile 的 workspace 解析不一致）。

## 没取的部分，以及为什么

### ① `src-tauri/`（上游 Rust 侧）—— **不要**

那是 deepseek-harness 的 Rust 实现（进程管理、profile、updater、插件安装器）。
本仓的 Rust 侧是 `apps/neobot-desktop/src/`，且 `crates/neotrix-neobot` 是
**更完整的超集**（比上游对应库多 4 个模块：nt_panel / nt_pdf_ground / nt_qwen_mm /
nt_sidebar，30,091 行 vs 26,581 行，466 个测试全绿）。
⇒ 换过去是**倒退**，且会丢掉本仓已验证的决策面板基准校验。

### ② `source/`（git submodule，当前为空）—— 取不到

`source/deepseek-harness` 是**对话界面本体**。上游自己的架构文档写明
「Most of the product UI remains upstream Harness」——
它的主界面是一个 `<iframe>`，指向 Harness Node 运行时起的 `service_url`
（`src/layout/components/iframe.tsx` → `store.harness.iframeSrc`）。
那个运行时不在本仓，submodule 也没拉。

⇒ **仓里真实存在的外壳**（导航 / 设置对话框 / 工作台面板 / 恢复页 /
BongoCat 宠物 / 10 个插件）已 1:1 到位；**对话区要自己写**（用户已确认这个取舍）。

### ③ 上游前端 invoke 的命令 —— 已逐条登记（2026-09-30）

上游 Rust 侧 96 个命令（`src-tauri/desktop/builder.rs` 的 `generate_handler!`），
本仓 `src/api.rs` 已逐条登记为 Implemented / Stub / Planned，无未展开项。
分母不是「前端调了几个」（没调≠缺口），对账门见 `nt_check_api.mjs`。

## 构建

```sh
cd apps/neobot-desktop/frontend
pnpm install        # 16 个 workspace 项目
pnpm build:plugins  # 先构建 packages/*，否则 dsh-tauri/client 解析不到 dist/
pnpm build          # tsc && vite build
```

⚠️ `pnpm build:plugins` **不能跳过**：`packages/*` 的 `exports` 指向 `dist/`，
不先构建则 854 个 `TS2307`。`pnpm build` 自带 `prebuild` 钩子会跑它。

⚠️ **不要手挑文件列表拷贝**。我第一次手挑，漏了 `tailwind.plugins.config.js`，
导致 Tailwind 产出空 CSS（`TAILWIND_EMPTY_OUTPUT`）而 `dsh-tauri-ui` 构建失败。
⇒ 现在是「根目录全量拷入 + 用脚本逐项比对上游」，漏拷项为 0。
