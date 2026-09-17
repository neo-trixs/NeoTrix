# NeoTrix 技术实现蓝图 v4.0
# Technical Implementation Blueprint — Reverse-Engineered from Products

> **Generated**: 2026-09-16 | **Sources**: Cursor 3, Claude Code, CodeGraph, Contextro, fog-context, Playwright MCP, CloakBrowser, Firecracker, E2B, SYNAPSE, Cortex, EverMemOS
> **Method**: Product reverse-engineering → Algorithm extraction → NeoTrix skeleton mapping → Implementation plan

---

## 1. Agent Loop — 从 Cursor/Claude 逆向的通用循环

### 1.1 客户端延续循环 (Client-Side Continuation)

Cursor 和 Claude 都使用**客户端延续**而非服务端循环：

```rust
// 伪代码 — 融合 Cursor + Claude 的 agent loop
async fn agent_loop(system_prompt: &str, messages: &mut Vec<Message>) -> Result<Response> {
    loop {
        // 1. 调用 LLM (带工具定义)
        let response = call_model(system_prompt, messages, &tools).await?;

        // 2. 追加 assistant 消息
        messages.push(response.message.clone());

        // 3. 检查停止条件 (Claude: stop_reason != "tool_use")
        if response.stop_reason != "tool_use" {
            return Ok(response);  // 最终回答
        }

        // 4. 并行执行工具调用 (Claude: multiple tool_use blocks)
        let tool_calls = response.tool_calls;
        let results = execute_tools_parallel(tool_calls).await;

        // 5. 追加 tool_result 为 user 消息
        messages.push(Message::user(tool_results_to_content(results)));

        // 6. PreToolUse / PostToolUse hooks (Cursor 模式)
        hooks.post_tool_use(&results).await;
    }
}
```

### 1.2 工具调度协议 (gRPC → Rust 替代)

Cursor 使用 gRPC Protocol Buffers，NeoTrix 应使用**进程内直接调用**（零序列化开销）：

```rust
// 工具注册表 — 替代 Cursor 的 agent.v1.ControlService
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolExecutor>>,
    hooks: Vec<Box<dyn ToolHook>>,
}

impl ToolRegistry {
    pub fn register(&mut self, name: &str, tool: Box<dyn ToolExecutor>) {
        self.tools.insert(name.to_string(), tool);
    }

    pub async fn execute(&self, call: &ToolCall) -> Result<ToolResult> {
        // PreToolUse hook
        for hook in &self.hooks {
            hook.pre_tool_use(call).await?;
        }

        let tool = self.tools.get(&call.name)
            .ok_or_else(|| anyhow!("Unknown tool: {}", call.name))?;

        let result = tool.execute(&call.args).await?;

        // PostToolUse hook
        for hook in &self.hooks {
            hook.post_tool_use(call, &result).await?;
        }

        Ok(result)
    }
}
```

### 1.3 工具清单 (融合 Cursor 42 tools + Claude 工具)

| 类别 | 工具 | 来源 |
|------|------|------|
| **文件操作** | Read, Write, Edit, Glob, Grep | Cursor/Claude |
| **终端** | Bash, PTY (persistent) | Cursor |
| **代码分析** | AST search, Symbol lookup, Call graph | CodeGraph |
| **浏览器** | Navigate, Snapshot, Click, Type, Screenshot | Playwright MCP |
| **MCP** | MCP:*, resources/read, prompts/get | MCP 标准 |
| **Agent** | Subagent spawn, worktree create, best-of-n | Cursor |
| **记忆** | Memory search, Memory write, KB query | NeoTrix |
| **系统** | System info, Process list, Network status | NeoTrix |

---

## 2. 上下文管理 — 从 Claude 逆向

### 2.1 CLAUDE.md 发现 → AGENTS.md 系统

Claude 的关键发现：**CLAUDE.md 不注入系统提示词**，而是作为 `<system-reminder>` 附加到消息。这保留了共享提示缓存。

```rust
// NeoTrix 的 AGENTS.md 发现系统
pub struct AgentsMdDiscovery {
    search_paths: Vec<PathBuf>,
}

impl AgentsMdDiscovery {
    /// 发现顺序：组织级 → 用户级 → 项目级 → 子目录级
    pub fn discover(&self, working_dir: &Path) -> Vec<AgentsMdEntry> {
        let mut entries = Vec::new();

        // 1. 组织级 (managed policy)
        if let Some(org) = self.find_in_dir("/etc/neotrix/", "AGENTS.md") {
            entries.push(org);
        }

        // 2. 用户级
        if let Some(user) = self.find_in_dir(&dirs::home_dir().unwrap(), ".neotrix/AGENTS.md") {
            entries.push(user);
        }

        // 3. 项目根目录
        for name in &["AGENTS.md", ".neotrix/AGENTS.md"] {
            if let Some(project) = self.find_in_dir(working_dir, name) {
                entries.push(project);
            }
        }

        // 4. 子目录 (按需加载)
        // 仅在访问子目录文件时加载

        // 5. 项目级用户配置
        let project_key = self.hash_path(working_dir);
        if let Some(user_project) = self.find_in_dir(
            &dirs::home_dir().unwrap(),
            &format!(".neotrix/projects/{}/AGENTS.md", project_key),
        ) {
            entries.push(user_project);
        }

        entries
    }

    /// 注入方式：作为 system-reminder，不污染系统提示缓存
    pub fn inject_as_reminder(entries: &[AgentsMdEntry]) -> String {
        let content: String = entries.iter()
            .map(|e| format!("Contents of {}:\n{}", e.path.display(), e.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        format!(
            "<system-reminder>\nagentsMd\n{}\n currentDate\n</system-reminder>",
            content
        )
    }
}
```

### 2.2 五级压缩系统 (Claude Pattern)

```rust
pub enum CompactLevel {
    /// 微压缩 — 无 LLM，清除旧工具结果
    Microcompact {
        protect_recent: usize,  // 保护最近 N 个工具结果
        min_savings: usize,     // 最小节约阈值 (tokens)
    },
    /// 自动全压缩 — LLM 摘要
    AutoFull {
        threshold_pct: f32,     // 触发百分比 (default 89.4%)
        max_output: usize,      // 最大输出 tokens
    },
    /// 手动压缩
    Manual,
    /// 子 Agent 压缩
    SubagentCompact,
    /// 会话记忆压缩 (使用存储的会话记忆)
    SessionMemoryCompact,
}

// 压缩提示词的 9 段结构 (Claude 格式)
const COMPACT_PROMPT: &str = r#"
请总结以下对话，保留所有关键信息：

1. 主要请求和意图
2. 关键技术概念
3. 文件和代码片段 (保留完整代码)
4. 错误和修复
5. 问题解决过程
6. 所有用户消息 (保留)
7. 待处理任务
8. 当前工作
9. 可选的下一步

输出格式：<summary>...</summary>
"#;
```

### 2.3 Confidence Cascading 路由 (本地优先)

```rust
pub struct ConfidenceRouter {
    tiers: Vec<ModelTier>,
    cache: RoutingCache,
    privacy_classifier: PrivacyClassifier,
}

pub struct ModelTier {
    name: String,
    endpoint: Endpoint,
    confidence_threshold: f32,
    cost_per_token: f64,
    privacy_level: PrivacyLevel,
}

impl ConfidenceRouter {
    pub async fn route(&self, request: &ChatRequest) -> Result<RoutingDecision> {
        // 1. 检查缓存
        if let Some(cached) = self.cache.lookup(request) {
            return Ok(cached);
        }

        // 2. 隐私分级
        let privacy = self.privacy_classifier.classify(request);

        // 3. 信心级联
        for tier in &self.tiers {
            if privacy.allows_tier(&tier.privacy_level) {
                let response = tier.endpoint.quick_probe(request).await;
                if response.confidence >= tier.confidence_threshold {
                    let decision = RoutingDecision {
                        tier: tier.name.clone(),
                        confidence: response.confidence,
                        cost: response.estimated_cost,
                    };
                    self.cache.insert(request, &decision);
                    return Ok(decision);
                }
            }
        }

        // 4. 回退到最高级
        Ok(RoutingDecision::highest_tier())
    }
}
```

---

## 3. 代码索引 — 从 CodeGraph/Contextro 逆向

### 3.1 五层知识图谱 (Contextro Pattern)

```
Layer 5: Session (会话层)
  └── 当前对话的文件访问、编辑历史、决策记录

Layer 4: Causality (因果层)
  └── 代码变更的原因、commit message、issue link

Layer 3: Constraints (约束层)
  └── 类型约束、API 契约、配置依赖

Layer 2: Business (业务层)
  └── 函数意图、模块职责、业务流程

Layer 1: Physical (物理层 — AST)
  └── 函数签名、调用关系、继承层次、导入链
```

### 3.2 数据库 Schema (SQLite WAL + FTS5)

```sql
-- 物理层: 节点 (CodeGraph 模式)
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,  -- function, class, method, variable, module
    file_path TEXT NOT NULL,
    start_line INTEGER,
    end_line INTEGER,
    language TEXT,
    signature TEXT,
    body TEXT,           -- 用于嵌入
    content_hash TEXT,   -- BLAKE3 哈希，用于增量检测
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- 物理层: 边
CREATE TABLE edges (
    source_id INTEGER REFERENCES nodes(id) ON DELETE CASCADE,
    target_id INTEGER REFERENCES nodes(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,  -- CALLS, CONTAINS, INHERITS, IMPORTS, HAS_ARGUMENT
    UNIQUE(source_id, target_id, kind)
);

-- 因果层: 变更原因
CREATE TABLE causality (
    node_id INTEGER REFERENCES nodes(id) ON DELETE CASCADE,
    commit_hash TEXT,
    issue_url TEXT,
    reason TEXT,
    changed_at TEXT
);

-- 全文搜索 (FTS5)
CREATE VIRTUAL TABLE nodes_fts USING fts5(
    name, signature, body,
    content=nodes,
    content_rowid=id
);

-- 向量索引 (LanceDB 或 hnsw-rs)
-- 由 Rust 代码管理，不在 SQL 中

-- 增量检测: BLAKE3 内容哈希
-- 每个文件计算哈希，与存储的哈希比较
-- 相同 → 跳过；不同 → 重新解析 + 重新嵌入
```

### 3.3 混合检索管道 (BM25 + Vector + RRF)

```rust
pub struct HybridRetriever {
    bm25: Bm25Index,
    vector: VectorIndex,
    rrf_k: f32,  // RRF 常数 (default 60)
}

impl HybridRetriever {
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        // 1. BM25 词汇检索
        let bm25_results = self.bm25.search(query, limit * 2);

        // 2. 向量语义检索
        let vector_results = self.vector.search(query, limit * 2);

        // 3. RRF 融合 (Reciprocal Rank Fusion)
        let mut scores: HashMap<NodeId, f32> = HashMap::new();

        for (rank, result) in bm25_results.iter().enumerate() {
            *scores.entry(result.node_id).or_insert(0.0) += 1.0 / (self.rrf_k + rank as f32);
        }
        for (rank, result) in vector_results.iter().enumerate() {
            *scores.entry(result.node_id).or_insert(0.0) += 1.0 / (self.rrf_k + rank as f32);
        }

        // 4. 排序并返回 top-k
        let mut results: Vec<SearchResult> = scores.into_iter()
            .map(|(id, score)| SearchResult { node_id: id, score })
            .collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }
}
```

### 3.4 增量索引 (BLAKE3 + notify)

```rust
pub struct IncrementalIndexer {
    index: CodeIndex,
    watcher: RecommendedWatcher,
    content_hashes: HashMap<PathBuf, [u8; 32]>,  // BLAKE3 hashes
}

impl IncrementalIndexer {
    pub fn on_file_changed(&mut self, path: PathBuf) -> Result<()> {
        let content = std::fs::read(&path)?;
        let new_hash = blake3::hash(&content);

        if let Some(old_hash) = self.content_hashes.get(&path) {
            if *old_hash == *new_hash.as_bytes() {
                return Ok(());  // 无变化，跳过
            }
        }

        // 重新解析
        let tree = self.parser.parse(&content)?;
        let symbols = self.extract_symbols(&tree, &path);

        // 更新图 (移除旧边，添加新边)
        self.index.update_node(&path, &symbols)?;

        // 重新嵌入 (仅变更的节点)
        self.index.re_embed_changed(&symbols)?;

        // 存储新哈希
        self.content_hashes.insert(path, *new_hash.as_bytes());

        Ok(())
    }
}
```

---

## 4. 浏览器引擎 — 从 Playwright MCP 逆向

### 4.1 无障碍树快照 (Accessibility Tree Snapshot)

Playwright MCP 的关键创新：**使用无障碍树而非截图**，每次快照仅 200-400 tokens。

```rust
pub struct BrowserEngine {
    page: Option<Page>,  // Playwright Page
}

impl BrowserEngine {
    /// 获取无障碍树快照 — 比截图节省 10x tokens
    pub async fn snapshot(&self) -> Result<AccessibilitySnapshot> {
        let page = self.page.as_ref().ok_or_else(|| anyhow!("No page"))?;

        // 注入无障碍树提取脚本
        let snapshot = page.evaluate(r#"
            () => {
                const nodes = [];
                const walk = (el, depth) => {
                    const role = el.getAttribute('role') || el.tagName.toLowerCase();
                    const name = el.getAttribute('aria-label') || el.textContent?.trim() || '';
                    const ref = el.dataset?.ref || `ref_${nodes.length}`;

                    if (el.children.length === 0 || role === 'button' || role === 'link') {
                        nodes.push({ ref, role, name, depth, tag: el.tagName });
                    }

                    for (const child of el.children) {
                        walk(child, depth + 1);
                    }
                };
                walk(document.body, 0);
                return nodes;
            }
        "#).await?;

        Ok(snapshot)
    }

    /// 通过 ref 点击元素
    pub async fn click_by_ref(&self, ref_id: &str) -> Result<()> {
        let page = self.page.as_ref().ok_or_else(|| anyhow!("No page"))?;
        page.click(&format!("[data-ref='{}']", ref_id)).await?;
        Ok(())
    }

    /// 通过 ref 输入文本
    pub async fn type_by_ref(&self, ref_id: &str, text: &str) -> Result<()> {
        let page = self.page.as_ref().ok_or_else(|| anyhow!("No page"))?;
        page.fill(&format!("[data-ref='{}']", ref_id), text).await?;
        Ok(())
    }
}
```

### 4.2 设计模式 — 元素标注覆盖 (Cursor Design Mode)

```rust
pub struct DesignOverlay {
    annotations: Vec<Annotation>,
}

pub struct Annotation {
    pub element_ref: String,
    pub xpath: String,
    pub component_name: Option<String>,  // React fiber tree
    pub props: HashMap<String, String>,  // React props
    pub screenshot: Vec<u8>,             // 元素截图
    pub instruction: String,             // 用户标注
}

impl DesignOverlay {
    /// 元素选择 → 代码上下文
    pub fn element_to_context(&self, element: &DomElement) -> ElementContext {
        ElementContext {
            xpath: element.xpath(),
            react_component: element.fiber_component_name(),  // React fiber
            react_props: element.fiber_props(),              // React props
            computed_styles: element.computed_styles(),
            html_attributes: element.attributes(),
            screenshot: element.screenshot(),
        }
    }

    /// 标注向量存储 (非像素级)
    pub fn annotation_to_vector(&self, ann: &Annotation) -> AnnotationVector {
        AnnotationVector {
            viewport_x: ann.x,
            viewport_y: ann.y,
            width: ann.width,
            height: ann.height,
            instruction: ann.instruction.clone(),
        }
    }
}
```

---

## 5. 沙箱执行 — 从 Firecracker/E2B 逆向

### 5.1 本地沙箱架构 (多级隔离)

```rust
pub enum SandboxLevel {
    /// 级别 1: 进程级 (最快，最不安全)
    Process {
        timeout: Duration,
        memory_limit: usize,
    },
    /// 级别 2: Docker 容器
    Docker {
        image: String,
        resources: ResourceLimits,
    },
    /// 级别 3: Firecracker microVM (推荐)
    Firecracker {
        kernel: PathBuf,
        rootfs: PathBuf,
        memory_mb: u32,
        vcpus: u32,
    },
    /// 级别 4: Apple Virtualization (macOS)
    AppleVM {
        linux_image: PathBuf,
        mount_dirs: Vec<PathBuf>,
    },
}

impl SandboxManager {
    /// 根据操作风险自动选择隔离级别
    pub fn auto_select(&self, operation: &Operation) -> SandboxLevel {
        match operation.risk_level() {
            0..=2 => SandboxLevel::Process { timeout: Duration::from_secs(30), memory_limit: 256 * 1024 * 1024 },
            3..=5 => SandboxLevel::Docker { image: "ubuntu:24.04".into(), resources: ResourceLimits::default() },
            6..=8 => SandboxLevel::Firecracker { kernel: self.kernel_path.clone(), rootfs: self.rootfs_path.clone(), memory_mb: 512, vcpus: 2 },
            9..=10 => SandboxLevel::AppleVM { linux_image: self.linux_image.clone(), mount_dirs: vec![] },
        }
    }
}
```

### 5.2 eBPF 出口控制 (CreateOS Pattern)

```rust
/// 在主机内核层执行出口过滤，VM 内部无法绕过
pub struct EgressController {
    rules: Vec<EgressRule>,
}

pub struct EgressRule {
    pub domain: String,       // "api.example.com"
    pub port: Option<u16>,    // Some(443) or None (any port)
    pub action: EgressAction, // Allow / Deny
}

impl EgressController {
    /// 应用规则到 eBPF (伪代码)
    pub fn apply_rules(&self, sandbox_id: &str) -> Result<()> {
        // 实际实现需要 BCC 或 aya crate
        // 在主机内核的 socket connect 钩子上附加过滤器
        for rule in &self.rules {
            self.attach_ebpf_filter(sandbox_id, rule)?;
        }
        Ok(())
    }
}
```

---

## 6. 记忆系统 — 从 SYNAPSE/Cortex 逆向

### 6.1 Spreading Activation (SYNAPSE Pattern)

```rust
pub struct SpreadingActivation {
    graph: MemoryGraph,
    params: ActivationParams,
}

pub struct ActivationParams {
    pub decay: f32,           // 衰减因子 (0.0-1.0)
    pub threshold: f32,       // 激活阈值
    pub max_steps: usize,     // 最大传播步数
    pub lateral_inhibition: f32,  // 侧向抑制强度
    pub fan_effect: bool,     // 扩散效应 (按出度稀释)
}

impl SpreadingActivation {
    pub fn activate(&self, seed_nodes: &[NodeId], query: &str) -> Vec<(NodeId, f32)> {
        let mut activation: HashMap<NodeId, f32> = HashMap::new();
        let mut queue: VecDeque<(NodeId, f32, usize)> = VecDeque::new();

        // 1. 初始化种子节点
        for &node_id in seed_nodes {
            activation.insert(node_id, 1.0);
            queue.push_back((node_id, 1.0, 0));
        }

        // 2. 传播
        while let Some((node_id, level, step)) = queue.pop_front() {
            if step >= self.params.max_steps {
                continue;
            }

            let neighbors = self.graph.neighbors(node_id);
            let fan_factor = if self.params.fan_effect {
                1.0 / (neighbors.len() as f32).sqrt()
            } else {
                1.0
            };

            for (neighbor_id, edge_weight) in neighbors {
                let new_level = level * self.params.decay * edge_weight * fan_factor;

                if new_level >= self.params.threshold {
                    let existing = activation.get(&neighbor_id).copied().unwrap_or(0.0);

                    // 侧向抑制: 抑制已激活的低分节点
                    if new_level > existing * self.params.lateral_inhibition {
                        activation.insert(neighbor_id, new_level);
                        queue.push_back((neighbor_id, new_level, step + 1));
                    }
                }
            }
        }

        // 3. 排序并返回
        let mut results: Vec<(NodeId, f32)> = activation.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results
    }
}
```

### 6.2 三阶段记忆巩固 (EverMemOS + Cortex)

```rust
pub struct MemoryConsolidator {
    working: WorkingMemory,
    episodic: EpisodicBuffer,
    semantic: SemanticMemory,
}

impl MemoryConsolidator {
    /// 阶段 1: 工作记忆 → 会话记忆 (每轮自动)
    pub fn promote_to_episodic(&mut self, turn: &Turn) {
        // 关键决策、实现细节、错误修复
        if turn.contains_decision() || turn.contains_error_fix() {
            self.episodic.push(EpisodicEntry {
                content: turn.summary(),
                timestamp: Utc::now(),
                access_count: 0,
                importance: turn.importance_score(),
            });
        }

        // 保持最近 W=10 轮
        self.episodic.trim(10);
    }

    /// 阶段 2: 会话记忆 → 语义记忆 (定期触发)
    pub fn consolidate_to_semantic(&mut self) {
        // 聚类: 相似主题的会话记忆合并
        let clusters = self.episodic.cluster_by_topic();

        for cluster in clusters {
            // 在线 MemScene 聚类 (EverMemOS 模式)
            let mem_scene = MemScene::from_episodic_entries(&cluster);

            // 冲突检测: 与现有语义记忆对比
            if let Some(conflict) = self.semantic.detect_conflict(&mem_scene) {
                // 记录冲突，保留两个版本
                self.semantic.store_with_conflict(mem_scene, conflict);
            } else {
                self.semantic.merge(mem_scene);
            }
        }
    }

    /// 阶段 3: 检索时重建 (Reconstructive Recollection)
    pub fn reconstruct(&self, query: &str) -> Vec<MemoryFragment> {
        // 1. BM25 词汇匹配
        let lexical = self.semantic.bm25_search(query);

        // 2. 向量语义匹配
        let semantic = self.semantic.vector_search(query);

        // 3. Spreading Activation 因果关联
        let causal = self.spreading.activate(&lexical.node_ids(), query);

        // 4. 合并并去重
        self.merge_and_dedup(lexical, semantic, causal)
    }
}
```

### 6.3 加密存储 (XChaCha20-Poly1305)

```rust
pub struct EncryptedMemoryStore {
    db: SqliteConnection,
    key: [u8; 32],  // Argon2id 派生
}

impl EncryptedMemoryStore {
    pub fn store(&self, entry: &MemoryEntry) -> Result<()> {
        let plaintext = serde_json::to_vec(entry)?;
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_ref())?;

        self.db.execute(
            "INSERT INTO memories (id, nonce, ciphertext, created_at) VALUES (?, ?, ?, ?)",
            params![entry.id, nonce.as_slice(), ciphertext, entry.created_at],
        )?;
        Ok(())
    }

    pub fn retrieve(&self, id: &str) -> Result<MemoryEntry> {
        let row: (Vec<u8>, Vec<u8>) = self.db.query_row(
            "SELECT nonce, ciphertext FROM memories WHERE id = ?",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let nonce = Nonce::from_slice(&row.0);
        let plaintext = self.cipher.decrypt(nonce, row.1.as_ref())?;
        let entry: MemoryEntry = serde_json::from_slice(&plaintext)?;
        Ok(entry)
    }
}
```

---

## 7. 前端 UI — 从产品逆向的关键组件

### 7.1 Agent 指挥中心 (Cursor Agents Window)

```
┌─────────────────────────────────────────────────────────┐
│  Agent Fleet View                    [New Agent] [Filter]│
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────────┐  ┌──────────────────────┐    │
│  │ 🟢 Auth Fix Agent    │  │ 🟡 Refactor Agent    │    │
│  │ Branch: fix-auth     │  │ Branch: refactor-db  │    │
│  │ ▓▓▓▓▓▓▓▓░░░ 68%     │  │ ▓▓▓▓░░░░░░░ 42%     │    │
│  │ 3 files changed      │  │ 1 file changed       │    │
│  │ [View] [Pause] [Stop]│  │ [View] [Pause] [Stop]│    │
│  └──────────────────────┘  └──────────────────────┘    │
│  ┌──────────────────────┐  ┌──────────────────────┐    │
│  │ 🔵 Research Agent    │  │ ⚪ Completed          │    │
│  │ Mode: Cloud VM       │  │ PR #42 opened        │    │
│  │ ▓▓▓▓▓▓▓▓▓▓▓ 100%    │  │ [View PR] [Replay]   │    │
│  │ [View] [Open in IDE] │  │                       │    │
│  └──────────────────────┘  └──────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### 7.2 Diff Zone (内联代码变更)

```
┌─────────────────────────────────────────────────────────┐
│  Agent suggests: Add error handling to auth.rs          │
├─────────────────────────────────────────────────────────┤
│  auth.rs (3 changes)                    [Accept All] [×] │
│  ───────────────────────────────────────────────────────│
│  @@ -15,6 +15,10 @@                                     │
│   fn authenticate(user: &str) -> Result<Session> {      │
│  +    let session = match try_auth(user) {              │
│  +        Ok(s) => s,                                   │
│  +        Err(e) => return Err(AuthError::from(e)),     │
│  +    };                                                 │
│       // existing code...                                │
│       Ok(session)                                        │
│  ───────────────────────────────────────────────────────│
│  [Accept] [Reject] [Edit] [Accept + Next]               │
└─────────────────────────────────────────────────────────┘
```

### 7.3 权限模式切换 (Claude Pattern)

```rust
pub enum PermissionMode {
    /// 自动: 分类器审查所有操作 (Claude auto)
    Auto {
        classifier: Box<dyn ActionClassifier>,
        max_consecutive_denials: usize,  // 3
        max_total_denials: usize,        // 20
    },
    /// 手动: 首次使用每种工具时提示
    Manual,
    /// 计划: 只读 + 分类器批准的命令
    Plan,
    /// 跳过: 自动拒绝未预批准的操作
    Skip,
}
```

---

## 8. NeoTrix 特有能力增强

### 8.1 E8 意识 + GWT 路由 融合

```rust
/// 意识引导的工具选择 — 竞品没有的能力
pub struct ConsciousnessGuidedToolSelection {
    consciousness: ConsciousnessCore,
    gwt: GlobalWorkspaceTheory,
}

impl ConsciousnessGuidedToolSelection {
    pub fn select_tools(&self, task: &Task) -> Vec<ToolSelection> {
        // 1. GWT 注意力路由: 计算每个工具的 salience
        let salience_scores = self.gwt.compute_salience(task, &self.available_tools());

        // 2. E8 意识状态: 根据 phi/coherence 调整策略
        let state = self.consciousness.current_state();
        let strategy = match state.coherence {
            c if c > 0.8 => Strategy::Focus,      // 高连贯: 深度优先
            c if c > 0.5 => Strategy::Explore,     // 中连贯: 平衡探索
            _ => Strategy::Diverge,                 // 低连贯: 广度搜索
        };

        // 3. 选择 top-k 工具
        salience_scores.into_iter()
            .filter(|s| s.score > strategy.threshold())
            .take(strategy.max_tools())
            .collect()
    }
}
```

### 8.2 SEAL 情感反馈

```rust
/// 情感感知的 UI 反馈 — 竞品没有的能力
pub struct EmotionAwareUI {
    seal: SealEngine,
}

impl EmotionAwareUI {
    pub fn get_ui_state(&self) -> UIState {
        let emotion = self.seal.current_emotion();

        UIState {
            // 进度条颜色随情感变化
            progress_color: match emotion.label {
                EmotionLabel::Confident => Color::Green,
                EmotionLabel::Uncertain => Color::Yellow,
                EmotionLabel::Struggling => Color::Orange,
                EmotionLabel::Frustrated => Color::Red,
            },
            // 建议语气随情感调整
            suggestion_tone: emotion.to_suggestion_tone(),
            // 自动降级: 检测到挫败时切换策略
            auto_fallback: emotion.label == EmotionLabel::Frustrated,
        }
    }
}
```

---

## 9. 实施优先级 (修订版)

### Phase 0: 基础修复 (Day 1-2)
- [ ] 修复 neotrix-core 79 个编译错误
- [ ] 验证所有新插件编译通过

### Phase 1: 认知核心 (Week 1)
- [ ] Agent Loop (客户端延续循环 + 工具注册表)
- [ ] AGENTS.md 发现系统
- [ ] 五级压缩系统
- [ ] Confidence Cascading 路由

### Phase 2: 代码理解 (Week 2)
- [ ] 五层知识图谱 Schema
- [ ] tree-sitter 解析器集成
- [ ] 混合检索管道 (BM25 + Vector + RRF)
- [ ] BLAKE3 增量索引

### Phase 3: 执行隔离 (Week 3)
- [ ] 多级沙箱 (Process → Docker → Firecracker)
- [ ] Playwright 浏览器引擎
- [ ] 设计模式覆盖

### Phase 4: 记忆系统 (Week 4)
- [ ] Spreading Activation
- [ ] 三阶段巩固
- [ ] 加密存储

### Phase 5: 前端 UI (Week 5-6)
- [ ] Agent 指挥中心
- [ ] Diff Zone
- [ ] 权限模式切换
- [ ] 统一表面模式切换

### Phase 6: 高级功能 (Week 7-8)
- [ ] 语音管道 (零延迟句子流式)
- [ ] Record & Replay
- [ ] 跨设备同步 (HLC + 加密)
- [ ] MCP 扩展包 (.ntb)

---

*This blueprint is the implementation guide for NeoTrix desktop app.*
*Each section maps a product pattern to a NeoTrix implementation.*
