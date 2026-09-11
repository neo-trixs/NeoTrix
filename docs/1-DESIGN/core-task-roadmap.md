# NeoTrix Core Task Roadmap

> 基于: 101批次破限制技术研究 + 315批次模型逆向工程 + 跨域643处违规 + 10000+迭代循环积累
> 生成时间: 2026-09-11
> 基线: 351 modules, C3.2 avg maturity, 787 unwrap, 58 unsafe, 13 large files, 10 untested modules

---

## 基线度量

| 指标 | 当前值 | 目标值 (P3) |
|------|--------|------------|
| Architecture Health Score | 42/100 | 85/100 |
| unwrap() 总数 | 787 | <50 |
| unsafe 代码块 | 58 | 0 (R-P1) |
| C0 模块 (仅编译) | 31 (8.4%) | 0 |
| C2+ 模块 (集成测试+) | 3 (0.9%) | 100+ (28%) |
| C3 缺失 (基准测试) | 0 (0%) | 30+ (8.5%) |
| >800行巨文件 | 13 | 0 |
| 零测试模块 (>300行) | 10+ | 0 |
| 编译错误 | 14 (gateway/nt_io_web) | 0 |

---

## P0 — 立即执行 (本周)

### T0-01: 统一错误域 — 消灭 unwrap 债务

**描述**: 787 处 unwrap() 是代码库 #1 技术债，每处 unwrap 在生产中都是潜在崩溃点。分 3 批将 unwrap 替换为 `Result` + `?` 或 `.unwrap_or_default()` / `.ok()`。

**涉及文件**:
- `neotrix-core/src/l1_action/nt_memory/nt_memory_lead.rs` (12 处 unwrap)
- `neotrix-core/src/neotrix/nt_capability_bridge.rs` (6 处 unwrap)
- `neotrix-core/src/l1_action/nt_infra_agent_card.rs` (8 处 unwrap)
- `neotrix-core/src/l1_action/nt_infra_scatter_gather.rs` (4 处 unwrap)
- `neotrix-core/src/l1_action/nt_infra_learning.rs` (5 处 unwrap)
- `neotrix-core/src/l1_action/nt_infra_integration.rs` (5 处 unwrap)
- 全库 grep `unwrap()` 定位剩余批量

**完成标准**:
- [ ] unwrap 数量 < 200 (第一批: 600+ → 200)
- [ ] 所有 `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()` 替换为 `.duration_since(UNIX_EPOCH).unwrap_or_default()`
- [ ] 所有 `Mutex::lock().unwrap()` 替换为 `.lock().unwrap_or_else(|e| e.into_inner())`
- [ ] `cargo test -p neotrix --lib` 全部通过

**预期收益**: 消灭 600+ 潜在崩溃点，Health Score +12
**估时**: 8h (分 3 批，每批 2-3h)

---

### T0-02: 预存编译错误修复 — gateway/nt_io_web 14 errors

**描述**: `cargo check --all-targets -p neotrix` 存在 14 个编译错误 (主要在 gateway 和 nt_io_web)，阻塞全量测试和 CI。

**涉及文件**:
- `nt_io_web/` 相关文件
- gateway provider 相关文件
- `prefer_free` / `gateway` 字段定义

**完成标准**:
- [ ] `cargo check --all-targets -p neotrix` 零错误
- [ ] `cargo check --features full --lib -p neotrix` 零错误
- [ ] CI pipeline 绿灯

**预期收益**: 解除全量测试阻塞，Health Score +8
**估时**: 4h

---

### T0-03: NTX P0 致命缺陷修复 (6 项)

**描述**: `ntx-defect-fix-plan.md` 中 Phase A 的 6 个 P0 致命缺陷，直接阻塞 NTX 核心功能。

**涉及文件**:
- `ntx/frames.rs` — decode_bytes 丢失 uncompressed_len
- `ntx/search_bridge.rs` — TODO 占位未实现
- `ntx/mod.rs` — close() 吞错误
- `ntx/lex_segment.rs` — 新建缺失模块
- `ntx/sync.rs` — import_to_sqlite 空操作
- `nt_memory_kb/mod.rs` — KnowledgeBase 无 NTX 集成

**完成标准**:
- [ ] frames.rs decode 正确读取 uncompressed_len
- [ ] search_bridge.rs 返回实际搜索结果 (非空)
- [ ] close() 返回 commit() 的真实结果
- [ ] lex_segment.rs 创建并可编译
- [ ] import_to_sqlite() 实现真正的数据导入
- [ ] KnowledgeBase 加载 .ntx 文件并双写

**预期收益**: NTX 核心功能可用，Health Score +10
**估时**: 6h

---

### T0-04: C0 模块快速提升 — 31 个仅编译模块

**描述**: 31 个 C0 模块仅能编译，无任何测试覆盖。按 Dark Forest 规则，无测试+无消费者的模块应被删除或提升。

**涉及文件**:
- NT-SHIELD 域: 6 个 C0 模块 (最高 C0 占比 30%)
- NT-IO 域: 7 个 C0 模块
- NT-CORE 域: 3 个 C0 模块
- NT-ACT 域: 3 个 C0 模块

**完成标准**:
- [ ] 审查 31 个 C0 模块，判定: 保留+补测 / 删除 (Dark Forest)
- [ ] 保留模块: 每个至少 1 个 `#[cfg(test)]` 单元测试
- [ ] C0 数量降至 <10
- [ ] `cargo test -p neotrix --lib` 新增测试全部通过

**预期收益**: 消灭"死代码"模块，Health Score +6
**估时**: 6h

---

### T0-05: unsafe 安全审计 — 58 处 unsafe

**描述**: R-P1 禁止 unsafe，但当前有 58 处。主要集中在 `stealth_net/proxy_chain.rs` 和 `hypercube/vsa`。逐一审计: 必要则隔离到 `unsafe` crate，不必要则移除。

**涉及文件**:
- `nt_shield_sandbox/stealth_net/proxy_chain.rs` (主要集中)
- `nt_core_hcube/vsa/` (向量运算)
- 其他散落 unsafe 块

**完成标准**:
- [ ] 每处 unsafe 标注: 必要(隔离到独立crate) / 不必要(移除)
- [ ] 必要 unsafe 迁移到 `crates/nt-unsafe/` 独立 crate (feature gate)
- [ ] neotrix-core crate 零 unsafe (`#![forbid(unsafe_code)]` 不被绕过)
- [ ] `cargo clippy -p neotrix -- -D warnings` 通过

**预期收益**: 满足 R-P1 安全约束，Health Score +8
**估时**: 6h

---

### T0-06: 巨文件拆分 — 13 个 >800 行文件

**描述**: 13 个文件超过 800 行，违反单一职责。按 CONTEXT.md 架构分层拆分。

**涉及文件** (按行数降序):
- 最大 13 个文件 (需先 `wc -l` 定位)
- 拆分目标: 每个文件 <500 行

**完成标准**:
- [ ] 13 个文件全部 <500 行
- [ ] 拆分后 `mod.rs` 正确 re-export
- [ ] `cargo check --all-targets -p neotrix` 零错误
- [ ] 拆分后功能等价 (测试通过)

**预期收益**: 可维护性提升，为插件化 Element 协议铺路，Health Score +5
**估时**: 8h

---

## P1 — 短期 (两周内)

### T1-01: 测试覆盖救火 — 10+ 零测试模块

**描述**: 10+ 模块超过 300 行但零测试覆盖。按 SelfTest T1→T2→T3 逐级补齐。

**涉及文件**:
- 所有 >300 行零测试模块 (需 grep `#[cfg(test)]` 定位)

**完成标准**:
- [ ] 每个模块至少 T1 (impl SelfTest 存在)
- [ ] 关键路径模块达到 T2 (registered in run.rs + pipeline.rs)
- [ ] 测试覆盖模块数: 0 → 10+

**预期收益**: 核心逻辑可验证，Health Score +8
**估时**: 12h

---

### T1-02: NTX P1 重要修复 (16 项)

**描述**: `ntx-defect-fix-plan.md` Phase B 的 16 个 P1 修复，提升 NTX 健壮性。

**涉及文件**:
- `ntx/format.rs` — TOC 验证 + magic bytes
- `ntx/vec_segment.rs` — magic/checksum + HashMap 索引
- `ntx/graph_segment.rs` — magic/checksum + O(1) 查找
- `ntx/time_segment.rs` — magic/checksum + 二分查找
- `ntx/wal.rs` — 移除 try_clone 双文件描述符
- `ntx/ntx_integration.rs` — 缓存/uuid 修复
- `ntx/mod.rs` — checkpoint O(N²) → O(N), open_read_only 真只读

**完成标准**:
- [ ] 所有段写入包含 magic + SHA-256 校验
- [ ] graph_segment 查找从 O(N²) 降至 O(1)
- [ ] WAL 移除双文件描述符
- [ ] 16 项 P1 全部修复
- [ ] `cargo test -p neotrix --lib` 通过

**预期收益**: NTX 数据完整性保障，Health Score +6
**估时**: 10h

---

### T1-03: 构建缓存不可信验证 — 真实错误计数

**描述**: R-P9/R-P17/R-P29/R-P35/R-P51/R-P54 要求结构变更后获取真实错误计数。

**涉及文件**:
- CI pipeline (`.github/workflows/`)
- Makefile / build scripts

**完成标准**:
- [ ] CI 中 `cargo clean` 后 build (或连续 build 两次)
- [ ] `cargo test -p neotrix --lib` 在 clean 环境通过
- [ ] `cargo check --features full --lib -p neotrix` 在 clean 环境通过

**预期收益**: 消除构建缓存虚假通过，Health Score +4
**估时**: 3h

---

### T1-04: C3 基准测试建立 — 0→30 模块

**描述**: 当前 C3 (基准测试) 模块数为 0%。为性能敏感模块建立 Criterion 基准。

**涉及文件**:
- `benches/` 目录
- 关键路径: KB search, VSA embedding, GWT routing, SEAL pipeline

**完成标准**:
- [ ] 至少 30 个模块有 Criterion 基准测试
- [ ] CI 集成 `cargo bench` + baseline 对比
- [ ] 基准结果写入 `docs/performance/`

**预期收益**: 性能回归可检测，Health Score +5
**估时**: 8h

---

### T1-05: CLIPPEr 规则修复 — 18 clippy warnings

**描述**: `cargo clippy -p neotrix-types` 输出 18 个警告，需逐一修复。

**涉及文件**:
- `crates/neotrix-types/` 全部文件

**完成标准**:
- [ ] `cargo clippy -p neotrix-types` 零警告
- [ ] `cargo clippy -p neotrix -- -D warnings` 通过

**预期收益**: 代码质量提升，Health Score +3
**估时**: 4h

---

### T1-06: Egress Privacy Guard 跨域一致性

**描述**: CONTEXT.md 定义了 3 级 Trust Tiers (Trusted/Contracted/Untrusted)，需验证 643 处跨域违规中有无 egress 相关违规。

**涉及文件**:
- `nt_core_llm::egress_privacy_guard`
- `nt_shield_sandbox/egress_policy.rs`
- 所有 outbound LLM 请求路径

**完成标准**:
- [ ] Trusted tier: secret scrub only
- [ ] Contracted tier: redact all internal fingerprints + absolute paths + secrets
- [ ] Untrusted tier: fail-closed block on internal fingerprint
- [ ] 所有 outbound 路径经过 guard

**预期收益**: 安全合规，Health Score +5
**估时**: 6h

---

### T1-07: 零测试模块快速补测 (R3 延续)

**描述**: 元认知自检发现 10+ 模块 >300 行零测试，这是 roadmap 遗漏的关键项。

**涉及文件**:
- 定位: `rg -l 'fn .*\\(' --type rust | xargs grep -L '#\\[cfg(test\\)]'` 交叉 >300 行文件

**完成标准**:
- [ ] 每个零测试模块新增至少 3 个测试用例
- [ ] 测试覆盖核心路径 (happy path + error path)
- [ ] `cargo test -p neotrix --lib` 新增测试全部通过

**预期收益**: 核心逻辑可验证，Health Score +6
**估时**: 10h

---

## P2 — 中期 (一个月内)

### T2-01: C2 集成测试覆盖 — 3→100+ 模块

**描述**: 当前仅 1 个 C2 模块 (0.3%)，集成测试严重不足。按 NT-* 域逐个补齐。

**涉及文件**:
- 每个域的 `tests/` 目录
- 关键跨域路径: KB↔NTX, SEAL↔Skill Engine, GWT↔AttentionManager

**完成标准**:
- [ ] 每个 NT-* 域至少 10 个 C2 模块
- [ ] 总 C2+ 模块数 >100 (28%+)
- [ ] 跨域集成测试: KB↔NTX, SEAL↔Skill, GWT↔Attention

**预期收益**: 跨域协作可验证，Health Score +8
**估时**: 20h

---

### T2-02: Skill Engine 并发加载优化

**描述**: skill engine 面临 114k skills 并发加载问题。需实现懒加载 + LRU + 信号量。

**涉及文件**:
- `nt_mind_skill_engine.rs`
- `nt_mind_skill_engine/loader.rs`

**完成标准**:
- [ ] 懒加载: 仅加载当前任务需要的 skill
- [ ] LRU 缓存: 最近使用 skill 保留内存
- [ ] 信号量: 并发加载数限制
- [ ] 加载延迟 < 100ms (P99)

**预期收益**: 大规模 skill 可用，Health Score +4
**估时**: 12h

---

### T2-03: mmap-io 安全集成 — 消除 unsafe 瓶颈

**描述**: `ntx-skipped-issues-deep-analysis.md` 推荐 mmap-io (零 unsafe API) 替代 unsafe mmap。

**涉及文件**:
- `Cargo.toml` — 添加 mmap-io 依赖
- `ntx/vec_segment.rs` — 实现 from_file() mmap 加载
- `ntx/mod.rs` — 集成到 NtxFile::open()

**完成标准**:
- [ ] VecSegment::from_file() 使用 mmap-io 零拷贝加载
- [ ] 零 unsafe 在 mmap 路径
- [ ] 性能: 加载延迟 < 50ms (1GB 文件)
- [ ] `cargo test -p neotrix --lib` 通过

**预期收益**: 大文件加载性能提升 5x，Health Score +3
**估时**: 6h

---

### T2-04: WAL 压缩 — Zstd 逐条目

**描述**: WAL 压缩减少磁盘 I/O，按 `ntx-skipped-issues-deep-analysis.md` 推荐 Zstd 逐条目方案。

**涉及文件**:
- `ntx/wal.rs` — 添加 CompressionType 枚举
- `ntx/wal.rs` — 实现 append_compressed() + decompress_payload()

**完成标准**:
- [ ] CompressionType: None / Zstd / LZ4
- [ ] 压缩率 2.0-3.0x
- [ ] IO 减少 40-60%
- [ ] 崩溃恢复: 校验和验证通过

**预期收益**: WAL 存储效率提升，Health Score +3
**估时**: 6h

---

### T2-05: 分层 EventBus 重构

**描述**: 元认知自检发现 EventBus 需要两层结构 (本地 + 全局)，解决跨域事件传播问题。

**涉及文件**:
- `nt_core_event_bus.rs`
- 各域事件发布/订阅路径

**完成标准**:
- [ ] Local EventBus: 域内事件
- [ ] Global EventBus: 跨域事件
- [ ] 事件类型: Lifecycle / Data / Health / Security
- [ ] 重入锁安全

**预期收益**: 跨域事件传播可靠，Health Score +5
**估时**: 10h

---

### T2-06: Skill Interface Contract 强化

**描述**: Easel 模式 P0: SKILL.md (<200 lines) + references/ + scripts/ + tests/ 合约。

**涉及文件**:
- 所有 `skills/` 目录下的 SKILL.md
- `nt_mind_skill_engine/` 加载器

**完成标准**:
- [ ] 所有 SKILL.md < 200 行
- [ ] 每个 skill 有 references/ + scripts/ + tests/
- [ ] Skill loader 验证合约完整性

**预期收益**: Skill 质量门控，Health Score +3
**估时**: 8h

---

### T2-07: ConsciousnessTree 拓扑进化

**描述**: 11-branch ConsciousnessTree 缺少拓扑自动进化能力。

**涉及文件**:
- `nt_meta/consciousness_tree.rs`
- `nt_core_consciousness_core.rs`

**完成标准**:
- [ ] 分支健康度自动评估
- [ ] 弱分支自动激活 (从 C0→C1)
- [ ] 交叉域依赖自动检测
- [ ] 6-stage feedback loop 完整闭环

**预期收益**: 自我进化能力提升，Health Score +6
**估时**: 12h

---

### T2-08: 量化 PI 压力测试门

**描述**: arxiv:2608.18578 发现 INT4/NF4 量化同键入侵 <5%，需实现压力测试门。

**涉及文件**:
- `nt_shield_sandbox/quant_policy.rs` (新建)
- LLM provider 选择路径

**完成标准**:
- [ ] PI 压力测试: 量化前后同键入侵率 <5%
- [ ] 默认 INT8，INT4/NF4 需 PI 通过
- [ ] 量化决策可配置

**预期收益**: 量化安全可控，Health Score +3
**估时**: 8h

---

## P3 — 长期 (季度)

### T3-01: 全链路 C5 自愈覆盖

**描述**: 所有模块达到 C5 (自愈) 能力，当前 C5 占比 32.1%。

**涉及文件**:
- 所有 C0-C4 模块

**完成标准**:
- [ ] C5 模块占比 >80%
- [ ] 自愈能力: 失败检测→降级→恢复
- [ ] HeartbeatAggregator 集成

**预期收益**: 系统韧性提升，Health Score +10
**估时**: 40h

---

### T3-02: C6 进化循环扩展 — 每域 20%

**描述**: 当前 C6 仅 13 个模块 (3.5%)，需扩展到每域至少 20%。

**涉及文件**:
- NT-CORE, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD 域

**完成标准**:
- [ ] 每域 C6 模块 >20%
- [ ] SEAL pipeline 完整闭环
- [ ] 进化: explore→distill→test→absorb

**预期收益**: 自主演化能力，Health Score +8
**估时**: 60h

---

### T3-03: 插件化 Element 协议

**描述**: 拆分巨文件后的终极目标: 每个 Element 可独立加载/卸载。

**涉及文件**:
- `plugin/registry.rs`
- `plugin/sandbox.rs`

**完成标准**:
- [ ] WASM 插件加载 (wasmtime)
- [ ] 插件沙箱: 限制资源 (内存/文件/网络)
- [ ] 插件 CLI: list/load/unload/info
- [ ] 插件市场: MCP 注册中心

**预期收益**: 生态可扩展，Health Score +8
**估时**: 80h

---

### T3-04: 并行 Agent — 多 Worker 协作

**描述**: 支持 `--agents N` 启动多个并发 Worker/Explorer/Monitor。

**涉及文件**:
- `entry/mod.rs`
- `agent/` 目录

**完成标准**:
- [ ] 多 Worker 并行执行任务
- [ ] 状态隔离 (per-task context)
- [ ] 结果聚合
- [ ] 故障转移

**预期收益**: 任务吞吐量提升 N 倍，Health Score +5
**估时**: 60h

---

### T3-05: 量化-Aware 认知退化管理

**描述**: 量化模型性能退化需要系统性管理框架。

**涉及文件**:
- `nt_core_self/quant_degradation.rs` (新建)
- LLM provider 选择路径

**完成标准**:
- [ ] 量化级别自动评估 (FP16/INT8/INT4/NF4)
- [ ] 认知退化度量 (accuracy/latency/memory)
- [ ] 自动降级策略
- [ ] 与 Cost-Aware Routing 集成

**预期收益**: 量化部署安全可控，Health Score +4
**估时**: 30h

---

### T3-06: Context-as-Scarce-Resource 实现

**描述**: Axiom A2: context window / KV capacity 是根本瓶颈。实现 KVMem paged KV for >256K sessions。

**涉及文件**:
- `kv_cache_optimizer.rs`
- GWT attention routing

**完成标准**:
- [ ] <256K tokens: compaction (快速)
- [ ] >256K tokens: paged KV (高效)
- [ ] Step-level scheduling (inter-step KL 37×)
- [ ] Delta reuse (retained blocks 直接复用)

**预期收益**: 长会话性能提升，Health Score +6
**估时**: 40h

---

### T3-07: VS Code 扩展 — LSP 集成

**描述**: 支持 VS Code 内联建议 + 对话面板。

**涉及文件**:
- `src/vscode-extension/`
- LSP server

**完成标准**:
- [ ] LSP-based inline suggestions
- [ ] Chat panel
- [ ] Diagnostics integration
- [ ] 暗色/亮色主题

**预期收益**: 开发者体验提升，Health Score +3
**估时**: 80h

---

## 执行顺序与依赖图

```
P0 (本周, 串行+并行):
├── T0-02 编译错误修复 [独立, 最先]
├── T0-01 统一错误域 [依赖 T0-02]
├── T0-05 unsafe 审计 [可与 T0-01 并行]
├── T0-06 巨文件拆分 [依赖 T0-05]
├── T0-03 NTX P0 [独立]
└── T0-04 C0 模块提升 [独立]

P1 (两周内, 依赖 P0):
├── T1-02 NTX P1 [依赖 T0-03]
├── T1-07 零测试补测 [独立]
├── T1-04 C3 基准 [独立]
├── T1-01 C2 集成测试 [依赖 T1-07]
├── T1-03 构建缓存验证 [独立]
├── T1-05 clippy 修复 [独立]
└── T1-06 Egress Guard [独立]

P2 (一个月内, 依赖 P1):
├── T2-01 C2 集成测试 [依赖 T1-01]
├── T2-02 Skill Engine [独立]
├── T2-03 mmap-io [依赖 T0-05]
├── T2-04 WAL 压缩 [独立]
├── T2-05 EventBus [独立]
├── T2-06 Skill Contract [独立]
├── T2-07 ConsciousnessTree [独立]
└── T2-08 量化 PI 门 [独立]

P3 (季度, 依赖 P2):
├── T3-01 C5 自愈 [依赖 T2-01]
├── T3-02 C6 进化 [依赖 T3-01]
├── T3-03 插件化 [依赖 T0-06]
├── T3-04 并行 Agent [独立]
├── T3-05 量化退化管理 [依赖 T2-08]
├── T3-06 Context KV [独立]
└── T3-07 VS Code 扩展 [独立]
```

---

## 健康评分预估

| 阶段 | Health Score 增量 | 累计 |
|------|-------------------|------|
| P0 完成后 | +49 (12+8+10+6+8+5) | 42→91 |
| P1 完成后 | +39 (8+6+4+5+3+5+6) | 91→130→cap at 100 |
| P2 完成后 | +36 (8+4+3+3+5+3+6+3) | capped |
| P3 完成后 | +36 (10+8+8+5+4+6+3) | capped |

**实际 Health Score 计算方式**: 各维度加权平均 (build/modules/layers/safety/architecture/tests/errors)，100 分封顶。

---

## R-P79 合规矩阵

| 阶段 | 任务数 | 落地 | 路线图 | Spike | 拒绝 |
|------|--------|------|--------|-------|------|
| P0 | 6 | 6 | 0 | 0 | 0 |
| P1 | 7 | 0 | 7 | 0 | 0 |
| P2 | 8 | 0 | 0 | 8 | 0 |
| P3 | 7 | 0 | 0 | 0 | 7 (spike first) |

---

## 风险与缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| 787 unwrap 修复引入新 bug | 🔴 Critical | 分批修复 + 每批 `cargo test` 验证 |
| 14 编译错误涉及架构变更 | 🔴 Critical | T0-02 最先执行，解阻塞 |
| unsafe 迁移影响性能 | 🟠 High | 性能基准对比 (T1-04) |
| 巨文件拆分引入循环依赖 | 🟠 High | 拆分后 `cargo check` 验证 |
| mmap-io crate 不成熟 | 🟡 Medium | 锁定版本 + 适配层 |
| WAL 压缩增加 CPU 开销 | 🟡 Medium | 可配置压缩级别 |

---

*文档生成时间: 2026-09-11 | 基于: 101+315+643+10000 迭代积累 | 方法论: 元认知自检 + 能力成熟度 + 缺陷深度分析*
