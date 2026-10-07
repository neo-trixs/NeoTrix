# §用户指导：请先读 AGENTS.md（判据）+ §1 全局决策树 + §3 并行公约
# 交接文档入库是强制义务（R-P16、收工义务 3 步）

## 1. 会话标识

- **窗口**：opencode-session（具体 tty 名待补）
- **分支**：`feat/capability-absorb-20260828`
- **交接时间**：2026-10-07 20:40
- **最近 HEAD**：`c2732961 feat(gates): R2 执行器 —— 层名只有一个定义处（nt_review_types），拒绝第三份清单`（他窗提交）
- **我的最后提交**：`38d18247 docs(infra): 补 health_healthy 字段级裁定注释`

## 2. 目标（一句话）

完成外部 URL 吸收（TradingAgents / memvid / huashu / tester-army/e2e / Ix / leviathan / CarterPerez/Cybersecurity-Projects）+ 全域债务偿还（D1 unwrap・D2 死配置・D3 冗余・D4 命名・D5 能力）+ 元认知测试/审计工具化（新门 `check-fake-signal` / 复盘债务判据）。

## 3. 已完成

### 3.1 外部吸收（URL → 设计熔炼，零代码复制）
| URL | 裁决 | 落地点 |
|---|---|---|
| `memvid/memvid`（Apache-2.0） | ✅ 吸设计 | `7b3dfbbf` checkpoint 帧校验和（FNV-1a 64） |
| `tester-army/e2e`（Apache-2.0） | ✅ 吸设计 | 「录制—重放」不实施（无对齐的 Rust 用例面），只留下复用判据「录制—重放」 |
| `TradingAgents`（Apache-2.0） | ✅ 吸设计 | 「决策持久化 + 结算」不实施（需跨会话账本，属新设计），只留下概念 |
| `huashu-art-motion`（MIT） | ⛔ 不吸收 | 与本仓宪法/架构无直接能力对应 |
| `ix-infrastructure/Ix`（Apache-2.0） | ⛔ 不吸收 | 与 `.project-map` + `check-layer-deps` **重复**，会破坏单一真源 |
| `elstongun/leviathan`（Apache-2.0） | ⛔ 不吸收 | 2 天历史，证据不足以承重（R-P79 §URL-only 入参 B10.4 记录真伪是唯一硬停） |
| `CarterPerez-dev/Cybersecurity-Projects`（AGPL-3.0） | ⛔⛔ **法务拒绝** | 强 copyleft + 作者明写 "copy directly" ⇒ 复制即传染整个闭源仓 ⇒ ⚠️ **应写进 `.neotrix/LICENSE-EXCEPTIONS.md` 作为**已知 AGPL 拒绝项** |

### 3.2 债务偿还（横向清理）
- **D1** `check-unwrap` **4 → 0**，`--strict` **首次全绿**：
  - `30c2c9f1` `nt_io_output_style.rs::resolve→Option` + 三级兜底
  - `a8915b61` 删除 `HttpPool::standard()`（零调用方 panic 包装）+ 迁 2 调方
  - `4795e209` `NexusWeaver` KB 缺失改 Option；discocver "重开同路径重试必失败" bug
  - `7e421b9a` `start_goal` 改 `Option<&GoalTracker>`
  - 并修了 `check-unwrap.sh` 的 content-anchors 基线过渡（`319bd2c9`-era）
- **D3** 冗余下沉 Phase 1：`1e8039db` `nt_io_output_style.rs`（1091 → 791，删 22 项逐字重复）+ `3727cb45` `memory_types.rs`（132 → 85，删工作流/子任务死类型）⇒ 共 **-347 行**
- **D2** 死配置：bool 零读点 585→**585**（最新）、未接线规格 **5→0**
  - `f59c2ddb` 接线 `proxy_enabled` + 撤 5 个错误标注（`enabled`/`enable_auto_crystallize` 实为活字段；`system_proxy_enabled` 跨域错位；`geo*`/`agent_protocol_*` 功能未实现）
  - `84fdc924` 给门加第四类「未接线规格」
  - `9d7bdfa0` 接线 `enable_exploration`
  - `515cb99d`（他窗）同步：`spawn_handler` 校验函数
- **新门 `check-fake-signal`**（`055bbdb4`）：检出「**看起来在工作、实际与真实状态无关**」的四类伪信号
  - R1 结论型字段只有布尔字面量赋值
  - R2 `detect`/`probe` 伪探测
  - R3 评分函数字面量实参
  - R4 健康/节律字段只有字面量赋值
  - 现 findings **18** 条。探针 `_lib.sh` 契约 4/4 PASS
- **`check-executor-registry`**（`4c42ae7c` 或同期）：对账 manifest 数据类与真实注册表
- **`check-license`**（`394c585f`）：排除 `.worktrees/` + `.git/` 扫描范围 ⇒ worktree 副本不再被鬼访为独立 vendored 树
- **`check-dead-config-flag`**（`84fdc924`）：补第四类
- **evolution 评分闭环**：`a5d602b4` 引入 `nt_reward`、`3d17f139` 接入 `GoalContractStage`、`c69d5abd` E2E 测试
- **能力执行**（`net_crt` 接管）：
  - `4c42ae7c` `nt-core-capability-tree` 共享 dispatch 端口；manifest 加 5 条能力
  - `77e9688d` AsyncBoxFuture 形态
  - `5470e7a7` `tree_dispatch` 接两类 + `Once` 幂等 + runtime new 调用
  - `d74c6f1f` `ConsciousnessRuntime::new` 一次接统一生命周期
  - `f08355a6` 自洽生产 runtime 真实构造（不再 Option 占位）
  - `nt-crystal-serve capability-call` core 进程外入口；`neobot capability {list,never,canary}` 只读

### 3.3 元认知基础本能能力缺陷修补
- **`check-unwrap`** 基线锚点从行号 **改为「路径 + 函数名 + 归一化代码行」**（`c01cbdb5`）⇒ 行号漂移免疫
- **`check-dead-config-flag`** 补第四类「未接线规格」（`84fdc924`）
- **`check-doc-drift`** 死链与 module-doc 分账（保留 main 门绿）
- **每字段每条裁定**加入代码注释，避免下一轮重复触发

## 4. 正在改的文件（他窗 WIP，非我所有）

| 文件 | 改到何程度 | 是否可独立提交 |
|---|---|---|
| `neotrix-core/src/l0_substrate/nt_core_event_bus.rs` | 他窗 WIP（未 commit） | 看他窗 |
| `neotrix-core/src/l0_substrate/nt_judge.rs` | 他窗 WIP | 看他窗 |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/kb_search.rs` | 他窗 WIP | 看他窗 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | 他窗 WIP（加 `nt_bg_wiring_*` 宏调用）⇒ ⛔ **119 个编译错误**（宏未定义） | ⛔ 阻塞主 lib 编译 |
| `neotrix-core/src/l1_action/error_conversions.rs` | 他窗 WIP | 看他窗 |
| `scripts/gate-registry.tsv`、`check-arch-rules.sh`、`check-layout.sh`、`nt_gate_coverage.py`、`nt_tui_e2e.py`、`nt_worktree_gate.sh` | 他窗 WIP（registry 补 license-js 行等） | 看他窗 |
| `.neotrix/knowledge.db` | 他窗未跟踪 sqlite db | 看他窗 |
| `.neotrix/patches/*` | 他窗未提交 patches 7 个 | 看他窗 |
| `neotrix-core/tests/nt_meta_integration.rs.disabled` | 他窗临时禁用测试文件 | 看他窗 |
| `scripts/ops/nt_trusted_gate.sh` | 他窗新建脚本（未提交） | 看他窗 |

## 5. 下一步（按优先级）

1. **他窗 WIP 收口**：`run.rs` 的 `nt_bg_wiring_name_ok` / `nt_bg_wiring_body_ok` 宏未定义 ⇒ 119 个编译错误。等他窗完成更改（宏定义入库 + 测试 `#[test]` 解析）或帮他窗补宏
2. **`LICENSE-EXCEPTIONS.md` 加 AGPL-3.0 永久拒绝项**（CarterPerez/Cybersecurity-Projects）⇒ 表明已核查许可并明确不吸收
3. **D2 剩余 585 + 313 条零读点** 继续按分片渐进清理（bool: l1_action 84 + l6_meta 79 + l1_action 75 + l5_cognition/nt_core 70 + ...）
4. **命名债**：`check-naming` advisory 1,615 处
5. **能力未接线**：3 DeclaredOnly + 1 Scaffold 的业务 schema 等外部输入
6. **`.project-map` 体积策略**（354M）：`git check-attr` / LFS 决策

## 6. 阻塞点

- ⛔ **他窗 WIP 阻塞 `neotrix::lib` 编译**：`nt_mind_background_loop/run.rs` 加了宏调用但未补宏定义 ⇒ 119 E0433 错误。我的改动（`fix(neocodex)` / `docs(infra)` 等）均可独立编译，但整体 `cargo test --lib` 无法跑
- **AGPL 许可问题**：`.neotrix/LICENSE-EXCEPTIONS.md` 仍是 `status: void` ⇒ 无人签署 ⇒ 该路径未被允许
- **D5/D6**：能力清单的 DeclaredOnly 业务逻辑需外部 schema 才能锁定 fail-closed 输入解析

## 7. 给接手会话的话

- **恢复命令**：先读本文件 + `git status --porcelain` + `git log --oneline -8`
- **禁止**跑 `cargo check --all-targets`（全量构建会卡 swap）。`cargo build -p neotrix --lib` 即可
- **如果他窗 WIP 已合并**：先 `git pull --ff-only` 拿最新，再重测
- **如果 `run.rs` WIP 还在**：不要动 `run.rs`，等他窗完成宏定义后再合并我的方案（`docs(fake-signal)` 中 R3 的两个命中正是它要测试的）
- **更新门记录时注意**：`check-dead-config-flag` 对 `未接线规格` 的标注会让门记录非 0，但属**设计真空/功能未实现**而非死开关——中文注释里的「`未接线规格`」**会被 SPEC_MARK_RE 误判**（已在 `f59c2ddb` 提交说明里留痕）
- **历史教训关键点回顾**：
  - **「grep 只看第一个命中就下结论」= 未取证**（本会话 6 次：`CapabilityVector`、`EvolutionForecast`、`SocialPath::standard`、`PlannerExecutor::verify`、`hybrid_search`、`LocalEchoEngine::probe`）
  - **门的正则会污染自己的文档**（`SPEC_MARK_RE = nt-unwired-spec|未接线规格` ⇒ 写「撤销`未接线规格`标注」会再被门命中 ⇒ `f59c2ddb` 已记录）
  - **「改门让 PASS」= 不可接受**（`check-license.sh` 自带禁令）⇒ 本次我所有门改动都通过 `git stash` 前后对照验证效果

## 8. 收工自查（必填）

- **worktree**：本窗口未开新 worktree；🧹 无残留
- **未提交改动**：仅他窗 WIP（`nt_core_event_bus.rs`/`nt_judge.rs`/`kb_search.rs`/`error_conversions.rs`/`nt_mind_background_loop/run.rs`/`scripts/gate-registry.tsv`/`check-arch-rules.sh`/`check-layout.sh`/`nt_gate_coverage.py`/`nt_tui_e2e.py`/`nt_worktree_gate.sh` + untracked `.neotrix/knowledge.db` / `.neotrix/patches/*` / `tests.disabled` / `nt_trusted_gate.sh`）⇒ **应由他窗提交或归档**，本窗口不越权提交
- **Stash**：`git stash list` 检查是否为空
- **新门/探针的验证闭环**：`check-fake-signal` 有探针 4/4 PASS + `--strict` RC=0；`check-license` 用 `git stash` 前后 RC=4→2 验证
- **生产接线（R-P79）核查**：本窗口所有外部技术（memvid 校验和 / e2e 录制—重放 / TradingAgents 决策账本）⇒ 最终只有 memvid 的 1 条校验和真实接线（`7b3dfbbf`），其余 ⛔ 未接线且留档裁决，符合 R-P79「不算、只做设计级研究」
