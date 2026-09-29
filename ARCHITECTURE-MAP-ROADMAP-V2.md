# NeoTrix 架构台账（§11 起 · 可再生实测值）

> **2026-09-29 拆分**：原 `ARCHITECTURE-MAP-ROADMAP-V2.md` 的 **§1–§7**
> （308 行）已移入 `docs/architecture/_superseded/ARCHITECTURE-MAP-ROADMAP-V2-deathsnap-2026-09-19.md`。
>
> **为什么移**：§1–§7 是 2026-09-19 的快照，此后 221+ 次提交未回填过任何一格；
> 文中所有百分比（「外部对标覆盖 36%」「平均成熟度 C3.2」）**没有任何机制会更新它**
> ⇒ 永远是 36%。留在根目录只会让人误当现状引用。
>
> **本文只留 §11 起的事实对账层** —— 它的值来自实测与门，可再生、可证伪。
> 结构性门禁见 `scripts/check-truth-surface.sh`。
>
> 台账更新规则 **R-P199**，口径限 `neotrix-core` 的 L1–L6；
> `crates/neotrix-neobot` 是独立 crate、不占 L 层故不进此台账
> （正典记录见 `docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md`）。
>
> 五实体正典：`docs/architecture/FIVE-ENTITY-BLUEPRINT-V3.md`
> （迭代编号 E0–E4 与本文的 I/M/NODE 编号并存，执行以任务清单为准）。

---

## 11. 事实对账层 (2026-09-27 审计新增)

> 本节与 §1–§7 相反: **§1–§7 是意图, 本节是事实。**
> 本节所有数字都必须能被命令重新生成, 不接受人工填写 —— 否则重蹈 §7
> 「外部对标覆盖 36%」永远停在 36% 的覆辙。

### 11.1 可再生实测值

| 指标 | 实测值 | 再生命令 |
|---|---|---|
| Rust 文件 / LOC | 2,945 / 897,274 | `find neotrix-core/src crates -name '*.rs'` |
<!-- 2026-09-28: `apps`（apps/neobot-desktop，d5413335 删除）与 `src-tauri/src`
     （5c02e738 归档）已不在仓库里，留在命令里会让它直接报 no such file or directory。
     桌面端见 docs/api/README.md 的说明。台账数字仍为历史值，R-P199 口径见 AGENTS.md。 -->
| 0 字节 `.rs` | **0** | `find neotrix-core/src crates -name '*.rs' -size 0 \| wc -l` |
| 真值面门禁基线 | **1** | `grep -vc '^#' scripts/truth-surface-baseline.txt` |
| 门禁新增违规 | **0** | `bash scripts/check-truth-surface.sh --strict` |
| HEAD 能否独立编译 | **能** (2026-09-27 起) | `cargo check --tests -p neotrix` |
| `src/` 内 `#[test]` 数 | 13,318 | `grep -rc '#\[test\]\|#\[tokio::test\]' --include='*.rs' neotrix-core/src` |
| 全量套件通过 / 失败 / 忽略 | **12,042 / 50 / 37** | `cargo test -p neotrix --lib --no-run` 后跑二进制, `--test-threads=2` |
| 全量套件耗时 | 101s (2026-09-27 前**跑不完**) | 同上 |
| 永久挂起测试 | **0** (原 6) | `grep -c 'has been running for over' <log>` |
| HEAD 断裂 mod 引用 | **0** (原 9) | 遍历已入库 `mod X;` 取传递闭包, 与 `git ls-tree` 比对 |
| 本轮经验入库 | 16 条 / cycle `audit0927` | `neotrix-experience list --cycle audit0927` |

### 11.2 2026-09-27 审计已除的根 (均为「代码存在但工具链看不见」)

| # | 问题 | 规模 | 状态 |
|---|---|---|---|
| 1 | HEAD 无法独立编译: 已入库文件 `use` 了只存在于工作区的 `mod` 声明 | 29 文件 / 7,339 LOC | ✅ 已入库 |
| 2 | `cli/` 树已入库但 `pub mod cli;` 从未出现在任何 commit → 从未编译 | 87 文件 / 24,884 LOC | ✅ 已删 |
| 3 | 4 个抽取 crate 的 54 个 0 字节模块被 L5 当公开 API 再导出 | 54 文件 | ✅ 已摘 |
| 6 | 6 个 md5 相同的重复文件 (目录重构残留) | 1,769 LOC | ✅ 已删 |
| 7 | `nt_core_capability_tree` 未继承 workspace lint, 4,670 LOC 零约束 | 20 条告警 | ✅ 已修 |
| 8 | `nt_act_trade/tests/` 3 文件未声明 → 311 个测试从不编译 | 4,899 LOC | ✅ 复活 223 (107+116 全绿) / 删 88 |
| 9 | `auto_inspector` 内嵌 cargo 死锁 → 3 个测试永久挂起, 全量套件跑不完 | 3 测试 | ✅ 已修 |
| 10 | `.githooks/{post,pre-merge}` 悬空链接 → `reset --hard` 护栏一直没生效 | 2 hook | ✅ 已复活 |
| 11 | `.gitignore` 的 `tests/` 通配屏蔽 10 个源码目录 | — | ✅ 已解禁 |

**新增门禁** `scripts/check-truth-surface.sh` 卡 4 类:
`EMPTY` / `UNDECLARED` / `TRACKED` / `UNCOMMITTED_DEP`。
棘轮基线用**列表**而非计数 —— 计数基线挡不住「删一加一」, 新违规会隐身。
接线: `Makefile` (`make truth-surface` / `-strict` / `-baseline`) +
`ci.yml` check job (3 OS 矩阵, `--strict`)。

### 11.3 仍开放的一项 (需设计决策)

**`ExtractConfig` / `EmailConfig` / `PlatformRegistry` 三处双定义** —— 分别同时
存在于 `extractors/mod.rs` 与 `data_pipeline.rs`(`PlatformRegistry` 在
`data_pipeline.rs:211` 与 `platform_registry.rs:46`)。测试按所在模块各取一份,
`TradeDataPipeline::with_registry` 只认 `data_pipeline` 那份。这是随时会咬人的
坑, 但两个模块的语义确实不同, **收敛前需先确认二者是否本就该合并**。

### 11.3b 已被推翻的判断 (留档, 防止重犯)

审计中途我写过「`test_orchestration.rs` 的 116 个测试不可修, 属设计决策,
需先裁决两套 `WorkerType` taxonomy 的归属」。**该判断是错的。** 实为:
`DomainWorkerType` / `DomainWorkerResult` 是 `orchestrator_v2::{WorkerType,
WorkerResult}` 的**旧名**, 我把 import 路由到了 `workers` 模块才导致 120 个错。
两个同名符号确实存在 (两个 `WorkerType`、两个 `TradeWorker` trait), 但那是
**并存设计**而非冲突 —— 测试两个都要用, 分别引入作用域即可。

教训: 遇到「同名符号」先确认是否**旧名/新名**关系, 别直接上升为设计冲突。
判据: 若一个符号是另一个的子集且用法自洽, 它多半是重命名而非两套设计。

### 11.4 剩余 50 个失败 + 一个并发崩溃 (仅登记, 需单独一轮 triage)

按模块: nt_shield 7 / nt_core_capability 6 / nt_memory 5 / nt_feel 5 /
nt_core_aware 4 / nt_meta 3 / healing 3 / 其余 17 个各 1-2。
共同点多数是**断言与实现漂移**(如夹具自带 `Always` 规则、亚毫秒时长),
单看容易改, 但 50 个一起动风险大, 应独立成轮。

**并发崩溃 (pre-existing)**: `--test-threads=4` 时测试进程 **SIGSEGV**
(退出码 139), 在 `l6_meta::healing::predictive_maintenance::trend::tests`
之后; 同一模块单线程/两线程跑均不复现(311 passed)。属共享资源竞态或
栈/句柄耗尽, 非本次改动引入(未触碰相关模块)。**CI 暂用 `--test-threads=2`**;
根因需单独定位。

### 11.5 未除的已知债 (仅登记, 不在本次范围)

- `l5_cognition/lib.rs` 式的「目录模块旁挂 lib.rs」副本 (已随本次清掉 1 处;
  机制上仍可能再生, 故门禁只查 `mod` 绑定, 不查此类副本)。
- `sessions/` 80 个文件 / 8.2 MB, 而 `DOCUMENTATION-MAP.md` §一.1 规定根目录
  `TODO.md` 是唯一任务清单、§三.3 禁止每会话独立 TODO —— 规范自身被绕过 80 次。
- `DOCUMENTATION-MAP.md` §三.7 禁 >500 行的 md, 实测 67 个超限 (排除 node_modules)。
  这两条都说明**规范缺少强制点**; 门禁化 (退出码) 才是解药, 与 §11.2 同理。

## 迭代记录 · 2026-09-27 结构性审计（cycle `audit0927`）

**基线 HEAD** `ca814c7b` · 13 个提交 · 经验入库 16 条（`neotrix-experience list --cycle audit0927`）

### 净效果

| 维度 | 前 | 后 |
|---|---|---|
| HEAD 断裂 mod 引用 | 9 | **0**（全新 clone 可构建） |
| 真值面门禁棘轮基线 | 60（其中 57 为待裁决） | **0** |
| 永久挂起测试 | 6（全量套件跑不完） | **0**（101s 跑完） |
| 全量套件通过 / 失败 | 11,299 / 124 | **12,042 / 50** |
| 死码清除 | — | 31,652 LOC（24,884 从未编译 + 1,769 md5 重复 + 4,899 测已删功能） |
| 复活测试 | — | 223（107 + 116，全绿） |
| 真实实现缺陷修复 | — | 8 |

### 三条最可复用的结论（已入 KB）

1. **「存在 ≠ 生效」且编译器报绿** —— 已入库未编译 / 已编译未执行 / 已导出但为空，
   三种形态都让 `cargo test` 与 `cargo check` 失去信号价值。必须门禁化，不能靠文档。
2. **「无法编译」是最容易下错的结论** —— 先量再判。同一个文件 31 个错里 21 个是机械性的；
   120 个错里绝大多数是「import 指错模块」。遇同名符号先确认是否旧名/新名关系。
3. **指标类代码最容易被跳过验证** —— 8 个缺陷里多数表象是「函数都在、返回值也看着合理」，
   复活测试才把它们暴露出来。

### 本轮未除（已登记，见 §11.4 / §11.5）

- 50 个失败测试（需独立一轮 triage）
- `--test-threads=4` 时 SIGSEGV（pre-existing 并发问题，CI 暂用 2 线程）
- `ExtractConfig`/`EmailConfig`/`PlatformRegistry` 三处双定义
- `sessions/` 80 文件与 67 个超 500 行 md 绕过 `DOCUMENTATION-MAP.md` —— 规范缺强制点

### 安全网

`refs/audit/snapshot-20260927`（已 pin，GC 不可回收）+ 15 MB untracked 备份 tar。
全程未动他窗在途文件；期间 4 次撞到他人在途编辑导致的编译失败，均按「等稳定」处理，
仅在必要时最小修并明示可 revert。
