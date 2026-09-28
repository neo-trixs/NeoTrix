# Handoff · 单窗口汇总（2026-09-28 收口）— 请由**单一窗口**统一修复

> **本文是所有窗口任务的唯一汇总入口。** 目标：让**一个**窗口接手，统一修复下面全部条目，
> 避免多窗口在 16 GB 机器上并发编译（AGENTS.md：「16G 机必爆 swap」）。
>
> 汇总自：`handoff-neobot-absorption-20260928` · `handoff-test-debt-20260928` ·
> `handoff-cocoons-health-20260928` · `handoff-medical-absorb-20260928` ·
> `handoff-mimo-rlenv-absorb-20260928` · `handoff-hf-batch-absorb-20260928` ·
> `handoff-crystal-consolidate-20260928` · `handoff-s-audit0927` · 本会话（架构侧）
>
> 全部数据 2026-09-28 实测。带 `[实测]` 的是本人复核过的；带 `[转述]` 的是引用他窗结论。

---

## 1. 会话标识

- 本窗口：架构侧分析 + 隔离车道（`lane/batch-fix-20260928`）
- 主树分支：`feat/capability-absorb-20260828` @ `70356116`
- 交接时间：2026-09-28 15:1x

---

## 2. ⛔ 接手前必读的三条硬约束

| # | 约束 | 证据 |
|---|---|---|
| 1 | **内存闸 BLOCKED 时禁止 cargo** | `sh scripts/ops/nt_mem_gate.sh` → exit 2；实测两个 `rustc` 各约 2.5 GB |
| 2 | **禁多窗口并发 `--all-targets`/`--test`** | AGENTS.md 并行公约；16 GB 机器 |
| 3 | **禁 `git add -A` / `git reset --hard`** | 主树含他窗 961 个未提交改动；`handoff-neobot-absorption` 第 1 条也是这么写的 |

### 2.0 ⭐ 复核纪律（本会话两次翻车换来的）

1. **任何「某仓是死代码 / 是 stub / 是孤儿」的断言**，动手前先打开
   `docs/architecture/absorption-sources/repos.csv` 确认该仓确实在吸收清单内，
   再用 **构造点**（`Type::new(` 调用点）复核。**本会话凭旧文档删过一次活代码** ——
   `nt_core_gate/nt_tool_registry.rs` 被标 stub，实测有活消费者。
2. **任何许可判断**，先查 `absorption-sources/LICENSES.md`；表里没有就自己跑
   `curl -s https://raw.githubusercontent.com/<owner>/<repo>/HEAD/LICENSE | head -4`。
3. **不要凭印象补全 `owner/repo`** —— 本会话因此在许可台账里写出过 3 个不存在的仓库。

**⇒ 第一件事：确认无他窗在写。**

```bash
find neotrix-core/src crates src-tauri/src -name '*.rs' -mmin -5 | head   # 必须空
git status --porcelain | wc -l                                             # 确认规模
sh scripts/ops/nt_mem_gate.sh; echo $?                                     # 必须 0 才起 cargo
```

---

## 3. 🟢 零编译风险（现在就能做，不需 cargo）

| ID | 任务 | 落点 | 验收 |
|---|---|---|---|
| **S-1** | 合并隔离车道 | `lane/batch-fix-20260928`（3 commit） | 合并后 `check-layer-deps.sh --strict` 在**主树**重测 |
| **S-2** | ⛔ **基线只在主树测** | `scripts/layer-deps-baseline.txt` | 见 §6 事故：**车道测得 102、主树测得 94** |
| **S-3** | 裁决 `nt_file_ability` 层归属 | 声明 L1 却引 L2/L5/L6（10 处） | 改 `.neotrix/layer-map.json` 或移码 |
| **S-4** | 裁决 `ffi` 层归属 | 声明 L0 却引 L5：`consciousness_tree.rs:290,320,321` · `seal_pipeline.rs:131,132` | 同上 |
| **S-5** | 补 `nt_core_gate` 防误删注释 | `nt_core_gate/nt_tool_registry.rs` 文件头 | 写明活消费者 `nt_shield_enforcer.rs:388-390` |
| **S-6** | 统一提交 neobot 那 54 个路径 | `git add` **精确列出**，不用 `-A` | 见 `handoff-neobot-absorption` |
| **S-7** | 拍板 `/stop` 回执措辞 | **唯一需产品决策项**，见 §5 | — |

---

## 4. 🔵 需编译验证（等内存闸 OPEN）

### 4.1 冗余清理（本人实测复核，零消费者已验证）

| ID | 任务 | 证据 | 风险 |
|---|---|---|---|
| **B-1** | 删 `neotrix/nt_file_ability/capability.rs:185` 的 `CapabilityRegistry` | 精确搜索 **0 消费者** | 🟢 |
| **B-2** | 删 `l5_cognition/nt_core/capability/registry.rs:462` 的 `CapabilityRegistry` | 仅自测；删前查 `mod.rs` 的 `pub use` | 🔵 |
| **B-3** | `neotrix-types` 包内 `SkillRegistry` 2→1（`core/skill.rs:54` / `core/skills/mod.rs:25`） | 包内自重复 | 🟢 |
| **B-4** | `error_conversions.rs` **6 份 → 1** | `l1_action` 26 个 From impl / `l5` 5 / `l3` 5 / `neotrix` 4 / `l2` 3 / `l6` 3；**直接对应 DIR-AUDIT 记的 17 个 E0119** | 🔴 |

> ⛔ **不要删** `nt_core_gate/nt_tool_registry.rs` —— 09-27 路线图说它是 stub，**实测证伪**：
> `l3_embodiment/nt_shield_enforcer.rs:388-390` 在用，承载**写操作可逆性**，与 `nt_act`
> 的运行期 `ToolStats` **正交**。同理 `agentic_browse::ToolRegistry` 也正交。

### 4.2 跨域错位

| ID | 任务 | 落点 |
|---|---|---|
| **D-1** | `neotrix-sysctl`（唯一 `unsafe` FFI）从 L5 剥离 | 5 个 L5 包 `Cargo.toml`；只让 `l0_substrate` 依赖它 |
| **D-2** | `nt-lang` 孤儿（5 文件/273 行，**只有 `[[bin]]` 无 `[lib]`**，**0 个主树 manifest 依赖**） | 删或补 `[lib]`+消费者 |
| **D-3** | `ARCHITECTURE.md:97` 纠正 | 声称 `nt_computer/` 是「计算集群」；**实测是 fs/process trait**，`screenshot()` 默认 `None`，neobot 侧 `NoopBackend` 唯一后端 |
| **D-4** | `nt_file_ability` / `ffi` 层归属 | 同 S-3/S-4（解码层） |

### 4.3 能力补齐

| ID | 任务 | 落点 | 来源 |
|---|---|---|---|
| **E-1** | 证伪门 | `crates/neotrix-audit/` + CI | harness-engineering `evals/README.md` |
| **E-2** | 记忆权威头 + 五个留存标签 | `experience_tree/mod.rs:29` | loopx + nanobot |
| **E-3** | delta-ops 取代整体重写 | `experience_tree/mod.rs:241` `:355` | Hindsight `reflect/delta_ops.py` |
| **E-4** | 成本归因插点 | `anthropic/anthropic.rs:93`（P0-4 断点） | cost-xray |
| **E-5** | GUI 执行回路 5 项 | `l3_embodiment/nt_computer.rs` | agent-desktop + cua-driver + OpenAdapt |
| **E-6** | DNS qtype 白名单 | `egress_types.rs:14-21`（**全文件零 DNS 概念**） | OpenAI 2026-09-20 事故 |

---

## 5. 🟡 唯一需要人拍板的一项

**`/stop` 回执措辞** —— `handoff-neobot-absorption` §6 阻塞 2。

原问题：桌面停止键只置 `shell.stopRequested` 让渲染跳过增量，`spawn_blocking` 照跑 ——
回执却推荐「按发送键」，**把一个不取消任何东西的键钉成了契约**。
已改为「只说事实 + 给真能生效的路径（退出 App）」，测试改为**禁止**出现动作短语。
**⇒ 需要产品侧确认这个措辞是否可接受。**

---

## 6. ⚠️ 本会话自己的两次错误（务必避免重犯）

### 6.1 测量台错误（R-SCAN-3 实例）

| 测量台 | 分层违规数 |
|---|---|
| 主树**脏**（含他窗 561 个未提交改动） | **94** ← 我最初测的，写进了基线 |
| 隔离车道**干净 HEAD** | **102** |

**真因**：主树那批未提交改动恰好**修好了** 8 处 `l1_action → l2_perception` 违规。
⇒ **同样的代码，脏树比干净树「更干净」。脏树不是合法测量台。**

**规约**：
1. 基线只在**主树**测，且测时主树相对其 HEAD **无未提交 `.rs` 改动**。
2. **隔离车道不得提交基线更新** —— 合并时先合代码，再在主树跑 `--update-baseline`。
3. 门记录要写**测量台**，不只是时间戳。

### 6.2 机器读文件不能加注释

我给 `scripts/layer-deps-baseline.txt` 加了 `#` 注释头 ——
门的读法是 `grep -c . $BASELINE`（`check-layer-deps.sh:153`），
**注释行被计为条目**，94 → 105，棘轮失真。已 revert。

**⇒ 警示写文档，不要写进机器读的 ledger。**

### 6.3 共享 index 会竞争

`git add` 后立刻 `git diff --cached`，暂存区里出现了 **46 个不是我暂存的 `.rs`**
—— 他窗在我 add 与 check 之间跑了 add。
⇒ **worktree 有独立 index**（`.git/worktrees/<name>/index`），这才是真隔离。

---

## 7. 各窗口剩余债（转述，供参考，未逐条复核）

> 以下标 `[转述]`。**接手前请自己复核**，不要照单全收 ——
> 本会话已抓到两次「转述与实测不符」（`nt_tool_registry` 非 stub；`nt_jev` 是活路径）。

| 来源 | 声明的剩余项 |
|---|---|
| `handoff-neobot-absorption` [转述] | 提交 54 路径 · 拍板 `/stop` · `nt_smoke.sh` **6 步编排从未整体执行过**（各步手工跑绿）· `edit_of` 真正生效 · IM 的 `/stop` 兑现（并发化，见 `DESIGN-CHANNEL-DISPATCH.md` §11/§13） |
| `handoff-test-debt` [转述] | 三笔债：`nodes` 表无法存双时间历史（2 条测试）· Noise 握手协议 · 92 分层违规 |
| `handoff-cocoons-health` [转述] | **id 分配无互斥**：8 个吸收脚本各自独立 `fast_max_mid(COCOONS)` 算 start id，涉事 8 个茧时间戳集中在 5 分钟内 |
| `handoff-s-audit0927` [转述] | 50 个失败测试（独立成轮，勿与结构清理混做）· `--test-threads=4` SIGSEGV（崩在 `l6_meta::healing::predictive_maintenance::trend::tests`）· 三处同名双定义 `ExtractConfig`/`EmailConfig`/`PlatformRegistry` · `nt-lang` · `nt_core_capability_tree` 归属 |

**⇒ 注意其中 3 项已被本会话实测覆盖或推翻**：
`nt-lang`（实测 0 依赖，确认孤儿）· `nt_core_capability_tree` 归属（**已移至 `crates/`，且是活路径，勿删**）· 分层违规数（**已从 92 降到 84 → 纳入第二棵树后 94/102，取决于测量台**）。

---

## 8. 建议执行顺序

```
S 组（零编译）—— 现在就能做完，包括合并车道
  └─→ S-2 基线在主树重测
        └─→ B-1/B-3（零风险删）──→ B-2 ──→ B-4（解 17 个 E0119）
              └─→ D-1/D-2 ──→ C 组（脱 stub / orchestrator 收敛）
                    └─→ E 组（能力补齐）
```

**每组之间跑一次**：

```bash
sh scripts/ops/nt_mem_gate.sh; echo $?              # 0 才起 cargo
bash scripts/check-layer-deps.sh --strict           # PASS 0 new
bash scripts/check-truth-surface.sh --strict       # 干净检出应 0
python3 scripts/ops/nt_lock_audit.py neotrix-core/src   # 0
cargo check --lib -p neotrix
```

---

## 9. 回滚

```bash
git switch -c fix/batch-consolidated-20260928
git diff --stat                          # 先看改了什么
git checkout -- <具体文件>                # 只回滚自己的
```

⛔ **绝不用 `git reset --hard` / `git checkout .`** —— 主树含他窗大量未提交改动，
一条 `reset --hard` 就是 2026-09-22 的第三次覆盖事故。

---

## 10. 正典文档（已入库）

| 文档 | 作用 |
|---|---|
| **`docs/architecture/absorption-sources/`** | **吸收源清单（483 仓 CSV + 436 条榜单排名 + 许可台账 + 5 论文）** ⭐ 复核任何结论都从这开始 |
| `docs/architecture/EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md` | 正典路线图（483 仓 + 5 arXiv） |
| `docs/architecture/DIR-REMEDY-2026-09-28.md` | 目录架构解法（层归属显式化，不搬目录） |
| `docs/architecture/OWNERSHIP.md` | 唯一裁决表（**以构造点取证**，含 §9 测量台事故） |
| `docs/architecture/BATCH-FIX-CHECKLIST-2026-09-28.md` | A/B/C/D/E 任务清单 + 三道闸 + 回滚 |
| `.neotrix/layer-map.json` | 层归属真源 |
