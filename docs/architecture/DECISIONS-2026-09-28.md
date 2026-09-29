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

> 状态快照 **2026-09-28 20:0x**（较 18:3x 那版有实质变化）：
> **P0「提交通道」已解除** —— 解法是 `.worktrees/ratchet` 独立 worktree
> （L7-b）。B-1 + 分层棘轮首批已落 `71e1c412`（P0 门 ✅ 通过）。
> 主工作树仍被另一 agent 的 WIP 占用（`nt_core_capability_tree` 断链等），
> **我不在主树继续施工**。

## P0 · ✅ 已解除：提交通道（L7-b 的真解）

| 项 | 结论 |
|---|---|
| 解法 | `git worktree add --detach .worktrees/ratchet HEAD` + `git checkout -b` ⇒ **独立 index**，不再卷走他窗暂存项，且他窗搞红主树不影响本车道 P0 门 |
| 效果 | 主树 P0 红 ×3 拒；同改动在 worktree 上 `cargo check --tests` **exit=0**、门 ✅、提交 `71e1c412` 一次成功 |
| 当前车道 | 分支 `fix/bitemporal-and-layer-ratchet`，基于 `d5413335` |
| 合入时机 | 待他窗重构收口后合（合入后**必须复核分层门数字**，见 L8「Auto-merging 静默降级」） |
| 注意 | 独立 `CARGO_TARGET_DIR` 建议导出到 `/tmp/...`，避免与对方抢 cargo 锁 |

## P1 · 本车道可继续（已定方案，无需再决策）

| 序 | 任务 | 状态 | 方案 | 验收 |
|---|---|---|---|---|
| 1 | `nodes` 真双时间（B-1） | ✅ **已完成** `71e1c412` | 每版本独立 `id` + `supersedes` 链，**零 schema 变更**（L11：两列早已存在）；69 处生产写入 / 5 处外键未动 | 2 条 `#[ignore]` 已摘除；`12154 passed / 0 failed @4 线程` |
| 2 | 分层违规棘轮 | ✅ **93/101**（101→92→89→80→50→47→40→26→25→8） | **必须经「消费方自己那层」的 facade** —— 走目标层 facade 无效（路径仍含层名）。`OneObserver` 等 14 个符号 l5 facade 已有；缺的按「消费方原本就在用的路径」补 `pub use` | 基线**只向下**：8/8 `PASS 0 new`、RC=0；**L1–L5 真引用全清，剩余 8 条均为已记录不可改道项** |
| 3 | Noise IK 对齐 spec（B-2） | ⛔ **阻塞于外部输入** | 改名到 `Noise_IKpsk2_25519_ChaChaPoly_SHA256`(39B)，用**官方测试向量**交叉验证 `es/ee/s` 派生次序 | `full_handshake` 绿并摘 `#[ignore]`；加**握手对称性**测试 |
| 4 | CAD 假证据面（B-3） | ✅ **早已完成**（勿重做） | 判据从「路径含 `:`」改成「**文件真实存在**」 | 已改 `CARGO_MANIFEST_DIR` + `is_file()` |
| 5 | `/stop` 过期测试前提（B-5） | ✅ **早已完成**（勿重做） | `nt_channel_cmd.rs` 已禁「按发送键」类假建议 | 4 条反撒谎契约绿 |

### 分层违规剩余 8 条：全部是已记录不可改道项（真引用清零）

| # | 条目 | 原因 | 处置 |
|---|---|---|---|
| 1 | `nt_file_ability/tests.rs [l2]` VSAEngine/VsaBackend 4 处 | l0 无 VSA，被引的是 l2 真实现 | 留基线 |
| 2 | `ffi/consciousness_tree.rs [l5]` metacalib 2 处 | l0 无 brier/ece，真实现在 l5 | 留基线 |
| 3 | `ffi/seal_pipeline.rs [l5]` training_cycle 3 处 | l0 无对应；且该文件 ios-bridge 下有预存 `types::` 解析错（未动） | 留基线 |
| 4 | `nt_file_ability/tests.rs [l5]` BranchKind 2 处 | l0 无，真实现在 l5 | 留基线 |
| 5 | `ffi/consciousness_tree.rs [l6]` register_absorbed_modules 1 处 | l6 本地真实现，不可下沉 | 留基线 |
| 6-8 | nt_agent_session[l5]、arch_fitness[l6]、orchestration_taxonomy[l6] | 剥离后 0 命中：字符串/参数字面量 | 留基线（L14） |

**累计** 101 → 92 → 89 → 80 → 50 → 47 → 40 → 26 → 25 → **8**（清 93 条）。
真引用清零的验证口径：每批改道后门计数实际下降（L19），而非分类器断言。

| 类别 | 文件 | 引用 | 状态 |
|---|---|---|---|
| **L1** | 36 | 70 | ⏳ 唯一剩余。`l1_action/nt_action_facade.rs` **0 条 `pub use`**（全是内部 `use`）⇒ 需**从零建 barrel**，一次导出数十符号，**撞名风险最高**。做法：按目标层分子批（先挑符号少、单目标层的叶子），每批 `check`+`test` 后棘轮 |
| 不可改道误报 | 2 | — | ⛔ 保持原样，见 L14 |

**累计** 101 → 92 → 89 → 80 → **50**（清 51 条）；四层（L2/L3/L4/L5）已清零。

**已确立的做法（照做即可）**
1. 提取映射的脚本**必须与门同口径**（都剥 `//` 与字符串）—— 见 L13；
2. facade 按**消费方原本就在用的路径** `pub use`（原代码能编译 ⇒ 路径可证），
   **不重新定位定义处**；
3. 对 **facade 里已存在**的符号**必须核来源**（同名 ≠ 同一符号，换错了
   `cargo check` 不报错）—— 见 L15；
4. 改完自查 diff：注释内改道数须为 **0**（且**排除 facade 文件**）—— 见 L16；
5. 每批 `cargo check --tests` + `cargo test` 全量，再棘轮下调。

**独立待办（勿在棘轮批次里顺手做）**
- 给 `check-layer-deps.sh` 加「剥离字符串字面量」过滤，使那 2 条真消失。
  ⚠️ **不能**用 `-v '"[^"]*l6_meta[^"]*"'`：会连带滤掉同行真引用
  （`l7_l1_bridge.rs:201-202` 即「字符串 + 真实路径」同行）。

## P2 · 阶段工程（单独立项，不混做）

| 项 | 为什么独立 |
|---|---|
| 185 个未核验删除 | 需逐条核验才能提交；全在 `stash@{0}` |
| 32 个未跟踪 `scripts/ops` | 含活路径 `nt_graph_audit.py`；全在 stash |
| 5 个陈旧 worktree | 实测全部 dirty ⇒ 删=毁在制品；需先导出内容再删 |
| `TextEmbedder` 换真实现 | 5 个生产调用方，改 `embed` 会**重排全部检索结果**，须先评估影响面 |
| `publish gateway` 真实上传 | 加 `dry_run` 是 B-4（低成本）；投 OAuth2+reqwest 是独立工程 |
| `cascade` `length_score` 改分档 | 改的是记忆晋升评分，需领域评估，不宜顺手改 |
| `edges` 绑定到具体版本 | B-1 的**明示取舍**：`edges` 仍按 `nodes.id` 绑定，不指向版本。要「边绑定到某版」是**独立 schema 议题** |

## P3 · 需你拍板 / 外部输入

| 项 | 状态 |
|---|---|
| **Noise IK 官方测试向量** | ⛔ B-2 唯一卡点。**不接受手写"看起来能跑"的 crypto**；需从 spec 或参考实现取向量 |
| 另一 agent 的并发重构 | 主树被占用（`nt_core_capability_tree` 断链 4+ 处、`d5413335` 删 `apps/neobot-desktop`）。**非我窗任务，我不去修**（L9） |
| 分支合入时机 | `fix/bitemporal-and-layer-ratchet` 何时合入主干 |
| CI `--test-threads` 2→4 | **暂不改**。4 连绿是强证据**非证明**；再观察数轮 |
| `stash@{0}` 是否整体恢复 | 759 tracked + 240 untracked；B-11/C-2/阶段工程都卡在这里 |
| 185 个删除是否恢复 | 同上 |

## 已彻底关闭（勿再翻）

- ⛔ 12 项前提证伪 / 已被外部解决（见 A 组）
- ✅ 本轮 commit：`bdf1e9f1`(门+102 基线) · `f23177de`(记录校正) · `a9ad48f5`(S-1 并车道) ·
  `8eac716e`+`ecd10d3e`(晶体+9 测试归零) · `024c2ae0`(B-1 死 `CapabilityRegistry`/B-4) ·
  `7676f6f8`(streaming 去外网) · `d5edd461`(门清单/2 examples/HOME 竞态/测试门转严格) ·
  `89769661`(4 条 flaky 归零) · `26749ac0`(经验+裁决+Yootta 移除) ·
  `71e1c412`(B-1 真双时间 + 棘轮 102→101) · `c02b2358`/`14db935d`/`67c83e7a`/`95cae243`
  (分层棘轮 **101 → 50**，L2/L3/L4/L5 四层清零) · `c9c1da9d`/`40f5cdda`/`109028bb`(经验+待办)
- ⛔ Yootta gated 401（用户指令移除，理由记录保留）
- ⛔ B-2 之外，B-3 / B-4 / B-5 均**早已完成**，勿当待办重做
