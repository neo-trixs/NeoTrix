# NeoTrix 五实体并行实施方案

> 正典蓝图 V3＋任务清单 T01–T37。约束：16G 机器禁多窗口并行全量构建；
> 同一工作区只留 1 watcher；真并行走 `.worktrees/` 隔离（AGENTS.md 并行公约）。

---

## 1. 原则

1. **编码并行，验证串行**：各车道同时写码，`cargo xl` 全量门同一时间只跑一个（swap 必爆，否则）。
2. **一文件一车道**：同文件任务强制同车道串行（已发现：T14/T20 同 `agent.rs`）。
3. **依赖边不可逆**：跨波次边（T03→E1、T22←T10–T21）必须等闸门绿。
4. **小步快走＋handoff**：关窗口前写 `sessions/handoff-<窗口>.md`，收齐＋stash 兜底再关。

## 2. 车道划分（按文件归属，互不交叠已验）

| 车道 | 任务 | 独占文件 | 工作区 |
|---|---|---|---|
| Lane A（L0/L1 基座） | T03→T18→T19→T20→T14（T05 待 T04 后插入） | task_dispatcher/8 imports；scheduler；agent.rs（T20→T14 串行）；error_conversions | `.worktrees/lane-a` |
| Lane B（L5 认知） | T04→T12→T13/T15→T27a/b/c→T29 | l1_facade；agent_card；presets；dispatch/orchestrator | `.worktrees/lane-b` |
| Lane C（工作区/技能） | T10→T11；T16→T17 | nt_core_ws；skill_evolution＋3 改名 | `.worktrees/lane-c` |
| Lane D（桌面/前端） | T09→T31–T34→T35–T37 | model_router/gateway；browser_host；frontend；registry 市场字段 | `.worktrees/lane-d` |
| Lane E（晶体/事件） | T22→T23/T24/T25→T26/T28 | crystal_state；core tick；entry；event | `.worktrees/lane-e` |
| Lane S（串行特区） | T06a→T06b→T06c（最后） | nt_approval＋L3 回调 | 主工作区（最后合流） |

注：T14/T20 同 `agent.rs` → 同 Lane A，T20 先 T14 后。T35 依赖 T13/T16/T20 跨车道 → 等三车道闸门绿后 Lane D 执行。

## 3. 波次表

### Wave 0（1 窗，0.5–1 天）：地基开门
- Lane A：T03（8 处 import＋CoT trait）。闸门：`cargo xl` ＋ rg 零命中旧路径。

### Wave 1（5 窗并行，约 2 天）：无交叠扇出
| 车道 | 任务 |
|---|---|
| A | T18（Task 字段＋同步镜像；T05 等 T04，排 Wave 2） |
| B | T04（facade 收敛） |
| C | T10（Workspace 字段） |
| B2 | T12（Agent 六维）——B 内与 T04 串行（同 L5，不同文件，可同窗先后） |
| C2 | T16（SkillCandidate＋5 步管线） |
| D | T09（模型档位层） |
- 闸门（串行执行）：`cargo xl` 一次；各窗先 `rustfmt` 自查。

### Wave 2（扇出收敛，约 3 天）
- T05←T04；T11←T10；T13/T15←T12；T17←T16；T19←T18；T21←T20；T20←T03；T14←T20（同文件串行）。
- Lane S 待命（不动，等 Wave 2 闸门）。

### Wave 3（E2/E3，约 4 天）
- T22（投影＋AgentDirectory 外观）→ T23/T24/T25 并行（三文件无交叠）→ T26/T28 → T27a/b/c → T29（含 42 词复核）。
- Lane S：T06a→T06b→T06c（最后合流）。

### Wave 4（收尾）
- T30 ✅ 已提前；T31–T37 桌面线（Lane D）；E4 文档同步；全量 `cargo clean && cargo build` ×2。

## 4. 验证门协议

| 门 | 执行者 | 命令 | 通过标准 |
|---|---|---|---|
| 自查门（每窗每次提交前） | 各窗 | `rustfmt --check`（我区）＋ `rg` 验收式 | 零 diff（我区） |
| 波次门（每波一次） | 指定一窗 | `cargo xl` | 通过；失败则全线停、修 Hull |
| 合流门（worktree→主） | 主窗 | `git diff --stat` 评审＋`cargo xl` | 无冲突＋绿 |
| 终门 | 主窗 | `cargo clean && cargo build` ×2＋全量 test（CI） | 真实错误数清零 |

### 验证门专窗方案（本机 10min 超时后的替代路径）

> 背景：neotrix 全量 check 在本机多次超时（ warm 缓存 48s 通过一次，其余全超时），
> 且多窗并发 cargo 会抢锁＋爆 swap。专窗独占跑门，其余窗只做自查门。

1. 专窗独占：开一干净 worktree（只含已合流代码），`cargo xl`，一次只跑一个 cargo 进程；
   其他所有窗在此期间禁跑任何 cargo 命令（rustfmt/rg 不限）。
2. 分级降载（全量超时仍失败时）：`cargo check -p neotrix --lib` → `--tests` →
   `cargo test -p neotrix --lib <module>` 逐模块收敛；tauri 侧只跑 `tsc`＋`vitest`。
3. 独立验证法（已验证有效×3）：纯逻辑抽取 `rustc --test`（门禁逻辑 2/2、轮转 1/1 通过）；
   仅适用于零外部依赖或 std-only 代码段，harness 必须注明与源文件逐字一致性校验方式。
4. 门禁结果回填清单对应行（✅＋命令＋耗时），超时记"待 CI/专窗"不记通过。

## 5. 冲突处理

1. 开工前 `git status --short <path>`，脏则喊一声（2026-09-22 桌面清单覆盖事故）。
2. 同文件两任务相撞 → 后者 rebase 到前者之后，同车道化（先例：T14/T20）。
3. 估时总和：关键路径 T03→T12→T22→T23→T27c ≈ 8 天；全量约 49 天·人/单窗，并行约 12–15 天 wall-clock（5 窗）。
