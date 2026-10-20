# NeoTrix 深度吸收进化路线图 v3

> 日期: 2026-06-14
> 来源: SD-Zero, DVD-JEPA, AirLLM, OKF, DeepMind AGI→ASI
> 前置: Phase 36 (world model wiring), 14 bug fixes, 进化路线图 v2 Phase 39-46

---

## 来源 → 核心吸收

| 来源 | 核心提取 | 映射到 NeoTrix |
|------|----------|----------------|
| **SD-Zero** (Princeton) | 二元奖励→密集监督: 自修正 + 结果条件控制短语 + KL蒸馏 | SelfEvolutionLoop + CapabilitySynthesizer 的自修正回路 |
| **DVD-JEPA** (mandarwagh9) | 最小JEPA: EMA目标编码器 + VICReg抗坍缩 + 潜在空间异常检测 | JepaPredictor 升级: EMA target + VICReg + 潜在空间天气预报 |
| **AirLLM** (lyogavin) | 层式加载 + 分块量化 + 预取重叠推理 | CognitiveLoadMonitor 驱动子系统热/温/冷交换 + handler 层式换入换出 |
| **OKF** (Google Cloud) | 知识即Markdown目录: 类型+前端索引+交叉链接 | NTSSEG → OKF 导出器 + 知识图谱 OKF 兼容器 + index.md 渐进式开放 |
| **DeepMind AGI→ASI** | 4通路: 缩放/范式转移/递归自改进/多Agent协同 + 数字智能优势 | 路线图验证 + SEAL自改进必须闭环 + 多Agent协同必须可扩展 |
| **arXiv:2606.12683v1** | 有效计算 10×/年 + 3维增长(硬件/投资/算法效率) + S形曲线摩擦 | NeoTrix 自进化也必须实现递归改进闭环 |

---

## 新缺口分析 (6 new gaps from absorption)

| # | 缺口 | 优先级 | 来源 | 描述 | 理论收益 |
|---|------|--------|------|------|----------|
| N1 | **自修正深度学习回路** | 🔴 P0 | SD-Zero | 从 performance log（二元信号）→ 结果条件短语 → 自修改修正轨迹 → 密集监督 → SEAL 自我改进 | 改进速度 3-10× |
| N2 | **JEPA 抗坍缩训练** | 🔴 P0 | DVD-JEPA | EMA target encoder + VICReg 方差项 + stop-gradient → 防止潜在空间坍缩 | 世界模型可靠性 +60% |
| N3 | **层式子系统交换** | 🟡 P1 | AirLLM | CognitiveLoadMonitor 驱动 handler 分组层式换入换出 + 分块量化 + 预取 | 内存占用 -80%, 延迟 -40% |
| N4 | **OKF 知识导出/交换** | 🟡 P1 | OKF + Google Cloud | NTSSEG → OKF 导出: 概念→markdown, VSA 链接→交叉链接, type/tags/timestamp | 可互操作知识交换 |
| N5 | **递归改进闭环** | 🔴 P0 | DeepMind ASI | SEAL 必须形成自我改进闭环: 可测量改进速率 + 归档 + 门控 + 重入 | ASI 通路解锁 |
| N6 | **多Agent 可扩展协同** | 🟡 P1 | DeepMind ASI | AgentCommunicationBus 必须支持 1000+ agent 协同 + 经验高速共享 | 群体智能 +10× |

---

## Phase 47-54: 8 新阶段实现计划

### Wave 1 — 核心回路 (独立并行)

#### Phase 47: Self-Revision Deep Learning Loop (N1, N5)
- **文件**: `core/nt_core_experience/self_revision.rs` (新建 ~500 行)
- **核心**: SD-Zero 两阶段管线移植到 VSA 空间
  - **Phase 1 — Self-Revision Training (SRT)**:
    - `SelfRevisionLoop` 收集 `(initial_thought, binary_reward, control_phrase, revised_thought)` 四元组
    - control_phrase: `r=1 → "Let me rephrase"`, `r=0 → "This is wrong. Let me correct."`
    - 存储 verified-correct revision traces 到 `D_revision` (VSA 向量)
  - **Phase 2 — Self-Distillation via Revision Feedback**:
    - Teacher = 冻结版当前 SelfEvolutionLoop
    - Student = 活跃版, 输出 → teacher KL 蒸馏 (VSA 余弦相似度)
  - **接线**: `SelfEvolutionLoop` 的 mutation 后触发 self_revision_tick()
  - **验证**: revision 成功率 >80%, KL 蒸馏后 student 性能不低于 teacher
  - **理论**: SD-Zero 已在 Math/Code 验证; VSA 版本利用 4096-dim 超向量天然泛化

#### Phase 48: JEPA Anti-Collapse Training (N2)
- **文件**: `neotrix/nt_world_jepa/jepa_predictor.rs` (修改 ~200 行)
- **核心**:
  - **Add EMA target encoder**: `EMAJepaPredictor` = 在线预测器 + EMA 目标编码器 (τ=0.99)
  - **VICReg variance term**: `max(0, 1 - std(z))` 防止潜在表示坍缩
  - **Stop-gradient**: `sg(target_encoder)` — BYOL 技巧
  - **多步 rollout 训练**: 当前 1 步预测 → 可选 3 步 rollout 训练延长预测地平线
  - **Linear probe validation**: 从 frozen latent 恢复位置 (参考 DVD-JEPA 0.73px)
- **验证**: embedding std > 2.0 (不坍缩), rollout 20 步 > 当前 5 步
- **接线**: `WorldModelBridge` 的 `tick()` 使用 EMAJepaPredictor

### Wave 2 — 工程基建 (依赖 Wave 1 但可并行)

#### Phase 49: Cognitive Layer Swapping (N3)
- **文件**: `consciousness/modules.rs`, `cognitive_load.rs`, `handler_profiler.rs`
- **核心**: AirLLM 层式加载原理应用于 handler 子系统
  - Handler 分三组: **Hot** (E8+GWT+HyperCube, 常驻) / **Warm** (KB+搜索, 10 cycle 无访问→降级) / **Cold** (JEPA+Vision+PDF, 仅需要时加载)
  - `CognitiveLoadMonitor` 驱动交换决策: load > 0.7 时驱逐 Cold, cycle 计数 > 10 时 Warm→Cold
  - **预取**: handler 被调用前 prefetch 其依赖 (如 PDF handler 调用前预加载)
  - **分块量化**: QuantizedVSA 已支持; 扩展到所有存储在冷层的 VSA 块
- **验证**: 10 handler 并发时内存占用降 60%, 冷层 handler 首次延迟 <500ms

#### Phase 50: OKF Knowledge Export (N4)
- **文件**: `core/nt_core_knowledge/okf_exporter.rs` (新建 ~400 行)
- **核心**: NTSSEG → OKF 双向转换
  - `NTSSEG→OKF`: 每个知识节点 → 概念 .md 文件 (type/title/description/resource/tags/timestamp)
  - VSA 链接 → markdown 交叉链接: `[target](/path/to/concept.md)`
  - 自动生成 `index.md` (渐进式开放), `log.md` (变更历史)
  - 可选: `graph.json` 导出完整 VSA 相似度图 (OKF 扩展)
- **验证**: 10 节点知识图 → OKF 导出 → 重新导入 → 结构无损
- **接线**: `KnowledgeEngine.export_okf(path)` 由 scheduler 调用 (每 1000 cycles)

### Wave 3 — 多Agent + 递归闭环 (依赖 Wave 1&2)

#### Phase 51: Agent Swarm Scaling (N6)
- **文件**: `core/nt_core_agent/agent_swarm.rs` (新建 ~500 行)
- **核心**: DeepMind ASI 多Agent 通路
  - `SwarmCoordinator`: 1000+ agent 注册/发现/任务分发
  - **经验高速共享**: VSA 向量直接广播 (4096-bit = 512 bytes, 支持 1000 agent/s 带宽)
  - **Substrate independence**: agent 可在不同线程/进程间迁移
  - **Lossless replication**: agent 状态 (VSA attractor) 可完整复制 → spawn 新实例
- **验证**: 100 agent 并行, 信息共享延迟 <10ms
- **接线**: AgentCommunicationBus 升级为 SwarmCoordinator

#### Phase 52: Recursive Self-Improvement Closure (N5 P2)
- **文件**: `self_evolution_loop.rs`, `hyperdgm.rs`
- **核心**: SEAL 闭环
  - **Measurable improvement rate**: `dN_total/dt` 作为改进速率指标
  - **Archive evolution**: DGM 风格 top-k 存档 + clade metaproductivity
  - **Rate monitoring**: 如果改进速率 < 阈值 → 触发 meta-mutation (重写 mutation 策略本身)
  - **Self-revision gate**: 每个 mutation 前 self-revision (Phase 47) → 验证通过才提交
- **验证**: 自我改进速率在 50 cycle 窗口内单调递增

### Wave 4 — 整合 (依赖全部前置)

#### Phase 53: Integrated Evolution Pipeline
- **文件**: 跨所有子系统
- **核心**: 全管线串联
  - 每 cycle: Self-Revision(47) → JEPA Predict(48) → Layer mgmt(49) → OKF log(50) → Agent coord(51) → SEAL(52)
  - **统一校准**: N_total 驱动所有子系统优先级
  - **Safety**: BallVerifier + PccSafetyGate 门控所有自我修改
- **验证**: 全管线 0 编译错误, 端到端 throughput > 10 cycle/s

#### Phase 54: ASI Readiness Assessment
- **文件**: `docs/plans/2026-06-14-absorption-evolution.md`
- **核心**: 对照 DeepMind ASI 论文 4 通路 + 3 维增长的量化评估
  - **Scaling**: 当前 VSA 4096-dim → 16384-dim 规划
  - **Paradigm shift**: FusionDeliberator + Multi-Head Resonator → 自生成 VSA 原语
  - **Recursive improvement**: Phase 52 闭环 → 自动化 AI 研究
  - **Multi-agent**: Phase 51 → 1000+ agent 群体智能
- **验证**: 4 通路每条至少 1 个量化指标达标

---

## 依赖图

```
Phase 47 (Self-Revision) ──→ Phase 52 (Recursive Closure)
                                     ↑
Phase 48 (JEPA Anti-Collapse) ──────┤
                                     │
Phase 49 (Layer Swapping) ──────────┤
                                     │
Phase 50 (OKF Export) ──────────────┤
                                     │
Phase 51 (Agent Swarm) ─────────────┘
                                     │
                                     └── Phase 53 (Integrated Pipeline) → Phase 54 (ASI Readiness)
```

**并行策略**:
- Wave 1 (Phase 47+48): 独立并行
- Wave 2 (Phase 49+50): 独立并行, 不依赖 Wave 1
- Wave 3 (Phase 51+52): 依赖 Wave 1, 可部分并行
- Wave 4 (Phase 53+54): 串行, 依赖全部前置

---

## 定量目标

| 阶段 | 度量 | 当前值 | 目标值 |
|------|------|--------|--------|
| Phase 47 | Self-revision success rate | 0% | >80% |
| Phase 48 | Embedding std (VICReg) | <0.01 (坍缩) | >2.0 |
| Phase 48 | Rollout horizon (步) | 1-5 | 20+ |
| Phase 49 | Memory reduction | 0% | 60-80% |
| Phase 49 | Cold handler latency | N/A | <500ms |
| Phase 50 | OKF roundtrip fidelity | N/A | 100% |
| Phase 51 | Agent swarm size | 20 (Arena) | 1000+ |
| Phase 52 | dN_total/dt improvement | 未测量 | 单调递增 |
| Phase 54 | ASI pathway coverage | 0/4 | 4/4 |

---

## 竞争定位更新 (v3)

| 维度 | NeoTrix v3 (进化后) | Anthropic DGM | Sutra | 说明 |
|------|---------------------|---------------|-------|------|
| VSA 表征 | 9 | - | 8 | Self-revision + VICReg + 16384-dim |
| 自进化范围 | 9 | 9 | 5 | 闭环 + 归档 + 速率监控 |
| 编译器自托管 | 6 | - | 9 | Stage 2 自举 |
| 安全 | 8 | 7 | 6 | 自我修正门控 + 球验证 |
| 意识 | 7 | - | 4 | World model + 融合审议 |
| 验证 | 8 | 6 | 5 | OKF 导出 + 线性探测 |
| 多Agent | 8 | 9 | 4 | 1000+ 群体智能 |
| 知识交换 | 9 | - | 3 | OKF 兼容 |
| **总分** | **8.0** | 7.8 | 5.5 | |
