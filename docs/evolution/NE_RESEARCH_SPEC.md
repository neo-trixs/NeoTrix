# .ne-research Specification v1

**文件格式**: `.ne-research.json` (JSON 序列化)
**应⽤场景**: 技能结晶、研究追踪、可复现实验包装、跨层法医绑定

## 1. 设计原则

- **4层镜像**: 遵循 ARA 四层结构 — logic/ (claims + experiments), trace/ (探索 DAG), evidence/ (证据表), src/ (可执行内核)
- **跨层法医绑定**: 每个实体带类型化引用, 形成完全可遍历的溯源图
- **VSA 索引**: 512-bit 语义指纹, 支持相似性发现
- **可验证性**: 所有绑定必须可解析, VSA 索引必须覆盖所有 claim

## 2. 文件结构

```
{
  "manifest": { ... },         // 元数据头
  "claims": [ ... ],           // logic/ — 研究主张
  "trace_nodes": [ ... ],      // trace/ — 探索 DAG
  "evidence_tables": [ ... ],  // evidence/ — 证据表
  "experiments": [ ... ],      // logic/experiments/ — 实验记录
  "bindings": [ ... ],         // 跨层绑定 (溯源图边)
  "vsa_index": [ ... ]         // VSA 语义索引
}
```

## 3. Manifest (清单)

```ne
manifest {
    name:          String        // 可读名称, e.g. "vsa-kernel-mutation-optimization"
    version:       String        // 语义版本, e.g. "0.1.0"
    description:   String        // ⾃由描述
    created_at:    u64           // Unix 纳秒时间戳
    author:        String        // ⽣产者, e.g. "NeoTrix SelfEvolution"
    entry_claims:  Vec<String>   // 顶级 claim ID 列表
    tags:          Vec<String>   // VSA 发现标签
    stats:         PackageStats  // 结构统计
}
```

### PackageStats

```ne
stats {
    trace_nodes:     usize   // 迹节点数
    claims:          usize   // 主张数
    experiments:     usize   // 实验数
    evidence_records: usize  // 证据记录数
}
```

## 4. Claims Layer (logic/)

### ResearchClaim

```ne
claim {
    id:                  String            // 唯⼀ ID, e.g. "claim-001"
    statement:           String            // ⼈类可读主张
    status:              FalsifiabilityStatus  // 可证伪性状态
    confidence:          f64               // 聚合置信度 [0, 1]
    source:              ClaimSource       // 来演
    anchor:              Option<ExecutableAnchor>  // 可执行锚点
    trace_bindings:      Vec<String>       // 关联的 trace node IDs
    experiment_bindings: Vec<String>       // 关联的实验 IDs
    evidence_bindings:   Vec<String>       // 关联的证据表 IDs
    code_bindings:       Vec<String>       // 关联的代码路径
}
```

### FalsifiabilityStatus

```ne
status: Hypothesized | InProgress | Confirmed | Refuted | Superseded
```

### ClaimSource

来演枚举（继承自 evidence::ClaimSource）:
- `UserInput` — ⽤户提供
- `WebSearch` — ⽹络搜索
- `GeneratedContent` — ⽣成内容
- `Reasoning` — 推理产⽣
- `Observation` — 观察
- `Inference` — 推断
- `SelfReflection` — ⾃我反思

### ExecutableAnchor

```ne
anchor {
    language:         String   // "ne", "rust", "python"
    code:             String   // 可执行代码/伪代码
    expected_pattern: String   // 确认模式/输出
    verified:         bool     // 是否已验证
}
```

## 5. Trace DAG Layer (trace/)

### TraceNode

```ne
trace_node {
    id:          String            // 唯⼀ ID, e.g. "trace-1"
    node_type:   TraceNodeType     // 节点类型
    label:       String            // ⼈类可读标签
    description: String            // 详细描述
    timestamp:   u64               // Unix 纳秒
    parent_id:   Option<String>    // 父节点 (None 为根)
    child_ids:   Vec<String>       // ⼦节点列表
    bindings:    TraceBindings     // 跨层绑定
    metadata:    HashMap<String, String>  // ⾃由元数据
}
```

### TraceNodeType

```ne
node_type: Question | Hypothesis | Decision | Experiment | DeadEnd | Pivot | Result
```

### TraceBindings

```ne
bindings {
    spawned_claims:      Vec<String>  // 本节点产⽣的 claim IDs
    triggered_experiments: Vec<String> // 本节点触发的实验 IDs
    produced_evidence:   Vec<String>  // 本节点产⽣的证据表 IDs
    reason:              Option<String> // DeadEnd/Pivot 原因
}
```

## 6. Evidence Layer (evidence/)

### EvidenceTable

```ne
evidence_table {
    id:                   String                    // 唯⼀ ID, e.g. "ev-table-001"
    claim_id:             String                    // 所⽀持/反驳的 claim ID
    experiment_id:        Option<String>             // 产⽣此表的实验 ID
    records:              Vec<ResearchEvidenceRecord> // 证据记录
    aggregated_confidence: f64                       // 聚合置信度
}
```

### ResearchEvidenceRecord

```ne
evidence_record {
    id:           u64                // 证据 ID
    source_url:   String             // 来 URL
    source_name:  String             // 来演名称
    assertion:    String             // 断⾔内容
    quotation:    Option<String>     // 引⽤原⽂
    confidence:   f64                // 置信度 [0, 1]
    state:        EvidenceState      // 验证状态
    scoring:      Option<ScoringDimensions>  // 评分维度
}
```

### EvidenceState

```ne
state: PendingReview | Validated | Disputed | Contradicted | Superseded | Irrelevant
```

### ScoringDimensions

```ne
scoring {
    relevance:             f64  // 相关性
    evidence_confidence:   f64  // 证据置信度
    recency:               f64  // 时效性
    source_authority:      f64  // 来权威度
    cross_references:      f64  // 交叉引⽤
    contradiction_penalty: f64  // 矛盾惩罚
}
```

## 7. Experiment Layer (logic/experiments/)

### ExperimentRecord

```ne
experiment {
    id:               String              // 唯⼀ ID, e.g. "exp-001"
    claim_id:         String              // 被测试的 claim ID
    name:             String              // 实验名称
    hypothesis:       String              // 假设
    type:             ExperimentType      // 实验类型
    baseline:         String              // 基准条件
    intervention:     String              // ⼲预条件
    metrics:          Vec<String>         // 度量名称列表
    status:           ExperimentStatus    // 状态
    evidence_table_id: Option<String>     // 关联的证据表 ID
}
```

### ExperimentType

```ne
type: Ablation | Comparison | ParameterSweep | Replication
```

### ExperimentStatus

```ne
status: Planned | Running | Completed | Failed
```

## 8. Cross-Layer Bindings

绑定系统是 .ne-research 的关键差异化特征——每个绑定都是溯源的⼀条有向边。

### Binding

```ne
binding {
    from_type: BindingEntityType  // 起实体类型
    from_id:   String             // 起实体 ID
    to_type:   BindingEntityType  // ⽬标实体类型
    to_id:     String             // ⽬标实体 ID
    relation:  String             // 关系语义, e.g. "spawned", "tested_by", "supports"
}
```

### BindingEntityType

```ne
entity: Claim | Experiment | TraceNode | EvidenceTable | CodePath
```

### 预定义关系语义

| from → to | relation | 含义 |
|-----------|----------|------|
| TraceNode → Claim | spawned | 迹节点产⽣了 claim |
| Claim → Experiment | tested_by | claim 被实验测试 |
| Claim → EvidenceTable | supported_by | claim 被证据⽀持 |
| Claim → CodePath | implements | claim 被代码实现 |
| Experiment → EvidenceTable | produced | 实验产⽣了证据 |
| TraceNode → EvidenceTable | produced | 迹节点收集了证据 |
| TraceNode → TraceNode | child_of | 迹 DAG 的⽗⼦关系 |

## 9. VSA Index (语义索引)

### VsaIndexEntry

```ne
vsa_index_entry {
    fingerprint: Vec<u8>   // 64 字节 (512-bit) VSA 指纹
    claim_ids:   Vec<String> // 此指纹索引的 claim IDs
    tags:        Vec<String> // 语义标签
}
```

指纹由确定性算法产⽣: `seed = fold(31 * acc + byte(name))`, 然后使⽤ `QuantizedVSA::seeded_random(seed, 64)`。

## 10. 验证规则

1. **绑定完整性**: 每个 binding 的 `to_id` 必须在对应类型的实体集合中存在 (`CodePath` 除外)
2. **VSA 覆盖**: 每个 claim 必须⾄少出现在⼀个 VSA index entry 中
3. **Manifest 完整性**: `manifest.name` 不能为空
4. **⾳符文件命名**: `{name}-{version}.ne-research.json`

## 11. ⽣命周期

```ne
1. export_skill_crystal(skill, trace_steps, evidence, scoring)
        ↓
2. ResearchPackage (内存对象)
        ↓
3. validate() → validate_bindings() + validate_vsa_index()
        ↓
4. save() → write JSON to disk
        ↓
5. load() / list() → 后续检索与引⽤
```

## 12. 可扩展性

- **CodePath 实体**: 当前仅在 binding 中引⽤ `src/kernel.ne` 等路径, 未来可扩展为 `CodeEntity { path, content, language }`
- **Ne 编译器集成**: `.ne-research` 的声⾔式格式天然可作为 Ne 编译器的⾃举⽬标
- **MemoryLattice 注⼊**: 已验证的 ResearchPackage 可注⼊ MemoryLattice 作为语义记忆
