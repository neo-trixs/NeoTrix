# 融合设计：how-to-train-your-gpt + codex-plusplus → NeoTrix

> 2026-05-28 | 来源: raiyanyahya/how-to-train-your-gpt (2.1K⭐) + b-nnett/codex-plusplus (2.4K⭐)

---

## 1. 全景差距图

| 维度 | how-to-train-your-gpt | codex-plusplus | NeoTrix 现状 | 差距 |
|------|----------------------|----------------|-------------|------|
| **教学清晰度** | 100% 代码注释 WHAT/WHY | — | 注释稀疏 | ❌ |
| **渐进披露推理** | 类比→数学→代码→图 | — | 一次性 dump | ❌ |
| **插件/工具生命周期** | — | `start(api)` / `stop()` | 工具注册无生命周期 | ❌ |
| **权限声明模型** | — | manifest `permissions` 数组 | 无权限模型 | ❌ |
| **沙箱文件系统** | — | 每 tweak `tweak-data/<id>/` | 无 per-tool 沙箱 | ❌ |
| **热重载迭代** | — | 存 tweak 目录, Cmd+R 刷新 | 需重启 | ❌ |
| **MCP per-plugin** | — | manifest 可声明 MCP server | MCP 集中注册 | ❌ |
| **更新安全性** | MIT, 鼓励 fork | advisory-only, 不自动安装 | 无更新机制 | ❌ |
| **叙事式 Walkthrough** | "A Token's Journey" 全文追踪 | — | 无推理可视化 | ❌ |
| **比对表决策辅助** | 23+ 技术对比表 | — | 无决策比对 | ❌ |

## 2. 优先级矩阵

| ID | 特性 | Impact (1-5) | Urgency (1-5) | 来源 | 推荐 Phase |
|----|------|-------------|---------------|------|-----------|
| **G1** | 推理 trace 增加 WHAT/WHY 注释 | 4 | 4 | gpt-book | **P1** |
| **G2** | 渐进披露推理输出 (类比→细节→代码) | 3 | 3 | gpt-book | P2 |
| **G3** | ReasoningBank 增加"解释器"知识条目 | 2 | 2 | gpt-book | P3 |
| **C1** | Agent 工具生命周期: `start(api)` / `stop()` | 5 | 4 | codexpp | **P1** |
| **C2** | Per-tool 沙箱文件系统 | 4 | 3 | codexpp | P2 |
| **C3** | Agent 工具权限声明 + 验证 | 5 | 4 | codexpp | **P1** |
| **C4** | MCP per-tool 声明 (manifest 嵌入) | 3 | 2 | codexpp | P3 |
| **C5** | 工具热重载 (监 control tweaks/ 目录) | 3 | 2 | codexpp | P2 |
| **C6** | Advisory-only 更新模型 | 2 | 1 | codexpp | P3 |

## 3. 深度融合方案

### G1: 推理 Trace 增加 WHAT/WHY 注释

**问题**: NeoTrix 的 `ReasoningTrace` 记录推理步骤，但缺少 WHY 解释。

**设计**:
```
// 当前
TraceStep { action: "query_llm", input: "...", output: "..." }

// 目标
TraceStep {
  action: "query_llm",
  what: "Distill raw LLM response into structured capability update",
  why: "LLM output is unconstrained; we must normalize it into CapabilityVector space before absorb() to prevent dimension pollution",
  framework: "The variance argument: raw LLM scores have arbitrary scale, normalize() brings variance to 1.0",
  input: "...",
  output: "...",
  alternatives: ["Skip normalization → ability vector collapse", "Clamp without normalize → loss of relative signal"],
  decision: "normalize() chosen because it preserves relative ranking while bounding magnitude",
}
```

**对接点**: `reasoning_brain/reasoning_engine.rs` → `ReasoningTrace` struct

**实现**: 添加 `what: String`, `why: String`, `framework: Option<String>`, `alternatives: Vec<String>`, `decision: String` 到 `TraceStep`。在 `SelfIteratingBrain::run_seal_loop()` 中各步骤填充。

**测试**: 验证 trace 序列化后包含 WHY 字段，用户可 /trace 查看

---

### C1: Agent 工具生命周期

**问题**: 当前工具 (`mcp_tools.rs`) 是纯函数注册，无 `start(api)` / `stop()` 生命周期。

**设计**:
```rust
// 借鉴 codex++ 的 Tweak 生命周期
pub trait AgentTool: Send + Sync {
    fn id(&self) -> &str;
    fn manifest(&self) -> ToolManifest;
    fn start(&mut self, api: ToolApi) -> Result<()>;
    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput>;
    fn stop(&mut self) -> Result<()>;
}

pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub version: String,       // semver
    pub permissions: Vec<ToolPermission>,
    pub mcp: Option<McpServerDecl>,
    pub min_runtime: String,
}

pub struct ToolApi {
    pub storage: Box<dyn ToolStorage>,  // per-tool sandboxed KV
    pub fs: Box<dyn ToolFs>,            // per-tool sandboxed filesystem
    pub ipc: Box<dyn ToolIpc>,          // namespaced IPC
    pub log: Box<dyn ToolLogger>,
}
```

**对接点**: `agent/tools/mod.rs` → 注册 `register_builtin()` 改为 `register_tool(tool: Box<dyn AgentTool>)`

**实现**:
1. 创建 `agent/tool/lifecycle.rs` — `AgentTool` trait + `ToolManifest`
2. 创建 `agent/tool/sandbox.rs` — per-tool 文件沙箱 + KV 存储
3. 创建 `agent/tool/permissions.rs` — `ToolPermission` enum + 验证
4. 迁移现有工具到新生命周期模式
5. `cargo check --lib` 门控

---

### C3: Agent 工具权限声明 + 验证

**问题**: 当前任何工具可调用任何操作，无权限门控。

**设计**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolPermission {
    Network,              // 可访问网络
    FileSystem,           // 可读写文件
    Shell,                // 可执行 shell 命令
    SystemConfig,         // 可修改系统配置
    UserData,             // 可访问用户数据
    McpServer,            // 可启动 MCP server
}

pub struct ToolPermissionSet(HashSet<ToolPermission>);

impl ToolPermissionSet {
    pub fn verify(&self, required: &[ToolPermission]) -> Result<()> {
        for perm in required {
            if !self.0.contains(perm) {
                return Err(PermissionDenied {
                    tool_id: "...",
                    missing: perm.clone(),
                });
            }
        }
        Ok(())
    }
}
```

**对接点**: `security/permissions.rs` — 已有 Permission 基础设施，扩展 `Permission::Tool(ToolPermission)`

**实现**:
1. 添加 `ToolPermission` 枚举到 `security/permissions.rs`
2. 创建 `agent/tool/permissions.rs` — `ToolPermissionSet` + `verify()`
3. `Orchestrator` 执行工具前调用 `verify()`
4. 拒绝未经授权的操作返回 `PermissionDenied` 错误

---

## 4. 实现计划

### Phase 1 (当前 session)

| 任务 | 文件 | 预计 LOC | 依赖 |
|------|------|---------|------|
| **C1+G3**: AgentTool trait + 生命周期 | `agent/tool/lifecycle.rs` | 150 | 无 |
| **C3**: ToolPermission enum + verify | `agent/tool/permissions.rs` | 80 | 无 |
| **C1**: ToolManifest 结构 | `agent/tool/lifecycle.rs` | 50 | 无 |
| 迁移 3 个现有工具到新模式 | `agent/tools/` | 200 | 上面 |
| **G1**: TraceStep 增加 WHAT/WHY | `reasoning_engine.rs` | 100 | 无 |

### Phase 2 (下一 session)

| 任务 | 文件 | 预计 LOC | 依赖 |
|------|------|---------|------|
| **C2**: Per-tool 文件沙箱 | `agent/tool/sandbox.rs` | 200 | C1 |
| **C5**: 工具热重载 | `agent/tool/watcher.rs` | 150 | C1 |
| **G2**: 渐进披露推理输出 | `reasoning_brain/renderer.rs` | 120 | G1 |

### Phase 3 (季度内)

| 任务 | 文件 | 预计 LOC | 依赖 |
|------|------|---------|------|
| **C4**: MCP per-tool 声明 | `agent/tool/lifecycle.rs` | 80 | C1 |
| **C6**: Advisory-only 更新模型 | `agent/tool/updater.rs` | 150 | C1 |
| **G3**: ReasoningBank 解释器条目 | `core/knowledge/sources.rs` | 60 | 无 |

---

## 5. KnowledgeSource 注册

| 来源 | 核心能力 | 维度映射 | Provenance |
|------|---------|---------|------------|
| `GptBook` | pedagogical_clarity: 0.95, progressive_disclosure: 0.90, annotated_reasoning: 0.92 | learning {clear_teaching, trace_annotation} | how-to-train-your-gpt |
| `CodexPlusPlus` | tool_lifecycle: 0.95, permission_model: 0.93, sandbox_storage: 0.90, hot_reload: 0.85 | agent {lifecycle, sandbox, permissions} | codex-plusplus |

---

*最终更新: 2026-05-28 | G 系列 3 特性 + C 系列 6 特性 | 优先级: C1+C3+G1 → Phase 1*
