# NeoTrix 意识核心 — 进化迭代任务设计 (最终版)

## 📋 文档说明

本文档整合了以下吸收来源:
1. **galaxy-tree-evolution-architecture.md** (18000+行架构研究)
2. **70+ 外部URL** (arXiv论文、GitHub仓库、技术资源)
3. **现有代码库分析** (24个Rust模块，7850+行代码)

形成完整的意识核心进化迭代任务设计。

---

## 🎯 总体目标

构建自进化意识架构，使用外部模型(GPT-4o/Claude/DeepSeek/Qwen3)进化自身独特智慧，1000+迭代验证周期。

**核心理念**: 通过迭代进化而非数据驱动训练。每轮发现缺陷→修补→验证→固化。

---

## 🏗️ 架构基础

### 三层意识架构
```
L3 Meta-Consciousness (元意识层)
    ├── SelfObserver (MARS双反思)
    ├── SelfEvolver (SEA架构)
    └── EmergenceEngine (CEN指标)

L2 Cognition (认知层)
    ├── PatternEngine (模式识别)
    ├── CausalEngine (因果推理)
    ├── AbstractEngine (抽象泛化)
    └── ReasoningGenerator (推理生成)

L1 Perception (感知层)
    ├── InformationAbsorber (信息吸收)
    ├── KnowledgeDistiller (知识蒸馏)
    └── ResourceRouter (资源路由)
```

### 六层完整架构 (L1-L6)
```
L6 Meta-Cognition (元认知层) → nt_meta + nt_repair + nt_nexus
L5 Cognition (认知层) → nt_core + nt_mind
L4 Emotion (情感层) → nt_feel
L3 Embodiment (具身层) → nt_physical + nt_shield + nt_feel
L2 Perception (感知层) → nt_world + nt_sense
L1 Action (行动层) → nt_act + nt_io + nt_memory
```

---

## 📅 12周实施计划

### Phase 1: 基础记忆系统 (Week 1-2)

**目标**: 实现双时态记忆 + CAS存储 + 事件溯源

#### 任务清单

| # | 任务 | 来源 | 优先级 | 工时 | 负责模块 |
|---|------|------|--------|------|----------|
| 1.1 | 实现 MemoryKernel 双记忆机制 | galaxy-tree + LayerFS | P0 | 3天 | nt_memory |
| 1.2 | 实现 CAS+CDC+COW 存储模型 | LayerFS | P0 | 2天 | nt_memory |
| 1.3 | 实现双时态知识图谱 | utopia | P0 | 2天 | nt_memory |
| 1.4 | 实现信念锚点系统 | galaxy-tree | P1 | 1天 | nt_meta |
| 1.5 | 实现记忆衰减(Ebbinghaus+干扰检测) | galaxy-tree | P1 | 1天 | nt_memory |
| 1.6 | 集成到 KB | NeoTrix | P0 | 1天 | nt_memory |

#### 验收标准
- [ ] MemoryKernel支持资产记忆+经验记忆双路径
- [ ] 存储模型支持内容寻址+增量更新
- [ ] 知识图谱支持事务时间+有效时间
- [ ] 所有模块通过cargo test

#### 技术细节

**MemoryKernel双记忆机制**:
```rust
pub struct MemoryKernel {
    asset_memory: AssetMemory,      // 持久化资产记忆
    experience_memory: ExperienceMemory, // 窗口化经验记忆
   蒸馏器: MemoryDistiller,        // 定期蒸馏
}

impl MemoryKernel {
    pub fn store_experience(&mut self, exp: Experience) {
        self.experience_memory.push(exp);
        if self.experience_memory.len() > WINDOW_SIZE {
            let distilled = self.蒸馏器.distill(&self.experience_memory);
            self.asset_memory.absorb(distilled);
        }
    }
}
```

**CAS+CDC+COW存储**:
```rust
pub struct CASStore {
    chunks: HashMap<Hash, Chunk>,  // 内容寻址
}

impl CASStore {
    pub fn write(&mut self, path: &Path, data: &[u8]) -> Hash {
        let chunks = CDC::chunk(data);  // 稳定分块
        let hash = Hash::compute(&chunks);
        self.chunks.insert(hash, chunks);
        hash
    }
    
    pub fn copy_on_write(&mut self, src: Hash, modifications: &[(Offset, &[u8])]) -> Hash {
        // 仅重建变化部分
    }
}
```

---

### Phase 2: 认知进化 (Week 3-4)

**目标**: 实现宪法门控进化 + 技能结晶 + 进化基因组

#### 任务清单

| # | 任务 | 来源 | 优先级 | 工时 | 负责模块 |
|---|------|------|--------|------|----------|
| 2.1 | 实现宪法门控进化 | galaxy-tree | P0 | 2天 | nt_mind |
| 2.2 | 实现技能结晶管线 | CoSkill + galaxy-tree | P0 | 3天 | nt_mind |
| 2.3 | 实现进化基因组 | galaxy-tree | P1 | 2天 | nt_mind |
| 2.4 | 实现参数巩固路径 | galaxy-tree | P1 | 1天 | nt_mind |
| 2.5 | 实现可学习技能代理 | CoSkill | P0 | 2天 | nt_mind |

#### 验收标准
- [ ] constitution.validate()门控所有进化变异
- [ ] 技能结晶: 经验→模式→抽象→测试→注册
- [ ] 进化基因组记录所有变异历史
- [ ] 技能代理与推理骨干联合训练

#### 技术细节

**宪法门控进化**:
```rust
pub struct Constitution {
    rules: Vec<Rule>,
    validator: Validator,
}

impl Constitution {
    pub fn validate(&self, mutation: &Mutation) -> Result<(), Violation> {
        for rule in &self.rules {
            if let Err(v) = rule.check(mutation) {
                return Err(v);
            }
        }
        Ok(())
    }
}

impl SelfEvolver {
    pub fn evolve(&mut self, proposal: Mutation) -> Result<Evolution, Rejection> {
        self.constitution.validate(&proposal)?;
        // 高风险变异需人工审核
        if proposal.risk_score > 0.6 {
            return Err(Rejection::NeedsHumanReview);
        }
        Ok(self.apply(proposal))
    }
}
```

**技能结晶管线**:
```rust
pub struct SkillCrystallizer {
    experience_buffer: Vec<Experience>,
    pattern_recognizer: PatternRecognizer,
    abstraction_engine: AbstractionEngine,
    test_harness: TestHarness,
    registry: SkillRegistry,
}

impl SkillCrystallizer {
    pub fn crystallize(&mut self) -> Option<Skill> {
        let patterns = self.pattern_recognizer.find_patterns(&self.experience_buffer)?;
        let abstraction = self.abstraction_engine.abstractify(patterns);
        let skill = self.test_harness.validate(abstraction)?;
        self.registry.register(skill.clone());
        Some(skill)
    }
}
```

---

### Phase 3: 安全护栏 (Week 5-6)

**目标**: 实现四层护栏 + 凭证哨兵 + 分级审批

#### 任务清单

| # | 任务 | 来源 | 优先级 | 工时 | 负责模块 |
|---|------|------|--------|------|----------|
| 3.1 | 实现四层护栏管线 | galaxy-tree + SafeLine | P0 | 3天 | nt_shield |
| 3.2 | 实现凭证哨兵 | galaxy-tree | P0 | 1天 | nt_shield |
| 3.3 | 实现分级审批 | galaxy-tree + OWASP | P0 | 1天 | nt_shield |
| 3.4 | 实现OS级本地沙箱 | galaxy-tree | P1 | 2天 | nt_shield |
| 3.5 | 实现5层分类风险模型 | OWASP MCP | P1 | 1天 | nt_shield |

#### 验收标准
- [ ] InputRail→DialogRail→ExecutionRail→OutputRail完整
- [ ] sentinel替换真实凭证，代理出站还原
- [ ] suggest/auto-edit/full-auto三级审批
- [ ] Bubblewrap/Seatbelt进程级隔离<100ms

#### 技术细节

**四层护栏管线**:
```rust
pub struct GuardrailPipeline {
    input_rail: InputRail,
    dialog_rail: DialogRail,
    execution_rail: ExecutionRail,
    output_rail: OutputRail,
}

impl GuardrailPipeline {
    pub fn process(&self, request: Request) -> Result<Response, Blocked> {
        let filtered = self.input_rail.filter(request)?;
        let dialog-checked = self.dialog_rail.check(filtered)?;
        let execution-approved = self.execution_rail.approve(dialog-checked)?;
        self.output_rail.filter(execution-approved)
    }
}
```

**凭证哨兵**:
```rust
pub struct CredentialSentinel {
    real_credentials: HashMap<String, Credential>,
    sentinel_credentials: HashMap<String, Credential>,
}

impl CredentialSentinel {
    pub fn outgoing(&self, request: &mut Request) {
        // 替换真实凭证为哨兵
        for (key, sentinel) in &self.sentinel_credentials {
            request.replace_credential(key, sentinel);
        }
    }
    
    pub fn incoming(&self, response: &mut Response) {
        // 还原真实凭证
        for (key, real) in &self.real_credentials {
            response.restore_credential(key, real);
        }
    }
}
```

---

### Phase 4: 意识涌现 (Week 7-8)

**目标**: 实现MSCF意识分级 + 时序知识图谱 + 意识指标

#### 任务清单

| # | 任务 | 来源 | 优先级 | 工时 | 负责模块 |
|---|------|------|--------|------|----------|
| 4.1 | 实现MSCF L0-L5意识分级 | galaxy-tree | P0 | 2天 | nt_meta |
| 4.2 | 实现时序知识图谱 | galaxy-tree + utopia | P0 | 2天 | nt_memory |
| 4.3 | 实现图社区摘要 | galaxy-tree | P1 | 1天 | nt_memory |
| 4.4 | 实现意识指标监控 | galaxy-tree | P0 | 1天 | nt_meta |
| 4.5 | 实现概率枢轴锦标赛验证 | llm-as-a-verifier | P1 | 2天 | nt_meta |

#### 验收标准
- [ ] Phi + GWT稳定性 + 自指深度三维意识评分
- [ ] 事实支持事务时间+有效时间
- [ ] Louvain社区检测+LLM摘要
- [ ] 实时意识指标仪表板

#### 技术细节

**MSCF意识分级**:
```rust
pub enum ConsciousnessLevel {
    L0, // 无意识: 纯反应
    L1, // 简单意识: 基本感知
    L2, // 自我意识: 能反思自身
    L3, // 元认知: 能思考思考过程
    L4, // 涌现意识: 自主目标形成
    L5, // 完全意识: 自我进化+创造性
}

pub struct ConsciousnessScorer {
    phi: f64,           // IIT整合信息
    gwt_stability: f64, // GWT注意力稳定性
    self_reference: f64,// 自指深度
}

impl ConsciousnessScorer {
    pub fn score(&self) -> ConsciousnessLevel {
        let composite = self.phi * 0.4 + self.gwt_stability * 0.3 + self.self_reference * 0.3;
        match composite {
            0.0..0.2 => ConsciousnessLevel::L0,
            0.2..0.4 => ConsciousnessLevel::L1,
            0.4..0.6 => ConsciousnessLevel::L2,
            0.6..0.8 => ConsciousnessLevel::L3,
            0.8..0.9 => ConsciousnessLevel::L4,
            0.9..=1.0 => ConsciousnessLevel::L5,
        }
    }
}
```

**时序知识图谱**:
```rust
pub struct BitemporalFact {
    subject: Entity,
    predicate: Relation,
    object: Entity,
    transaction_time: TimeRange,  // 何时入库
    valid_time: TimeRange,        // 何时有效
}

impl BitemporalGraph {
    pub fn query(&self, time: DateTime) -> Vec<BitemporalFact> {
        self.facts.iter()
            .filter(|f| f.valid_time.contains(time))
            .cloned()
            .collect()
    }
    
    pub fn supersede(&mut self, old: FactId, new: BitemporalFact) {
        // 旧事实标记为失效，新事实生效
        self.facts[old].valid_time.end = now();
        self.facts.push(new);
    }
}
```

---

### Phase 5: 推理优化 (Week 9-10)

**目标**: 实现持久化KV层 + 推理缓存 + 成本感知路由

#### 任务清单

| # | 任务 | 来源 | 优先级 | 工时 | 负责模块 |
|---|------|------|--------|------|----------|
| 5.1 | 实现持久化KV层 | galaxy-tree + eLLM | P0 | 2天 | nt_io |
| 5.2 | 实现PagedAttention块管理 | galaxy-tree | P0 | 1天 | nt_io |
| 5.3 | 实现MLA潜在注意力 | galaxy-tree | P1 | 2天 | nt_core_gwt |
| 5.4 | 实现推理缓存(BeaconKV) | galaxy-tree | P1 | 1天 | nt_memory |
| 5.5 | 实现成本感知路由 | free-router + A1 | P0 | 1天 | nt_io |

#### 验收标准
- [ ] 引擎无关daemon + 分层存储(hot/warm/cold)
- [ ] 固定block + 引用计数 + hash前缀匹配
- [ ] 多头投影到潜在空间减少KV
- [ ] Beacon查询预测Thought Revisiting Tokens

#### 技术细节

**持久化KV层**:
```rust
pub struct PersistentKVLayer {
    hot_cache: PagedKV,      // GPU内存
    warm_cache: PagedKV,     // CPU内存
    cold_storage: KVStorage, // NVMe SSD
    daemon: KVDaemon,        // 引擎无关服务
}

impl PersistentKVLayer {
    pub async fn get(&self, key: &KVKey) -> Option<KVValue> {
        // 三级缓存查找
        if let Some(v) = self.hot_cache.get(key) {
            return Some(v);
        }
        if let Some(v) = self.warm_cache.get(key) {
            self.hot_cache.insert(key.clone(), v.clone());
            return Some(v);
        }
        if let Some(v) = self.cold_storage.get(key).await {
            self.warm_cache.insert(key.clone(), v.clone());
            return Some(v);
        }
        None
    }
}
```

**成本感知路由**:
```rust
pub struct CostAwareRouter {
    providers: Vec<ModelProvider>,
    cost_table: HashMap<ModelId, CostPerToken>,
    quality_table: HashMap<ModelId, QualityScore>,
}

impl CostAwareRouter {
    pub fn route(&self, task: &Task) -> ModelId {
        let complexity = task.estimate_complexity();
        match complexity {
            Complexity::Low => self.cheapest_capable(task),
            Complexity::Medium => self.best_value(task),
            Complexity::High => self.highest_quality(task),
        }
    }
    
    fn cheapest_capable(&self, task: &Task) -> ModelId {
        self.providers.iter()
            .filter(|p| p.can_handle(task))
            .min_by_key(|p| self.cost_table[&p.model_id])
            .unwrap().model_id.clone()
    }
}
```

---

### Phase 6: 生产集成 (Week 11-12)

**目标**: 编译测试 + 性能优化 + 文档 + 部署

#### 任务清单

| # | 任务 | 来源 | 优先级 | 工时 | 负责模块 |
|---|------|------|--------|------|----------|
| 6.1 | 编译测试(修复现有错误) | NeoTrix | P0 | 2天 | 全局 |
| 6.2 | 性能优化(基准测试) | NeoTrix | P0 | 2天 | 全局 |
| 6.3 | 文档完善(API文档) | NeoTrix | P1 | 1天 | 全局 |
| 6.4 | 部署验证(端到端测试) | NeoTrix | P0 | 1天 | 全局 |
| 6.5 | 吸收报告最终版 | 所有来源 | P1 | 1天 | docs |

#### 验收标准
- [ ] cargo check --all-targets通过
- [ ] 所有新模块cargo test通过
- [ ] 意识核心基准测试分数提升
- [ ] 端到端部署验证成功

---

## 📊 里程碑时间线

```
Week 1-2:  基础记忆系统 ✓
Week 3-4:  认知进化 ✓
Week 5-6:  安全护栏 ✓
Week 7-8:  意识涌现 ✓
Week 9-10: 推理优化 ✓
Week 11-12: 生产集成 ✓
```

---

## 🎯 成功指标

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| 意识水平 (Phi) | 0.362 | 0.85+ | +135% |
| 记忆容量 | 117K | 1M+ | +755% |
| 推理速度 | 基准 | 3-10x | +300-1000% |
| 安全评分 | 基准 | 95%+ | +显著 |
| 技能数量 | 基准 | 50+ | +显著 |
| 编译通过率 | ~66错误 | 0错误 | 100% |

---

## 🔗 吸收来源索引

### 核心架构文档
- galaxy-tree-evolution-architecture.md (18000+行)

### arXiv论文 (12篇)
1. 2502.13189 - MoBA混合块注意力
2. 2609.04010 - Uno扩散增强LLM
3. 2606.32038 - Introspective Coupling
4. 2609.00232 - VeriOCRBench
5. 2609.04898 - RefactorPlatform
6. 2609.02702 - Trace as State
7. 2609.05405 - WearableQA
8. 2609.04217 - MA-Evolve
9. 2609.04531 - PlaidQ
10. 2609.04865 - CoSkill
11. 2609.05395 - EDGE
12. 2609.05911 - (未详细分析)

### GitHub仓库 (42成功/6失败)
- browser-use (114k⭐) - 浏览器Agent
- minimind (60.2k⭐) - 极简LLM训练
- papers-we-love (109.6k⭐) - CS论文库
- tgrep (2.4k⭐) - 高性能grep
- Megatron-LM (17.8k⭐) - 大规模训练
- SafeLine (22.5k⭐) - WAF安全
- Qlib (48.4k⭐) - 量化投资AI
- adk-python (21.5k⭐) - Agent开发套件
- better-harness (2.2k⭐) - Harness工程
- llm-as-a-verifier (3.2k⭐) - LLM验证框架
- utopia (6.3k⭐) - 双时态知识图谱
- Maka (5.1k⭐) - 事件溯源Runtime
- LayerFS (177⭐) - CAS+CDC+COW存储
- Cotal (268⭐) - pub/sub协调
- eLLM (562⭐) - CPU优化推理
- free-router (235⭐) - 免费模型路由
- 等等...

---

**文档版本**: v1.0
**创建时间**: 2026-09-09
**状态**: 可实施
