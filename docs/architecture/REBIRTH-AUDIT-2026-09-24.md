# NeoTrix 全项目审计 + 蜕皮方案（2026-09-24，同日第三刷 13:10）

> 方法：rev-officer（FPAM + 7 维并行 squad），只读审计 + 最小安全执行。
> 基线：2939 `.rs` / 904,902 行 → 现约 2830 `.rs` / ~87 万行；
> 分支 `feat/capability-absorb-20260828`； codemap 索引 3740 文件
> （`.project-map/codemap.json`，`make project-audit-map` 双零）。
> 蜕皮累计：删 ~100 文件 / ~3.1 万行，提交 9 个（neobot×2、audit×1、rebirth×5、map×1）。

## 一、文件代码 Map

### 1.1 顶层

| 路径 | 规模 | 职责 |
|---|---|---|
| `neotrix-core/src/` | 2415 rs | 主 crate（`neotrix`）：l0–l6 + entry + 15 bins |
| `crates/` | 285 rs | 叶/中间 crate（types/sysctl/consciousness/reasoning/gateway/multi-agent/abilities/game/neobot/audit/nt-lang） |
| `src-tauri/` | 95 rs + TS 前端 | 桌面壳（`neotrix_tauri` lib + main） |
| `games/neotrix-guixu` | 18 rs | 唯一游戏体（swords 更名，消费 game+abilities） |
| `ntos/src` | 212K（TS） | 第二壳（node_modules 65M 已忽略） |
| `fuzz/` | 82 行 | 独立 fuzz 包（未进 workspace，正确） |
| `_archive/` | 3.7M | 唯一历史备份（已忽略，勿删） |
| `thirdparty/` | 233M（未入库） | kev/agent-jev 吸收参照；`runs/`+`evals/` 约 196M 可剪 |
| `skills/` | 199 文件 | 58 skills / 147 triggers 单一事实源 |
| `target/` | 25G | 构建缓存（勿 `cargo clean`，各窗口增量共用） |

### 1.2 `neotrix-core` l0–l6

| 层 | 职责 | 代表模块 |
|---|---|---|
| l0_substrate | 零上层依赖地基（错误/事件/状态/缓存/ECS） | `nt_core_state.rs`, `nt_core_event.rs`, `nt_ecs.rs` |
| l1_action | 动作与 IO（分发/facade/`nt_io/` 含模型池+web 服务） | `nt_core_task_dispatcher.rs`, `nt_io/` |
| l2_perception | 感知检索（WorldModel/KB/向量/E8） | `nt_world/`, `nt_core_knowledge/` |
| l3_embodiment | 具身护栏（Shield/沙箱/computer） | `nt_shield/`, `nt_sandbox.rs` |
| l4_emotion | 情感+记忆（最小层） | `nt_feel/`, `nt_memory/` |
| l5_cognition | 推理大脑（最大层，nt_mind/网关/多 Agent） | `nt_mind/`, `nt_cognition_facade.rs` |
| l6_meta | 自治理（编排/审批/成本/守护） | `nt_auto_orchestrator.rs`, `coordination/` |

装配：`main.rs → entry/（3019 行组合根）→ use neotrix::（lib.rs）`；
`src-tauri → neotrix-core` 单向（core 永不回指）。

### 1.3 crates 依赖链（无环）

```text
types/sysctl → consciousness → gateway → reasoning → multi-agent → neotrix-core → tauri
abilities → game → swords        nt-lang（孤岛）  neobot/audit（叶）
```

workspace 14/14 对齐；`decision-engine` 意图性隔离（编译红，见 §三.1）。

## 二、七维评分

| 维度 | 分 | 一句话 |
|---|---|---|
| D1 构建 | 7 | members 对齐、game→abilities 闭合；decision-engine 隔离未根治 |
| D3/D5 分层 | 4 | L0 干净唯一亮点；L1/L2→L5/L6 上翻普遍，L5↔L6 成环 |
| D2/D44 死代码 | 4 | 真 `todo!`≈0；78 strict-dead（有实现无接线）待 owner 认领 |
| D4 安全 | 5 | 顶层 forbid + unsafe 隔离 sysctl；纯生产仍 580 处 unwrap/expect/panic |
| D7 测试 | 8 | 14.7k 测试，65% 文件内嵌；34 ignore 多可解释 |
| D9 供给 | 5 | 1404 包，多版本并存（axum/reqwest/wasmtime）；workspace 仅收敛 7 件 |

## 三、Top 发现（证据优先）

1. 🛑 **L5↔L6 成环**：`l5/l1_facade_observer.rs:8-25` ↔ `l6/healing/*monitor.rs:4-5`、`nt_meta/arch_optimizer.rs:1`。修法：事件总线单向化（L6 只订阅 L5 事件，不直引类型）。
2. 🛑 **Tauri 整包红**：`src-tauri/src/domain/plugins/chat.rs:5` 引不存在的 `nt_core_consciousness_core`（类型现住 `l5_cognition::consciousness_core`，但 `AttemptOutcome` 全仓无定义——对方重构未完成）。修法：等 owner，不可代修。
3. ⚠️ **God-file Top3**：`nt_io_browser_engine.rs:4559`、`bin/experience.rs:4195`、`nt_memory_kb/mod.rs:3839`。修法：按职责拆模块（先 experience.rs，bin 最易拆）。
4. ⚠️ **78 strict-dead**：见 `repo-analyses/dead-code-registry-20260923.md:18-102`，需 owner 认领删除（本轮未动）。
5. 📢 **供给膨胀**：`reqwest 0.11+0.12+0.13`、`wasmtime 41+42` 并存；`crystal_game_disabled` 死 feature（`neotrix-core/Cargo.toml:265`）；swords 两处 spire 残留（`build_font_pixel.py:29`、`STYLE-FORMULA.md:31`）。
6. 📢 **磁盘**：`thirdparty/kev/{runs,evals}` 196M + `target/` 25G（不清，后者是增量命根子）。

## 四、蜕皮路线（P0–P2 已执行，P3 待立项）

- [x] P0（2026-09-24）：5 crate 根补 `forbid(unsafe_code)`；删 `notes/`；`neotrix-audit` 入库；本报告。
- [x] P1：Tauri 红已修（`b05061c0` 前序 `3bba2507`）；decision-engine 复活（113 测试）；78 strict-dead 清零（7+60+2兼容+3归档+3已接线+1保留+2豁免）。
- [x] P2 分层：L1/L2 上翻 20+4 处——改道 7 行（nt_core_state×6、hex×1）/ 注记 8 处 / 其余认证不动；L5↔L6 环判定为**设计如此**（门面+trait 实现双向桥，不拆）；双生消除（causal_inventor、DimensionAxis→types）。
- [x] P3（供给）：`deny.toml` 落地（bans/licenses/sources 全绿）+ workspace 依赖 7→20 收敛 + serde_yml 换血（RUSTSEC-2025-0068）；多版本台账（tauri/otlp 系第三方强制，oauth2 4→5 另立项）。
- [x] 游戏三件套入库（abilities/game/guixu，零密钥零 unsafe）；版本纠错（workspace 实 0.21.0）；swords→guixu 更名跟进。
- [x] P4（God-file）：experience.rs→7 模块、browser_engine→8 模块、memory_kb→18 模块，
  全部行为零变更验证（单测+冒烟+黄金输出）。

## 六、后续核心建议（2026-09-24 第三刷，P0–P3 已结项）

只剩三类活，按先决关系排序：

1. **experience 回归线 ✅（已交付）**：`scripts/experience-smoke.sh` 12 命令 12/12。
2. **God-file 拆分 ✅（已交付）**：experience.rs→7 模块、browser_engine→8 模块、
   memory_kb→18 模块，全部单测/smoke/黄金验证。
3. **认知下沉（不做，有依据）**：TTC/PRM/TD 全是具体引擎（无 trait 可抽），
   "下沉"实为整体搬家 + E8 使用方全改；且 E8 片区有 session 并行施工。
   DimensionAxis 模式不可复制（那次是纯数据类型）。等邻居收工+产品拍板再立项。
   dispatcher 签名同理（LocalCrtPlan/LocalTrace 已有，换签名即断 API）。
4. **oauth2 4→5（已完成，需方认领提交）**：reqwest 0.11 已退出 lock；
   `social_access/auth.rs` 完整 v5 化（typestate 构建器+显式端点类型+blocking 同步交换），
   `cargo check -p neotrix --lib` 0 error 验证通过。文件仍是未提交状态，等 owner 收。
5. **施工纪律（立即，零成本）**：脏区仍 ~900——handoff 恢复写；门禁本轮 3 次真拦截
   （consciousness 笔误、axis 漏消费者、game 版本号），保持常开；selective-stage 走
   blob 手术（`scripts/ops/PATCHING.md`）。

## 七、codemap 定位入口（新增）

- `.project-map/codemap.json`（3740 文件：area/layer/modpath/items/loc）。
- `make project-locate COMPONENT=Foo | FILE=bar.rs | QUERY=xxx`（0.2s）。
- `make project-audit-map`（rs 2863/2863、missing/extra 双零；断裂 symlink 自动标出）。

## 五、纠错与保留清单

- `video-decode`/`crystal_game_disabled` 并非死 feature（thumbnail.rs×4 / crystal_integration.rs 在用）。
- 空占位 `consciousness_core.rs`/`coordination.rs`（0 行）是 T39-A4 腾名设计，保留。
- 兼容垫片 `orchestrator_compat`/`unified_types_compat`、archive 3 文件、已接线 3 文件保留。
- `nt_infra_unified_search`/`nt_io_protocol_bridge`/`dual_track` 有新鲜改动，豁免删除。
- `infrastructure/tests/` 系本地忽略目录（其中 1 处 axis 引用已在工作区同步，不入库）。

## 五、本轮验证

- `cargo check -p neotrix-{consciousness,reasoning,gateway,multi-agent} -p nt-lang` 全绿。
- `cargo test -p neotrix-audit` 10/10；`nt-audit` 三命令端到端通过。
- 未碰：他人脏文件、删除类操作（除 `notes/`）、`target/`、任何 `checkout --`。
