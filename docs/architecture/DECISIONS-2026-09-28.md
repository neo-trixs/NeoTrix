# 待解决任务总清单 + 技术裁决 — 2026-09-28 18:1x

> **本文是裁决记录，不是愿望清单。** 每条都标注了「裁决」与「依据」，
> 依据一律是**读现场/实测**，不是台账转述。
>
> **本轮实测基线**（干净检出 + 4 连跑）：
> `cargo test -p neotrix --lib -- --test-threads=4` → **12233 passed / 0 failed / 41 ignored**
>
> **元规律（本轮第 6、7 次撞上）**：凡「N 份同名类型 → 合并到 1」的任务，
> **先比字段集**。本仓的 `CapabilityRegistry`×4 / `SearchResult`×9 /
> `ExtractConfig`×2 / `EmailConfig`×2 / `PlatformRegistry`×3 / `SemanticRouter`×2
> **全部是「同名不同型」**。机械合并会毁掉正在被用的东西。

---

## A. ⛔ 裁决：**不做**（前提证伪 / 已被外部解决）

| # | 台账要求 | 我的裁决 | 依据（实测） |
|---|---|---|---|
| A-1 | 「4 份 `CapabilityRegistry` 收敛到 1」 | **只删 1 份** | 4 份里 3 份活：①`nt_core_capability_tree`（dispatch/loop/cad/maintenance 在用）②`l5_cognition/.../capability`（`cluster_self_test.rs` 是 **pub mod 生产自测**）③`l0_substrate/nt_core_capability_types`。只有 `nt_file_ability` 那份零消费者 ⇒ 已删（`024c2ae0`） |
| A-2 | 「`neotrix-types` 包内 `SkillRegistry` 2→1」 | **不合并** | 字段集不同：`HashMap<String,Skill>` + `skill_tree` vs `HashMap<String,SkillDefinition>` + 查重 `register()`；后者经 `core/mod.rs:116` 公开导出 |
| A-3 | 「`ExtractConfig`/`EmailConfig`/`PlatformRegistry` 三处同名双定义定正典」 | **不合并** | `ExtractConfig`：A=`page_size:Option+max_records+filters` / B=`page_size:u32+max_pages+customer_ids`；`EmailConfig`：A=时间窗 `since/until` / B=分页+`box_id`；`PlatformRegistry` 3 份分别存 `Arc<dyn PlatformAdapter>`+configs / `Arc<dyn ExternalPlatformExtractor>` / 裸 `PlatformConfig` ⇒ 全部正交 |
| A-4 | 「`SemanticRouter` ×2 定正典」 | **不合并** | A=`rules:Vec<RouteRule>+provider_scores`（规则+供应商打分）/ B=`confidence_threshold+route_table+fallback_model`（置信度路由）⇒ 两个不同轴 |
| A-5 | 「`nt_core_gate::ToolRegistry` 是 stub，删」 | **保留，已加防误删注释** | 活消费者 `nt_shield_enforcer.rs:388-411`，承载写操作**可逆性**；与 `nt_act` 的运行期 `ToolStats` 正交 |
| A-6 | 「分层违规 94 条」 | **真值 102** | 94 是**脏树**测量；照抄会让 CI `FAIL: 8 new`。已改（`bdf1e9f1`） |
| A-7 | 「`naming` 1,644」 | **真值 1,646** | 同为脏树值 |
| A-8 | 「台账 §3 的 29 条一行级生产 bug」 | **27/27 证伪，0 确认** | 绝大多数已由 `f4a4eecd` 修（25 文件正对应）；8 条前提错（详见台账 §3） |
| A-9 | 「`nt-lang` 无 `[lib]`，加还是删」 | **无需裁决** | 已被外部 agent 删除 |
| A-10 | 「6 个脚本硬编码 `M-477231~479942`」 | **作废** | 那 6 个脚本**在树与 stash 中均已不存在**；且活库 max 已从 `M-065651` 涨到 **`M-070817`** |
| A-11 | 「`shared_types::Severity` 的 `Ord` 该翻」 | **绝不翻** | `l2_perception/nt_world/osint/sweep.rs:225` 显式依赖该约定做 `min_severity` 过滤，翻转会**静默反转过滤器** |
| A-12 | 「删 5 个内容型 worktree」 | **一个都不删** | 实测 5 个全部 dirty（2/1/12/6/8 文件）且 0 个未合并 commit ⇒ 删掉 = 毁掉在制品。改为登记内容 |

---

## B. ✅ 裁决：**执行**（我已定方案，按序做）

| # | 任务 | 我的技术方案 | 为什么是这个方案 |
|---|---|---|---|
| **B-1** | `nodes` 真双时间 | **照 `nt_temporal_facts.rs` 已验证形态**：每版本独立 `id` + `supersedes`/`superseded_by` 指针链，**不改主键、不动外键** | 复合主键方案已实测崩在 5 处外键（`foreign key mismatch`）并回退过一次；per-version-id 是本仓唯一有 91 测试全绿背书的版本链实现 |
| **B-2** | Noise IK 握手 | **对齐 spec**（`Noise_IKpsk2_25519_ChaChaPoly_SHA256`，39B）并用**官方测试向量**交叉验证 `es/ee/s` 派生次序 | crypto 不该降级；现实现 `es` 角色+时序双接反。手写"看着绿"的错实现比红测试更坏 ⇒ 必须有向量 |
| **B-3** | CAD 假证据面 | **删 `cad_wiring_map()` 证据表**，并把判据从「路径含 `:`」改成「**文件真实存在**」 | 12 项里 8 项指向不存在的文件；把死 `file:line` 当接线证据喂晋升门 = **自我欺骗面，假「通过」比红测试更糟** |
| **B-4** | publish gateway | **加一等公民 `dry_run`**（不是删测试） | 投 OAuth2+reqwest 是独立工程；`dry_run` 让「能发布」成为真的，且不藏能力 |
| **B-5** | `/stop` 回执 | **采用「只说事实 + 给真能生效的路径（退 App）」**，并修 `nt_agent.rs:2283` 那条**前提已过期**的反向测试 | 反撒谎优先：宁可让用户退 App，也不推荐一个**不取消任何东西**的键 |
| **B-6** | 3 处本地 `Severity` enum | **对齐承重的 `shared_types` 约定**（改声明序），**不动 shared 版** | 本地 enum 全仓无排序/比较调用点 ⇒ 可安全对齐 |
| **B-7** | `cascade.rs` `length_score` | `len/200` 改为长度分档，且 `tick():126` 不再**永久丢弃**未达阈样本（改为保留待累积） | 现实现让长记忆与短记忆得分几乎无差，且样本一旦未达阈就消失，学习信号断链 |
| **B-8** | 102 条分层违规 | **按文件聚类**：先做 84 个「单文件单规则」叶子，再看能否改走 `facade`（门已排除 `*facade*`/`*l1_facade*`/`traits.rs` 为官方跨层通道） | 棘轮已就位，**只向下**；调大基线等于藏债 |
| **B-9** | hh-rlhf 2 条 `[redteam]` | **留作拒答样本**（不删） | 下游过滤会丢掉「拒答模式」这个信息本身 |
| **B-10** | CI 线程数 | **暂不改 `--test-threads=2`**，再观察数轮 | 4 连绿是强证据，**不是**对间歇性崩溃的证明；改早了 CI 抖动会重新训练所有人忽略它 |
| **B-11** | 32 个未跟踪 `scripts/ops` | **等 stash 恢复后统一入库**（含活路径 `nt_graph_audit.py`） | 当前工作树里已 0 个未跟踪 scripts/ops，全在 `stash@{0}` |
| **B-12** | `TextEmbedder` | ⏸️ **独立一轮，暂不动** | 5 个生产调用方；改 `embed` 会**重排全部检索结果**，需先评估影响面 |

---

## C. 🚫 真正需要外部输入（我无解）

| # | 阻塞 | 原因 |
|---|---|---|
| **C-2** | 185 个未核验删除（skills 75 / src-tauri 67 / games 38） | 需逐条核验才能提交；全在 `stash@{0}`，我不动 |
| **C-3** | 另一 agent 正在同分支并发重构 | `5c02e738` 归档 src-tauri、删 6 个 crate。我在其半成品上提交 = 覆盖事故路径 |

> ~~**C-1 · Yootta/World-SimReady-Home（3.0 TB gated，401 物理不可达）**~~
> **2026-09-28 用户指令移除该任务。** 理由记录保留在
> `RFC-CRYSTAL-REASONING-CHAIN-20260928.md` 的处置矩阵（"跳过（强制），401"）
> 与 `TODO.md` 处置矩阵里 —— 那是**已做的决定**，不是待办。

---

## D. 📊 本轮已完成（勿重做）

`bdf1e9f1` 门修复+102 基线 · `f23177de` 记录校正 · `a9ad48f5` S-1 并车道 ·
`8eac716e` 晶体入库+8 测试归零 · `024c2ae0` B-1/B-4 · `7676f6f8` streaming 去外网依赖 ·
`ecd10d3e` 晶体第 9 条+台账 §3 证伪 · `d5edd461` 门清单/2 examples/HOME 竞态/测试门转严格 ·
`89769661` 4 条 flaky 归零，`--test-threads=4` 恢复（4 连绿，107s vs 2 线程 216s）

## E. 📌 待恢复

`stash@{0}`：**759 tracked + 240 untracked**（4 天多窗口产出）。
外部备份 `/tmp/nt-backup-20260928`（manifest + tracked.patch + untracked.tar.gz）。
B-11 与 C-2 都卡在这里。

---

# 后续待解决问题（按可执行顺序梳理）

> 状态快照 2026-09-28 18:3x：`cargo check --lib -p neotrix` **exit=0**（已转绿），
> 但共享 index 仍有 **177 个他窗暂存项**、最近 3 分钟仍有文件在被改
> ⇒ **提交通道不安全**（`git commit` 会连他那 177 个一起卷走）。

## P0 · 唯一阻塞：提交通道（不是代码问题）

| 项 | 现状 | 我的处置 |
|---|---|---|
| 提交我这 5 个文件（TODO / OPEN-TASKS / DECISIONS / 新 LESSONS / handoff） | lib 已绿，但 index 被占 | **等 index 清空再提**。不 `--no-verify`、不在 177 个暂存上提交 |

> 判据：提交前 `git diff --cached --name-only \| wc -l` 必须是 **0**（或只含我的文件）。

## P1 · 通道一开就做（已定方案，无需再决策）

| 序 | 任务 | 方案 | 验收 |
|---|---|---|---|
| 1 | `nodes` 真双时间（B-1） | 照 `nt_temporal_facts.rs`：**每版本独立 `id` + `supersedes`/`superseded_by` 指针链**，**不改主键、不动 5 处外键** | 两条 `#[ignore]` 测试转绿并**删掉 `#[ignore]`**；`nodes_as_of`/`node_history` 接到真实调用方 |
| 2 | Noise IK 对齐 spec（B-2） | 改名到 `Noise_IKpsk2_25519_ChaChaPoly_SHA256`(39B)，用**官方测试向量**交叉验证 `es/ee/s` 派生次序 | `full_handshake` 绿且**删掉 `#[ignore]`**；加**握手对称性**测试（两侧 chaining key 必须相等） |
| 3 | CAD 假证据面（B-3） | 删 `cad_wiring_map()`；判据从「路径含 `:`」改成「**文件真实存在**」 | 8 个指向不存在文件的条目不再被当"接线证据" |
| 4 | `/stop` 过期测试前提（B-5） | 修 `nt_agent.rs:2283` 那条**假设"取消尚未接线"**的反向测试（现已接线） | 4 条反撒谎契约仍绿 |
| 5 | 102 条分层违规（B-8） | **先做 84 个「单文件单规则」叶子**；能改走 `facade` 的优先（门已排除 `*facade*`/`*l1_facade*`/`traits.rs`） | 基线**只向下**：`check-layer-deps.sh --strict` 报数递减 |

## P2 · 阶段工程（单独立项，不混做）

| 项 | 为什么独立 |
|---|---|
| 185 个未核验删除 | 需逐条核验才能提交；全在 `stash@{0}` |
| 32 个未跟踪 `scripts/ops` | 含活路径 `nt_graph_audit.py`；全在 stash |
| 5 个陈旧 worktree | 实测全部 dirty（2/1/12/6/8 文件）⇒ 删=毁在制品；需先导出内容再删 |
| `TextEmbedder` 换真实现 | 5 个生产调用方，改 `embed` 会**重排全部检索结果**，须先评估影响面 |
| `publish gateway` 真实上传 | 加 `dry_run` 是 B-4（低成本）；投 OAuth2+reqwest 是独立工程 |
| `cascade` `length_score` 改分档 | 改的是记忆晋升评分，需领域评估，不宜顺手改 |

## P3 · 需你拍板/外部输入

| 项 | 状态 |
|---|---|
| 另一 agent 的并发重构 | 我全程避让，未在其半成品上提交 |
| CI `--test-threads` 2→4 | **暂不改**。4 连绿是强证据**非证明**（修前 3 跑 2 崩）；再观察数轮 |
| `stash@{0}` 是否整体恢复 | 759 tracked + 240 untracked；B-11/C-2/阶段工程都卡在这里 |
| 185 个删除是否恢复 | 同上 |

## 已彻底关闭（勿再翻）

- ⛔ 12 项前提证伪 / 已被外部解决（见 A 组）
- ✅ 9 个 commit 已落盘：分层门 102 基线 · 记录校正 · S-1 并车道 · 晶体入库+9 测试归零 ·
  B-1/B-4 · streaming 去外网依赖 · 台账 §3 证伪 · 门清单/2 examples/HOME 竞态/测试门转严格 ·
  4 条 flaky 归零（`12233 passed / 0 failed @4 线程` 4 连跑）
- ⛔ Yootta gated 401（用户指令移除，理由记录保留）
