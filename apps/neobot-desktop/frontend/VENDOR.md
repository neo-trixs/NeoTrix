# 前端来源与许可

本目录（`apps/neobot-desktop/frontend/`）的代码 **1:1 取自**：

| 项 | 值 |
|---|---|
| 项目 | [dsh-tauri/deepseek-harness-desktop](https://github.com/dsh-tauri/deepseek-harness-desktop) |
| 版本 | **0.19.1**（用户提供的本地件 `/Users/neo/Downloads/deepseek-harness-desktop-0.19.1`） |
| 许可 | **MIT License** |
| 版权 | Copyright (c) 2026 deepseek-harness-desktop contributors |
| 引入日期 | 2026-09-30 |
| 引入理由 | 用户指示：「把这个代码吃透，使用这个原代码来完善构建，移除之前的脏乱代码」 |

## 保留了哪些（上游原件，逐字）

`src/`（136 文件 / 15,719 行）· `packages/`（15 个包 / 774 个 ts）· `public/` ·
`assets/` · `patches/` · `test/` · `types/` · `scripts/` · 全部根配置文件
（含 `tailwind.config.js` / `tailwind.plugins.config.js` / `pnpm-workspace.yaml` /
`pnpm-lock.yaml` / `vite.config.ts` / 4 个 `vitest.*.config.ts`）·
`LICENSE` · `LICENSE.details` · `THIRD_PARTY_NOTICES.md` · `AGENTS.md`

⛔ **未改动任何上游源码。** 唯一例外是包名（原 `deepseek-harness-desktop`）保持原样，
因为改名会与上游 lockfile 的 workspace 解析不一致。

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

### ③ 上游前端 invoke 的 80 个 Tauri 命令 —— 尚未接

前端调用 80 个命令（插件管理 / profile 备份 / updater / 安全模式 / 核心下载…），
本仓 Rust 侧只有 12 个 `neobot_*`，**名字零重叠**。
⇒ 这是「后端接口微调」的具体工作量，见 `STATUS.md`。

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
