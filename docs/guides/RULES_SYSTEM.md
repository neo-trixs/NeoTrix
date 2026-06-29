# NeoTrix Rules System — 完善计划

**目标**: 所有规则必须有阻断机制，进化迭代（SEAL + MAPPA + BES）在规则下执行。

---

## 现状：定义 vs 执行的 Gap

```
已定义 (17 组件)  →  已接入执行 (6)  →  有阻断 (6)
                   ↘ 未接入 (11)     →  零阻断
```

## 完善方案：ShieldEnforcer + ProjectLaws

### 1. 创建统一执行入口 `ShieldEnforcer`

新文件: `cli/shield_enforcer.rs`

```rust
pub struct ShieldEnforcer {
    sandbox: SandboxEnforcer,
    guard: SecurityGuard,       // 3 层 DenyList + session + audit
    policy: ActionPolicy,       // 4 个 profile
    guardrails: GuardrailSystem, // 6 维护栏
    approval: ApprovalEngine,   // 3 种审批模式
    laws: ProjectLaws,          // 项目法则
}
```

**检查链** (短路优先):
```
SecurityGuard::check()       → DENY     → 立即阻断
ActionPolicy::evaluate()     → DENY     → 立即阻断  
GuardrailSystem::validate()  → BLOCKED  → 立即阻断
SandboxEnforcer::check()     → READONLY → 阻断写入
ApprovalEngine::check()      → ASK      → 等待批准
ProjectLaws::check()         → VIOLATION→ 记录/警告/阻断
```

### 2. 创建 `ProjectLaws` — 代码化的项目法则

新文件: `cli/laws.rs`

| 法则 ID | 来源 | 阻断级别 | 检查内容 |
|---|---|---|---|
| L001 | AGENTS.md | ERROR | 新模块必须带 `nt_` 前缀 |
| L002 | AGENTS.md | ERROR | 新模块必须注册 mod.rs |
| L003 | AGENTS.md | ERROR | 不允许 unsafe 代码 |
| L004 | AGENTS.md | WARN | 不允许 .unwrap() 在生产代码 |
| L005 | AGENTS.md | ERROR | 不允许提交密钥 |
| L006 | SOUL.md | WARN | 不允许代码注释（Agent 生成规则） |
| L007 | SOUL.md | WARN | 不允许空泛安慰语 |
| L008 | SOUL.md | ERROR | 报错必须带 file:line |
| L009 | AGENTS.md | WARN | 浮点数用 .max(0.0).min(1.0) 代替 .clamp() |
| L010 | AGENTS.md | WARN | VecDeque::windows() 禁用，用 .as_slices() |

`ProjectLaws::check(path: &str, content: &str) -> Vec<LawViolation>`

### 3. 接入执行点

| 接入点 | 位置 | 检查内容 |
|---|---|---|
| CLI 命令派发 | `types.rs:120 execute()` | Guard + Policy + Guardrails + Sandbox + Laws |
| 文件写入 | `file_cmds.rs` | Guard (DenyList) + Policy (write_file) + Laws |
| 命令执行 | `types.rs:125` | Guard (blocked_cmds) + Policy (exec_cmd) + Guardrails |
| SEAL 管道执行 | `pipeline.rs BrainPipeline::execute()` | Policy + Guardrails + Laws (每 stage) |
| LLM 调用 | `engine_core.rs` | Guardrails (max_input/output) + Laws |
| MCP 工具 | `mcp_tools.rs` | ToolPermission + Policy |
| 网络请求 | Provider 调用链 | Policy (network_request) + Guard (blocked_domains) |
| git 操作 | `/git` 命令 | Sandbox + Approval + Laws |

### 4. CI 集成

| 检查 | 时机 | 阻断 |
|---|---|---|
| `ProjectLaws::check()` | pre-commit hook | 阻塞提交 |
| `cargo check --deny warnings` | CI | 阻塞合并 |
| `cargo audit` | CI | 阻塞合并 |
| `gitleaks` | CI | 阻塞合并 |
| `nt_shield_audit::scan_directory()` | CI | 阻塞合并 |

### 5. 进化迭代执行规则

所有 P0-P3 进化迭代（MAPPA、BES、AdaCoM 等）必须：

1. **PR 前**: `ProjectLaws::check_all()` 通过
2. **每次 stage 执行**: `ShieldEnforcer::check_stage()` 通过
3. **文件操作**: 走 `file_cmds.rs` 审批链而非直接 fs::write
4. **命令执行**: 走 `types.rs` 派发而非直接 Command::new
5. **LLM 调用**: 必须通过 PromptGuard + Guardrails
6. **密钥/凭证**: 必须通过 Vault/KeyVault，不得硬编码

---

## 执行顺序（纳入统一重构流程）

| Phase | 改动 | 涉及文件 |
|---|---|---|
| **4a** | 创建 `ShieldEnforcer` + `ProjectLaws` | `cli/shield_enforcer.rs` (新) + `cli/laws.rs` (新) |
| **4b** | Wire into CLI dispatch | `types.rs` execute() 插入 ShieldEnforcer 链 |
| **4c** | Wire into file operations | `file_cmds.rs` 插入 Guard + Policy + Laws |
| **4d** | Wire into SEAL pipeline | `pipeline.rs` execute() 插入每 stage 检查 |
| **4e** | Wire into LLM/Provider calls | Provider 调用链插入 Guardrails + Laws |
| **4f** | Wire into MCP tools | `mcp_tools.rs` 插入 ToolPermission check |
| **4g** | CI pre-commit hook | 添加 `project-laws` 检查脚本 |
| **4h** | E2E test: `test_shield_blocks_dangerous_ops` | 验证所有阻断点 |
