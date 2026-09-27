# NeoTrix 全域 Code Map（2026-09-27，只读侦察＋本窗裁决）

> 源报告：map agent 全量表（L0-L6/四端/九 worktree/God Top5/重复 3/死码 3）。
> 本文件：旧新定位＋支脉优劣＋补齐点＋本窗执行裁决。零改码，纯落盘。

## 0. 基线

- HEAD `64f3e3d0`（docs loop 证据）；链上另有 `3b3fa10c` 合体＋tags、`8ebfb491` EVO-04、
  `3d080a96` VER 0.22.0、`84e98aff` C 批；2526+ `.rs` 量级，799k 行。
- 活体：soul online＋sidecar `:8149` ready（AgentJev-0.6B）＋NeoBot.app（本轮自退后重开）。

## 1. 旧新定位（层）

| 层 | 新（近两月/正典） | 旧（迁移改名/残留） | 判定 |
|---|---|---|---|
| L0 | nt_judge/nt_data_gateway（EVO） | nt_core_* 27 个多为改名；hex 1530 行 God | 底座新皮，芯待拆 |
| L1 | nt_io 合体/nt_dialogue_tui | gateway/mod 1678 行；nt_act_trade 双生 1495/1471 | 最厚，债最重 |
| L2 | nt_code_graph/nt_intel_digest（EVO） | nt_core_e8/vector/sense＋三套路由 | 路由三分叉 |
| L3 | nt_near_field/shield 八拆范本 | guard_chain＋osint/ztnet 死码区 | 拆法已有，复制即可 |
| L4 | nt_sim_eval/nt_feel（减后） | unify 1815＋confidence 1520 | 拆一半，续拆 |
| L5 | nt_dspy/nt_mind | 707 文件最大；nt_core/* 54＋SemanticRouter 双生 | 冻结新增，先收敛 |
| L6 | evolution/nt_law_gate | nt_auto_orchestrator 1037 行新 God＋三重门面 | 新 God 已现，立拆 |

## 2. 支脉（端/worktree）优劣

- neobot（新）：local-first＋fail-closed＋tags；缺：dialog.rs ?? 320 行＋bin 1466 行新 God 风险。
- desktop（全新 ??）：薄壳解耦；缺：一仓双 Tauri 必分叉（与 src-tauri）。
- src-tauri（旧）：全功能；缺：im/plugin 1987＋proxy 1757 双 God，三端重叠。
- core（旧身新肢）：分层＋EVO 全＋7 连拆范本；缺：crate 级 allow(dead_code) 静音＋serve 2002 行 ??。
- 九 worktree：HEAD 全被主链超前（MERGED）；3 可删（drift 双胞胎/cap-absorb/model-gw 先 diff），
  split 双轨二合一，在途三件（nt-act/resilience/typed）入库即删。——本窗不动他人窗，删留待各窗（单窗口令下可由主线程代删，另行确认）。

## 3. 重灾区（证据）

- God Top5：pdf 2142＞file_ability 2017＞serve 2002(??)＞im/plugin 1987＞unify 1815。
- 重复 3：MemoryItem×6、CapabilityRegistry×4、SemanticRouter×2（＋MemoryEntry/Lifecycle 双生）。
- 死码 3：agent.rs run_agent_mode 真死、run_agent_tui 空桩、crate 级 allow 系统性掩盖。

## 4. Top5 核心建议（ROI 序，本窗裁决）

1. 一仓双 Tauri 二选一（desktop 转正，src-tauri 冻结＋抽双 God）——**裁决：方案接受，执行另约窗**（改动面大，需独占）。
2. 关全局静音（删 crate allow，分批清 20 处，首刀 agent.rs 死码转接 run_dialog）——**裁决：转接句是行为变更，park 等产品确认**。
3. 立 L0 正典（Registry/Router/MemoryItem 各留一处，余转 alias＋deprecate）——**裁决：接受，排 P3（需跨层改动＋全量门）**。
4. God 按序拆（serve 先入库再拆→file_ability→unify→pdf→im/plugin）——**裁决：serve 入库需其主确认，park；余按 8a11227a 手法排队**。
5. 收工作树（删 3、合双轨、在途入库即删）——**裁决：单窗口可代劳，SOLE 待你一句话（删错不可逆，虽可恢复但扰人）**。

## 5. 本轮活体证据（B agent）

- 前端 rebuild：tsc 零错＋vite 131ms＋tauri debug 包（11:03）；277 单测绿；soul online；sidecar ready。
- App 自退一次（非本窗所杀），已重开待目视；tags 前后端齐，包内即所见。
