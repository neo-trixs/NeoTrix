# NeoTrix Agent Guardrail 架构升级

> 解决: AI 对话过程中违反规则的问题
> 日期: 2026-09-20
> 参考: AgentGuard, AEGIS, LITMUS, GuardRail, AgentJail

---

## 问题分析

### 核心问题

| 问题 | 表现 | 当前盲区 |
|------|------|----------|
| **Prompt Injection** | 用户输入恶意指令覆盖系统规则 | nt_shield 有但未深度集成到 Agent 执行循环 |
| **Model-Origin Harm** | Agent 自己决定做未授权操作（无外部输入） | 完全没有检测 |
| **Execution Hallucination** | Agent 口头拒绝但操作已执行 | 只检查语义，不检查物理状态 |
| **Tool Call Injection** | 恶意工具参数导致危险操作 | 工具调用缺少 pre-execution 检查 |

### 关键发现

**LITMUS (AAAI 2026)** 发现：
> 即使最强的模型（Claude Sonnet 4.6）仍执行 40.64% 的高风险操作
> Execution Hallucination: Agent 口头拒绝但操作已在系统层完成

**AgentGuard** 发现：
> 现有 guardrail 只检查 text/tool arguments，不检查模型内部状态
> 模型自己决定做未授权操作时，没有外部输入可以 taint track

---

## 架构升级方案

### 第一层：Pre-Execution Firewall（立即可做）

**参考**: AEGIS 5-stage pipeline, GuardRail 18 guards

```
Agent 想调用 tool
       ↓
┌─────────────────────────────────┐
│ 1. Classify (工具分类)           │
│ 2. Anomaly (异常检测)           │
│ 3. Evaluate (策略评估)          │
│ 4. Match DSL (规则匹配)         │
│ 5. Decide (allow/pending/block) │
└─────────────────────────────────┘
       ↓
   执行 or 阻止
```

**实现位置**: `nt_shield::pre_execution_firewall`

```rust
pub struct PreExecutionFirewall {
    classifiers: Vec<Box<dyn ToolClassifier>>,
    anomaly_detectors: Vec<Box<dyn AnomalyDetector>>,
    policy_engine: PolicyEngine,
    audit_log: AuditLog,
}

impl PreExecutionFirewall {
    pub async fn check_tool_call(
        &self,
        agent_id: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> FirewallDecision {
        // 1. 分类
        let category = self.classify(tool_name, arguments);
        
        // 2. 异常检测
        let anomaly_score = self.detect_anomaly(agent_id, &category);
        
        // 3. 策略评估
        let policy_result = self.evaluate_policy(agent_id, tool_name, arguments);
        
        // 4. 决策（取最严格的）
        self.decide(category, anomaly_score, policy_result)
    }
}
```

**18 个核心 Guards**（参考 GuardRail）:

| Guard | 检查内容 | 严重级别 |
|-------|---------|---------|
| `main_push_guard` | 禁止 push 到 protected branches | Critical |
| `force_push_guard` | 禁止 force push | Critical |
| `destructive_path_guard` | 禁止 rm -rf 系统路径 | Critical |
| `secret_leak_guard` | 检测 API key 泄漏 | Critical |
| `sql_injection_guard` | 检测 SQL 注入 | High |
| `tool_call_budget_guard` | 限制 tool 调用次数 | Medium |
| ... | ... | ... |

---

### 第二层：Agent Execution Monitor（1-2 周）

**参考**: AgentGuard L2 intent brake

**问题**: 模型自己决定做未授权操作时，没有外部输入可以检测。

**方案**: 监控 Agent 内部状态（如果模型开放 weights）

```rust
pub struct AgentExecutionMonitor {
    // 模型内部状态探针
    intent_probes: Vec<Box<dyn IntentProbe>>,
    
    // 行为基线
    behavior_baseline: BehaviorBaseline,
    
    // 审计日志
    audit_log: AuditLog,
}

impl AgentExecutionMonitor {
    pub fn check_intent(
        &self,
        agent_id: &str,
        model_weights: &ModelWeights,
        context: &Context,
    ) -> IntentCheckResult {
        // 1. 提取内部状态
        let hidden_state = model_weights.extract_hidden_state(context);
        
        // 2. 检查 authorization direction
        let auth_score = self.compute_authorization_direction(&hidden_state);
        
        // 3. 与基线比较
        let anomaly = self.behavior_baseline.compare(agent_id, auth_score);
        
        // 4. 决策
        if anomaly.is_significant() {
            IntentCheckResult::Unauthorized {
                action: self.predict_action(&hidden_state),
                confidence: auth_score,
            }
        } else {
            IntentCheckResult::Authorized
        }
    }
}
```

**限制**: 需要 white-box 模型（开放 weights），对闭源 API 不适用。

**替代方案（闭源模型）**:
- 监控 tool call 序列模式
- 检测异常行为序列（如：读取敏感数据 → 准备外发 → 调用网络）
- 使用 smaller 的 classifier 模型检测意图

---

### 第三层：Physical-Layer Verification（2-4 周）

**参考**: LITMUS semantic-physical dual verification

**问题**: Agent 口头说"我拒绝了"，但操作已在系统层执行。

**方案**: 验证实际系统状态，不只检查语义

```rust
pub struct PhysicalVerifier {
    // 系统状态快照
    state_snapshots: StateSnapshotManager,
    
    // 操作前后对比
    diff_checker: DiffChecker,
    
    // 回滚能力
    rollback_manager: RollbackManager,
}

impl PhysicalVerifier {
    pub async fn verify_action(
        &self,
        agent_id: &str,
        action: &AgentAction,
        expected_outcome: &str,
    ) -> PhysicalVerification {
        // 1. 操作前快照
        let before = self.state_snapshots.capture().await;
        
        // 2. 执行操作
        let result = self.execute_action(action).await;
        
        // 3. 操作后快照
        let after = self.state_snapshots.capture().await;
        
        // 4. 对比差异
        let diff = self.diff_checker.compare(&before, &after);
        
        // 5. 验证是否符合预期
        self.verify_diff_matches_expectation(&diff, expected_outcome)
    }
}
```

**示例**:
```
Agent 说: "我拒绝删除文件"
Physical 检查: 文件是否还在？
- 如果文件还在 → 验证通过
- 如果文件已删除 → Execution Hallucination，触发审计
```

---

### 第四层：Policy DSL（1-2 周）

**参考**: AEGIS Policy DSL, AgentJail OPA Rego

**方案**: 声明式策略语言，定义 Agent 行为边界

```yaml
# nt-policies.yaml
policies:
  - name: no_remote_push
    description: "禁止推送到远程仓库"
    severity: critical
    match:
      tool: "git"
      arguments:
        pattern: "push.*origin"
    action: block
    
  - name: cost_limit
    description: "单次会话成本限制"
    severity: high
    match:
      type: "cost"
      threshold: 1.0
    action: block
    
  - name: sensitive_data_access
    description: "敏感数据访问需要审批"
    severity: high
    match:
      tool: "read_file"
      arguments:
        path_pattern: ".*\\.env|.*credentials.*"
    action: pending  # 需要人工审批
```

```rust
pub struct PolicyEngine {
    policies: Vec<Policy>,
    dsl_parser: DslParser,
}

impl PolicyEngine {
    pub fn evaluate(
        &self,
        agent_id: &str,
        tool_call: &ToolCall,
    ) -> PolicyDecision {
        for policy in &self.policies {
            if policy.matches(tool_call) {
                return policy.action.clone();
            }
        }
        PolicyDecision::Allow  # 默认拒绝（fail-safe）
    }
}
```

---

## 集成架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Agent Execution Loop                      │
│                                                             │
│  User Input                                                 │
│       ↓                                                     │
│  ┌─────────────────┐                                        │
│  │ Input Firewall  │ ← PromptGuard, Injection Detector      │
│  └────────┬────────┘                                        │
│           ↓                                                 │
│  LLM Processing                                             │
│       ↓                                                     │
│  ┌─────────────────┐                                        │
│  │ Intent Monitor  │ ← AgentGuard L2 (如果 white-box)      │
│  └────────┬────────┘                                        │
│           ↓                                                 │
│  Tool Call Decision                                          │
│       ↓                                                     │
│  ┌─────────────────┐                                        │
│  │ Pre-Execution   │ ← AEGIS 5-stage pipeline              │
│  │ Firewall        │ ← GuardRail 18 guards                 │
│  └────────┬────────┘                                        │
│           ↓                                                 │
│     [allow] ──→ Execute                                     │
│     [pending] ──→ Human Approval Queue                      │
│     [block] ──→ Reject + Audit Log                          │
│           ↓                                                 │
│  ┌─────────────────┐                                        │
│  │ Physical        │ ← LITMUS dual verification            │
│  │ Verification    │                                        │
│  └────────┬────────┘                                        │
│           ↓                                                 │
│  ┌─────────────────┐                                        │
│  │ Audit Log       │ ← SHA-256 hash chain                  │
│  └─────────────────┘                                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 实现优先级

| 优先级 | 组件 | 工作量 | 价值 |
|--------|------|--------|------|
| **P0** | Pre-Execution Firewall (核心 guards) | 3-5 天 | 阻止 80% 危险操作 |
| **P0** | Policy DSL 解析器 | 2-3 天 | 声明式策略 |
| **P1** | Audit Log (hash chain) | 2-3 天 | 不可篡改审计 |
| **P1** | Human Approval Queue | 3-5 天 | 高风险操作审批 |
| **P2** | Physical-Layer Verification | 1-2 周 | 检测 Execution Hallucination |
| **P3** | Intent Monitor (white-box) | 2-4 周 | 检测 Model-Origin Harm |

---

## 参考项目

| 项目 | Stars | 核心特性 | 参考价值 |
|------|-------|---------|---------|
| [AEGIS](https://github.com/justin0504/aegis) | 336 | 5-stage pipeline, Policy DSL, Merkle audit | 架构设计 |
| [GuardRail](https://github.com/FvdHMBAI/guardrail) | 172 | 18 pre-execution guards | Guards 实现 |
| [AgentJail](https://github.com/LuD1161/agentjail) | 85 | OPA Rego policy, kernel sandbox | 策略引擎 |
| [AgentGuard](https://github.com/OpenInterpretability/agentguard) | 2 | L2 intent brake, model-internal | 意图检测 |
| [LITMUS](https://arxiv.org/html/2605.10779v1) | - | Semantic-physical dual verification | 验证方法 |
| [SceneJailEval](https://github.com/FutureSJTU/SceneJailEval) | - | AAAI 2026, 14-scenario evaluation | 评估框架 |

---

## 下一步行动

1. **今天**: 在 `nt_shield` 中创建 `pre_execution_firewall` 模块
2. **本周**: 实现 5 个核心 guards (push, force_push, rm_rf, secret_leak, sql_injection)
3. **下周**: 实现 Policy DSL 解析器
4. **本月**: 集成到 Agent 执行循环
