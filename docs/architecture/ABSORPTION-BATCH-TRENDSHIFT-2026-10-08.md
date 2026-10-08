# 吸收批次：Trendshift Daily 榜单（2026-10-08）

> **入参**：`https://trendshift.io/` Daily 榜 25 行 + 赞助位 + live mentions。
> **范式**：NTS-B10 URL-only；零克隆取证；raw 许可核实；四字段模式。
> **原始证据**：本文 §0 + §1（过程文件亦可落 `notes/`，此处直接内联）。

---

## 0. 信号初筛（raw 实测）

| 源 | stars | language | 许可 | 判定 |
|---|---|---|---|---|
| docker/docker-agent | 3,907 | Go | Apache-2.0 | 🟢 强化候选 |
| farion1231/cc-switch | 141,199 | Rust | MIT | 🟢 强化候选 |
| mattpocock/skills | 280,122 | Shell | MIT | 🟢 强化候选 |
| openai/math | 10,617 | Lean | Apache-2.0 | ⚪ intake（README 实测=数学证明稿，非 eval 框架，**修正了 plan 的误判**） |
| storytold/* ×6 | 2.6k–19.9k | Rust | **MIT OR Apache-2.0**（`LICENSE-APACHE`+`LICENSE-MIT` 双件，探针初判「无」为假阴性） | 🟡 设计级吸收 |
| storytold/artcraft | 5,403 | Rust | **ArtCraft License (WIP)** 自定义 | ⛔ 受限，仅取设计 |
| shader-effects-inc/shaders | 2,930 | TypeScript | MIT | 🟡 |
| robbietilton/Compositor | 12,112 | Swift | MIT | 🟡 |
| threerocks/hand-drawn-styles | 1,694 | Python | MIT | 🟡 |
| eternity4719/HowToLiveBetter | 52,429 | HTML | CC-BY-4.0 | ⚪ （内容站） |
| GetBusbar/busbar | 175 | Rust | Apache-2.0 | ⭐ 赞助位观察，不入 |
| 3 已在前两批入库 | — | — | — | 跳过（morluto/rea、yetone/magpie、alchaincyf/huashu-art-motion） |
| 噪声 8 源 | — | — | — | intake |

### ⭐ 本批两条实测教训

1. **`LICENSE-APACHE`+`LICENSE-MIT` 双件是 Rust 生态常态**，探针必须覆盖该文件名
   组合，否则 storytold 全族会被误判「无 LICENSE」→ **已记录为探针补丁建议**
   （`nt_absorption_enrich.py` 未覆盖该组合）。
2. **`openai/math` 是「数学证明稿仓库」不是 eval 框架**——README 第一行即真相。
   计划阶段「openai/math → 强化 nt_agent_eval」的判定被实测推翻 ⇒ **不接线，降级 intake**。
   同型于 10-07 批次「读现场推翻子代理判定」。

## 1. 四字段矩阵（🟢 + 🟡）

| Source | Pattern | NeoTrix 映射 | 判定 |
|---|---|---|---|
| docker/docker-agent | AI Agent Builder+Runtime（MCP） | NT-ACT/orchestrate（l6_meta loop 契约对照） | 强化（📋 路线图） |
| farion1231/cc-switch | 跨平台多 agent 桌面助手统一切换 | neobot 桌面发布 C2 规格 | 强化（📋 路线图） |
| mattpocock/skills | Skills for Real Engineers（.agents 直出） | skills/ 生态兼容（open-steps 同族） | 强化 |
| storytold/* ×6 | Rust 纯净室重实现 PS/AI/PR/LR/AI/Illus | NT-WORLD/retrieve（大型应用逆向参照） | 🟡 intake（设计级） |
| shaders | WebGPU 组件库（React/Vue/Svelte...） | 设计 token/视觉层对照 | 🟡 intake |
| Compositor | Mac Photoshop 替代（Swift） | NT-IO/ui 桌面 | 🟡 intake |
| hand-drawn-styles | 手绘画风 skill 配方 | skills 设计资产（image 生图提示词配方） | 🟡 强化（设计域） |
| HowToLiveBetter | 循证生活指南（CC-BY） | 内容结构参照 | ⚪ intake |
| artcraft | ArtCraft 创作引擎 | — | ⛔ 受限，仅设计 |

## 2. 熔炼五段

0 信号初筛 ✅（§0）· 1 零克隆取证 ✅（raw 11/11 可达 + API 交叉，0 clone）·
2 熔炼成束 ✅（同 `context_bundle()` 原语）· 3 化为已有 ✅（零新建模块）·
4 落账 ✅（本篇入库 + KB absorb-node）。

## 3. 判定总账

| 类别 | 数 |
|---|---|
| 强化 | 4（docker-agent、cc-switch、skills、hand-drawn-styles） |
| intake | 12（openai/math、storytold×5+artcraft、shaders、Compositor、HowToLiveBetter、busbar、噪声 8 的代表记账） |
| 合计 | 16 行判定，3 源跳过（已入库） |

## 4. 后续接线裁决

- `mattpocock/skills` 可直接被我们的 skills 加载器消费（MIT、.agents 目录），
  建议在 `skills/` 做一次兼容性抽样。
- `docker/docker-agent` 的 Runtime 契约与 `nt-io-agent-loop` 对照，
  checkpoint/recover 为本轮最有价值的 intake 项。
- 探针补丁：给 raw LICENSE 探测加 `LICENSE-APACHE`/`LICENSE-MIT` 双件。
