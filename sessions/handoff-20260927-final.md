# 交接 · 卡死/内存专项 + 长尾修复（2026-09-27 收口）

> 交接对象：下一个对话 / 下一个 agent 窗口。
> 本对话**已完成**：8 条卡死/内存根因除根、73 处生产缺陷修复、125 条失败全量分诊、
> 29 项已改未验修复、17 条硬规则沉淀、三份决策/清单文档。
> 本对话**未完成**：最后 29 项因内存门阻塞**未提交**（见 §1，这是接手第一件事）。

---

## 1. 🔴 最高优先：29 项已改未验，尚未提交

**状态**：`git status` 有 430 个文件 `M`（大部分是他窗改动），其中**本线的 29 项修复
尚未 commit**。原因是提交门禁要跑 `cargo`，而当时内存门 BLOCKED（free < 1.6G）。

**接手第一步（严格按序）**：

```bash
sh scripts/ops/nt_mem_gate.sh; echo $?        # 必须 0；非 0 就等，别硬上
CARGO_BUILD_JOBS=1 cargo test -p neotrix --lib -- <模块前缀> --test-threads=1
git commit -- <具体路径>                      # pathspec 限定，禁 git add -A
```

**验证进度**（已跑，无需重跑）：
- 第一次覆盖 20+ 模块：**802 passed / 2 failed**（29 项里 28 项绿）
- 补修 `speculative_decoding` 类型歧义后：**65 passed / 1 failed**

**唯一残留失败**：`l3_embodiment::nt_shield::nt_shield_ztnet::crypto::noise_handshake::tests::full_handshake`
- 位置 `noise_handshake.rs:420` — `_initiator._consume_message2(&msg2).unwrap()`
- 进展：已从"构造第一行就 panic"推进到 msg2 消费失败（协议名长度 bug 已修，见 §3）
- 这是真 crypto 缺口（非测试问题），接手方需读 `_consume_message2` 的解密/状态推进

**本线改动的文件清单**（提交时照此 pathspec）：
```
neotrix-core/src/l6_meta/nt_core_capability/{dependency,integrator,monitor}.rs
neotrix-core/src/l6_meta/nt_meta/gwt_router/{cost_weight,attention}.rs
neotrix-core/src/l6_meta/nt_core_self/dynamic_params.rs
neotrix-core/src/l6_meta/nt_agent_identity.rs
neotrix-core/src/l6_meta/nt_core_aware/mod.rs
neotrix-core/src/l6_meta/nt_core_guardian/repair.rs
neotrix-core/src/l4_emotion/nt_feel/{cognitive_bridge/feedback,writing_style,salesperson_profiling}.rs
neotrix-core/src/l4_emotion/nt_memory/{cascade/cascade,consolidation/cache,distillation/distiller}.rs
neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/{memory_orchestrator,nt_memory_distill}.rs
neotrix-core/src/l3_embodiment/nt_shield/nt_shield_sandbox/stateful_bench.rs
neotrix-core/src/l3_embodiment/nt_shield/nt_shield_ztnet/crypto/noise_handshake.rs
neotrix-core/src/l3_embodiment/nt_shield/compliance/requirement.rs
neotrix-core/src/l3_embodiment/nt_shield/nt_shield_audit/threat_modeler.rs
neotrix-core/src/l3_embodiment/nt_shield/shield_core/{audit,safety_kernel}.rs
neotrix-core/src/l0_substrate/nt_core_speculative_decoding.rs
neotrix-core/src/l2_perception/nt_core_vector_store/store_hnsw.rs
neotrix-core/src/l5_cognition/nt_codegen.rs
crates/neotrix-types/src/core/shared_types.rs
Cargo.toml + neotrix-core/Cargo.toml        # 新增 serde_yaml
```
⚠️ `Cargo.lock` 会在首次构建时自动更新（`serde_yaml 0.9.34` 已在 lock 中，离线可解）。
`--locked` 构建会失败，CI 注意。

---

## 2. 📋 剩余 51 条待修（有完整根因）

全量基线：**11493 绿 / 51 红**（`cargo test -p neotrix --lib --no-fail-fast -- --test-threads=1`）。
逐条根因 + file:line + 最小修法：**`sessions/handoff-disease-list-20260927.md` §9**
（分布：P 生产 bug 17 / S 契约漂移 27 / U 未接线 stub 4 / E 环境依赖 3）。

**建议的下一批顺序**（都是低风险高回报）：
1. P0 一行级：`gwt_router::attention` water-fill、`dynamic_params` 逗号解析、
   `check_ip` CIDR（已改待验）、`cost_weight` 预算兜底
2. P1：`store_hnsw` 指标一致性、`nt_codegen` serde_yaml（已改待验）
3. S 类夹具：`nt_core_aware` 4 条、熔断器 3 条（**单位漂移**：new(1,1) 是 1 **秒**
   而测试 sleep 2 **毫秒**）、`predictive_maintenance` 2 条（测试自己的算术错）
4. U 类：`noise_handshake` msg3 接线（已改待验）、`TextEmbedder` 真实现
   （~30 行可做，但有 5 个生产调用方，改了会重排全部检索结果，**需评估**）

---

## 3. ✅ 本对话已完成并提交（HEAD 链）

| 提交 | 内容 |
|---|---|
| `5639c08f` | 11 项领域缺陷 + sidecar 改按需 + `nt_mem_gate.sh` |
| `a82f3084` | `chunk_planner` 无限循环（内存爆炸真凶）|
| `bb020d71` | `DeferredLoader` 自死锁 |
| `1e74a89c` | KB 搜索自死锁 + BM25 标题回表 |
| `ee0729cc` | 自愈测试内嵌 cargo 死锁 + 注入桩 |
| `9113e21c` | RISE 预演自死锁 + `nt_lock_audit.py` 死锁扫描器 |
| `0f829ca8` | 架构守卫二次方空转 |
| `1cca0af7` | 迷雾四分支 SelfTest 补接（fog 曾卡 0.15）|
| `4beba4b3` / `4ca59fce` | 提示路由补真洞 / 训练环死 stub |
| `98ca28f2` | 11 处解析与数据缺陷 |
| `f4a4eecd` | 3 代理并行 25 处逻辑缺陷（154 绿）|
| `a3a8292d` | 4 代理并行 4 stub + 6 环境依赖 + 12 契约（209 绿）|
| `51355bca` | 17 条硬规则（`RUST-STANDARDS.md` §17）|

**全量推进轨迹**：`564 绿` → `1452` → `8917` → `10113 绿/125 红` → `11493 绿/51 红`。
8 条卡死逐条清除，每修一条解锁下一段。

---

## 4. 🧭 需人工决策（技术决策书已备好）

`sessions/handoff-decision-20260927.md` —— 三项都不是一行能改的，各含事实/代价/推荐：
1. **双时态节点 PK** → 推荐**删 `nodes_as_of`/`node_history`**（零生产调用方，
   为它重建 5 张表不值）；附带发现 `SCHEMA_VERSION=10` 与代码里 v11 迁移漂移
2. **CAD 接线证据表** → 推荐**砍到真实 4 个**。关键不是测试红，而是
   `cad_wiring_map()` 把**不存在的 file:line 当证据**喂给 D16 晋升门 —— 假通过更危险
3. **publish_gateway** → 推荐加一等公民 `dry_run`（现在测试断言的是硬编码 `false`，
   等于什么都没测）

---

## 5. 📌 环境事实（接手方必读）

- **本机 16G，三个并发 agent 窗口 ≈ 4.4G**，加 9B `llama-server` 670M（供 NeoBot，**不可停**）。
  `cargo` 会被挤爆。**起重型构建前先查 `pgrep -c rustc`**，并看 `ps -o %cpu,time`
  判断其它窗口是否**真在干活**（本轮实测两个"已停"的窗口实际各占 0.4 核）。
- **sidecar 已改按需**：`sh scripts/ops/nt_sidecar.sh {start|stop|status}`，
  当前 DOWN（这是对的，不是故障）。
- **死锁扫描器**：`python3 scripts/ops/nt_lock_audit.py neotrix-core/src` → 当前 0 命中；
  `--selftest` 可验规则本身。改动锁相关代码后必跑。
- 提交门禁会在 test 构建失败时拒绝提交并给出 `/tmp/precommit-cargo-check.log`。
  **禁止 `--no-verify`**（本轮被拒 3 次全是他窗并发重构弄红，用
  `cargo check --p neotrix --lib` 排除 test cfg 验证自己的生产改动，等对方提交后再跑测试）。

---

## 6. 经验沉淀位置

- 规则（含强制工具）：`RUST-STANDARDS.md` §17，共 **35 条**，分锁/构建/卡死判别/Git/
  修 bug 判据/字节安全/编译期七族
- 三道闸（阻塞项）：`AGENTS.md` → `nt_mem_gate.sh` / `nt_lock_audit.py` / `nt_sidecar.sh`
- 事故与分诊全表：`sessions/handoff-disease-list-20260927.md`（§1 8 条根因、§9 51 条待修）
