# Batch3 2026-08-26 — 47 源全域进化迭代统一 TODO

> 来源: `notes/absorption-batch-20260826.md` 吸收矩阵 (46 新增 + 1 历史)
> 关系: 与 `docs/evolution-master-roadmap-2026-08-25.md` (上批 52 源) **去重合并** — 重叠源 (freellmapi↔CLIProxyAPI / nativePDF↔marker / Ix↔双轨代码图谱) 标注合并而非重复立项
> 纪律: R-P42 (强化现有节点, 目标模块已 grep 接地) · R-P79 (消费者具名) · R-P100 (落地时 `neotrix-capability` bud/strengthen)

---

## Wave 0 — P0 机制修复 (本批发现的缺陷, 脚本级, 可同 session 闭环)

- [x] **W0.1 kb_batch_absorb.py 批计数器双计修复**
  - 缺陷: 批摘要报 `inserted=94`, 逐条日志实际 47 — 内部计数器把 Rust CLI 返回与本地循环重复累计
  - 位置: `scripts/kb_batch_absorb.py` (main 汇总处)
  - 判据: 单次运行 N URL → 批摘要 inserted == SQL `COUNT(nodes WHERE url IN batch)` == 日志条数
  - 源追踪: 本批实测 | 类型: defect

- [x] **W0.2 arxiv 镜像源 ID 归并 dedup** ✅ 2026-08-26 实测: 镜像 URL 归一化 + 批内去重 + 存量三重平行清零 (2608.23552 三节点→canonical 1 节点, 校正元数据已迁移)
  - 缺陷: paperswithcode.co / alphaxiv.org 与 arxiv.org 同 ID 论文产生平行节点 (本批 2608.23552 双节点)
  - 位置: `scripts/kb_batch_absorb.py` dedup 步骤 — URL 规范化后追加 arxiv ID 二次归并 (`/(abs|pdf|paper)/(\d{4}\.\d{4,5})` 提取)
  - 判据: 同 arxiv ID 多镜像仅 1 节点, 其余判 `duplicate-mirror`
  - 源追踪: 本批 pattern | 类型: rule

- [x] **W0.3 KNOWN_REPOS 确定性键补充 (本批高置信仓库)**
  - 动作: `scripts/absorb_to_capability.py` KNOWN_REPOS 新增: `NVIDIA/SkillSpector→SHIELD/audit`, `1N3/Sn1per→SHIELD/audit`, `tashfeenahmed/freellmapi→IO/delegate`, `microsoft/agent-lightning→IO/delegate`, `tickernelz/opencode-mem→MEMORY/recall`, `agentforce314/clawcodex→ACT/execute`
  - 判据: 重跑 `--report` 这些仓库不再走 keyword 路径; 新增键查重防 dict 覆盖 (Cycle 208 教训)
  - 源追踪: 本批校正门 32 条中提炼

---

## Wave 1 — P0 高信号接线 (本周, 全部强化现有节点)

- [ ] **W1.1 压缩断崖阈值量化 → context_budget 截断器** 
  - 源: arxiv 2608.22752 *The Compaction Cliff in Long-Running AI Agent Memory* (MEMORY/compress)
  - 目标模块: `neotrix-core/src/core/nt_core_context/mod.rs` (`apply_context_budget`)
  - 动作: 引入 compaction-cliff 信号 — 长会话压缩事件前后任务成功率衰减监测; 截断预算低于断崖阈值时告警并保留锚点段
  - 消费者 (R-P79): 后台循环 context budget Pass1 已调用该路径
  - 能力树: strengthen `nt_core_context::compaction_cliff_guard`
  - 判据: 注入长对话测试集, 压缩后关键事实保留率可量化输出

- [ ] **W1.2 SkillSpector skill 静态审计方法 → P6 vetting gate 规则集**
  - 源: NVIDIA/SkillSpector (SHIELD/audit)
  - 目标模块: `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind_skill_engine.rs` (`scan_skill_content`) + `l1_body_impl/nt_shield/tool_inspection_stack.rs`
  - 动作: 以 `trust_rule!` 模式新增 SkillSpector 类检测规则 (skill 元数据欺骗/权限提升模式); 零 LLM、纯 regex 可审计
  - 消费者 (R-P79): skill 加载路径已调用 `scan_skill_content`, rejected 即拒载
  - 能力树: strengthen `nt_shield::skill_vetting_ruleset`
  - 判据: 构造 3 个恶意样本 skill 被 reject, 3 个良性 skill 通过; 规则单测覆盖

- [ ] **W1.3 免费 LLM provider 聚合路由 + failover → LlmProvider 层** 〔与主路线图 2.1 CLIProxyAPI 合并〕
  - 源: tashfeenahmed/freellmapi (IO/delegate) — 34 provider / 635 endpoint / key 加密 / per-key 用量追踪 / 限流 failover
  - 目标模块: `neotrix-core/src/core/nt_core_llm.rs` (`trait LlmProvider`) + provider factory
  - 动作: 在现有 provider 抽象上加 ordered-fallback 路由器 (限流→下一 provider) + 免费档用量记账; 不新建平行网关 (R-P42, 合并 CLIProxyAPI OAuth 工作为同一任务族)
  - 消费者 (R-P79): `nt_io_provider` 生产调用面
  - 能力树: bud `nt_io::provider_failover_router`
  - 判据: 模拟 429 → 自动 failover 成功率 100%; per-provider 用量落 KB

- [ ] **W1.4 Ingest-time 索引编译 → KB 入库管线策略**
  - 源: arxiv 2608.20845 *RAG Deserves an Index* (MEMORY/persist) — ingest 时编译优于查询时解释
  - 目标模块: KB 入库路径 (`neotrix-experience absorb-node` + `nodes_fts` 写入)
  - 动作: 入库时同步生成结构化摘要字段 + 概念标签 (替代查询时全文扫描); 对齐既有显式 FTS 插入纪律
  - 消费者 (R-P79): 本批 47 节点即生产数据; `query --kw` 检索延迟对比
  - 能力树: strengthen `nt_memory_kb::ingest_index_compilation`
  - 判据: 1000 节点级 query 延迟下降可测; 摘要字段入库即存在

---

## Wave 2 — P1 结构强化 (下周)

- [ ] **W2.1 poka-yoke 约束披露门 → approval confirmation_gate**
  - 源: rainmanjam/poka-yoke (SHIELD/constrain) — "fix makes impossible" 披露率 45%→80%
  - 目标模块: `neotrix-core/src/neotrix/l6_self_impl/nt_shield_approval/confirmation_gate/mod.rs`
  - 动作: 审批请求 schema 增加 `forecloses` 字段 — 高影响编辑必须声明其关闭的可能性; 未披露降级为 Warning 级 andon 信号
  - 能力树: bud `nt_shield::foreclosure_disclosure_gate`
  - 判据: 高危编辑审批流中出现 forecloses 字段; 严重度三级 (Control/Warning/Detection) 映射测试

- [ ] **W2.2 测试时算力自适应分配 → AttentionManager**
  - 源: arxiv 2608.20256 *Learning When to Think* (CORE/route)
  - 目标模块: `neotrix-core/core/nt_core_self/attention_head.rs` (`AttentionManager`)
  - 动作: Dual Specialization 路由增加 difficulty-proportional 思考预算 — 简单任务 System1 直通, 复杂任务触发 System2 迭代; 对齐已有 mars_system1/2 激活计数
  - 能力树: strengthen `nt_core_self::adaptive_compute_route`
  - 判据: 分级任务集上 System2 触发率与任务难度正相关; phi/coherence 无回归

- [ ] **W2.3 零OCR PDF 结构化 → doc-parse 数字原生分支** 〔与主路线图 2.6 marker 任务族互补合并〕
  - 源: crunz-ai/nativePDF-structurer (MIND/transform) — 零 OCR 零模型依赖, 页眉尾去重/参数表/步骤抽取
  - 目标模块: `neotrix-core/src/neotrix/nt_file_ability/core.rs` (doc-parse 分支)
  - 动作: 为数字原生大体量 PDF (设备手册/维修手册类) 增加零依赖结构化路径 — 与 marker (扫描件 OCR 路) 形成 dual-path
  - 能力树: strengthen `nt_file_ability::doc_parse_native_pdf`
  - 判据: 100+ 页数字原生 PDF 结构化输出章节/表格树; 与 OCR 路径自动分流

- [ ] **W2.4 Stateful 业务 agent 沙箱基准 → shield_sandbox 验证集**
  - 源: arxiv 2608.19741 *Thinkingbox* (SHIELD/verify)
  - 目标模块: `nt_shield_sandbox` (Egress Policy 所在域)
  - 动作: 引入 stateful 业务流程基准用例 (多轮状态依赖任务), 作为沙箱 egress/权限策略的回归验证集
  - 能力树: bud `nt_shield::stateful_sandbox_bench`
  - 判据: 基准在 CI 可跑; 策略回归被用例捕获

- [ ] **W2.5 代码库上下文图强化 → 统一代码图谱** 〔与主路线图 1.10 双轨图谱合并〕
  - 源: ix-infrastructure/Ix (MEMORY/recall) — "virtual cartographer", 定时扫描→持续维护模型
  - 目标模块: `neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind/infrastructure/code_graph.rs`
  - 动作: 吸收 scheduled-scan→四视图 (L2/L3/workloads/apps 类比: 文件/符号/依赖/调用) 的持续文档化思路, 给 BlastRadiusIndex 加 staleness 信号
  - 能力树: strengthen `nt_mind::unified_code_graph`
  - 判据: 图谱 staleness 可观测; 过期子图标记并在检索时降权

- [ ] **W2.6 23 角色 subagent 模板集 → 子代理编排**
  - 源: garrytan/gstack (ACT/execute) — CEO/Designer/EngManager/QA 等 23 个 opinionated 角色
  - 目标模块: `nt_core_subagent` 角色注册表
  - 动作: 精选与本仓库工作流契合的角色模板 (QA/DocEngineer/ReleaseManager 优先) 进 subagent profile 库; 经 P6 vetting 后入库
  - 能力树: bud `nt_act::role_template_library`
  - 判据: ≥5 角色可通过 skill vetting 并被派发契约 (C1-C6) 引用

---

## Wave 3 — P2 Spike 探索 (两周后, 先 spike 后 bud)

- [ ] **W3.1 跨模型记忆迁移** — arxiv 2608.17050 (target-side reader adaptation) → KB embedding 跨版本迁移 spike; 判据: 换 embedding 后旧向量召回保持 >70%
- [ ] **W3.2 学术插图自动化** — arxiv 2601.23265 PaperBanana → data-viz/fireworks-tech-graph 论文配图 spike; 判据: 从 markdown 生成可编辑架构图
- [ ] **W3.3 通达信行情数据源** — handsomejustin/easy_tdx (毫秒级 A 股行情) → `nt_world_search` 金融数据 fetcher spike; 判据: 日线/分钟线拉取入库
- [ ] **W3.4 全球情报 MCP 工具面评估** — marc-shade/world-intel-mcp (120 tools) → 评估 30+ 域免费 API 中 NeoTrix 缺口工具, 按 Egress Policy 准入; 判据: 选型报告 + ≤10 工具准入清单
- [ ] **W3.5 编排失败类分类法** — BraxisAI/braxis-blueprint (失败类 failure classes) → orchestration 韧性知识注入 SelfTest 检测族; 判据: ≥8 失败类有对应检测规则
- [ ] **W3.6 无真值步级信用审计** — arxiv 2608.19760 → ConsciousnessTree 归因链审计 spike (执行轨迹 vs 声明一致性); 判据: 步级 credit 分配可解释输出
- [ ] **W3.7 自改进 harness 对照** — alphaxiv/pwc 2608.23552 Prime Agent RLM → SEAL pipeline 自改进循环对照实验; 判据: 一项技能经自动循环 C0→C2
- [ ] **W3.8 学习式 CLI 补全** — jacobpowaza/adaptive-zsh-completions → neotrix CLI zsh 补全动态生成 spike; 判据: 子命令补全无需手写 completion 脚本

---

## ❌ 拒绝 / 📋 仅记录项 (R-P79 拒绝依据)

| 源 | 裁决 | 依据 |
|---|---|---|
| an2tha/onerep (健身 OS) | ❌ 拒 | 无能力网消费者; WORLD/observe 泛化无实体接线点 |
| MengTo/sketchbook | 📋 记录 | 单文件创意前端, neocodex-frontend 动效参考即可, 不立项 |
| Alishahryar1/free-claude-code / kyegomez/OpenMythos | 📋 记录 | 对照实现/方法论参考, 现有能力覆盖 |
| Yu9191/wloc / YuJunZhiXue/dsh-purge | 📋 记录 | NT-SHIELD 威胁面知识保留, 不接线攻击工具 |
| huytieu/COG-second-brain / tickernelz/opencode-mem | 📋 记录 | 与 neotrix-experience 同构, 作设计对照 |
| topics/awesome-list / ripienaar/free-for-dev | 📋 记录 | 目录型来源; free-for-dev 已在上批路线图 2.9 立项 |

---

## 跨域元模式 (本批新增)

| 元模式 | 涉及域 | 本批证据 | 对齐动作 |
|---|---|---|---|
| **Ingest-time > Query-time** | MEMORY/WORLD | RAG Index 论文 + nativePDF 入库前结构化 | W1.4 + W2.3 |
| **约束披露 = 可靠性前提** | SHIELD/CORE | poka-yoke 45%→80% + Thinkingbox 沙箱 | W2.1 + W2.4 |
| **算力分配即注意力** | CORE/MIND | When-to-Think + Compaction Cliff | W2.2 + W1.1 |
| **持续扫描文档化** | MEMORY/SHIELD | Ix cartographer + scanopy 四视图 | W2.5 |
| **镜像源归一** | MEMORY | 2608.23552 平行节点 | W0.2 |

---

## 执行依赖与顺序

```text
W0.1/W0.2/W0.3 (脚本修复, 无依赖, 可立即)
   ↓
W1.1 ← 无硬依赖 (context_budget 已在生产)
W1.2 ← 无硬依赖 (vetting gate 已在生产)
W1.3 ← 依赖主路线图"预存编译错误修复"(gateway/nt_io_web) 解除 cargo test 阻塞
W1.4 ← 依赖 W0.2 (镜像归并先于索引编译策略)
   ↓
Wave 2 各项独立, 按 W2.1 → W2.2 → W2.3 优先 (安全门 > 注意力 > 解析)
   ↓
Wave 3 spike 并行, 每个 spike 出 Given/When/Then 判定后才允许 bud
```

## 验收门 (每 Wave 收尾)

1. `cargo check --lib -p neotrix` 0 errors (R-P9/R-P17)
2. 受影响模块 `cargo test --lib` 通过
3. 落地项 `neotrix-capability` bud/strengthen 登记 (R-P100)
4. 经验五阶段吸收 → cycle 递增落 KB
5. 本文件 checkbox 勾选 + TODO.md 指针同步
