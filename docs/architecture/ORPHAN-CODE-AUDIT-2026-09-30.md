# 幽灵代码特性判定 —— ORPHAN-CODE-AUDIT-2026-09-30

- **范围**：`/tmp/ghost.txt` 的 **279** 个「未参与 `cargo check --workspace` 编译」的 `.rs`
- ⛔ **本次未删、未改、未 `git mv` 任何文件**；本文只产出判定与清单。
- 清单直接采用给定 `/tmp/ghost.txt`，**未重新推导**。
- 「参与编译 / 未参与编译」由**自建 Rust 2018 模块树解析器**独立判定，
  与 rustc dep-info 交叉验证后 **0 处分歧**（§6.1）。

---

## 1. 统计

| 类别 | 含义 | 文件数 | 行数 | 行占比 |
|---|---|---:|---:|---:|
| **A** | A 真的死代码 | 212 | 50,454 | 82.9% |
| **B** | B `#[path]` 挂载 | 0 | 0 | 0.0% |
| **C** | C Cargo 入口目标 | 43 | 5,907 | 9.7% |
| **D** | D cfg / feature 门控 | 23 | 4,500 | 7.4% |
| **X** | X 口径产物（rustc 确实编过） | 1 | 10 | 0.0% |
| — | **合计** | **279** | **60,871** | 100% |

### 1.1 各类要点

**A（真的死）= 212 个 / 50,454 行**，占绝对多数，分布在 **98 棵孤儿子树**里。
其中 **81 个**的 `mod` 声明是被同一个 commit `477bf669` 删掉的 —— 那是个自称
只做文档的 commit，实际改了 4,535 个文件。详见 §4.1。

**B = 0。** 全仓 60 个 `#[path]` 挂载目标**全部**在 dep-info 里 ⇒
「dep-info 可能漏 `#[path]` 目标」这条假设**被证伪**（§3.5）。

**C = 43 个 / 5,907 行**，其中 **41 个是 `neotrix-core/examples/`**。
根 `Cargo.toml` 与全部 10 个成员 crate 的 `Cargo.toml` 都**没有** `autoexamples = false`
⇒ Cargo 自动发现生效，这些都是货真价实的 example target；`cargo check` 不带
`--all-targets` 不编 example ⇒ **整类误判**。

**D = 23 个 / 4,500 行。** feature 分布：`ios-bridge` ×12、`sandbox` ×3、`simd-vsa` ×2、
`desktop` ×2、`rkyv-storage` ×1、`telemetry` ×1、`#[cfg(test)]` ×1、
`required-features` 目标 ×1。本次 `default = ["keyring", "anydoc", "stealth-net"]`，以上全未开。

**X = 1 个 / 10 行 —— 给定口径的假阳性。** `neotrix-core/build.rs` 确实被 rustc 编译了，
它的 dep-info 落在 `target/debug/build/neotrix-*/build_script_build-*.d`，
而口径只扫了 `target/debug/deps/`。详见 §6.4。

---

## 2. 判据

| 类别 | 判据 | 检验方式 |
|---|---|---|
| A | ①最近的活 `mod.rs`/`lib.rs` 无 `mod X;` 声明 ②全 history 搜 `mod X;` 找到删除该声明的 commit ③无 `#[path]` 引用 ④不是任何 Cargo target | 模块树解析 + `git log -S` |
| B | 存在 `#[path = "..."] mod X;` 把它接进模块树 | 模块树解析（**结果 0 个**） |
| C | `examples/` `tests/` `benches/` `src/bin/` 下的自动发现目标，或 `Cargo.toml` 里的 `[[example]]`/`[[bin]]`/`[[test]]`/`[[bench]]` 段 | 读全部 12 个 `Cargo.toml` |
| D | 声明它的那一行带 `#[cfg(...)]` 且该 cfg 本次为假；或 target 带未满足的 `required-features` | 模块树解析记录 cfg 上下文 |

---

## 3. 「看起来死、其实是活的」实例（附证据原文行）

### 3.1 `neotrix-core/src/neotrix/ffi/*` —— 12 个文件 / 2,752 行（D 类）

证据原文（`neotrix-core/src/neotrix/mod.rs:15-16`）：
```
#[cfg(feature = "ios-bridge")]
pub mod ffi;
```
`lib.rs:41-47` 还把这 12 个 impl 全量 re-export（`E8ReasoningImpl`、
`VSAHyperCubeImpl`、`ConsciousnessTreeImpl` …）⇒ 它们是 iOS/Swift FFI 的**唯一实现体**。
⚠️ 改这一棵树必须另跑 `cargo check -p neotrix --features ios-bridge`
（AGENTS.md §4.2 已记录此坑）。

### 3.2 `crates/neotrix-neobot/src/nt_testutil.rs` —— 46 行（D 类）

证据原文（`crates/neotrix-neobot/src/lib.rs:60-62`）：
```
/// 仅测试目标编译：测试期临时目录唯一化工具（见模块头「为什么需要它」）。
#[cfg(test)]
mod nt_testutil;
```
该模块在 **30 处**被调用（`nt_agent.rs:1580`、`nt_vision.rs:324`、`nt_git.rs:532` …）。
若按「未参与编译 ⇒ 死」删掉，**整个 crate 的测试会集体编译失败**。

### 3.3 `neotrix-core/examples/demo/{mcp_tools_demo,reasoning_brain_demo,seal_loop_demo}.rs` —— 781 行（C 类）

这三个文件既不在 `examples/` 顶层（不是独立 target），也不在 `demo/main.rs` 里以
文件模块形式出现。证据原文（`neotrix-core/examples/demo/main.rs:9-11`）：
```
mod reasoning_brain_demo;
mod seal_loop_demo;
mod mcp_tools_demo;
```
它们是 example target `demo` 的子模块 ⇒ `cargo run --example demo` 会编。

### 3.4 `neotrix-core/src/l3_embodiment/nt_shield/shield_core/{keyvault,vault}.rs` —— 668 行（D 类）

证据原文（`shield_core/mod.rs:15-18`）：
```
#[cfg(feature = "sandbox")]
pub mod keyvault;
#[cfg(feature = "sandbox")]
pub mod vault;
```
### 3.5 负例：B 类一个都没有 —— 你的假设被证伪

全仓 `#[path = "..."]` 出现在 6 个文件里、共 **60** 个挂载目标。逐个核对：
**60/60 都在 dep-info 里**，rustc 一个都没漏（`l1_facade_meta.rs`、
`handlers_consciousness/*.rs`、`src/bin/experience/exp_*.rs`、`engine/nt_*.rs` …）。
⇒ dep-info 对 `#[path]` 可靠，**不需要**为它做人工复核。

---

## 4. 「真的是死的」实例（附 `git log` 证据）

### 4.1 头号批量现场：`477bf669` 一次弄丢 81 个

```
$ git log -1 --format=%h%n%ad%n%s --date=iso 477bf669
477bf669270b72a49d316dc40590143402dbcb39
2026-09-20 16:23:11 +0800
docs: establish documentation standard and cleanup

$ git show --stat --oneline 477bf669 | tail -1
 4535 files changed, 174964 insertions(+), 703249 deletions(-)

$ git show --diff-filter=A --name-only --format="" 477bf669 | grep -c "\.rs$"
486
```
该 commit 把 `neotrix-core/src/l4_emotion/nt_memory/mod.rs` **新建**
（diff header 为 `new file mode`）并只声明了部分子模块，于是同批落地的 486 个 `.rs`
里有一大批留在磁盘上、没被任何 `mod` 引用。

**逐例 1 —— `.../nt_memory/paged_kv/mod.rs`（116 行）**
```
$ rg -n "paged_kv" neotrix-core/src/l4_emotion/nt_memory/mod.rs
(0 hits —— 父 mod.rs 确实没声明)

$ git log --oneline -S "paged_kv" --all -- "*.rs"
477bf669 docs: establish documentation standard and cleanup   <- 删掉声明
6c93c459 feat: P0融合实现+跨域修复+6来源吸收+循环318-320     <- 当初加的声明

$ git log --oneline --diff-filter=A -- .../paged_kv/mod.rs
477bf669 docs: establish documentation standard and cleanup   <- 同一 commit 落地文件
```
**逐例 2 —— `.../nt_shield/defense/ring_core/`（4 文件）**
```
$ rg -n "ring_core" neotrix-core/src/l3_embodiment/nt_shield/defense/mod.rs
(0 hits)
$ git log --oneline -S "ring_core" --all -- "*.rs"
477bf669 docs: establish documentation standard and cleanup
```
**逐例 3 —— `neotrix-core/src/l6_meta/lib.rs`（68 行，3 文件子树）**

这一条是**另一种死法**，值得单独点出：`neotrix-core/src/l6_meta/` 下同时存在
`lib.rs` **和** `mod.rs`。`src/lib.rs:69` 的 `pub mod l6_meta;` 在 Rust 2018 下解析到
`l6_meta/mod.rs`（编译中），`l6_meta/lib.rs` 是旧 `lib.rs` 布局时代的**残留孪生文件**，
shadowed，从未参与编译；它还带着自己的子树（`nt_core_qtest.rs`）。
```
$ git log --oneline -1 -- neotrix-core/src/l6_meta/lib.rs
477bf669 docs: establish documentation standard and cleanup

$ git log --oneline -1 -- neotrix-core/src/l6_meta/mod.rs
2bbed32c chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库
```
（`l6_meta/lib.rs` 与 `mod.rs` 并存本身是 Rust 2018 的 E0761 冲突条件，只因声明
走的是 `mod.rs` 才没炸。）

### 4.2 53 个「生下来就没接线」

`git log -S "mod X;" --all` **在全部历史里都搜不到** —— 这些文件从被 `git add` 的
那一刻起就没有任何 `mod.rs` 声明过它们。例：

- `neotrix-core/src/l1_action/nt_act/agent_loop/`（9 文件，根 `mod.rs` 45 行）
  —— `nt_act/mod.rs` 有 25 条 `mod` 声明，`agent_loop` 不在其中；由 `e20ebe34` 落地。
- `neotrix-core/src/l2_perception/nt_world/temporal_kg/`（8 文件，根 20 行）
  —— `nt_world/mod.rs` 无 `temporal_kg`。
- `crates/neotrix-types/src/cleanup_types.rs`（51 行）
  —— `neotrix-types/src/lib.rs` 无 `cleanup_types`；由 `dfdc4d48` 落地。

### 4.3 逐 commit 的 A 类分布

| 删掉 `mod` 声明的 commit | 文件数 |
|---|---:|
| `477bf669` | 81 |
| `NEVER-DECLARED`（全历史从未声明过） | 53 |
| `e20ebe34` | 10 |
| `5c02e738` | 9 |
| `2bbed32c` | 7 |
| `1b5d5e6b` | 6 |
| `8aab76f6` | 6 |
| `13dfd9a8` | 4 |
| `acaf0941` | 4 |
| `2a480283` | 3 |
| `848539fe` | 3 |
| `831c3203` | 3 |
| `b291f3d5` | 3 |
| `d4850dba` | 2 |
| `4ab738e7` | 2 |
| `9d2b84c8` | 2 |
| `41f2fc2a` | 2 |
| `15b12f96` | 1 |
| `4527f202` | 1 |
| `3a3ea364` | 1 |
| `e75272be` | 1 |
| `84e98aff` | 1 |
| `32efc110` | 1 |
| `74ad4869` | 1 |
| `f52238d2` | 1 |
| `50799ccc` | 1 |
| `2ad3cf7f` | 1 |
| `d3b58247` | 1 |
| `1e97e730` | 1 |

---

## 5. 逐文件清单

### 5.1 C 类 —— Cargo 入口目标（43 个 / 5,907 行）

| 路径 | 行数 | 类别 | 证据 | 建议 |
|---|---:|:--:|---|---|
| `crates/neotrix-types/examples/extract_pdf.rs` | 70 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `crates/nt-core-capability-tree/tests/epistemic_wiring.rs` | 87 | C | `tests/` 自动发现 ⇒ 集成测试 target | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/auto_evolve.rs` | 67 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/batch_ingest_nomad.rs` | 12 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/build_real_world.rs` | 190 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/check_template.rs` | 25 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/chinese_knowledge_inject.rs` | 168 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/civilization_replica.rs` | 182 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/comprehensive_earth_mine.rs` | 167 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/consciousness_kernel.rs` | 173 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/cortex_mine.rs` | 138 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/deep_web_miner.rs` | 274 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/demo/main.rs` | 234 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/demo/mcp_tools_demo.rs` | 233 | C | `neotrix-core/examples/demo/main.rs:9-11` 的 `mod` ⇒ 隶属 example target `demo` | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/demo/reasoning_brain_demo.rs` | 247 | C | `neotrix-core/examples/demo/main.rs:9-11` 的 `mod` ⇒ 隶属 example target `demo` | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/demo/seal_loop_demo.rs` | 301 | C | `neotrix-core/examples/demo/main.rs:9-11` 的 `mod` ⇒ 隶属 example target `demo` | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/depth_methodology.rs` | 150 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/earth_civilization_mine.rs` | 211 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/earth_dimension_chain.rs` | 246 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/earth_mine_quick.rs` | 112 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/esoteric_chinese_knowledge.rs` | 216 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/evolve.rs` | 256 | C | `neotrix-core/Cargo.toml` 的 `[[example]]` 显式段 | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/evolve_test.rs` | 53 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/expand_world_knowledge.rs` | 291 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/final_comprehensive.rs` | 108 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/god_level_knowledge.rs` | 135 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/guji_embed_fill.rs` | 65 | C | `neotrix-core/Cargo.toml` 的 `[[example]]` 显式段 | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/inject_knowledge_json.rs` | 92 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/kb_flocktest.rs` | 54 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/kb_probe.rs` | 139 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/knowledge_distiller.rs` | 329 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/knowledge_engine_mine.rs` | 3 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/knowledge_mine.rs` | 64 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/nt_whisper_spike.rs` | 39 | C | `neotrix-core/Cargo.toml` 的 `[[example]]` 显式段 + `required-features = ['onnx']` | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/pdf_enhance_test.rs` | 150 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/process_url.rs` | 32 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/proxy.rs` | 94 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/quick_start.rs` | 59 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/test_semantic.rs` | 22 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/todo_parallel.rs` | 54 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/unified_exploration_loop.rs` | 66 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/v2_quick_start.rs` | 112 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |
| `neotrix-core/examples/web_mine_deep.rs` | 187 | C | `examples/` 自动发现（无 `autoexamples = false`） | 保留（活的目标；`--examples` / `--tests` 会编） |

### 5.2 D 类 —— cfg / feature 门控（23 个 / 4,500 行）

| 路径 | 行数 | 类别 | 证据（声明原文位置） | 建议 |
|---|---:|:--:|---|---|
| `crates/neotrix-neobot/src/nt_testutil.rs` | 46 | D | `crates/neotrix-neobot/src/lib.rs:61-62` → `#[cfg(test)]` / `mod nt_testutil;` | 保留（活的；改动须带 feature 复验） |
| `crates/neotrix-types/src/core/nt_core_hcube/vsa_holon.rs` | 50 | D | `crates/neotrix-types/src/core/nt_core_hcube/mod.rs:7-8` → `#[cfg(feature = "simd-vsa")]` / `pub mod vsa_holon;` | 保留（活的；改动须带 feature 复验） |
| `crates/neotrix-types/src/core/nt_core_rkyv.rs` | 155 | D | `crates/neotrix-types/src/core/mod.rs:40-41` → `#[cfg(feature = "rkyv-storage")]` / `pub mod nt_core_rkyv;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/bin/uniffi-bindgen.rs` | 8 | D | `neotrix-core/Cargo.toml:19-22` → `[[bin]] name="uniffi-bindgen"` + `required-features = ["ios-bridge"]` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/l1_action/nt_io/nt_io_desktop/mod.rs` | 9 | D | `neotrix-core/src/l1_action/nt_io/mod.rs:54-55` → `#[cfg(feature = "desktop")]` / `pub mod nt_io_desktop;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/l1_action/nt_io/nt_io_desktop/updater_signing.rs` | 355 | D | `nt_io_desktop/mod.rs` 的子模块，同上 `#[cfg(feature = "desktop")]` 门控 | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/l1_action/nt_io/nt_io_plugin/wasm.rs` | 108 | D | `.../nt_io_plugin/mod.rs:7-8` → `#[cfg(feature = "sandbox")]` / `pub mod wasm;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/l1_action/nt_io/nt_io_telemetry.rs` | 65 | D | `.../nt_io/mod.rs:64-65` → `#[cfg(feature = "telemetry")]` / `pub mod nt_io_telemetry;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/l2_perception/nt_core_hcube/vsa_holon.rs` | 113 | D | `.../l2_perception/nt_core_hcube/mod.rs:23-24` → `#[cfg(feature = "simd-vsa")]` / `pub mod vsa_holon;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/l3_embodiment/nt_shield/shield_core/keyvault.rs` | 343 | D | `.../nt_shield/shield_core/mod.rs:15-16` → `#[cfg(feature = "sandbox")]` / `pub mod keyvault;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/l3_embodiment/nt_shield/shield_core/vault.rs` | 325 | D | `.../nt_shield/shield_core/mod.rs:17-18` → `#[cfg(feature = "sandbox")]` / `pub mod vault;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/consciousness_tree.rs` | 476 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/constellation_system.rs` | 85 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/dual_specialization.rs` | 91 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/e8_reasoning.rs` | 206 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/gwt_attention.rs` | 251 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/kb_bridge.rs` | 172 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/mod.rs` | 343 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/rune_socketing.rs` | 77 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/seal_pipeline.rs` | 209 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/skill_tree.rs` | 352 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/types.rs` | 490 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |
| `neotrix-core/src/neotrix/ffi/vsa_hypercube.rs` | 171 | D | `neotrix-core/src/neotrix/mod.rs:15-16` → `#[cfg(feature = "ios-bridge")]` / `pub mod ffi;` | 保留（活的；改动须带 feature 复验） |

### 5.3 X 类 —— 口径产物（1 个 / 10 行）

| 路径 | 行数 | 类别 | 证据 | 建议 |
|---|---:|:--:|---|---|
| `neotrix-core/build.rs` | 10 | X | rustc 编过：`target/debug/build/neotrix-a53e03f6c9673e6f/build_script_build-a53e03f6c9673e6f.d` 首行即 `neotrix-core/build.rs` | **从幽灵清单剔除**；勿删 |

### 5.4 A 类 —— 真的死代码（212 个 / 50,454 行），按孤儿子树根分组

A 类共 **98 棵**孤儿子树。下表每行一棵；子文件逐条列在 §5.5。

| 孤儿子树根 | 文件数 | 行数 | 删声明的 commit | 建议 |
|---|---:|---:|---|---|
| `neotrix-core/src/l1_action/nt_act/agent_loop/mod.rs` | 9 | 2096 | `e20ebe34`×6 / `5c02e738`×2 | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/mod.rs` | 8 | 1270 | `477bf669`×6 / `84e98aff` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/mod.rs` | 6 | 1028 | `477bf669`×5 / `8aab76f6` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_memory/mod.rs` | 6 | 2073 | `477bf669`×3 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_inner/mod.rs` | 5 | 610 | `477bf669`×4 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/mod.rs` | 5 | 560 | `477bf669`×4 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/admission_control/mod.rs` | 5 | 581 | `acaf0941`×2 / `5c02e738` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/decay_forgetting/mod.rs` | 5 | 734 | `477bf669`×3 / `5c02e738` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/mod.rs` | 5 | 1248 | `1b5d5e6b`×4 / `74ad4869` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/memory/mod.rs` | 5 | 1166 | `477bf669`×2 / `9d2b84c8`×2 | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/coordinator/mod.rs` | 5 | 1441 | `848539fe`×2 / `f52238d2` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/graph_orch/mod.rs` | 5 | 1735 | `477bf669`×4 / `2bbed32c` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/mod.rs` | 5 | 1479 | `477bf669`×5 | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/cross_domain/mod.rs` | 5 | 2644 | `477bf669`×4 / `831c3203` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/context_router/mod.rs` | 5 | 913 | `477bf669`×3 / `5c02e738` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/mod.rs` | 5 | 1166 | `477bf669`×4 / `8aab76f6` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/geo_seo/mod.rs` | 4 | 587 | `13dfd9a8`×2 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/nt_act_dev_tools/mod.rs` | 4 | 578 | `477bf669`×3 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/semantic_routing/mod.rs` | 4 | 844 | `3a3ea364` / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/mod.rs` | 4 | 340 | `477bf669`×2 / `5c02e738` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_core/mod.rs` | 4 | 642 | `477bf669`×3 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/mod.rs` | 4 | 1450 | `477bf669`×3 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/add_only_writes/mod.rs` | 4 | 789 | `477bf669`×3 / `8aab76f6` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/self_improvement/mod.rs` | 4 | 2125 | `477bf669`×3 / `acaf0941` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/coordination/nt_meta_cleanup/mod.rs` | 4 | 343 | `e20ebe34` / `848539fe` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/nt_meta/eval_engine/mod.rs` | 4 | 650 | `477bf669`×3 / `2ad3cf7f` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/mod.rs` | 4 | 1149 | `477bf669`×2 / `5c02e738` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/prompt_manager/mod.rs` | 4 | 632 | `477bf669`×4 | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_cleanup/mod.rs` | 3 | 165 | `d4850dba`×2 / `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/lib.rs` | 3 | 1292 | `50799ccc` / `477bf669` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/io_skills/mod.rs` | 2 | 85 | `32efc110` / `1b5d5e6b` | 待裁决（先确认 `docs/` 无引用再动） |
| `crates/neotrix-types/src/cleanup_types.rs` | 1 | 51 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `crates/neotrix-types/src/core/nt_core_bank/compressor.rs` | 1 | 204 | `477bf669` | 待裁决（先确认 `docs/` 无引用再动） |
| `crates/neotrix-types/src/core/nt_core_knowledge/provider.rs` | 1 | 229 | `15b12f96` | 待裁决（先确认 `docs/` 无引用再动） |
| `crates/neotrix-types/src/core/nt_crypto_util.rs` | 1 | 51 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `docs/standards/templates/FITNESS-FN-TEMPLATE.rs` ⛔**不是死代码，是文档模板** | 1 | 40 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/actions/media/nt_3d_task_spec.rs` | 1 | 207 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/nt_act_scheduler.rs` | 1 | 747 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/nt_act_trade/orchestrator_compat.rs` | 1 | 32 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/nt_act_trade/unified_types_compat.rs` | 1 | 41 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/nt_act_workspace_isolator.rs` | 1 | 719 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_act/provider_abstraction/models.rs` | 1 | 257 | `4527f202` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_infra_unified_search.rs` | 1 | 485 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_io/nt_io_eli5.rs` | 1 | 82 | `1b5d5e6b` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l1_action/nt_io/nt_io_protocol_bridge.rs` | 1 | 473 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l2_perception/error_conversions.rs` | 1 | 24 | `e75272be` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l2_perception/nt_world/crawl/dom_extractor/mod.rs` | 1 | 251 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l2_perception/nt_world/crawl/ordered_backend_router/mod.rs` | 1 | 386 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l2_perception/nt_world/sense/tests/integration.rs` | 1 | 93 | `13dfd9a8` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l2_perception/nt_world/traits.rs` | 1 | 41 | `2a480283` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l3_embodiment/nt_shield/adversarial_pipeline.rs` | 1 | 660 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_feel/cognition_bridge/config.rs` | 1 | 74 | `5c02e738` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_feel/cognition_bridge/emotion_state.rs` | 1 | 158 | `477bf669` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/coverage_ledger.rs` | 1 | 522 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/git_memory/mod.rs` | 1 | 110 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/context_budget.rs` | 1 | 76 | `13dfd9a8` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/process_skill_memory.rs` | 1 | 92 | `e20ebe34` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_knowledge_graph/audit_memory/mod.rs` | 1 | 276 | `4ab738e7` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_knowledge_graph/experience_memory/mod.rs` | 1 | 285 | `4ab738e7` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_knowledge_graph/mod.rs` | 1 | 8 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/paged_kv/mod.rs` | 1 | 116 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_pipeline/mod.rs` | 1 | 122 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_consciousness/mod.rs` | 1 | 38 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/archive/crystal_consciousness_state_machine.rs` | 1 | 560 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/archive/crystal_learning_loop.rs` | 1 | 260 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/archive/crystal_unified_pipeline.rs` | 1 | 259 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/consciousness_core.rs` | 1 | 173 | `2a480283` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/crystal_memory_distillation.rs` | 1 | 699 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_evolution.rs` | 1 | 161 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_learning.rs` | 1 | 259 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_memory.rs` | 1 | 221 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_pipeline.rs` | 1 | 125 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_state_machine.rs` | 1 | 492 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core_gwt/cost_ladder/mod.rs` | 1 | 103 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core_gwt/decision_layer/mod.rs` | 1 | 138 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_core_gwt/evidence_gating/mod.rs` | 1 | 128 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/dual_track/mod.rs` | 1 | 113 | `acaf0941` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/jit_harness/mod.rs` | 1 | 116 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/dao_engine.rs` | 1 | 314 | `b291f3d5` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/ethical_intuition.rs` | 1 | 443 | `41f2fc2a` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/federation.rs` | 1 | 700 | `41f2fc2a` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge_miner.rs` | 1 | 585 | `b291f3d5` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/core/execution_trace.rs` | 1 | 127 | `831c3203` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline/tests.rs` | 1 | 175 | `2bbed32c` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/self_evolver.rs` | 1 | 654 | `477bf669` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_crystal.rs` | 1 | 126 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_dual_track.rs` | 1 | 163 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine/domain_modeling/mod.rs` | 1 | 504 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine/flow_router/mod.rs` | 1 | 400 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine/progressive_disclosure/mod.rs` | 1 | 351 | `831c3203` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/seal/domain_mapper.rs` | 1 | 653 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l5_cognition/nt_mind/seal/source_adapter.rs` | 1 | 329 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/l6_meta/nt_meta/dream_replay/mod.rs` | 1 | 186 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/neotrix/nt_crystal_core/agent_orchestrator.rs` | 1 | 192 | `d3b58247` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/neotrix/nt_crystal_core/memory_orchestrator.rs` | 1 | 311 | `e20ebe34` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/neotrix/nt_crystal_core/multi_graph_memory.rs` | 1 | 545 | `NEVER` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/neotrix/nt_crystal_core/observability.rs` | 1 | 152 | `1e97e730` | 待裁决（先确认 `docs/` 无引用再动） |
| `neotrix-core/src/neotrix/nt_crystal_core/self_healing.rs` | 1 | 372 | `477bf669` | 待裁决（先确认 `docs/` 无引用再动） |

> ⛔ `docs/standards/templates/FITNESS-FN-TEMPLATE.rs`（40 行）虽落在 A 类，
> 但它**不是死代码** —— 文件头第 1 行原文 `//! TEMPLATE — do NOT compile standalone.`，
> 它按设计就不该被编译。**归入 A 类仅表示「不参与编译」，绝不等于可删。**

### 5.5 A 类逐文件（折叠）

<details>
<summary>展开：212 个 A 类文件逐条（路径 / 行数 / 证据）</summary>

| 路径 | 行数 | 类别 | 证据 | 建议 |
|---|---:|:--:|---|---|
| `crates/neotrix-types/src/cleanup_types.rs` | 51 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `dfdc4d48` (2026-09-11) 落地 | 可删候选（**本任务不删**） |
| `crates/neotrix-types/src/core/nt_core_bank/compressor.rs` | 204 | A | `git log -S "mod compressor;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `94a4770e` (2026-07-06) 落地 | 可删候选（**本任务不删**） |
| `crates/neotrix-types/src/core/nt_core_knowledge/provider.rs` | 229 | A | `git log -S "mod provider;"` → 最后一次改动该声明的 commit `15b12f96` (2026-09-24) refactor(godfile): entry/mod.rs 3027 行拆 15 模块；文件由 `94a4770e` (2026-07-06) 落地 | 可删候选（**本任务不删**） |
| `crates/neotrix-types/src/core/nt_crypto_util.rs` | 51 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `docs/standards/templates/FITNESS-FN-TEMPLATE.rs` | 40 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `e63e7404` (2026-09-21) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/actions/media/nt_3d_task_spec.rs` | 207 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `e75272be` (2026-09-27) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/auditor.rs` | 275 | A | `git log -S "mod auditor;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/checker_executor.rs` | 302 | A | `git log -S "mod checker_executor;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/config.rs` | 104 | A | `git log -S "mod config;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/executor.rs` | 198 | A | `git log -S "mod executor;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/manager.rs` | 272 | A | `git log -S "mod manager;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/mod.rs` | 45 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/operator_executor.rs` | 333 | A | `git log -S "mod operator_executor;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/planner_executor.rs` | 360 | A | `git log -S "mod planner_executor;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/agent_loop/trait_.rs` | 207 | A | `git log -S "mod trait_;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/geo_seo/geo.rs` | 156 | A | `git log -S "mod geo;"` → 最后一次改动该声明的 commit `13dfd9a8` (2026-09-17) fix: build clean — 0 errors, resolve all phantom module references and type mismatches；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/geo_seo/mod.rs` | 15 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/geo_seo/search_engine.rs` | 185 | A | `git log -S "mod search_engine;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/geo_seo/visibility.rs` | 231 | A | `git log -S "mod visibility;"` → 最后一次改动该声明的 commit `13dfd9a8` (2026-09-17) fix: build clean — 0 errors, resolve all phantom module references and type mismatches；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_dev_tools/build_desktop.rs` | 251 | A | `git log -S "mod build_desktop;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_dev_tools/daemon_monitor.rs` | 191 | A | `git log -S "mod daemon_monitor;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_dev_tools/git_hook.rs` | 122 | A | `git log -S "mod git_hook;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_dev_tools/mod.rs` | 14 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_scheduler.rs` | 747 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_trade/orchestrator_compat.rs` | 32 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_trade/unified_types_compat.rs` | 41 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/nt_act_workspace_isolator.rs` | 719 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/provider_abstraction/models.rs` | 257 | A | `git log -S "mod models;"` → 最后一次改动该声明的 commit `4527f202` (2026-09-09) refactor(nt_act): delete 66 dead flat files, complete Batch 3f-3h safe moves；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/semantic_routing/catalog.rs` | 227 | A | `git log -S "mod catalog;"` → 最后一次改动该声明的 commit `3a3ea364` (2026-09-28) fix(build): 落盘 6 个根因的修复, 使干净检出的 lib+bins+tests 全部编译通过；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/semantic_routing/mod.rs` | 185 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/semantic_routing/pattern.rs` | 143 | A | `git log -S "mod pattern;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_act/semantic_routing/router.rs` | 289 | A | `git log -S "mod router;"` → 最后一次改动该声明的 commit `2bbed32c` (2026-09-28) chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_infra_unified_search.rs` | 485 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_io/nt_io_eli5.rs` | 82 | A | `git log -S "mod nt_io_eli5;"` → 最后一次改动该声明的 commit `1b5d5e6b` (2026-09-09) refactor(nt_core): EVO-77 — 44 flat files → 8 subdirectories；文件由 `d4850dba` (2026-09-08) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l1_action/nt_io/nt_io_protocol_bridge.rs` | 473 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/error_conversions.rs` | 24 | A | `git log -S "mod error_conversions;"` → 最后一次改动该声明的 commit `e75272be` (2026-09-27) fix(build): 补齐 HEAD 依赖却未入库的 29 个文件 + 2 处缺失 mod 声明；文件由 `9ed792a3` (2026-09-23) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/crawl/dom_extractor/mod.rs` | 251 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `d4850dba` (2026-09-08) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/crawl/ordered_backend_router/mod.rs` | 386 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `d4850dba` (2026-09-08) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/sense/tests/integration.rs` | 93 | A | `git log -S "mod integration;"` → 最后一次改动该声明的 commit `13dfd9a8` (2026-09-17) fix: build clean — 0 errors, resolve all phantom module references and type mismatches；文件由 `d4850dba` (2026-09-08) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/entity.rs` | 127 | A | `git log -S "mod entity;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/fact_extractor.rs` | 238 | A | `git log -S "mod fact_extractor;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/graph_traversal.rs` | 235 | A | `git log -S "mod graph_traversal;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/knowledge_graph.rs` | 155 | A | `git log -S "mod knowledge_graph;"` → 最后一次改动该声明的 commit `84e98aff` (2026-09-27) feat(crystal): C批混合落地（生成式注册行+gap-fill代际快照+cli迁移，handoff §17）；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/mod.rs` | 20 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/pagerank.rs` | 200 | A | `git log -S "mod pagerank;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/relation.rs` | 83 | A | `git log -S "mod relation;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/temporal_kg/temporal_query.rs` | 212 | A | `git log -S "mod temporal_query;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l2_perception/nt_world/traits.rs` | 41 | A | `git log -S "mod traits;"` → 最后一次改动该声明的 commit `2a480283` (2026-09-27) chore(dead): 移除 24,884 LOC「已入库但从未编译」的 cli/ 树 + 1 个陈旧副本；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/adversarial_pipeline.rs` | 660 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/circuit_breaker.rs` | 109 | A | `git log -S "mod circuit_breaker;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/containment.rs` | 92 | A | `git log -S "mod containment;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/escape_detector.rs` | 112 | A | `git log -S "mod escape_detector;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/mod.rs` | 27 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_core/asi_compliance.rs` | 177 | A | `git log -S "mod asi_compliance;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_core/mod.rs` | 30 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_core/reasoning_shield.rs` | 189 | A | `git log -S "mod reasoning_shield;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_core/trust_anchor.rs` | 246 | A | `git log -S "mod trust_anchor;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_inner/input_sanitizer.rs` | 196 | A | `git log -S "mod input_sanitizer;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_inner/intent_remapper.rs` | 155 | A | `git log -S "mod intent_remapper;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_inner/mod.rs` | 23 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_inner/retrieval_gate.rs` | 115 | A | `git log -S "mod retrieval_gate;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_inner/trust_classifier.rs` | 121 | A | `git log -S "mod trust_classifier;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/behavior_monitor.rs` | 143 | A | `git log -S "mod behavior_monitor;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/guardrail_sentinel.rs` | 149 | A | `git log -S "mod guardrail_sentinel;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/mod.rs` | 23 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/output_filter.rs` | 124 | A | `git log -S "mod output_filter;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/resonance_detector.rs` | 121 | A | `git log -S "mod resonance_detector;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/input_validator.rs` | 433 | A | `git log -S "mod input_validator;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/mod.rs` | 108 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/output_validator.rs` | 442 | A | `git log -S "mod output_validator;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l3_embodiment/nt_shield/guard/agent_guardrails/policy_engine.rs` | 467 | A | `git log -S "mod policy_engine;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_feel/cognition_bridge/config.rs` | 74 | A | `git log -S "mod config;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_feel/cognition_bridge/emotion_state.rs` | 158 | A | `git log -S "mod emotion_state;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/add_only_writes/memory_entry.rs` | 207 | A | `git log -S "mod memory_entry;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/add_only_writes/mod.rs` | 20 | A | `git log -S "mod add_only_writes;"` → 最后一次改动该声明的 commit `8aab76f6` (2026-09-21) neotrix-core编译修复 + 清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/add_only_writes/temporal_query.rs` | 213 | A | `git log -S "mod temporal_query;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/add_only_writes/temporal_store.rs` | 349 | A | `git log -S "mod temporal_store;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/admission_control/config.rs` | 157 | A | `git log -S "mod config;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/admission_control/gate.rs` | 109 | A | `git log -S "mod gate;"` → 最后一次改动该声明的 commit `acaf0941` (2026-09-27) refactor(l5): 拆掉 4 个抽取 crate 的空壳门面 (54 个 0 字节模块)；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/admission_control/mod.rs` | 9 | A | `git log -S "mod admission_control;"` → 最后一次改动该声明的 commit `8aab76f6` (2026-09-21) neotrix-core编译修复 + 清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/admission_control/policy.rs` | 127 | A | `git log -S "mod policy;"` → 最后一次改动该声明的 commit `acaf0941` (2026-09-27) refactor(l5): 拆掉 4 个抽取 crate 的空壳门面 (54 个 0 字节模块)；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/admission_control/scorer.rs` | 179 | A | `git log -S "mod scorer;"` → 最后一次改动该声明的 commit `2bbed32c` (2026-09-28) chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/coverage_ledger.rs` | 522 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/decay_forgetting/config.rs` | 190 | A | `git log -S "mod config;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/decay_forgetting/curves.rs` | 111 | A | `git log -S "mod curves;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/decay_forgetting/mod.rs` | 16 | A | `git log -S "mod decay_forgetting;"` → 最后一次改动该声明的 commit `8aab76f6` (2026-09-21) neotrix-core编译修复 + 清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/decay_forgetting/pruner.rs` | 158 | A | `git log -S "mod pruner;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/decay_forgetting/salience.rs` | 259 | A | `git log -S "mod salience;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/git_memory/mod.rs` | 110 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/bm25_search.rs` | 181 | A | `git log -S "mod bm25_search;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/entity_search.rs` | 162 | A | `git log -S "mod entity_search;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/fusion_engine.rs` | 240 | A | `git log -S "mod fusion_engine;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/mod.rs` | 114 | A | `git log -S "mod hybrid_retrieval;"` → 最后一次改动该声明的 commit `8aab76f6` (2026-09-21) neotrix-core编译修复 + 清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/semantic_search.rs` | 193 | A | `git log -S "mod semantic_search;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/hybrid_retrieval/temporal_scoring.rs` | 138 | A | `git log -S "mod temporal_scoring;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_cleanup/history_log.rs` | 88 | A | `git log -S "mod history_log;"` → 最后一次改动该声明的 commit `d4850dba` (2026-09-08) fix: compilation errors - Severity conflicts, derive attributes, unused imports, module registration；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_cleanup/mod.rs` | 11 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_cleanup/rule_store.rs` | 66 | A | `git log -S "mod rule_store;"` → 最后一次改动该声明的 commit `d4850dba` (2026-09-08) fix: compilation errors - Severity conflicts, derive attributes, unused imports, module registration；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/context_budget.rs` | 76 | A | `git log -S "mod context_budget;"` → 最后一次改动该声明的 commit `13dfd9a8` (2026-09-17) fix: build clean — 0 errors, resolve all phantom module references and type mismatches；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/process_skill_memory.rs` | 92 | A | `git log -S "mod process_skill_memory;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_knowledge_graph/audit_memory/mod.rs` | 276 | A | `git log -S "mod audit_memory;"` → 最后一次改动该声明的 commit `4ab738e7` (2026-08-25) feat(desktop): 吸收 DSH/Minke 特性 + Terminal 面板 + 架构审计修复；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_knowledge_graph/experience_memory/mod.rs` | 285 | A | `git log -S "mod experience_memory;"` → 最后一次改动该声明的 commit `4ab738e7` (2026-08-25) feat(desktop): 吸收 DSH/Minke 特性 + Terminal 面板 + 架构审计修复；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/nt_memory_knowledge_graph/mod.rs` | 8 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/paged_kv/mod.rs` | 116 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_memory/mod.rs` | 75 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_memory/router.rs` | 545 | A | `git log -S "mod router;"` → 最后一次改动该声明的 commit `2bbed32c` (2026-09-28) chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_memory/tier_archival.rs` | 496 | A | `git log -S "mod tier_archival;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_memory/tier_core.rs` | 336 | A | `git log -S "mod tier_core;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_memory/tier_recall.rs` | 421 | A | `git log -S "mod tier_recall;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_memory/traits.rs` | 200 | A | `git log -S "mod traits;"` → 最后一次改动该声明的 commit `2a480283` (2026-09-27) chore(dead): 移除 24,884 LOC「已入库但从未编译」的 cli/ 树 + 1 个陈旧副本；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l4_emotion/nt_memory/tiered_pipeline/mod.rs` | 122 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_consciousness/mod.rs` | 38 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/io_skills/mod.rs` | 3 | A | `git log -S "mod io_skills;"` → 最后一次改动该声明的 commit `32efc110` (2026-09-10) feat: 深度审计+外部研究吸收+P0/P1修复 (cycle 212)；文件由 `1b5d5e6b` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/io_skills/nt_io_eli5.rs` | 82 | A | `git log -S "mod nt_io_eli5;"` → 最后一次改动该声明的 commit `1b5d5e6b` (2026-09-09) refactor(nt_core): EVO-77 — 44 flat files → 8 subdirectories；文件由 `1b5d5e6b` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/mod.rs` | 4 | A | `git log -S "mod knowledge;"` → 最后一次改动该声明的 commit `74ad4869` (2026-09-17) feat(crystal): CrystalState 统一状态空间 — 最小可行实现；文件由 `1b5d5e6b` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/nt_core_kg_traversal.rs` | 312 | A | `git log -S "mod nt_core_kg_traversal;"` → 最后一次改动该声明的 commit `1b5d5e6b` (2026-09-09) refactor(nt_core): EVO-77 — 44 flat files → 8 subdirectories；文件由 `1b5d5e6b` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/nt_core_knowledge_mgmt.rs` | 261 | A | `git log -S "mod nt_core_knowledge_mgmt;"` → 最后一次改动该声明的 commit `1b5d5e6b` (2026-09-09) refactor(nt_core): EVO-77 — 44 flat files → 8 subdirectories；文件由 `1b5d5e6b` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/nt_core_knowledge_repr.rs` | 309 | A | `git log -S "mod nt_core_knowledge_repr;"` → 最后一次改动该声明的 commit `1b5d5e6b` (2026-09-09) refactor(nt_core): EVO-77 — 44 flat files → 8 subdirectories；文件由 `1b5d5e6b` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/nt_core_rag.rs` | 362 | A | `git log -S "mod nt_core_rag;"` → 最后一次改动该声明的 commit `1b5d5e6b` (2026-09-09) refactor(nt_core): EVO-77 — 44 flat files → 8 subdirectories；文件由 `1b5d5e6b` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/memory/consolidation.rs` | 381 | A | `git log -S "mod consolidation;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `9d2b84c8` (2026-09-13) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/memory/emotional.rs` | 295 | A | `git log -S "mod emotional;"` → 最后一次改动该声明的 commit `9d2b84c8` (2026-09-13) feat: cognitive evolution system - memory, emotion, metacognition；文件由 `9d2b84c8` (2026-09-13) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/memory/episodic.rs` | 262 | A | `git log -S "mod episodic;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `9d2b84c8` (2026-09-13) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/memory/mod.rs` | 9 | A | `git log -S "mod memory;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `9d2b84c8` (2026-09-13) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/memory/semantic.rs` | 219 | A | `git log -S "mod semantic;"` → 最后一次改动该声明的 commit `9d2b84c8` (2026-09-13) feat: cognitive evolution system - memory, emotion, metacognition；文件由 `9d2b84c8` (2026-09-13) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/aggregation.rs` | 410 | A | `git log -S "mod aggregation;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/coordinator/coordinator.rs` | 400 | A | `git log -S "mod coordinator;"` → 最后一次改动该声明的 commit `848539fe` (2026-09-20) 架构优化: double-envelope修复 + 事件命名统一 + 死模块移除 + 路径集中管理 + .expect()清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/coordinator/load_balancer.rs` | 332 | A | `git log -S "mod load_balancer;"` → 最后一次改动该声明的 commit `f52238d2` (2026-09-28) refactor: 删除 GWT MoELoadBalancer（207 行）—— 被同目录继任者取代；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/coordinator/mod.rs` | 22 | A | `git log -S "mod coordinator;"` → 最后一次改动该声明的 commit `848539fe` (2026-09-20) 架构优化: double-envelope修复 + 事件命名统一 + 死模块移除 + 路径集中管理 + .expect()清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/coordinator/monitor.rs` | 295 | A | `git log -S "mod monitor;"` → 最后一次改动该声明的 commit `2bbed32c` (2026-09-28) chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/coordinator/task_routing.rs` | 392 | A | `git log -S "mod task_routing;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/crew.rs` | 436 | A | `git log -S "mod crew;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/delegation.rs` | 389 | A | `git log -S "mod delegation;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/graph_orch/dag.rs` | 478 | A | `git log -S "mod dag;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/graph_orch/mod.rs` | 22 | A | `git log -S "mod graph_orch;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/graph_orch/monitor.rs` | 416 | A | `git log -S "mod monitor;"` → 最后一次改动该声明的 commit `2bbed32c` (2026-09-28) chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/graph_orch/optimizer.rs` | 486 | A | `git log -S "mod optimizer;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/graph_orch/scheduler.rs` | 333 | A | `git log -S "mod scheduler;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/mod.rs` | 35 | A | `git log -S "mod multi_agent;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/multi_agent/role.rs` | 209 | A | `git log -S "mod role;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/archive/crystal_consciousness_state_machine.rs` | 560 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/archive/crystal_learning_loop.rs` | 260 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/archive/crystal_unified_pipeline.rs` | 259 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/consciousness_core.rs` | 173 | A | `git log -S "mod consciousness_core;"` → 最后一次改动该声明的 commit `2a480283` (2026-09-27) chore(dead): 移除 24,884 LOC「已入库但从未编译」的 cli/ 树 + 1 个陈旧副本；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/crystal_memory_distillation.rs` | 699 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_evolution.rs` | 161 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_learning.rs` | 259 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_memory.rs` | 221 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_pipeline.rs` | 125 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/unified_state_machine.rs` | 492 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core_gwt/cost_ladder/mod.rs` | 103 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core_gwt/decision_layer/mod.rs` | 138 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_core_gwt/evidence_gating/mod.rs` | 128 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/cross_domain/cross_domain_transfer.rs` | 518 | A | `git log -S "mod cross_domain_transfer;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/cross_domain/disagreement_gate.rs` | 604 | A | `git log -S "mod disagreement_gate;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/cross_domain/entity_mapping.rs` | 819 | A | `git log -S "mod entity_mapping;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/cross_domain/mod.rs` | 42 | A | `git log -S "mod cross_domain;"` → 最后一次改动该声明的 commit `831c3203` (2026-06-26) Phase 42: Project restructuring + strict wiring + stash integration；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/cross_domain/reason_retrieve_refine.rs` | 661 | A | `git log -S "mod reason_retrieve_refine;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/dual_track/mod.rs` | 113 | A | `git log -S "mod dual_track;"` → 最后一次改动该声明的 commit `acaf0941` (2026-09-27) refactor(l5): 拆掉 4 个抽取 crate 的空壳门面 (54 个 0 字节模块)；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/jit_harness/mod.rs` | 116 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/dao_engine.rs` | 314 | A | `git log -S "mod dao_engine;"` → 最后一次改动该声明的 commit `b291f3d5` (2026-08-28) Delete neotrix-core directory；文件由 `12035725` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/ethical_intuition.rs` | 443 | A | `git log -S "mod ethical_intuition;"` → 最后一次改动该声明的 commit `41f2fc2a` (2026-08-26) chore: concurrent session changes (nt_act wave3 + skill engine + shield)；文件由 `12035725` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/federation.rs` | 700 | A | `git log -S "mod federation;"` → 最后一次改动该声明的 commit `41f2fc2a` (2026-08-26) chore: concurrent session changes (nt_act wave3 + skill engine + shield)；文件由 `12035725` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/knowledge_miner.rs` | 585 | A | `git log -S "mod knowledge_miner;"` → 最后一次改动该声明的 commit `b291f3d5` (2026-08-28) Delete neotrix-core directory；文件由 `12035725` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/core/execution_trace.rs` | 127 | A | `git log -S "mod execution_trace;"` → 最后一次改动该声明的 commit `831c3203` (2026-06-26) Phase 42: Project restructuring + strict wiring + stash integration；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline/tests.rs` | 175 | A | `git log -S "mod tests;"` → 最后一次改动该声明的 commit `2bbed32c` (2026-09-28) chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库；文件由 `50799ccc` (2026-09-01) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/self_evolver.rs` | 654 | A | `git log -S "mod self_evolver;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `12035725` (2026-09-09) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_crystal.rs` | 126 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `e75272be` (2026-09-27) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_dual_track.rs` | 163 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine/domain_modeling/mod.rs` | 504 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `50799ccc` (2026-09-01) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine/flow_router/mod.rs` | 400 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `50799ccc` (2026-09-01) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_skill_engine/progressive_disclosure/mod.rs` | 351 | A | `git log -S "mod progressive_disclosure;"` → 最后一次改动该声明的 commit `831c3203` (2026-06-26) Phase 42: Project restructuring + strict wiring + stash integration；文件由 `50799ccc` (2026-09-01) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/seal/domain_mapper.rs` | 653 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/seal/source_adapter.rs` | 329 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/self_improvement/memory_filesystem.rs` | 342 | A | `git log -S "mod memory_filesystem;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/self_improvement/mod.rs` | 43 | A | `git log -S "mod self_improvement;"` → 最后一次改动该声明的 commit `acaf0941` (2026-09-27) refactor(l5): 拆掉 4 个抽取 crate 的空壳门面 (54 个 0 字节模块)；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/self_improvement/self_edit_loop.rs` | 439 | A | `git log -S "mod self_edit_loop;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l5_cognition/nt_mind/self_improvement/sleep_compute.rs` | 1301 | A | `git log -S "mod sleep_compute;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/coordination/nt_meta_cleanup/concurrency_detector.rs` | 86 | A | `git log -S "mod concurrency_detector;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/coordination/nt_meta_cleanup/coordinator.rs` | 167 | A | `git log -S "mod coordinator;"` → 最后一次改动该声明的 commit `848539fe` (2026-09-20) 架构优化: double-envelope修复 + 事件命名统一 + 死模块移除 + 路径集中管理 + .expect()清理；文件由 `d4850dba` (2026-09-08) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/coordination/nt_meta_cleanup/mod.rs` | 13 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `d4850dba` (2026-09-08) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/coordination/nt_meta_cleanup/runtime_monitor.rs` | 77 | A | `git log -S "mod runtime_monitor;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `e20ebe34` (2026-09-16) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/lib.rs` | 68 | A | `git log -S "mod l6_meta;"` → 最后一次改动该声明的 commit `50799ccc` (2026-09-01) chore: unify architecture migration checkpoint；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_core_qtest.rs` | 538 | A | `git log -S "mod nt_core_qtest;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `d82ad31b` (2026-09-18) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/dream_replay/mod.rs` | 186 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/eval_engine/dataset_manager.rs` | 209 | A | `git log -S "mod dataset_manager;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/eval_engine/experiment_tracker.rs` | 244 | A | `git log -S "mod experiment_tracker;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/eval_engine/llm_judge.rs` | 188 | A | `git log -S "mod llm_judge;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/eval_engine/mod.rs` | 9 | A | `git log -S "mod eval_engine;"` → 最后一次改动该声明的 commit `2ad3cf7f` (2026-09-29) feat(evolution): D-3 裁决 — 融合评测能力为单一自进化验证底座；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/cost_tracker.rs` | 316 | A | `git log -S "mod cost_tracker;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/health_monitor.rs` | 454 | A | `git log -S "mod health_monitor;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/mod.rs` | 17 | A | `git log -S "mod otel_bridge;"` → 最后一次改动该声明的 commit `8aab76f6` (2026-09-21) neotrix-core编译修复 + 清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/prompt_manager/mod.rs` | 9 | A | `git log -S "mod prompt_manager;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/prompt_manager/prompt_eval.rs` | 206 | A | `git log -S "mod prompt_eval;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/prompt_manager/prompt_registry.rs` | 226 | A | `git log -S "mod prompt_registry;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/prompt_manager/prompt_version.rs` | 191 | A | `git log -S "mod prompt_version;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/otel_bridge/trace_pipeline.rs` | 362 | A | `git log -S "mod trace_pipeline;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/budget_manager.rs` | 303 | A | `git log -S "mod budget_manager;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/context_router/analyzer.rs` | 247 | A | `git log -S "mod analyzer;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/context_router/context.rs` | 167 | A | `git log -S "mod context;"` → 最后一次改动该声明的 commit `5c02e738` (2026-09-28) refactor(app)!: 归档 src-tauri (NeoTrix 桌面端), neobot 成为唯一桌面 app；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/context_router/mod.rs` | 11 | A | `git log -S "mod context_router;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/context_router/replay.rs` | 198 | A | `git log -S "mod replay;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/context_router/router.rs` | 290 | A | `git log -S "mod router;"` → 最后一次改动该声明的 commit `2bbed32c` (2026-09-28) chore: 文档回填 + 删 nt_core_capability 死引擎 4,640 行 + Cargo.lock 入库；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/cost_dashboard.rs` | 273 | A | `git log -S "mod cost_dashboard;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/event_log.rs` | 278 | A | `git log -S "mod event_log;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/mod.rs` | 14 | A | `git log -S "mod session_replay;"` → 最后一次改动该声明的 commit `8aab76f6` (2026-09-21) neotrix-core编译修复 + 清理；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_meta/session_replay/replay.rs` | 298 | A | `git log -S "mod replay;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/l6_meta/nt_nexus/checkpoint.rs` | 686 | A | `git log -S "mod checkpoint;"` → 最后一次改动该声明的 commit `b291f3d5` (2026-08-28) Delete neotrix-core directory；文件由 `13dfd9a8` (2026-09-17) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/neotrix/nt_crystal_core/agent_orchestrator.rs` | 192 | A | `git log -S "mod agent_orchestrator;"` → 最后一次改动该声明的 commit `d3b58247` (2026-08-05) chore(cleanup): remove archive_star dead-code zone (152 files, -34.9k lines)；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/neotrix/nt_crystal_core/memory_orchestrator.rs` | 311 | A | `git log -S "mod memory_orchestrator;"` → 最后一次改动该声明的 commit `e20ebe34` (2026-09-16) Fusion v2 baseline: dual architecture state before merge；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/neotrix/nt_crystal_core/multi_graph_memory.rs` | 545 | A | `git log -S` 全历史**从未**有过该 `mod` 声明 ⇒ 生下来未接线；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/neotrix/nt_crystal_core/observability.rs` | 152 | A | `git log -S "mod observability;"` → 最后一次改动该声明的 commit `1e97e730` (2026-09-12) docs: targeted research cycle 475 — 3 more internal pain points；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |
| `neotrix-core/src/neotrix/nt_crystal_core/self_healing.rs` | 372 | A | `git log -S "mod self_healing;"` → 最后一次改动该声明的 commit `477bf669` (2026-09-20) docs: establish documentation standard and cleanup；文件由 `477bf669` (2026-09-20) 落地 | 可删候选（**本任务不删**） |

</details>

---

## 6. 方法、自检与口径评估

### 6.1 怎么判的（不靠 grep 猜）

1. 自建 Rust 2018 模块树解析器：从 10 个 workspace crate 的 `lib.rs` / `main.rs` /
   `[[bin]]` / `[[example]]` / `[[test]]` / `[[bench]]` / `build.rs` 出发，
   按 2018 规则解析 `mod X;`、`#[path = "..."] mod X;`、`#[cfg(..)] mod X;`、
   `include!()`，记录每个文件的可达路径 + 沿途 cfg 上下文。
2. 与 rustc dep-info 交叉验证：**「在 dep-info 里但我的树走不到」= 0 个**。
3. 只有树走不到、且不是 Cargo target 的才进 A 类候选，再用 `git log -S` 补第二条判据。

### 6.2 ⛔ 工具自身踩过的 4 个坑（都已修，且每一个都会产出**假死代码**判决）

按 AGENTS.md §5 R-SCAN-1 / R-SCAN-2，这是「扫描器告警先证伪」的实例，记录在案：

| 坑 | 现象 | 若不修会怎样 |
|---|---|---|
| raw string 终止符差一 | `r#"…"#` 的终止符算成 `"##` | 整个文件被吞成空 ⇒ 该文件所有子模块被误判死 |
| 抹掉字符串字面量 | `#[path = "x.rs"]` 的**值**被删 | **B 类彻底隐形**，`#[path]` 全被当成普通 `mod` |
| `#[path]` 基准目录 | 用了「模块的子目录」而非「文件所在目录」 | `src/bin/experience.rs` 的 8 个 `#[path]` 子件全被判死 |
| 属性与 `mod` 之间可夹 `pub` | `#[path=…]` 与 `pub mod` 之间的 `pub ` 未被正则容纳 | `l1_facade_meta.rs` / `l1_facade_observer.rs` 被判死 |

（另有一个性能坑：`re.match(pat, src[i:])` 的切片让任何含撇号文件退化成 O(n²)，
1,345 行的 `run.rs` 就能跑满 2 分钟；改成 `re.compile(pat).match(src, i)` 后全仓 2.4 秒。）

### 6.3 残余不确定性

- **1 个文件解析不可信**：`neotrix-core/src/neotrix/nt_file_ability/test_helpers.rs`
  （词法器报 unterminated）。它**在 dep-info 里、且不在幽灵清单内** ⇒ 对本次结论无影响。
- A 类判据依赖 `mod` 声明的**文本**。若存在 `include!()` 拼接模块，本工具会漏；
  已全仓验证：**`include!()` 出现 0 次**。
- A 类**不等于可删**。98 棵孤儿子树里可能含「故意归档」的文件
  （例：`.../nt_consciousness_core/archive/` 下 3 个带 `archive` 字样的文件）。
  **删前必须逐棵确认 `docs/` 与 `sessions/` 无引用。**

### 6.4 对 `/tmp/ghost.txt` 口径的评估：**口径本身可精确复现，但有三处系统性偏差**

**先说复现结论**：你的口径 = `git ls-files "*.rs"` 减去 `target/debug/deps/**/*.d`
里命中的工作区路径。我独立跑了一遍：
```
$ git ls-files "*.rs" | wc -l
2859                       <- 磁盘全集（tracked）
$ <deps/**/*.d 里的工作区 .rs 去重>
2580                       <- "参与编译"
2859 - 2580 = 279          <- 与 /tmp/ghost.txt 完全一致，零残留
```
⇒ 清单**内部自洽且完整**（相对这个口径）。你正文里的「2,858 / 2,579」是转写误差，
真实数字是 **2,859 / 2,580**，差值 279 无误。

下面是三处偏差：

**偏差 ①（假阳性，1 个文件 / 10 行）—— 清单内唯一的错误**

`neotrix-core/build.rs` 被列为幽灵，但 **rustc 确实编了它**：
```
$ find target -name "build_script_build-*.d" -path "*neotrix-*" | xargs grep -l "neotrix-core/build.rs"
target/debug/build/neotrix-a53e03f6c9673e6f/build_script_build-a53e03f6c9673e6f.d
target/debug/build/neotrix-1983451fee15dedb/build_script_build-1983451fee15dedb.d

$ cat target/debug/build/neotrix-a53e03f6c9673e6f/build_script_build-a53e03f6c9673e6f.d
...: neotrix-core/build.rs
```
原因：build script 的 dep-info 落在 `target/debug/build/`，而口径只扫了
`target/debug/deps/`。**⇒ 真实的「未参与编译」数是 278，不是 279。**

**偏差 ②（假阴性，34 个文件）—— 清单不完整，但不影响死代码结论**

`target/debug/deps/` 里 **1,644 / 1,678** 个 `.d` 早于本次 check（本次只重写了 34 个，
因为命中缓存）。这些陈旧 `.d` 混入了**历史 `cargo test` / `cargo bench` 的产物**：

- **26** 个 `neotrix-core/tests/*.rs`（集成测试 target）
- **8** 个 bench：`neotrix-core/benches/{mail_benchmark,neotrix_benchmarks,act_c3,memory_bench,repair_c3,security_bench,shield_c3}.rs` + `crates/neotrix-types/benches/pdf_c3.rs`

它们被当成「参与编译」而**从幽灵清单里漏掉了**。这 34 个都是货真价实的 C 类
（`cargo test` / `cargo bench` 会编），**所以不改变 A/B/C/D 的死代码结论**，
但清单作为「未参与本次编译的全集」是**不完整**的。

**偏差 ③（假阴性，3 个文件）—— untracked 文件不在 `git ls-files` 里**

另有 **3 个文件**同样未编译却不在清单内：`apps/neobot-desktop/{build.rs,src/lib.rs,src/main.rs}`。
两个原因叠加：

1. 口径的磁盘全集用 `git ls-files`，**不含 untracked 文件**（该目录目前是 `??` 状态）；
2. `apps/neobot-desktop` **不在 workspace `members` 里**（根 `Cargo.toml` 也没有
   `exclude` 表），所以 `cargo check --workspace` 永远不会碰它 —— 它只通过
   `neotrix-neobot = { path = "../../crates/neotrix-neobot" }` 单向依赖工作区。

⇒ 这 3 个 `.rs` 是货真价实的「未参与编译」文件，但既不在清单里、也不受任何
工作区级门禁保护。

**建议的正确口径**（若要重做）：

1. 扫 `target/debug/{deps,build,examples}/**/*.d` 三处，**再按 mtime 过滤**到本次
   check 之后 —— 只扫 `deps/` 会漏 build script，不按 mtime 过滤会吃进陈旧的
   test/bench 产物。
2. 更稳的做法：**别用 dep-info 做「未编译」判定**。dep-info 是「曾经编过」的累积
   目录，`target/` 跨窗口共享（AGENTS.md §2 禁止多窗口全量构建正是为此）。
   改用**模块树解析 + Cargo target 表**：输入只有工作树，无缓存污染，
   且能直接区分「被 `#[cfg]` 门控」与「真没接线」——这正是本次 279 个文件
   里最需要的信息，而 dep-info 原生给不出。


### 7.0 最大指控的独立复核（我亲自跑，非子代理结论）

「`477bf669` 自称文档清理却删了 70 万行」——复跑确认：

```
$ git show --shortstat --format='' 477bf669
 4535 files changed, 174964 insertions(+), 703249 deletions(-)
$ git show --numstat --format='' 477bf669 | awk '$3 ~ /\.rs$/ …'
 .rs 新增行 123226   .rs 删除行 99886
```

标题是 `docs: establish documentation standard and cleanup`
⇒ **一个「文档标准化」commit 实际重写了 20 万行 Rust**，
并使 212 个文件（50,454 行）脱离模块树成为孤儿。

⚠️ 本审计发现的**最大单点风险**：不是「有死代码」，
而是「**一个看起来无害的文档 commit 移除了 20 万行代码而无人察觉**」。
本仓纪律 R-P41（门记录声称已做而实现从未入库）管的是另一半 ——
这条是「**commit 消息与内容不符**」，需要独立纪律。

---

## 7. 附带发现：`cargo check --all-targets` 编译不过（与本审计同源）

### 7.1 `neotrix-core/benches/` 引用不存在的模块

```
error[E0432]: unresolved import `neotrix::l5_cognition::nt_mind::nt_mind::evolution::experiment`
error: could not compile `neotrix` (bench "neotrix_benchmarks")
  --> neotrix-core/benches/neotrix_benchmarks.rs:20
```

`nt_mind/nt_mind/evolution/` 下**没有 `experiment` 模块**（实测该目录 26 个 `.rs` 无它）。

⛔ **⇒ `cargo bench` 编译不过。** 而本审计的 279 个「未参与编译」文件里，
8 个 bench 之所以没被 `cargo check --workspace` 编到，正是因为
**带 `--all-targets` 才会去编它们** —— 一编就炸。

### 7.2 这是 `9bbc9dc2` 的漏网

`9bbc9dc2`（2026-09-28，`fix(build): 补齐 HEAD 缺失文件 + … 恢复干净检出的可构建性`）
恢复了 **lib** 的可构建性，**漏了 bench**。

⇒ **「恢复可构建性」这类提交必须用 `--all-targets` 验，否则只验了 lib 一条腿。**
与 `R-DISK-8`「测逻辑 ≠ 测可达性」同族：那次验的是「lib 能编」，
没验「**cargo bench 能编**」。

### 7.3 待办

- [ ] `neotrix_benchmarks.rs:20` 的 import 要么改到真实模块，要么删掉该 bench
- [ ] `check-fresh-build.sh` 应加 `--all-targets` 档（当前只有 metadata / lib 两档）
