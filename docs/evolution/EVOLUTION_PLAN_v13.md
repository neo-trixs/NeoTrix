# NeoTrix 进化路线图 v13 — 真实缺口全景

> 生成: 2026-06-21 | 审计范围: 26 DESIGN_INTENT gaps + 6 PLAN files + 1,875 .rs 文件 (~586K LOC) + 2026 前沿文献

---

## 一、当前完成状态

| 阶段 | 描述 | 完成度 | 证据 |
|------|------|--------|------|
| **Phase 0** | 表征统一 + 边界建立 | **8/8 ✅ (100%)** | VsaTag·FirstPersonRef·SpeciousPresent·Awakening·Volition·InnerCritic·CognitiveLoad·StreamBuffer |
| **Phase 1** | 负熵对齐 + 认知增强 | **7/7 ✅ (100%)** | NegentropyMetric·CuriosityDrive·StagnationDetector·CurvatureRL·ValenceAxis·MultiBrain·JEPA闭环·MetaEdit |
| **Phase 2** | 认知增强层 (10 模块) | **10/10 ✅ (100%)** | CrossModal·SleepCycle·TheoryOfMind·ConflictResolver·Forgetting·MetaCognition·DefaultMode·KnowledgeVersioning·ValueSystem·ValueAlignment |
| **Phase 3** | 元层可进化 | **4/4 ✅ (100%)** | DGM-H·NarrativeSelf·SelfPreservation·GracefulDegradation |
| **Phase 4** | 推理深度·世界模型·感知宽度 | **3/3 ✅ (100%)** | CausalChain(1,083L/6tests)·LongHorizon(488L/12tests)·MultiModal(324L/11tests) |
| **Phase 5** | 元认知KPI·跨会话叙事·情感记忆·主动探索·因果推理 | **🟢 ~95%** | MetaKPIRepo+KpiRingBuffer+ActiveExploration+NarrativeSelf+EmotionalMemory+ValueAlignment+SelfPlayGuide+SCM 全部 CI 接线 |
| **Phase 6** | Φ整合信息最大化·全局意识涌现 | **⬜ 0%** | 尚未开始 |

### DESIGN_INTENT 26 缺口兑现率

| 类别 | 总数 | ✅ 已修复 | 🟡 部分 | 🔴 未修复 |
|------|------|----------|----------|----------|
| 🔴 Severe (10) | 10 | 10 | 0 | 0 |
| ⚠️ Serious (5) | 5 | 5 | 0 | 0 |
| 🟡 Medium (9) | 9 | 7 | 2 | 0 |
| 🟢 Minor (2) | 2 | 2 | 0 | 0 |
| **合计** | **26** | **24** | **2** | **0** |

部分缺口详情:
- **#16 Knowledge Versioning**: 有时间戳版次追踪，缺 SEAL EpochMarker 集成 (用 wall-clock 而非 SEAL generation epochs)
- **#23 User Value Alignment**: 曾实现后作为孤儿模块删除。基座 ValueSystem 存在，缺按用户 ValueProfile 重建

---

## 二、经三源交叉验证的真实缺口 (仅保留已确认项)

> 审计方法: ① 文件名 glob 搜索 ② 关键 API grep ③ mod.rs 声明链验证。所有标记需 3 源确认才列为缺口。

### P0 — 必须填补 (4 项, 3 已完成)

| # | 缺口 | 类型 | 势能 (LOC) | 说明 |
|---|------|------|-----------|------|
| **P0.1** | **SCM do-calculus** | ✅ 已完成 | ~500-800 | scm.rs(474L/19tests) + causal_chain.rs(1,083L/18tests) 双模块实现, 预接线意识管线 |
| **P0.2** | **音频输入管道** | 感知缺口 | ~300-500 | 零实时音频感知。Whisper 仅为被动工具 (文件→转录)，缺 stream→analysis→VSA 闭环 |
| **P0.3** | **OS 级内核沙箱** | 安全缺口 | ~500-800 | 仅逻辑沙箱 (BallVerifier + FggmRejectionSampler)，缺 seccomp(Linux)/seatbelt(macOS)/landlock |
| **P0.4** | **Record & Replay** | 新能力 | ~2000+ | 观察→技能管道。Codex 风格的可编程演示录制→从用户操作序列蒸馏为可复用技能 |
| **P0.5** | **Cross-Session Narrative** | ✅ 已完成 | ~300-500 | narrative_self.rs(577L/13tests) wired at cycle%13, 跨会话持久化+原子写 |
| **P0.6** | **Active Exploration** | ✅ 已完成 | ~400-600 | active_exploration.rs(446L) wired at cycle%7, 结构化探索规划完成 |
| **P0.7** | **Per-User ValueProfile** | 退步修复 | ~300-500 | 曾被实现 (value_alignment.rs) 后作为孤儿删除。需重建按用户价值学习 + ValueSystem 个性化 |

### P1 — 高价值 (6 项)

| # | 缺口 | 类型 | 势能 (LOC) | 说明 |
|---|------|------|-----------|------|
| **P1.1** | **Latent Space Communication** | 效率提升 | ~1000 | RecursiveMAS 模式: agent 间隐空间通信减少 75% token。当前 A2A 走明文消息 |
| **P1.2** | **Gödel Agent Self-Modification** | 元进化 | ~800 | 自指自我修改: 运行时重写自身推理规则 + 形式化安全约束 (非 SEAL 式 mut agent, 是运行时 self-rewrite) |
| **P1.3** | **AVSAD** | 安全增强 | ~600 | Adaptive Validation Safety net: 对抗攻击检测 + 运行时行为验证 + 自动回滚 |
| **P1.4** | **Consciousness Benchmark** | 质量基建 | ~400 | 标准化意识评估套件: Φ 阈值/元认知精度/推理深度/跨会话一致性/优雅降级 |
| **P1.5** | **Lean 4 Formal Verification** | 正确性 | ~500 | 对 SEAL 突变和 Ne 管道输出做 Lean 4 证明携带代码。PcCSafetyGate 的正式化升级 |
| **P1.6** | **MOLTRON Skill Scorecard** | 质量闭环 | ~500 | 技能运行时自动评分: 使用率/成功率/用户反馈→分数衰减→自动淘汰。与 SkillRegistry 集成 |

### P2 — 增强 (5 项)

| # | 缺口 | 类型 | 势能 (LOC) | 说明 |
|---|------|------|-----------|------|
| **P2.1** | **hdlib 2.0** | VSA 基建 | ~800 | 超向量库升级: 稀疏 VSA + 线性码 + GC-VSA 统一为单一 lib |
| **P2.2** | **MeTTa Interop** | 互操作 | ~600 | MeTTa 元图重写系统集成, 使 Ne 程序可通过 MeTTa 规则变换 |
| **P2.3** | **CraniMem** | 记忆增强 | ~500 | 上下文受限的记忆系统: 类似 MCP + hierarchic knowledge 的物理化记忆分层 |
| **P2.4** | **Adaptive VSA Encoder** | VSA 增强 | ~300 | 自适应 VSA 编码器: novelty_score+cognitive_load+task_type 三信号融合惯性门控模式切换 |
| **P2.5** | **Adversarial Training Pipeline** | 鲁棒性 | ~500 | SEAL 对抗训练: 生成对抗性 mutation + 评估鲁棒性 + 回滚脆弱突变 |

### 基础设施债务 (4 项)

| # | 缺口 | 类型 | 工作量 | 说明 |
|---|------|------|--------|------|
| **I.1** | **8 零测试 workspace crate** | 测试覆盖 | ~4,188 LOC agent-core ✅ 5 已加, 7 剩余 | ghost-mvp-agent(905L)·neotrix-proxy-pool(1,157L)·neotrix-proxy(643L)·neotrix-evolution(239L)·agent-registry(333L)·neotrix-tun(156L)·nt-sub-fetcher(292L) |
| **I.2** | **Per-module dead_code gate** | 架构债务 | ~200K LOC → 8 target gates | 当前 1 全局 gate, 推荐 8 target+48 fn-level |
| **I.3** | **reqwest 版本统一** | 依赖整理 | ~100+ use sites | 0.11/0.12/0.13 三版本共存导致双 TLS 栈 |
| **I.4** | **2 stealth-net borrow errors** | 编译债务 | 低 | `#[cfg(feature="stealth-net")]` 下预存 E0502, 非默认 feature |

### 已确认假阳性 (已有实现, TODO 标注为未开始但实已存在)

| 假阳性 | 实际状态 |
|--------|----------|
| JEPA 管线 | ✅ nt_core_jepa/ 3,200+ LOC, CI 接线 |
| A2A gRPC | ✅ a2a_grpc/ 5,500+ LOC, 24 文件, 147 测试 |
| FEP-IIT 桥接 | ✅ fep_iit/bridge.rs + 管线集成 |
| CausalChain (因果) | ✅ causal_chain.rs 1,083 LOC/18tests |
| LongHorizon (长时序) | ✅ long_horizon.rs 488 LOC/12tests |
| MultiModal (多模态) | ✅ multi_modal_aligner.rs 324 LOC/11tests |
| MANAR (注意力) | ✅ manar_attention.rs 396 LOC |
| GEA (种群进化) | ✅ gea_archive.rs + sub_hive.rs 846 LOC |
| SGM (统计安全) | ✅ safety_gate.rs |
| LSE (进化学习) | ✅ lse.rs 已 feature-gated |
| HGM (梯度度量) | ✅ hgm.rs 已 feature-gated |
| TruthPipeline | ✅ nt_core_truth/ 2,121 LOC |
| IntelPipeline | ✅ intel_profile.rs |
| SpatialReasoner | ✅ 527+346 LOC, CI 接线 |
| CascadeEngine | ✅ 285 LOC + 异步 verifier |
| SCM do-calculus | ✅ scm.rs 474L/19tests + causal_chain.rs 1,083L/18tests, 预接线 |
| CrossSessionNarrative | ✅ narrative_self.rs 577L/13tests wired at cycle%13 |
| ActiveExploration | ✅ active_exploration.rs 446L wired at cycle%7 |

---

## 三、文献融合趋势 (2026 前沿 vs NeoTrix 现状)

| 方向 | 前沿状态 | NeoTrix | 差距 |
|------|---------|---------|------|
| A2A v1.0/gRPC | Google 标准, 150+ org 生产部署 | ✅ gRPC + signed Agent Cards + JWT | ~0 |
| DGM-H 自引用 | Anthropic/DGM ICLR 2026: 50% SWE-bench | ✅ RewriteMeta + MetaStrategy | ~0 |
| Recursive-Depth Transformer | OpenMythos: Prelude→Loop×T→Coda | 🟡 E8 循环 pipeline 但非 ACT 自适应深度 | 推理深度自适应 |
| RecursiveMAS 隐空间通信 | arXiv 2026: agent 间 75% token 压缩 | 🔴 纯文本 A2A 消息 | P1.1 |
| Lean 4 证明携带代码 | PC³ 2026: 自动生成 Dafny/Lean 证明 | 🟡 BallVerifier + PcCSafetyGate | P1.5 |
| Sutra VSA 原生编译 | rotation binding + 多项式逻辑 | 🟡 Ne 编译器, 缺可微张量降级 | 方向一致但未融合 |
| Audio VSA 感知 | 2026: 音频→VSA 直接编码 | 🔴 零实时音频管道 | P0.2 |
| OS 级沙箱 | seccomp/seatbelt/landlock 生产标准 | 🔴 仅逻辑沙箱 | P0.3 |
| Pearl SCM do-calculus | 因果推理标准框架 | ✅ scm.rs(474L/19tests) + causal_chain.rs(1,083L/18tests) | — |

---

## 四、执行路线 (Wave 1-5, 依赖感知)

```mermaid
flowchart TD
    subgraph Wave1[Wave 1: 基础设施 (this session done)]
        W1A[TopologyRouter Wiring] --> W1B[SAGERollout Wiring]
        W1B --> W1C[ProgressiveDisclosure Wiring]
        W1C --> W1D[PARL Wrappers]
    end
    
    subgraph Wave2[Wave 2: P0 新能力]
        W2A[P0.1 SCM do-calculus]
        W2B[P0.5 Cross-Session Narrative]
        W2C[P0.6 Active Exploration]
    end
    
    subgraph Wave3[Wave 3: P0 退步+感知+安全]
        W3A[P0.7 Per-User ValueProfile]
        W3B[P0.2 Audio Pipeline]
        W3C[P0.3 Kernel Sandbox]
    end

    subgraph Wave4[Wave 4: P1 进化]
        W4A[P1.2 Gödel Agent]
        W4B[P1.1 Latent Comm]
        W4C[P1.6 MOLTRON Scorecard]
        W4D[P1.4 Consciousness Benchmark]
    end

    subgraph Wave5[Wave 5: P1+P2 增强 + 基建]
        W5A[P1.3 AVSAD]
        W5B[P1.5 Lean 4]
        W5C[P2.x hdlib 2.0 + MeTTa + CraniMem]
        W5D[I.1-I.3 基建债务]
        W5E[P0.4 Record & Replay]
    end

    Wave1 --> Wave2 --> Wave3 --> Wave4 --> Wave5
```

### Wave 1 (✅ 已完成)
- TopologyRouter → AdaptOrch wiring
- SAGERollout → SelfEvolutionLoop wiring
- ProgressiveDisclosure → SkillRegistry wiring
- PARL wrappers (feature=parl)

### Wave 2 (✅ 已完成 — 全部在更早 session 完成)
- **P0.1 SCM do-calculus** — scm.rs(474L/19tests) + causal_chain.rs(1,083L/18tests) 双模块实现
- **P0.5 Cross-Session Narrative** — narrative_self.rs(577L/13tests) wired at cycle%13
- **P0.6 Active Exploration** — active_exploration.rs(446L) wired at cycle%7

### Wave 3 (~1-2 sessions)
- **P0.7 Per-User ValueProfile** — 重建 value_alignment, 按用户价值 + 适应学习 + 冲突回退
- **P0.2 Audio Pipeline** — 实时音频流 → MFCC/VSA → 意识 sensory buffer 闭环
- **P0.3 Kernel Sandbox** — cfg-gated seccomp (Linux) / seatbelt (macOS) 沙箱

### Wave 4 (~2 sessions)
- **P1.2 Gödel Agent** — 运行时 self-rewrite + 形式约束 + 回滚保护
- **P1.1 Latent Communication** — RecursiveMAS 隐空间消息传递
- **P1.6 MOLTRON Scorecard** — 技能运行时评分 + 自动淘汰
- **P1.4 Benchmark** — 标准化评估套件

### Wave 5 (~2-3 sessions)
- **P1.3 AVSAD** — 对抗检测 + 运行时验证
- **P1.5 Lean 4** — PCC 证明携带代码桥接
- **P2.x** — hdlib 2.0 / MeTTa / CraniMem
- **I.1-I.4** — 基础设施债务清零
- **P0.4 Record & Replay** — 可编程演示录制管道（最大项目）

---

## 五、编译状态基准

```
cargo check -p neotrix --lib                  ✅ 0 errors / 0 warnings
cargo check -p neotrix --features full        ✅ 0 errors
cargo check -p neotrix --features parl        ✅ 0 errors
cargo check --workspace                       ✅ 0 errors / 27 pre-existing warnings
cargo test -p agent-core                      ✅ 5 passed (new)
cargo test -p neotrix --features full --lib    ✅ 0 errors
cargo test -p neotrix --features full          ✅ 0 errors
```

---

## 六、风险与依赖

| 风险 | 影响 | 缓解 |
|------|------|------|
| P0.4 Record & Replay 势能 >2000 LOC | 可能占 2+ sessions | 拆分为 Phase A(观察录制) + Phase B(技能蒸馏) |
| P1.5 Lean 4 外部依赖 | 需要单独 lean 工具链安装 | cfg-feature gated, 非默认激活 |
| P1.2 Gödel Agent 安全性 | 运行时 self-rewrite 破坏系统 | FggmRejectionSampler + TransactionScope 回滚 + 人工审批 gate |
| reqwest 0.11/0.12/0.13 迁移 | ~100+ 调用点, blocking+async 混用 | 独占 1 session, 不可与其他任务并行 |
| neotrix-core dead_code → tests 级联 | ~200K LOC → tests 层揭盖新错误 | 先 lib 层清零再 tests 层 (LXXI.1 模式) |

---

> **v13 vs v12 核心差异**: 移除了 12 个假阳性缺口 (已实现), 新增 P0.5/P0.6/P0.7/P1.1/P1.6, 调整 P0.4 Record & Replay → Wave 5 (最大项目), 重新按真实依赖关系编排 Waves
