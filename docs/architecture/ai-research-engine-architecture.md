# AI Agent 工程化核心技术架构蓝图 — 统一可视化与模板

## Mermaid 可视化

### 统一参考架构总览

```mermaid
graph TB
    subgraph Human["🧑 人类边界层"]
        H1["选问题 / 方向校准"]
        H2["最终 Review / 决策"]
        H3["找人协作 / 社区反馈"]
    end

    subgraph ControlPlane["🎛️ Orchestration & Control Plane (可训练的研究引擎)"]
        direction TB
        
        subgraph Dispatcher["Dispatcher / Orchestrator"]
            D1["状态机 + Workflow-as-Code"]
            D2["Outcome-Driven Router<br/>(反向推理: 目标 → 实验)"]
            D3["Gate Keeper<br/>(G1-G8 + Human Gates + Escalation)"]
            D4["Multi-Role / Multi-Model Scheduler"]
        end

        subgraph Memory["Memory & Logging System (写下来 + 盯输出)"]
            M1["Append-only Event Log<br/>Hypothesis → Expectation → Result → Belief"]
            M2["Project Memory<br/>(规范 / Journal / 失败库 / 架构图)"]
            M3["Checkpoint + Reproducible Config"]
        end

        subgraph Eval["Deterministic Evaluator + Analyzer"]
            E1["纯规则评分 + Failure Clustering"]
            E2["Self-healing / Ablation 支持"]
            E3["Taste Trainer<br/>(预测 vs 实际 → 校准)"]
        end

        subgraph Context["Context Loader (三层注入)"]
            C1["全局上下文 (角色 / 规范)"]
            C2["任务上下文 (当前 Goal / 状态)"]
            C3["历史上下文 (相关 Log / 失败案例)"]
        end
    end

    subgraph Execution["⚙️ Execution Layer"]
        direction TB
        
        subgraph Agents["多 Agent 团队"]
            A1["PM Agent"]
            A2["Architect Agent"]
            A3["Engineer Agent"]
            A4["QA / Evaluator Agent"]
            A5["Researcher Agent"]
            A6["Interpreter Agent"]
        end

        subgraph Tools["Tools & Sandbox"]
            T1["可重现沙箱 (Docker / 隔离环境)"]
            T2["一键启动实验"]
            T3["代码 / 搜索 / 数据工具"]
        end

        subgraph Obs["Observability"]
            O1["Transcripts (完整对话记录)"]
            O2["Failure Cases 聚合"]
            O3["Distributions / 统计分析"]
        end
    end

    Human -->|"激活 Skill / Prompt / Goal"| ControlPlane
    ControlPlane --> Execution
    Execution -->|"结果 + Log"| ControlPlane
    ControlPlane -->|"更新信念 + 规范"| Human
```

### 增强研究循环 (Tighten the Loop)

```mermaid
stateDiagram-v2
    [*] --> PickProblem: 用户输入 / Outcome Goal
    
    PickProblem --> RunExperiment: Orchestrator 拆解 Workflow
    RunExperiment --> WriteEverything: 子 Agent 执行
    WriteEverything --> StareAtOutputs: 自动 + 手动 Log
    StareAtOutputs --> UpdateAndIterate: Evaluator 聚类失败案例
    UpdateAndIterate --> PublicOrShare: 判断收敛
    
    UpdateAndIterate --> PickProblem: 自修复 / Ablation
    UpdateAndIterate --> RunExperiment: 继续迭代
    PublicOrShare --> [*]: 输出文档 / Tool
```

### Vivek 研究技能 → Harness 映射

```mermaid
flowchart LR
    subgraph Skills["Vivek 研究技能栈"]
        S1["Pick own problems"]
        S2["Upgrade inputs"]
        S3["Write everything"]
        S4["Tighten the loop"]
        S5["Stare at outputs"]
        S6["Wander on purpose"]
        S7["Find your people"]
        S8["Long game"]
    end

    subgraph Harness["Harness 实现"]
        H1["Outcome-Driven Workflow<br/>Goal → Experiments"]
        H2["分层上下文 + 跨领域注入"]
        H3["Append-only Log + 项目记忆"]
        H4["一键实验 + 可重现 Config"]
        H5["Failure Clustering + 阅读 Transcript"]
        H6["多角色 / 多模型路由"]
        H7["协作 Gate + 共享规范"]
        H8["持久化积累 + 规范复利"]
    end

    subgraph Tools["对应工具方案"]
        T1["Fabro DOT / Timovi Pipeline"]
        T2["阿里三层 + Trellis spec"]
        T3["Anthropic Session + Trellis"]
        T4["所有 Harness + Sandbox"]
        T5["Deterministic Evaluator"]
        T6["Timovi 虚拟团队"]
        T7["社区 Trellis / Fabro / Eigen"]
        T8["项目记忆 + Event Log 重放"]
    end

    S1 --> H1 --> T1
    S2 --> H2 --> T2
    S3 --> H3 --> T3
    S4 --> H4 --> T4
    S5 --> H5 --> T5
    S6 --> H6 --> T6
    S7 --> H7 --> T7
    S8 --> H8 --> T8
```

### Gate Keeper 详细架构 (G1-G8)

```mermaid
stateDiagram-v2
    [*] --> G1_InputValid: Agent 输出
    G1_InputValid --> G2_SyntaxCheck: 通过
    G1_InputValid --> Reject: 输入格式无效
    
    G2_SyntaxCheck --> G3_SemanticCheck: 语法正确
    G2_SyntaxCheck --> Reject: 语法错误
    
    G3_SemanticCheck --> G4_SecurityScan: 语义合理
    G3_SemanticCheck --> Reject: 语义异常
    
    G4_SecurityScan --> G5_QualityGate: 安全通过
    G4_SecurityScan --> Escalate: 安全风险
    
    G5_QualityGate --> G6_ConsistencyCheck: 质量达标
    G5_QualityGate --> AutoFix: 质量不足
    
    G6_ConsistencyCheck --> G7_DeterministicEval: 一致性通过
    G6_ConsistencyCheck --> Reject: 上下文冲突
    
    G7_DeterministicEval --> G8_HumanGate: 评分通过
    G7_DeterministicEval --> AutoFix: 评分不达标

    G8_HumanGate --> Approve: 人类确认
    G8_HumanGate --> Reject: 人类驳回

    AutoFix --> G1_InputValid: 自修复后重入
    Escalate --> [*]: 升级人工

    Approve --> [*]: 最终输出
    Reject --> [*]: 拒绝
```

---

## 具体 Log 模板

### Append-only Event Log 条目

```yaml
event_log_entry:
  version: "1.0"
  timestamp: "2026-06-12T14:30:00Z"
  session_id: "sess_abc123"
  
  # === 核心循环 ===
  
  # Step 1: 预测（训练 Taste 的关键）
  hypothesis:
    statement: "使用分层上下文注入可以减少 Agent 的幻觉率"
    predicted_outcome: "HallucinationRate >= 40% reduction on code generation tasks"
    confidence: 0.65  # 必须写数字，训练校准能力
    reasoning: "Anthropic 论文显示 3-layer context reduces ambiguity by 37%"
  
  # Step 2: 实验
  experiment:
    workflow_id: "wf_codegen_v3"
    config_hash: "sha256:a1b2c3d4..."
    runner: "Timovi Virtual Team"
    agents_used: ["Architect", "Engineer", "QA"]
    duration_ms: 45200
    input:
      task: "Generate REST API for user management"
      context_layers: ["global_api_spec", "task_requirements", "past_failures"]
  
  # Step 3: 结果
  result:
    status: "partial_success"  # success / partial_success / failure / unexpected
    metrics:
      hallucination_rate: 0.12  # 相对于预期 0.40，实际降幅 70%
      pass_rate: 0.85
      first_pass_correctness: 0.72
    artifacts:
      - "generated_code/user_api_v3.rs"
      - "test_results/user_api_e2e.json"
    failure_cases:
      - category: "type_mismatch"
        count: 3
        examples: ["user_id: String vs i64"]
      - category: "missing_edge_case"
        count: 2
        examples: ["paginated response 没有 limit 参数"]
  
  # Step 4: 信念更新（必须写，抵抗确认偏误）
  updated_belief:
    previous_belief: "分层上下文对幻觉帮助有限 (~20%)"
    new_belief: "三层上下文在高复杂度任务上有显著效果 (>50%)，但边缘案例仍需加强"
    delta: "upward_revision"  # upward / downward / unchanged / overturned
    evidence_weight: 0.8  # 这次实验的可信度
    remaining_uncertainty: "不知道在低复杂度任务上是否也有同样效果"
  
  # 元信息
  meta:
    tokens_used: 12800
    cost_usd: 0.64
    iteration: 3  # 这个实验的第几次重复
    ablated_components: ["relevance_filter"]  # 这次关了什么
    reviewed_by_human: true
    human_notes: "failure clustering 非常好，但边缘案例分类颗粒度不够细"
```

### 项目记忆目录结构

```
.trellis/                          # 项目记忆根
├── SPEC.md                        # 当前规范（随时更新）
├── ARCHITECTURE.md                # 架构图 + 决策记录
├── JOURNAL.md                     # 按时间序的思考日志
│
├── hypotheses/                    # 假设库
│   ├── 2026-06-10_context-effectiveness.md
│   └── 2026-06-12_backpressure-gates.md
│
├── failures/                      # 失败案例库（最重要的目录）
│   ├── clusters/                  # 按类别聚类的失败
│   │   ├── type_mismatch.md
│   │   ├── missing_edge_case.md
│   │   └── hallucination_context.md
│   └── raw/                       # 原始失败日志
│       └── session_sess_abc123.json
│
├── experiments/                   # 可重现实验配置
│   ├── v3.1.context_layers.yaml
│   └── v3.2.backpressure_gates.yaml
│
├── beliefs/                       # 信念演化追踪
│   └── context_layers.md          # 随时间变化的信念 + 证据链
│
└── artifacts/                     # 产出物
    └── impl_codegen_v3/
```

### Session 级别工作流 (researched_workflow.json)

```json
{
  "workflow_id": "wf_codegen_v3",
  "goal": "Reduce hallucination in code generation by ≥40%",
  "created": "2026-06-12T14:00:00Z",
  "experiments": [
    {
      "id": "exp_001",
      "hypothesis": "分层上下文减少幻觉",
      "comparison": ["flat_context", "three_layer_context"],
      "status": "completed",
      "result": "hallucination dropped 0.40 → 0.12"
    },
    {
      "id": "exp_002",
      "hypothesis": "Backpressure gates 进一步提高成功率",
      "dependency": ["exp_001"],
      "status": "running"
    }
  ],
  "decisions": [
    {
      "at": "2026-06-12T14:05:00Z",
      "what": "优先攻击 'type_mismatch' 堆",
      "why": "占失败案例的 60%，修复后总 pass_rate 估计+15%",
      "evidence": "failures/clusters/type_mismatch.md"
    }
  ],
  "learnings": [
    "三层上下文在高复杂度任务显著有效",
    "type_mismatch 可被自动 type-check gate 避免"
  ],
  "next": "exp_002: 叠加 backpressure gates"
}
```

---

## 关键设计原则 — 一句话版

| 原则 | 一句话 |
|------|--------|
| **研究=可训练技能栈** | 每次运行前写预测，运行后更新信念，每周回顾失败聚类 — 这就是 deliberate practice |
| **Harness=研究基础设施** | 好 Harness 不是"让 Agent 跑得更快"，是"让研究者更快发现自己错了" |
| **Failure 聚类>指标** | 指标告诉你"差多少"，聚类告诉你"差在哪" — 后者才是迭代的起点 |
| **信念必须可追溯** | 没有 Updated Belief 的实验等于没做 — 你不知道自己现在相信什么、为什么 |
| **Taste 可校准** | 预测 vs 实际的偏差追踪 = 你自己的 calibration curve |
| **工程=研究方法** | 构建 Eval、Pipeline、Harness 本身就是最高 ROI 的研究活动 |
