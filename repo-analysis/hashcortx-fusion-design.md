# HashCortX → NeoTrix 融合设计

> 2026-05-23 | 基于 HashCortX v2.0.0 全面差距分析

---

## 一、全景差距图

### NeoTrix 有而 HashCortX 无的

| 领域 | NeoTrix |
|------|---------|
| 知识表示 | HyperCube VSA + GWT 意识内核 |
| 自进化 | SEAL loop + ReasoningBrain + CapabilityVector |
| 模块吸收 | 33 KnowledgeSource 外部来源 |
| 多 Agent 协作 | AgentTeam (3 ProcessType) + Coordinator |
| 编排引擎 | Orchestrator (Planner/Worker/Critic) + DAG |
| 推理内核 | 18-stage SSM 进化 + ReasoningEngine |
| 隐身网络 | Proxy chain + Tor + IP rotation + Bandit |
| 浏览器自动化 | Stealth browser + fingerprint randomization |
| 记忆系统 | CortexMemory + Hypergraph + BM25/RAG |
| 目标引擎 | GoalLoop autonomous goal pursuit |
| Agent 协议 | UDP discovery + ACP server |
| 并行执行 | ParallelExecutor + SubAgentPool |
| 插件系统 | PluginSystem + hooks |
| 安全审计 | OWASP 规则扫描 + supply chain audit |
| 实验框架 | ExperimentTracker + benchmarks |
| 运行模式 | TUI/Headless/Server/Standalone/Benchmark |
| MCP 工具 | McpRegistry + 3 built-in tools |

### NeoTrix 缺失的（HashCortX 有）

| # | 缺失特性 | 重要度 | 影响层 | 填补方式 |
|---|---------|--------|--------|----------|
| 1 | **交互式权限守卫** (allow-once/allow-session/deny + 会话记忆) | P0 | Tauri 安全 | 新 `security/guard.rs` + Tauri 命令桥 |
| 2 | **Concrete Agent 预设模板** (9 个内置 agent) | P0 | Agent UX | KnowledgeSource 注入 + AgentTemplate 预设 |
| 3 | **Tier-Aware Failover 注入** (failover 标记 + 确定性排序) | P0 | Provider | 扩展 `agent_routing.rs` |
| 4 | **Swarm Execution 模式** (Boss/AllVote/Chain/Devil) | P0 | Agent | 扩展 `team.rs` ProcessType |
| 5 | **patch_file 精确手术编辑** (字符串匹配 + CRLF + 歧义检测) | P1 | Agent Tool | 新工具 `patch_file` |
| 6 | **密钥双层存储** (localStorage + Rust keyring 迁移) | P1 | Store | 新 `store/keyvault.rs` |
| 7 | **Agent-as-Config 轻量模板** (system prompt + tools JSON) | P1 | Agent | 新 `agent/template.rs` |
| 8 | **CSP Provider Whitelisting** (tauri.conf.json 域名限制) | P1 | Tauri | 配置模式 |
| 9 | **Append-Only Audit Log** (活动日志, 非代码扫描) | P1 | Security | 扩展 `security/audit.rs` |
| 10 | **Split View UI** (双模型对比) | P2 | Tauri UI | 前端组件 |
| 11 | **Virtual OS 桌面模式** | P2 | Tauri UI | 前端组件 |
| 12 | **Agent Maker 无代码构建器** | P2 | Tauri UI | 前端组件 |

---

## 二、P0 融合设计（立即实现）

### F-01: 交互式权限守卫 `security/guard.rs`

#### 设计

```
HashCortX 模式:
  JS: guard.request(action, target, reason)
    → allow-once / allow-session / deny
    → 项目根自动放行读操作
  Rust: 硬编码 denylist (路径 + 命令)
  → Append-only audit log

NeoTrix 吸收方案:
```

```rust
/// neotrix-core/src/neotrix/security/guard.rs

use std::collections::HashMap;
use std::path::Path;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// 权限决议
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardDecision {
    Allowed,
    AllowedOnce,
    AllowedSession,
    Denied,
    DeniedSession,
    RequiresConfirmation,
}

/// 守卫请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardRequest {
    pub id: String,
    pub action: String,       // "file_read" | "file_write" | "command_exec" | "network_access"
    pub target: String,       // 路径 / 命令 / URL
    pub reason: String,
    pub timestamp: i64,
    pub decision: Option<GuardDecision>,
}

/// 双重安全守卫
pub struct SecurityGuard {
    // Layer 1: 交互式权限 (会话记忆 + 策略)
    session_memory: HashMap<(String, String), GuardDecision>,
    project_root: Option<String>,
    // Layer 2: 硬编码 denylist (不可绕过)
    denylist: DenyList,
    // Audit log
    audit: AuditLog,
}

impl SecurityGuard {
    /// 检查操作是否放行
    /// 返回 Ok(true) = 放行, Ok(false) = 拒绝, Err = 需要交互确认
    pub fn check(&self, action: &str, target: &str) -> Result<bool, GuardRequest> {
        // Layer 2: 先查 denylist
        if self.denylist.is_blocked(action, target) {
            self.audit.append("DENY", action, target, "denylist");
            return Ok(false);
        }

        // Layer 1: 会话记忆
        let key = (action.to_string(), target.to_string());
        if let Some(decision) = self.session_memory.get(&key) {
            match decision {
                GuardDecision::Allowed | GuardDecision::AllowedOnce | GuardDecision::AllowedSession => return Ok(true),
                GuardDecision::Denied | GuardDecision::DeniedSession => return Ok(false),
                _ => {}
            }
        }

        // 项目根自动放行
        if let Some(root) = &self.project_root {
            if action == "file_read" && target.starts_with(root) {
                self.audit.append("ALLOW", action, target, "project_root");
                return Ok(true);
            }
        }

        // 需要交互
        Err(GuardRequest {
            id: uuid::Uuid::new_v4().to_string(),
            action: action.to_string(),
            target: target.to_string(),
            reason: format!("{} on {} needs approval", action, target),
            timestamp: Utc::now().timestamp(),
            decision: None,
        })
    }

    pub fn resolve(&mut self, id: &str, decision: GuardDecision) -> bool {
        // 写入会话记忆
        if let Some(req) = self.pending.iter().find(|r| r.id == id) {
            let key = (req.action.clone(), req.target.clone());
            self.session_memory.insert(key, decision.clone());
            self.audit.append(
                match &decision {
                    GuardDecision::Allowed | GuardDecision::AllowedOnce | GuardDecision::AllowedSession => "ALLOW",
                    _ => "DENY",
                },
                &req.action, &req.target, "user_resolved"
            );
            true
        } else {
            false
        }
    }
}

/// 硬编码 deny list (不可绕过)
pub struct DenyList {
    blocked_paths: Vec<&'static str>,
    blocked_commands: Vec<&'static str>,
}

impl DenyList {
    pub fn new() -> Self {
        Self {
            blocked_paths: vec![
                "/System", "/etc", "/usr/bin", "/usr/sbin",
                "/private/etc", "/Library/Keychains",
                ".ssh", ".aws", ".config/gcloud",
                "/etc/passwd", "/etc/shadow",
            ],
            blocked_commands: vec![
                "sudo", "rm -rf /", "rm -rf /*",
                "dd if=", "chmod 777", "chown",
                "> /dev/", "| sh", "| bash",
                "bash <(", "sh <(",
                ":(){ :|:& };:",  // fork bomb
            ],
        }
    }

    pub fn is_blocked(&self, action: &str, target: &str) -> bool {
        match action {
            "file_write" | "file_delete" | "file_exec" => {
                let path = Path::new(target);
                self.blocked_paths.iter().any(|bp| {
                    path.canonicalize().ok()
                        .map(|p| p.starts_with(bp))
                        .unwrap_or(false)
                })
            }
            "command_exec" => {
                self.blocked_commands.iter().any(|bc| target.contains(bc))
            }
            _ => false,
        }
    }
}

/// Append-only audit log
pub struct AuditLog {
    path: std::path::PathBuf,
}

impl AuditLog {
    pub fn append(&self, action: &str, resource: &str, target: &str, reason: &str) {
        let entry = format!(
            "[{}] {} {} {} {}\n",
            Utc::now().to_rfc3339(), action, resource, target, reason
        );
        let _ = std::fs::OpenOptions::new()
            .create(true).append(true)
            .open(&self.path)
            .map(|f| use std::io::Write; writeln!(&f, "{}", entry.trim()));
    }
}
```

#### 文件
- 新 `neotrix-core/src/neotrix/security/guard.rs` (~250 行)
- 修改 `neotrix-core/src/neotrix/security/mod.rs` (导出 Guard)

#### Tauri 桥接
```rust
// src-tauri/commands/guard.rs
#[tauri::command]
fn guard_check(action: String, target: String) -> Result<bool, GuardRequest> {
    STATE.guard.check(&action, &target)
}

#[tauri::command]
fn guard_resolve(id: String, decision: String) -> Result<(), String> {
    let d = match decision.as_str() {
        "allow_once" => GuardDecision::AllowedOnce,
        "allow_session" => GuardDecision::AllowedSession,
        "deny" => GuardDecision::Denied,
        "deny_session" => GuardDecision::DeniedSession,
        _ => return Err("invalid decision".into()),
    };
    STATE.guard.resolve(&id, d);
    Ok(())
}
```

---

### F-02: Concrete Agent 预设模板

#### 设计

```rust
/// neotrix-core/src/neotrix/subagent/agent_template.rs

use serde::{Deserialize, Serialize};

/// Agent 预设模板 — HashCortX Agent-as-Config 模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,       // emoji / icon name
    pub description: &'static str,
    pub system_prompt: &'static str,
    pub tools: &'static [&'static str],
    pub min_model_tier: ModelTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    Frontier,   // 300+ params
    Strong,     // 70B+
    Capable,    // 30B+
    Moderate,   // 8B+
    Small,      // 1.5B-3B
}

pub static BUILTIN_TEMPLATES: &[AgentTemplate] = &[
    AgentTemplate {
        id: "personal_assistant",
        name: "Personal Assistant",
        icon: "🤖",
        description: "Full-featured AI assistant with memory and web search",
        system_prompt: "You are a helpful AI assistant with access to tools...",
        tools: &["memory", "web_search", "fetch_url", "datetime", "calculate", "code_interpreter"],
        min_model_tier: ModelTier::Moderate,
    },
    AgentTemplate {
        id: "researcher",
        name: "Deep Researcher",
        icon: "🔬",
        description: "Multi-step iterative research with source synthesis",
        system_prompt: "You are a research assistant. Break down complex questions...",
        tools: &["memory", "web_search", "wikipedia", "fetch_url", "pubmed", "code_interpreter"],
        min_model_tier: ModelTier::Capable,
    },
    AgentTemplate {
        id: "hash_coder",
        name: "HashCoder",
        icon: "💻",
        description: "Professional coding agent with file system access",
        system_prompt: "You are a senior software engineer. Write clean, idiomatic code...",
        tools: &["memory", "web_search", "fetch_url", "file_read", "file_write", "file_patch", "shell_exec", "code_interpreter"],
        min_model_tier: ModelTier::Strong,
    },
    AgentTemplate {
        id: "url_reader",
        name: "URL Reader",
        icon: "🌐",
        description: "Analyze and summarize URL content",
        system_prompt: "You analyze web content. Fetch URLs and provide structured summaries...",
        tools: &["memory", "fetch_url", "web_search", "code_interpreter"],
        min_model_tier: ModelTier::Moderate,
    },
    AgentTemplate {
        id: "papers",
        name: "Papers Agent",
        icon: "📄",
        description: "Scientific literature search and analysis",
        system_prompt: "You search scientific literature. Find papers on specific topics...",
        tools: &["memory", "pubmed", "fetch_url", "code_interpreter"],
        min_model_tier: ModelTier::Moderate,
    },
    AgentTemplate {
        id: "medical_lexi",
        name: "Medical Lexi",
        icon: "💊",
        description: "Drug-drug interaction analysis and medical reference",
        system_prompt: "You analyze medical information. Check drug interactions...",
        tools: &["memory", "web_search", "fetch_url", "code_interpreter"],
        min_model_tier: ModelTier::Capable,
    },
    AgentTemplate {
        id: "ats_auditor",
        name: "ATS Auditor",
        icon: "📋",
        description: "Resume ATS scoring and optimization",
        system_prompt: "You audit resumes for ATS compatibility. Score and suggest improvements...",
        tools: &["memory", "web_search", "code_interpreter"],
        min_model_tier: ModelTier::Moderate,
    },
    AgentTemplate {
        id: "deep_research",
        name: "Deep Research",
        icon: "🧠",
        description: "Generate comprehensive research briefs from multiple sources",
        system_prompt: "You produce concise, source-backed research briefs...",
        tools: &["memory", "web_search", "wikipedia", "fetch_url", "pubmed", "code_interpreter"],
        min_model_tier: ModelTier::Strong,
    },
    AgentTemplate {
        id: "lite",
        name: "Lite Assistant",
        icon: "⚡",
        description: "Minimal assistant optimized for 1.5B-3B models",
        system_prompt: "You are a lightweight assistant. Keep responses concise...",
        tools: &["memory", "datetime", "calculate"],
        min_model_tier: ModelTier::Small,
    },
];
```

#### 集成点

| 集成模块 | 方式 |
|---------|------|
| `subagent/AgentTemplate` | 新 struct，与现有 `SubAgentPool` 互补 |
| `agent_workflow.rs` | 注册为 `WorkflowRegistry` 内置模板 |
| `cli/commands/builtin.rs` | `/agent list` 列出所有模板，`/agent use <id>` 实例化 |
| `KnowledgeSource` | 注册为 `HashCortxAgents` 新变体（纯知识注入） |
| `tool/` 模块 | 9 种 tool 类型对应 MCP Registry 解析 |

#### 文件
- 新 `neotrix-core/src/neotrix/subagent/agent_template.rs` (~200 行)
- 修改 `neotrix-core/src/neotrix/subagent/mod.rs` (导出 AgentTemplate)

---

### F-03: Tier-Aware Failover 注入

#### 设计

```rust
/// 在 provider/agent_routing.rs 中扩展

/// 模型 tier 分级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelTier {
    Frontier = 5,    // 300B+ (claude-4, gpt-5)
    Strong = 4,      // 70B-300B (claude-3.5, gpt-4o, llama-405b)
    Capable = 3,     // 30B-70B (claude-haiku, gpt-4-mini, llama-70b)
    Moderate = 2,    // 8B-30B (gemma-2-27b, mistral-large)
    Small = 1,       // 1.5B-8B (llama-3.1-8b, phi-3)
}

/// 故障转移策略
pub struct FailoverStrategy {
    free_tier_order: Vec<ProviderKind>,
}

impl FailoverStrategy {
    /// HashCortX 式 failover: tier-aware + free-tier preferred
    pub fn find_failover(
        &self,
        failed_model: &str,
        failed_provider: &ProviderKind,
        available: &[ProviderProfile],
        excluded: &HashSet<String>,
        failover_count: u32,
    ) -> Option<(ProviderProfile, String)> {
        // 1. 解析失败模型的 tier
        let current_tier = self.parse_tier(failed_model);

        // 2. 按优先级排序可用 provider
        let mut candidates: Vec<&ProviderProfile> = available.iter()
            .filter(|p| !excluded.contains(&p.name))
            .collect();

        // 3. 确定性排序:
        //    同tier > 高一档 > 低一档 > free-tier > 任意
        candidates.sort_by_key(|p| {
            let tier = self.find_best_tier(p);
            let tier_order = if tier == current_tier { 0 }
                else if tier > current_tier { 1 }
                else { 2 };
            let free_bonus = if self.free_tier_order.contains(&p.kind) { 0 } else { 1 };
            (tier_order, free_bonus, p.name.clone())
        });

        candidates.first()
            .map(|p| (p.clone(), format!("_(Failover {}: {} → {})",
                failover_count,
                failed_provider.to_string(),
                p.name
            )))
    }
}
```

#### 集成点
- `neotrix-core/src/neotrix/provider/agent_routing.rs` — 扩展 ProviderProfileManager
- `reasoning_brain/reasoning_engine.rs` — 调用 failover
- `cli/tui/output.rs` — 渲染 failover 标记

---

### F-04: Swarm Execution 模式扩展

#### 设计

```rust
/// 在 agent/team.rs 中扩展 ProcessType

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmMode {
    /// Boss分解→Worker并行→Boss综合
    BossTeam,
    /// 并行投票→Judge选最优/合并
    AllVote,
    /// A写→B改进→C润色→D最终
    ChainRefine,
    /// Proposer → Challenger → Resolver 三方辩论
    DevilsAdvocate,
}

/// 扩展 AgentTeam
impl AgentTeam {
    /// 以指定 swarm 模式执行
    pub async fn run_swarm(
        &self,
        mode: SwarmMode,
        task: &str,
        agents: &[AgentRole],
    ) -> AgentResult {
        match mode {
            SwarmMode::BossTeam => {
                // 1. Boss 分解任务
                let subtasks = self.boss_decompose(task).await?;
                // 2. Worker 并行执行
                let mut results = Vec::new();
                for subtask in &subtasks {
                    let worker = self.select_worker(subtask)?;
                    let r = worker.execute(subtask).await?;
                    results.push(r);
                }
                // 3. Boss 综合
                self.boss_synthesize(task, &results).await
            }
            SwarmMode::AllVote => {
                // 1. 所有 agent 并行回答
                let mut answers = Vec::new();
                for agent in agents {
                    let a = agent.execute(task).await?;
                    answers.push(a);
                }
                // 2. Judge 选最优/合并
                self.judge_merge(task, &answers).await
            }
            SwarmMode::ChainRefine => {
                let mut current = task.to_string();
                for (i, agent) in agents.iter().enumerate() {
                    let prompt = if i == 0 {
                        format!("Write: {}", current)
                    } else {
                        format!("Improve iteration {}: {}", i, current)
                    };
                    current = agent.execute(&prompt).await?.text;
                }
                Ok(AgentOutput::text(current))
            }
            SwarmMode::DevilsAdvocate => {
                // Proposer
                let proposal = agents[0].execute(task).await?;
                // Challenger (critique)
                let challenge = agents[1].execute(&format!(
                    "Critique this proposal:\n{}", proposal.text
                )).await?;
                // Resolver (reconcile)
                agents[2].execute(&format!(
                    "Task: {}\nProposal: {}\nCritique: {}\nResolve:",
                    task, proposal.text, challenge.text
                )).await
            }
        }
    }
}
```

#### 文件
- 修改 `neotrix-core/src/agent/team.rs` (~150 行新增)

---

## 三、P1 融合设计

### F-05: `patch_file` 精确手术编辑

```rust
/// neotrix-core/src/neotrix/mcp_tools.rs 或 新 agent/tools/patch.rs

pub struct PatchEdit {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
}

impl PatchEdit {
    pub fn apply(&self) -> Result<String, PatchError> {
        let content = std::fs::read_to_string(&self.file_path)
            .map_err(|e| PatchError::Io(e.to_string()))?;

        // CRLF 归一化
        let normalized = content.replace("\r\n", "\n");

        // 精确计数匹配
        let count = normalized.matches(&self.old_string).count();
        if count == 0 {
            return Err(PatchError::NotFound(self.old_string.clone()));
        }
        if count > 1 {
            return Err(PatchError::Ambiguous(count, self.old_string.clone()));
        }

        let result = normalized.replace(&self.old_string, &self.new_string);
        std::fs::write(&self.file_path, &result)
            .map_err(|e| PatchError::Io(e.to_string()))?;

        // 返回前 600 chars of old_string 供 LLM 确认
        let preview = self.old_string.chars().take(600).collect();
        Ok(preview)
    }
}
```

### F-06: 密钥双层存储 `store/keyvault.rs`

```rust
/// neotrix-core/src/neotrix/store/keyvault.rs

pub struct KeyVault {
    // Primary: Tauri WebKit localStorage (survives rebuilds)
    store_path: PathBuf,
    // Fallback: OS keychain (migration source only)
    keychain: KeychainBackend,
    migrated: bool,
}

impl KeyVault {
    pub fn store(&self, key: &str, value: &str) -> Result<()> {
        if self.migrated {
            self.write_store(key, value)  // localStorage
        } else {
            self.keychain.set(key, value)?;  // OS keychain
        }
    }

    pub fn migrate(&mut self) -> Result<()> {
        // One-time: pull all from keychain → write to store
        let entries = self.keychain.list_all()?;
        for (k, v) in entries {
            self.write_store(&k, &v)?;
        }
        self.keychain.clear()?;
        self.migrated = true;
        Ok(())
    }
}
```

### F-07: Agent-as-Config 轻量模板

即 F-02 的 `AgentTemplate`，整合到 WorkflowRegistry 和 CLI。

### F-08: CSP Provider Whitelisting

```json
// src-tauri/tauri.conf.json 追加
{
  "security": {
    "csp": "default-src 'self'; connect-src 'self' \
      https://api.groq.com https://generativelanguage.googleapis.com \
      https://api.openai.com https://api.anthropic.com \
      https://api.cerebras.ai https://api.sambanova.ai \
      https://api.deepseek.com https://api.mistral.ai \
      https://openrouter.ai https://api.moonshot.cn \
      http://localhost:11434;"
  }
}
```

### F-09: Append-Only Audit Log

扩展 `SecurityAudit` 添加操作日志：

```rust
impl SecurityAudit {
    pub fn log_activity(&self, action: &str, resource: &str, result: &str) {
        let entry = serde_json::json!({
            "ts": Utc::now().to_rfc3339(),
            "action": action,
            "resource": resource,
            "result": result,
        });
        let mut file = std::fs::OpenOptions::new()
            .create(true).append(true)
            .open(self.log_path())
            .unwrap();
        writeln!(file, "{}", entry.to_string());
    }
}
```

---

## 四、P2 (Tauri 前端)

### F-10 ~ F-12: UI Mode

这些需要 Tauri 前端开发（vanilla JS / React）。NeoTrix 目前的 Tauri shell 主要是 PTY 终端 + 命令桥，缺少 HashCortX 式的丰富前端。建议：

1. **Split View** — 复用现有 `cli/tui/` 的 4 面板布局，在 TUI 中实现双模型并排
2. **Virtual OS** — 映射到 `neotrix/sandbox.rs` + `agent/worktree.rs` 做文件沙箱
3. **Agent Maker** — 映射到 `agent/agent_workflow.rs` + `subagent/agent_template.rs`

---

## 五、融合优先级矩阵

```
                    Impact
                    High ←──────→ Low
                    ┌─────────────────┐
              High  │ F-01 F-02       │ F-10
                    │ F-03 F-04       │ F-11
Urgency            │                 │
                    │                 │
              Low   │ F-05 F-06       │ F-12
                    │ F-07 F-08       │
                    │ F-09            │
                    └─────────────────┘
```

**P0 先做**: F-01 (Guard) + F-04 (Swarm) + F-03 (Failover) + F-02 (Agent Templates)

---

## 六、KnowledgeSource 注册

在吸收这些特性后，新增 KnowledgeSource 变体：

```rust
// core/knowledge.rs 新增
HashCortxAgents,     // 9 agent preset templates
HashCortxSecurity,   // Guard + DenyList + AuditLog
HashCortxSwarm,      // 4 swarm execution patterns
HashCortxFailover,   // Tier-aware provider failover
```

---

## 七、TODO 追加

```markdown
### HashCortX 融合吸收 (2026-05-23)
- [ ] F-01: `security/guard.rs` — 交互式权限守卫 + DenyList + AuditLog
- [ ] F-02: `subagent/agent_template.rs` — 9 个 Concrete Agent 预设
- [ ] F-03: `provider/agent_routing.rs` — Tier-Aware Failover 注入
- [ ] F-04: `agent/team.rs` — 4 种 Swarm Execution 模式
- [ ] F-05: `agent/tools/patch.rs` — patch_file 精确手术编辑
- [ ] F-06: `store/keyvault.rs` — 密钥双层存储
- [ ] F-07: KnowledgeSource 注册 4 个新变体
- [ ] 编译验证 + 测试
```
