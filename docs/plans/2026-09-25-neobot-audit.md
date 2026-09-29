# neobot 审计（2026-09-25）：全景 map + 对标最新标准 + 核心建议

规模：Rust 约 9.2k 行（`nt_store` 2068 / `bin/neobot` 1183 / `nt_http_engine` 1043 / `nt_agent` 941）+ 前端约 2.2k 行；单测 63（复审时）。

## 0. 复审（同日第二轮）：P0 验证 + 新增标准 + 新发现

- P0 三件全部在位：`WORD_NEEDLES`（nt_policy.rs:218）/ `BASH_TIMEOUT+ENV_ALLOW`（nt_agent.rs:605）/ `esc` 引号（main.ts:73）/ `valid_model_id`（nt_provider.rs:87）/ `busy_timeout+WAL`（nt_store.rs:117）；`cargo test -p neotrix-neobot --lib` **63 passed**。
- Tauri 仍 2.11.5（CVE-2026-42184 窗外）；CSP/capability（fs 仅 `~/.neobot/**+$TEMP`）未动；IPC 46 命令。
- 自上轮新增的互联网标准：
  - **OWASP Agentic Skills Top 10（AST01–AST10，2026-02 新）**：neobot skills 当前是惰性元数据（`nt_skills.rs` 只解析 name/description，未接入 agent 工具门）——误打误撞符合最小权限；一旦给 skills 接工具，须按 AST03（过度授权）/AST06（弱隔离）重审。
  - **OWASP AI Agent Security Cheat Sheet（滥用测试矩阵）**：prompt 覆盖/工具滥用/提权/记忆投毒/外泄/递归滥用/审批绕过——建议把现有 63 单测对号入座，缺口进 CI。
  - **ASI10 Rogue Agents**（2026 定稿新增第10项）：单 agent 本地设计天然限制漂移半径，记录即可。
- 新发现 **F8 [高]**：`shell:allow-open` 默认 scope + 前端 `open(p.url)`（main.ts:999，画布"外部打开"）对用户手输 URL **无协议校验**；对照 **CVE-2025-31477**（plugin-shell 协议校验不当 RCE，当前 plugin-shell 2.3.6）。`file://`/自定义 scheme 直达系统 handler。修法：调用前白名单 `https?://`（5 行），与插件版本无关、立刻做。
  - ✅ 同日已修：`open` 前 `/^https?:\/\//i` 校验，非 http(s) toast 拒（`main.ts`）；`window.open` 回退同享此门。

## 1. 全景 map

```
┌─ 桌面 App (Tauri 2.11.5, mac) ─────────────────────────────┐
│ main窗口(index.html) ←→ settings窗口 ←→ 右键/弹窗/画板        │
│   vanilla TS, 双 localStorage 命名空间, 事件同步              │
│   iframe 网页节点: sandbox(allow-scripts+same-origin+…)       │
│                        ↕ IPC (invoke/em events)               │
│ Rust core: nt_commands (60+命令, spawn_blocking 跑 agent)      │
└────────────────────────┬───────────────────────────────────┘
                         │ 同一 SQLite 文件 ~/.neobot/neobot.db
┌─ CLI (bin/neobot) ─────┴── 无 busy_timeout / journal 默认 ──┐
│ task/convo/provider/memory/attach/routine/skill/policy       │
└────────────────────────────────────────────────────────────┘
┌─ Agent (nt_agent) ───────────────────────────────────────────┐
│ prompt[MEMORY.md+skills+转录] → 模型池(http/openai兼容)        │
│   → toolcalls: bash/read/write/edit/computer(noop)           │
│   → 网关: policy(纯reducer,deny优先/缺省拒) → 先审计后执行      │
│   → 账本(Claude官方价)+预算(写/转录/tokens)+outbox            │
└────────────────────────────────────────────────────────────┘
信任边界: WebView(不可信, 远端iframe/模型输出流入) | IPC | Rust(全权) | 模型提供方(半可信, 可见prompt+工具回显)
```

## 2. 对标（2026 最新）

- **OWASP Agentic Top-10 2026**（ASI01 Goal Hijack / ASI02 Tool Misuse / ASI03 身份越权 / ASI05 RCE / ASI06 记忆投毒 / ASI08 级联故障）+ **LLM Top-10 2026**（LLM01 注入 / LLM06 敏感泄密 / LLM10 无界消耗）+ **Agent Control Standard**（运行时治理、HITL、可审计）。
- **Tauri 安全模型**：capability 最小域 + CSP + 不打包 WebView；**CVE-2026-42184**（`is_local_url`  origin 混淆，Win/Android）已在 2.10.3/2.11.1 修。
- **local-first 七理想**（Ink & Switch）：无转圈 / 数据不被绑架 / 网络可选 / 无缝协作 / 长期保存 / 隐私默认 / 用户拥有。

## 3. 已达标（保持）

- Tauri **2.11.5**：CVE-2026-42184 不在影响窗；capability 仅 `~/.neobot/**+$TEMP` 读写；CSP `default-src 'self'` + `object-src 'none'`；capability 无 `remote` → 远端 iframe **调不到 IPC**（CVE-2024-35222 类免疫）。
- fail-closed 网关 + 审计先行 + key 只存变量名 + 账本实测价 + 三级写预算/转录预算（LLM10 缓解）+ policy 纯函数可测 + 审批队列设计（ACS 的 HITL）。

## 4. 发现（按严重度）

| # | 严重 | 位置 | 问题 | 对标 |
|---|---|---|---|---|
| F1 | 高 | `nt_policy.rs:200` `looks_like_escape` | 子串启发式（`..`/`~`/`/etc/`）一捅就穿：绝对路径（`cat /Users/x/.ssh/id_rsa`）、`cd $HOME`、`env`、`curl … \| bash` 全放行；且步骤4对 bash 直接 `return Allow`，步骤5 的 default-deny 对 bash 不可达 | ASI02/LLM08 |
| F2 | 高 | `nt_agent.rs:601` `execute_bash` | `bash -c` 无超时（`.output()` 死等，`sleep 999` 卡死整轮）+ 全环境变量继承（`env` 回显进模型 → key 送提供方） | LLM06/LLM10 |
| F3 | 中 | `main.ts:71` `esc()` | 不转义 `"`：`data-mpick="${esc(prov)}"`（477）、iframe `src="${esc(p.url)}"`（1234）等 26 处内 HTML；模型 id 来自**远端池**，可构造 stored-XSS → 主窗口全 IPC | LLM02 |
| F4 | 中 | `nt_store.rs:113` `open()` | 无 `busy_timeout`、无 WAL：CLI 与 App 双进程同库，并发写即 `database is locked` | 稳定性 |
| F5 | 中 | — | 无导出/备份：local-first 第5条（长期保存）缺失；附件散盘 + 单 db，无快照 | local-first#5 |
| F6 | 低 | `main.ts:1234` iframe | `allow-same-origin+allow-scripts` 组合允许页内去沙盒（仅限自身）；展示型预览不需要 same-origin | 纵深防御 |
| F7 | 低 | prompt 组装 | MEMORY/skills/转录拼进 prompt 无来源标签边界（现均为用户本地源，风险低） | ASI06/LLM01 |

## 5. 核心建议（排序即优先级）

**P0（本周，半天量）—— 2026-09-25 已修复验证**
1. **bash 关笼子** ✅：`looks_like_escape` 改整词+路径双层（`nt_policy.rs`；`cd`/`env`/`curl`/`ssh`/绝对敏感根全拒，单测覆盖 10 攻 4 守）；`execute_bash` 60s 超时杀（`sleep 70` 实测 60.01s 击杀）+ `env_clear` 白名单（`PATH/HOME/USER/LANG/TMPDIR…`，`NEOBOT_TEST_ONLY_SECRET` 实测不透）；残余：启发式非真沙盒（OS 级隔离列 P1）。
2. **`esc()` 加引号** ✅ + 模型 id 入池格式校验（`[A-Za-z0-9._:/-]{1,128}`，`nt_provider.rs:valid_model_id`，脏串丢弃，单测覆盖）。
3. **SQLite `busy_timeout(5s)` + WAL** ✅（`nt_store.rs:open`）。

**P1（下轮）**
4. `neobot export`：db + 附件 + MEMORY 打包 zip；启动时若 db 缺失提示恢复。（堵 F5）
5. iframe 去掉 `allow-same-origin`（纯展示不需要）；`cargo audit` + `npm audit` 进发版前检查清单。（F6+供应链）
6. prompt 注入块加 `<local-memory>/<local-skill>/<transcript>` 来源标签（LLM01 的内外隔离）。（F7）

**P2（声明即可，多设备同步前先写进 README）**：local-first #4（无缝协作）有意不做——单机单库是定位不是缺陷，但要在 README 写明“换机靠 export 搬运”，否则用户会按云笔记心智期待自动同步。
