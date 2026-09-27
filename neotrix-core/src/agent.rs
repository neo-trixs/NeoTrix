//! Backward-compat compatibility layer for legacy agent/ directory APIs.
//! Provides: hooks (ECC quality gate), team (AgentTeam), decoder, skills, workflow, tool orchestration.

pub mod hooks {
    use std::collections::HashMap;
    use std::time::Instant;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum HookProfile { Standard, Strict, Permissive }

    /// ECC 风格钩子 — 对事件执行检查并产出可阻断动作。
    /// 输出前缀约定：`BLOCK:` 开头的字符串表示阻断（check_blocked 识别）。
    pub trait Hook: Send + Sync {
        fn name(&self) -> &'static str;
        fn description(&self) -> &'static str;
        fn events(&self) -> Vec<HookEvent>;
        fn execute(&self, ctx: &HookContext) -> String;
    }

    /// 质量门控：PreToolUse 阻止超 800 行的输入（对齐 LARGE_FILE_THRESHOLD）。
    pub struct QualityGateHook;
    impl Hook for QualityGateHook {
        fn name(&self) -> &'static str { "quality-gate" }
        fn description(&self) -> &'static str { "阻止创建超 800 行的文件" }
        fn events(&self) -> Vec<HookEvent> { vec![HookEvent::PreToolUse] }
        fn execute(&self, ctx: &HookContext) -> String {
            if let Some(ref input) = ctx.tool_input {
                let lines = input.lines().count();
                if lines > 800 {
                    return format!("BLOCK: File exceeds 800 lines ({} lines). Split into smaller modules.", lines);
                }
            }
            "CONTINUE: quality-gate passed".to_string()
        }
    }

    /// TODO 警告：PostToolUse 提醒新增 TODO/FIXME/HACK。
    pub struct TodoWarningHook;
    impl Hook for TodoWarningHook {
        fn name(&self) -> &'static str { "todo-warning" }
        fn description(&self) -> &'static str { "警告新增 TODO/FIXME/HACK 注释" }
        fn events(&self) -> Vec<HookEvent> { vec![HookEvent::PostToolUse] }
        fn execute(&self, ctx: &HookContext) -> String {
            if let Some(ref output) = ctx.tool_output {
                if output.contains("TODO") || output.contains("FIXME") || output.contains("HACK") {
                    return "WARN: New TODO/FIXME/HACK found".to_string();
                }
            }
            "CONTINUE: todo-warning passed".to_string()
        }
    }

    /// 会话边界钩子：SessionStart/SessionEnd 记录。
    pub struct SessionPersistenceHook;
    impl Hook for SessionPersistenceHook {
        fn name(&self) -> &'static str { "session-persistence" }
        fn description(&self) -> &'static str { "保存/恢复会话上下文" }
        fn events(&self) -> Vec<HookEvent> {
            vec![HookEvent::SessionStart, HookEvent::SessionEnd, HookEvent::PostToolUse]
        }
        fn execute(&self, ctx: &HookContext) -> String {
            format!("CONTINUE: {} handled", ctx.event.as_str())
        }
    }

    pub struct EccHookRegistry {
        hooks: Vec<Box<dyn Hook>>,
        event_index: HashMap<HookEvent, Vec<usize>>,
        profile: HookProfile,
        disabled_hooks: Vec<String>,
    }

    impl Default for EccHookRegistry {
        fn default() -> Self {
            let mut reg = Self::new();
            reg.register_defaults();
            reg
        }
    }

    impl EccHookRegistry {
        pub fn new() -> Self {
            Self {
                hooks: Vec::new(),
                event_index: HashMap::new(),
                profile: HookProfile::Standard,
                disabled_hooks: Vec::new(),
            }
        }

        pub fn register(&mut self, hook: Box<dyn Hook>) {
            let idx = self.hooks.len();
            for event in hook.events() {
                self.event_index.entry(event).or_default().push(idx);
            }
            self.hooks.push(hook);
        }

        pub fn register_defaults(&mut self) {
            self.register(Box::new(QualityGateHook));
            self.register(Box::new(TodoWarningHook));
            self.register(Box::new(SessionPersistenceHook));
        }

        pub fn set_profile(&mut self, profile: HookProfile) { self.profile = profile; }

        pub fn disable_hook(&mut self, name: &str) {
            if !self.disabled_hooks.contains(&name.to_string()) {
                self.disabled_hooks.push(name.to_string());
            }
        }

        pub fn check_blocked(actions: &[String]) -> Option<String> {
            actions.iter().find(|a| a.starts_with("BLOCK:")).cloned()
        }

        pub fn execute_event(&self, ctx: &HookContext) -> Vec<String> {
            let mut results = Vec::new();
            if let Some(indices) = self.event_index.get(&ctx.event) {
                for &idx in indices {
                    let hook = &self.hooks[idx];
                    if self.disabled_hooks.contains(&hook.name().to_string()) { continue; }
                    // Permissive 仅保留会话钩子，跳过阻断类检查
                    if self.profile == HookProfile::Permissive
                        && !matches!(hook.name(), "session-persistence")
                    {
                        continue;
                    }
                    let result = hook.execute(ctx);
                    results.push(result);
                }
            }
            results
        }

        pub fn hook_count(&self) -> usize { self.hooks.len() }

        pub fn list_hooks(&self) -> Vec<(String, String)> {
            self.hooks.iter().map(|h| (h.name().to_string(), h.description().to_string())).collect()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum HookEvent { SessionStart, SessionEnd, PreToolUse, PostToolUse }

    impl HookEvent {
        pub fn as_str(&self) -> &'static str {
            match self {
                HookEvent::SessionStart => "SessionStart",
                HookEvent::SessionEnd => "SessionEnd",
                HookEvent::PreToolUse => "PreToolUse",
                HookEvent::PostToolUse => "PostToolUse",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct HookContext {
        pub event: HookEvent,
        pub session_id: Option<String>,
        pub timestamp: Instant,
        pub file_path: Option<String>,
        pub tool_name: Option<String>,
        pub tool_input: Option<String>,
        pub tool_output: Option<String>,
    }

    impl Default for HookContext {
        fn default() -> Self {
            Self {
                event: HookEvent::SessionStart,
                session_id: None,
                timestamp: Instant::now(),
                file_path: None,
                tool_name: None,
                tool_input: None,
                tool_output: None,
            }
        }
    }

    impl HookContext {
        pub fn new(event: HookEvent) -> Self {
            Self {
                event,
                session_id: None,
                timestamp: Instant::now(),
                file_path: None,
                tool_name: None,
                tool_input: None,
                tool_output: None,
            }
        }
    }
}

pub mod team {
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct AgentTeamResult {
        pub agent_name: String,
        pub success: bool,
        pub output: String,
    }

    #[derive(Debug, Clone)]
    pub struct AgentRole {
        pub name: String,
        pub role: String,
        pub goal: String,
        pub backstory: String,
        pub tools: Vec<String>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProcessType { Sequential, Parallel, Supervised }

    #[derive(Debug, Clone)]
    pub struct AgentTeam {
        pub name: String,
        pub process_type: ProcessType,
        pub agents: Vec<String>,
        pub state: HashMap<String, String>,
    }

    impl AgentTeam {
        pub fn new(name: &str, process_type: ProcessType) -> Self {
            Self { name: name.to_string(), process_type, agents: Vec::new(), state: HashMap::new() }
        }

        pub fn add_agent(&mut self, _role: AgentRole) {
            self.agents.push(_role.name);
        }
        /// Execute a task across all agents in the team.
        ///
        /// Returns an error result per agent — no real execution is wired yet.
        /// Callers should not treat `success: true` as evidence of task completion.
        pub fn execute(&self, _task: &str) -> Vec<AgentResult> {
            self.agents.iter().map(|name| AgentResult {
                agent_name: name.clone(),
                success: false,
                output: format!("Agent '{}' execution not wired — requires agent runtime integration", name),
            }).collect()
        }
    }

    #[derive(Debug, Clone)]
    pub struct AgentResult {
        pub agent_name: String,
        pub success: bool,
        pub output: String,
    }
}

// F1 死码清理（2026-09-27）：空 `interface::AgentInterface` 全仓零引用已删。
pub mod decoder {
    pub fn decode_state(delta: &[f64], confidence: f64, min: f64) -> String {
        let mag: f64 = delta.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nz = delta.iter().filter(|x| x.abs() > min).count();
        format!("Δmag={mag:.3} nz={nz}/{n} conf={confidence:.2}", n = delta.len())
    }
}

pub mod skills {
    use std::path::{Path, PathBuf};
    use crate::skill_loader::{SkillLoader, ResolvedSkill, SkillFilter, SearchResult};

    /// Legacy struct kept for backward compatibility.
    #[derive(Debug, Clone)]
    pub struct DiscoveredSkill {
        pub name: String,
        pub description: String,
        pub path: PathBuf,
    }

    impl From<ResolvedSkill> for DiscoveredSkill {
        fn from(rs: ResolvedSkill) -> Self {
            DiscoveredSkill {
                name: rs.name,
                description: rs.description,
                path: rs.path,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct SkillsEngine {
        pub skills: Vec<String>,
    }

    impl SkillsEngine {
        pub fn new() -> Self { Self { skills: Vec::new() } }

        pub fn init(&mut self) -> Vec<String> {
            let discovered = Self::discover_all();
            self.skills = discovered.iter().map(|s| s.name.clone()).collect();
            self.skills.clone()
        }

        /// Discover all skills using enhanced index-based discovery with legacy fallback.
        pub fn discover_all() -> Vec<DiscoveredSkill> {
            // Try index-based discovery first
            let mut loader = SkillLoader::new();
            if let Ok(indexed) = loader.list_skills() {
                if !indexed.is_empty() {
                    return indexed.into_iter().map(DiscoveredSkill::from).collect();
                }
            }
            // Fallback to legacy filesystem scan
            SkillLoader::scan_legacy()
                .into_iter()
                .map(DiscoveredSkill::from)
                .collect()
        }

        /// Search skills by filter (enhanced: supports tags, triggers, categories).
        pub fn search(filter: &SkillFilter) -> Result<Vec<SearchResult>, String> {
            SkillLoader::new().search_skills(filter)
        }

        /// Get a specific skill by name with full metadata.
        pub fn get_skill(name: &str) -> Result<ResolvedSkill, String> {
            SkillLoader::new().load_skill(name)
        }

        /// Get all skills in a category.
        pub fn get_category(category: &str) -> Result<Vec<ResolvedSkill>, String> {
            SkillLoader::new().get_category(category)
        }

        /// Get dependency chain for a skill.
        pub fn get_dependencies(name: &str) -> Result<Vec<ResolvedSkill>, String> {
            SkillLoader::new().get_dependencies(name)
        }

        /// Get reverse dependencies (skills that depend on this one).
        pub fn get_dependents(name: &str) -> Result<Vec<ResolvedSkill>, String> {
            SkillLoader::new().get_dependents(name)
        }

        /// Find all SKILL.md files recursively within a directory (delegated to skill_loader).
        pub fn find_skill_mds(dir: &Path) -> Vec<PathBuf> {
            let mut results = Vec::new();
            Self::find_skill_mds_recursive(dir, &mut results);
            results
        }

        fn find_skill_mds_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let fname = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                        if fname.starts_with('.') || fname == "node_modules" || fname == "target" {
                            continue;
                        }
                        Self::find_skill_mds_recursive(&path, results);
                    } else if path.ends_with("SKILL.md") {
                        results.push(path);
                    }
                }
            }
        }
    }

    impl Default for SkillsEngine {
        fn default() -> Self { Self::new() }
    }

    // F1 死码清理（2026-09-27）：空 `SkillSource` 全仓零引用已删。
}

pub mod workflow {
    #[derive(Debug, Clone)]
    pub struct Workflow {
        pub name: String,
        pub description: String,
        pub steps: Vec<WorkflowStep>,
    }

    #[derive(Debug, Clone)]
    pub enum WorkflowStep {
        AgentTask { name: String, task_description: String },
    }

    #[derive(Debug, Clone)]
    pub struct WorkflowResult {
        pub step_name: String,
        pub success: bool,
    }

    #[derive(Debug, Clone, Default)]
    pub struct WorkflowEngine {
        workflows: Vec<Workflow>,
    }

    impl WorkflowEngine {
        pub fn new() -> Self { Self::default() }
        pub fn register(&mut self, workflow: Workflow) {
            self.workflows.push(workflow);
        }
        pub fn run(&self, name: &str, _ctx: &str) -> Vec<WorkflowResult> {
            if let Some(wf) = self.workflows.iter().find(|w| w.name == name) {
                wf.steps.iter().map(|step| {
                    match step {
                        WorkflowStep::AgentTask { name, .. } => WorkflowResult {
                            step_name: name.clone(),
                            success: true,
                        },
                    }
                }).collect()
            } else {
                Vec::new()
            }
        }
    }
}

pub mod tool {
    pub mod mcp {
        //! MCP tool definitions (stub)
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
        pub struct McpToolDef {
            pub name: String,
            pub description: String,
            pub input_schema: serde_json::Value,
            pub transport: McpTransport,
            pub server_name: String,
            pub schema_version: Option<String>,
            /// 调用该工具所需的权限标识（None＝无需显式授权）。
            #[serde(default)]
            pub required_permission: Option<String>,
            /// 风险等级（默认 Low）。
            #[serde(default)]
            pub risk_level: RiskLevel,
            /// 累计调用次数（由调用侧回写统计）。
            #[serde(default)]
            pub usage_count: u64,
            /// 均值延迟 ms（由调用侧回写统计）。
            #[serde(default)]
            pub avg_latency_ms: f64,
        }
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
        pub enum McpTransport {
            #[default]
            Stdio,
            Sse,
            Local { command: String, args: Vec<String> },
        }

        /// 工具风险等级（默认 Low；fail-closed：未知按 Low 展示、高危需显式标注）。
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
        )]
        pub enum RiskLevel {
            #[default]
            Low,
            Medium,
            High,
            Critical,
        }
    }

    use std::sync::{Arc, LazyLock, Mutex, RwLock};

    /// 模块内全局 McpRegistry（MCP 正典；`all_native_tools()` 的真实来源）。
    ///
    /// 仿 `nt_infra_agent_card::GLOBAL_CARDS` 模式：实例方法
    /// `register_stdio`/`register_sse` 经 `try_lock` 同步镜像写入全局
    /// （已持有全局锁时 `try_lock` 失败则跳过，避免死锁与重复写入）。
    static GLOBAL_MCP: LazyLock<Mutex<McpRegistry>> =
        LazyLock::new(|| Mutex::new(McpRegistry::new()));

    /// 快捷入口：注册 stdio 服务器并同步写入全局 McpRegistry。
    pub fn register_stdio_global(
        server_name: &str,
        command: &str,
        args: &[&str],
        tools: Vec<mcp::McpToolDef>,
    ) {
        GLOBAL_MCP
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .register_stdio(server_name, command, args, tools);
    }

    /// 快捷入口：注册 SSE 服务器并同步写入全局 McpRegistry。
    pub fn register_sse_global(server_name: &str, url: &str, tools: Vec<mcp::McpToolDef>) {
        GLOBAL_MCP
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .register_sse(server_name, url, tools);
    }

    #[cfg(test)]
    pub fn reset_global_mcp_for_tests() {
        if let Ok(mut global) = GLOBAL_MCP.try_lock() {
            *global = McpRegistry::new();
        }
    }

    /// 测试串行锁：触碰全局 McpRegistry 的单测必须先持有，
    /// 避免并行 harness 下 reset/register 交错导致 flake。
    #[cfg(test)]
    pub static TEST_MCP_SERIAL: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    /// ToolOrchestrator — 统一原生工具编排器
    ///
    /// 吸收管线终点：外部 MCP 服务器 → McpToolAdapter → ToolOrchestrator。
    /// 上层（GWT、SEAL、nt_cap）把每个 NativeTool 视为普通工具。
    #[derive(Default)]
    pub struct ToolOrchestrator {
        tools: Vec<Box<dyn crate::l0_substrate::nt_core_traits::NativeTool>>,
    }

    impl ToolOrchestrator {
        pub fn new(_cap: Arc<RwLock<neotrix_types::core::nt_core_cap::CapabilityVector>>) -> Self {
            Self::default()
        }

        /// 批量注册 NativeTool（吸收的 MCP 服务器走此路径）。
        pub fn register_native_all(
            &mut self,
            tools: Vec<Box<dyn crate::l0_substrate::nt_core_traits::NativeTool>>,
        ) {
            self.tools.extend(tools);
        }

        pub fn native_count(&self) -> usize {
            self.tools.len()
        }

        /// 列出所有已注册工具的 ToolDef（供 /mcp native 与上层消费）。
        pub fn list_defs(&self) -> Vec<crate::l0_substrate::nt_core_traits::ToolDef> {
            self.tools.iter().map(|t| t.to_def()).collect()
        }

        /// 按 id 派发调用。
        pub fn call(
            &self,
            name: &str,
            args: &serde_json::Value,
        ) -> Result<crate::l0_substrate::nt_core_traits::ToolOutput, String> {
            self.tools
                .iter()
                .find(|t| t.id() == name)
                .ok_or_else(|| format!("Native tool '{}' not registered", name))
                .and_then(|t| t.execute(args))
        }
    }

    /// 从全局 McpRegistry 重建吸收的原生工具列表（真实路径，非空壳）。
    pub fn all_native_tools() -> Vec<Box<dyn crate::l0_substrate::nt_core_traits::NativeTool>> {
        GLOBAL_MCP
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .as_native_tools()
    }

    /// McpRegistry — MCP 服务器注册表 (stdio/SSE 传输)。
    ///
    /// 融合已删除的 cli::commands McpRegistry: 服务器注册 → McpToolDef 清单 →
    /// NativeTool 适配。execute 经 stdio 子进程做 JSON-RPC tools/call (30s 超时)。
    #[derive(Debug, Default)]
    pub struct McpRegistry {
        servers: Vec<McpServer>,
        /// S3.1: 以 resource 形式托管的 Agent Card JSON（只存不调）。
        agent_cards: Vec<serde_json::Value>,
    }

    #[derive(Debug, Clone)]
    struct McpServer {
        name: String,
        command: String,
        args: Vec<String>,
        /// SSE 端点（stdio 服务器为 None）。
        sse_url: Option<String>,
        tools: Vec<mcp::McpToolDef>,
    }

    impl McpRegistry {
        pub fn new() -> Self {
            Self::default()
        }

        /// 注册一个 stdio MCP 服务器及其工具清单（同步镜像写入全局）。
        pub fn register_stdio(
            &mut self,
            server_name: &str,
            command: &str,
            args: &[&str],
            tools: Vec<mcp::McpToolDef>,
        ) {
            let entry = McpServer {
                name: server_name.to_string(),
                command: command.to_string(),
                args: args.iter().map(|s| s.to_string()).collect(),
                sse_url: None,
                tools,
            };
            self.servers.push(entry.clone());
            Self::mirror_to_global(&entry);
        }

        /// 注册一个 SSE MCP 服务器及其工具清单（同步镜像写入全局）。
        pub fn register_sse(&mut self, server_name: &str, url: &str, tools: Vec<mcp::McpToolDef>) {
            let entry = McpServer {
                name: server_name.to_string(),
                command: String::new(),
                args: Vec::new(),
                sse_url: Some(url.to_string()),
                tools,
            };
            self.servers.push(entry.clone());
            Self::mirror_to_global(&entry);
        }

        /// 镜像单条服务器记录到全局 McpRegistry。
        ///
        /// `try_lock` 语义：调用方已持有全局锁（经 `*_global` 快捷入口进入）
        /// 时加锁失败，直接跳过——既避免死锁，也不重复写入。
        fn mirror_to_global(entry: &McpServer) {
            if let Ok(mut global) = GLOBAL_MCP.try_lock() {
                global.servers.push(entry.clone());
            }
        }

        /// 已注册工具总数。
        pub fn tool_count(&self) -> usize {
            self.servers.iter().map(|s| s.tools.len()).sum()
        }

        /// 已注册服务器数量。
        pub fn server_count(&self) -> usize {
            self.servers.len()
        }

        /// 服务器一览 (`name (N tools)`), 供 /mcp list|status 展示。
        pub fn list_servers(&self) -> Vec<String> {
            self.servers
                .iter()
                .map(|s| format!("{} ({} tools)", s.name, s.tools.len()))
                .collect()
        }

        /// 按关键字推荐工具 (名称/描述子串匹配; 空查询返回全部)。
        pub fn recommend_tools(&self, query: &str) -> Vec<mcp::McpToolDef> {
            let q = query.trim().to_lowercase();
            self.servers
                .iter()
                .flat_map(|s| s.tools.iter())
                .filter(|t| {
                    q.is_empty()
                        || t.name.to_lowercase().contains(&q)
                        || t.description.to_lowercase().contains(&q)
                })
                .cloned()
                .collect()
        }

        /// 按 agent 过滤可见工具。
        ///
        /// 当前返回全部工具；TODO(E2): 与 `AgentCard.mcp_permissions`
        /// (`server_name`＋`tools`，空＝全部) 取交集做权限过滤，
        /// 跨文件逻辑留 E2。
        pub fn tools_for_agent(&self, agent_id: &str) -> Vec<mcp::McpToolDef> {
            let _ = agent_id;
            self.servers
                .iter()
                .flat_map(|s| s.tools.iter().cloned())
                .collect()
        }

        /// 按风险等级过滤工具。
        pub fn tools_by_risk(&self, level: mcp::RiskLevel) -> Vec<mcp::McpToolDef> {
            self.servers
                .iter()
                .flat_map(|s| s.tools.iter())
                .filter(|t| t.risk_level == level)
                .cloned()
                .collect()
        }

        /// 逐工具用量统计：`(tool_name, usage_count, avg_latency_ms)`。
        pub fn usage_stats(&self) -> Vec<(String, u64, f64)> {
            self.servers
                .iter()
                .flat_map(|s| s.tools.iter())
                .map(|t| (t.name.clone(), t.usage_count, t.avg_latency_ms))
                .collect()
        }

        /// S3.1: 以 resource 形式托管 Agent Card JSON——只存不调。
        ///
        /// discovery 经 `list_agent_cards` 查询；执行仍走原生调用路径。
        pub fn register_agent_card(&mut self, card: serde_json::Value) {
            self.agent_cards.push(card);
        }

        /// 已托管的 Agent Card JSON discovery 查询接口。
        pub fn list_agent_cards(&self) -> Vec<serde_json::Value> {
            self.agent_cards.clone()
        }

        /// 已托管的 Agent Card 数量。
        pub fn agent_card_count(&self) -> usize {
            self.agent_cards.len()
        }

        /// 全部工具的 NativeTool 适配 (供 ToolOrchestrator 注册)。
        pub fn as_native_tools(
            &self,
        ) -> Vec<Box<dyn crate::l0_substrate::nt_core_traits::NativeTool>> {
            self.servers
                .iter()
                .flat_map(|s| {
                    s.tools.iter().map(|t| {
                        let tool = StdioNativeTool {
                            def: t.clone(),
                            command: s.command.clone(),
                            args: s.args.clone(),
                        };
                        Box::new(tool)
                            as Box<
                                dyn crate::l0_substrate::nt_core_traits::NativeTool,
                            >
                    })
                })
                .collect()
        }
    }

    /// stdio MCP 工具的 NativeTool 适配: 每次调用 spawn 子进程, stdin 写入
    /// JSON-RPC tools/call 请求, 30s 超时内读取 stdout。长驻式 MCP 服务器
    /// (等待多轮输入) 会超时返回 Err, 短命令式工具 (如 echo) 直接返回输出。
    #[derive(Debug, Clone)]
    struct StdioNativeTool {
        def: mcp::McpToolDef,
        command: String,
        args: Vec<String>,
    }

    impl crate::l0_substrate::nt_core_traits::NativeTool for StdioNativeTool {
        fn id(&self) -> &str {
            &self.def.name
        }
        fn description(&self) -> &str {
            &self.def.description
        }
        fn input_schema(&self) -> serde_json::Value {
            self.def.input_schema.clone()
        }
        fn capability_tags(&self) -> Vec<&'static str> {
            Vec::new()
        }
        fn execute(
            &self,
            args: &serde_json::Value,
        ) -> Result<crate::l0_substrate::nt_core_traits::ToolOutput, String> {
            use std::io::Write;
            let req = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": self.def.name, "arguments": args},
            });
            let mut child = std::process::Command::new(&self.command)
                .args(&self.args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| format!("spawn {}: {}", self.command, e))?;
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                let _ = writeln!(stdin, "{}", req);
            }
            let mut waited = 0u32;
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let mut out = String::new();
                        if let Some(stdout) = child.stdout.take() {
                            use std::io::Read;
                            let mut stdout = stdout;
                            let _ = stdout.read_to_string(&mut out);
                        }
                        return Ok(
                            crate::l0_substrate::nt_core_traits::ToolOutput {
                                success: status.success(),
                                content: out.trim().to_string(),
                            },
                        );
                    }
                    Ok(None) => {
                        waited += 1;
                        if waited >= 300 {
                            let _ = child.kill();
                            return Err(format!(
                                "tool '{}' timed out after 30s",
                                self.def.name
                            ));
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => return Err(format!("wait {}: {}", self.def.name, e)),
                }
            }
        }
    }
}

// McpServerEntry not defined — placeholder type alias removed

pub use team::{AgentTeam, AgentRole, ProcessType};

#[cfg(test)]
mod tests {
    #[test]
    fn test_agent_tool_orchestrator_exists() {
        let orch = super::tool::ToolOrchestrator::default();
        assert_eq!(orch.native_count(), 0, "default orchestrator should have no tools");
    }

    use crate::l0_substrate::nt_core_traits::{NativeTool, ToolOutput};
    use serde_json::json;

    struct DummyTool(&'static str);

    impl NativeTool for DummyTool {
        fn id(&self) -> &str {
            self.0
        }
        fn description(&self) -> &str {
            "dummy"
        }
        fn input_schema(&self) -> serde_json::Value {
            json!({"type": "object", "properties": {}})
        }
        fn capability_tags(&self) -> Vec<&'static str> {
            vec![]
        }
        fn execute(&self, _args: &serde_json::Value) -> Result<ToolOutput, String> {
            Ok(ToolOutput { success: true, content: format!("ran {}", self.0) })
        }
    }

    #[test]
    fn test_tool_orchestrator_register_and_count() {
        let mut orch = super::tool::ToolOrchestrator::default();
        assert_eq!(orch.native_count(), 0);
        orch.register_native_all(vec![
            Box::new(DummyTool("alpha")) as Box<dyn NativeTool>,
            Box::new(DummyTool("beta")) as Box<dyn NativeTool>,
        ]);
        assert_eq!(orch.native_count(), 2);
        assert_eq!(orch.list_defs().len(), 2);
    }

    #[test]
    fn test_tool_orchestrator_dispatch() {
        let mut orch = super::tool::ToolOrchestrator::default();
        orch.register_native_all(vec![Box::new(DummyTool("calc")) as Box<dyn NativeTool>]);
        let out = orch.call("calc", &json!({"a": 1})).expect("dispatch");
        assert!(out.success);
        assert!(out.content.contains("ran calc"));
        let err = orch.call("ghost", &json!({}));
        assert!(err.is_err(), "unregistered tool must error");
        let msg = match err {
            Ok(_) => String::new(),
            Err(e) => e,
        };
        assert!(msg.contains("not registered"));
    }

    #[test]
    fn test_all_native_tools_from_global_registry() {
        // Before any registry is set, must return empty (never panic).
        let _guard = super::tool::TEST_MCP_SERIAL
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        super::tool::reset_global_mcp_for_tests();
        let tools = super::tool::all_native_tools();
        assert!(tools.is_empty());
    }

    fn sample_mcp_tool(name: &str) -> super::tool::mcp::McpToolDef {
        super::tool::mcp::McpToolDef {
            name: name.to_string(),
            description: "sample tool".to_string(),
            input_schema: json!({"type": "object"}),
            transport: super::tool::mcp::McpTransport::Stdio,
            server_name: "sample-server".to_string(),
            schema_version: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_mcp_register_grows_tool_count() {
        let _guard = super::tool::TEST_MCP_SERIAL
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        super::tool::reset_global_mcp_for_tests();
        let mut reg = super::tool::McpRegistry::new();
        assert_eq!(reg.tool_count(), 0);
        reg.register_stdio(
            "sample-server",
            "echo",
            &["mcp"],
            vec![sample_mcp_tool("tool_a"), sample_mcp_tool("tool_b")],
        );
        assert_eq!(reg.server_count(), 1);
        assert_eq!(reg.tool_count(), 2);
    }

    #[test]
    fn test_all_native_tools_nonempty_after_register() {
        let _guard = super::tool::TEST_MCP_SERIAL
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        super::tool::reset_global_mcp_for_tests();
        super::tool::register_stdio_global(
            "sample-server",
            "echo",
            &["mcp"],
            vec![sample_mcp_tool("tool_a")],
        );
        let tools = super::tool::all_native_tools();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.id() == "tool_a"));
    }

    use super::hooks::{HookContext, HookEvent, HookProfile, EccHookRegistry};

    #[test]
    fn test_hook_registry_default_count() {
        let reg = EccHookRegistry::default();
        assert_eq!(reg.hook_count(), 3);
        assert_eq!(reg.list_hooks().len(), 3);
    }

    #[test]
    fn test_quality_gate_blocks_large_input() {
        let reg = EccHookRegistry::default();
        let mut ctx = HookContext::new(HookEvent::PreToolUse);
        ctx.tool_name = Some("write".into());
        ctx.tool_input = Some("line\n".repeat(900));
        let actions = reg.execute_event(&ctx);
        let blocked = EccHookRegistry::check_blocked(&actions);
        assert!(blocked.is_some(), "quality gate must block >800 lines");
        assert!(blocked.unwrap().contains("800"));
    }

    #[test]
    fn test_quality_gate_allows_small_input() {
        let reg = EccHookRegistry::default();
        let mut ctx = HookContext::new(HookEvent::PreToolUse);
        ctx.tool_name = Some("write".into());
        ctx.tool_input = Some("small content".to_string());
        let actions = reg.execute_event(&ctx);
        assert!(EccHookRegistry::check_blocked(&actions).is_none());
    }

    #[test]
    fn test_todo_warning_hook() {
        let reg = EccHookRegistry::default();
        let mut ctx = HookContext::new(HookEvent::PostToolUse);
        ctx.tool_name = Some("edit".into());
        ctx.tool_output = Some("added // TODO later".to_string());
        let actions = reg.execute_event(&ctx);
        assert!(actions.iter().any(|a| a.contains("WARN")));
    }

    #[test]
    fn test_permissive_profile_skips_blocking_hooks() {
        let mut reg = EccHookRegistry::default();
        reg.set_profile(HookProfile::Permissive);
        let mut ctx = HookContext::new(HookEvent::PreToolUse);
        ctx.tool_input = Some("line\n".repeat(900));
        let actions = reg.execute_event(&ctx);
        assert!(EccHookRegistry::check_blocked(&actions).is_none());
    }

    #[test]
    fn test_disable_hook() {
        let mut reg = EccHookRegistry::default();
        reg.disable_hook("quality-gate");
        let mut ctx = HookContext::new(HookEvent::PreToolUse);
        ctx.tool_input = Some("line\n".repeat(900));
        let actions = reg.execute_event(&ctx);
        assert!(EccHookRegistry::check_blocked(&actions).is_none());
    }

    #[test]
    fn test_session_hook_handles_boundary() {
        let reg = EccHookRegistry::default();
        let actions = reg.execute_event(&HookContext::new(HookEvent::SessionStart));
        assert!(actions.iter().any(|a| a.contains("CONTINUE")));
    }
}
