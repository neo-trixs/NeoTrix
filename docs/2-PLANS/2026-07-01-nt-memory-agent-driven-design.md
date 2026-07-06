# nt_memory_agent_driven — Agent-Driven Memory Management

**Blind Spot**: Memory management is passive — the KB is written to by pipeline stages (`ConversationDistillStage`, `KBIngester`), but the E8 reasoning agent itself has no agency over its own memory. It cannot decide what to remember, what to forget, or how to organize its knowledge.

**Sources**: Letta/MemGPT (23.6k★) — self-editing memory blocks via function calls, OS-inspired virtual context management. Mem0 (59.8k★) — three-stage extraction/update/retrieval pipeline with graph-based memory.

**Layer**: L3 Memory (sibling to nt_memory_kb), depends on L0–L2 only. Consumed by L4 E8 and L5 GWT.

---

## 1. Core Architecture

Three-tier memory with agent-driven lifecycle. The E8 agent calls `MemoryTool` functions (exposed as MCP tools) to read, write, consolidate, forget, and search its own memory. This mirrors Letta's core mechanism: "the agent manages its own context window via tool calls."

### 1.1 Memory Tiers

```
┌─────────────────────────────────────────────────────────┐
│  TIER 1: CORE MEMORY (always in context)                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────────┐ │
│  │ Persona   │ │ Human    │ │ Working  │ │ Custom       │ │
│  │ (agent ID)│ │ (user)   │ │ (current)│ │ (extensible) │ │
│  │ limit:2K  │ │ limit:2K │ │ limit:4K │ │ limit:var    │ │
│  └──────────┘ └──────────┘ └──────────┘ └─────────────┘ │
├─────────────────────────────────────────────────────────┤
│  TIER 2: ARCHIVAL MEMORY (vector DB, retrieved on demand)│
│  ┌──────────────────────────────────────────────────────┐│
│  │  SQLite + embeddings (nt_memory_kb)                  ││
│  │  ADD-only (never delete), provenance tracked         ││
│  │  Retrieved via semantic search + BM25 hybrid         ││
│  └──────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────┤
│  TIER 3: RECALL MEMORY (full conversation history)       │
│  ┌──────────────────────────────────────────────────────┐│
│  │  Complete message history, searchable by date/query  ││
│  │  Stored in KB as ConversationRecord                  ││
│  │  Used for distillation and pattern detection         ││
│  └──────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

### 1.2 Memory Blocks

```rust
// neotrix-core/src/neotrix/l3_memory_impl/nt_memory_agent_driven/mod.rs

/// A named, editable memory block injected into the system prompt as structured JSON.
/// Analogous to Letta's memory_blocks (label + value + limit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub name: String,              // "persona", "human", "working", "custom:<name>"
    pub content: String,           // JSON-serializable structured text
    pub limit: usize,              // max characters before auto-consolidation
    pub token_count: usize,        // estimated tokens (cached)
    pub is_editable: bool,         // agent can modify via MemoryWrite
    pub is_pinned: bool,           // always in context (core memory)
    pub last_modified: SystemTime,
    pub version: u64,              // monotonic version for conflict detection
}

impl MemoryBlock {
    pub fn new(name: &str, content: &str, limit: usize, editable: bool) -> Self {
        Self {
            name: name.to_string(),
            content: content.to_string(),
            limit,
            token_count: estimate_tokens(content),
            is_editable: editable,
            is_pinned: true,
            last_modified: SystemTime::now(),
            version: 1,
        }
    }

    /// Check if block has exceeded its token budget
    pub fn needs_consolidation(&self) -> bool {
        self.token_count > self.limit
    }
}
```

### 1.3 Agent Memory Tools

Exposed to the E8 LLM as MCP tools. The agent calls these during its reasoning loop to manage its own memory.

```rust
/// Tools exposed to the LLM (via MCP) for agent-driven memory management.
/// Each variant maps to a tool definition with JSON Schema arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MemoryTool {
    /// Read a memory block's content
    MemoryRead {
        name: String,
    },
    /// Replace the content of a memory block.
    /// If append=true, content is appended instead of replaced.
    MemoryWrite {
        name: String,
        content: String,
        append: bool,
    },
    /// Compress a memory block to fit within its limit.
    /// Uses an LLM call to summarize the content.
    MemoryConsolidate {
        name: String,
        target_tokens: Option<usize>, // default: block.limit
    },
    /// Remove specific facts from a memory block by ID or content pattern.
    MemoryForget {
        name: String,
        fact_ids: Vec<String>,
    },
    /// Semantic search over archival memory (nt_memory_kb).
    /// Returns top-k matching passages with relevance scores.
    MemorySearch {
        query: String,
        k: usize,
        scope: SearchScope, // Core, Archival, or All
    },
    /// Store a new fact in archival memory with provenance.
    MemoryInsert {
        content: String,
        tags: Vec<String>,
        importance: f64, // 0.0–1.0
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchScope {
    Core,      // search only core memory blocks
    Archival,  // search archival memory (vector DB)
    Recall,    // search conversation history
    All,       // search all tiers, rank by relevance
}
```

### 1.4 MemoryManager

```rust
/// Central manager for agent-driven memory.
/// Injects core blocks into the system prompt, processes tool calls,
/// and manages auto-consolidation.
pub struct MemoryManager {
    /// Core memory blocks (always in context)
    core_blocks: Vec<MemoryBlock>,
    /// Reference to the archival KB backend
    kb: Arc<KnowledgeBase>,
    /// Token budget for the entire prompt
    total_budget: usize,
    /// Current estimated token usage
    current_tokens: usize,
    /// LLM client for consolidation calls
    llm: Box<dyn LlmProvider>,
    /// Cache of recent search results
    search_cache: LruCache<String, Vec<MemorySearchResult>>,
    /// Provenance chain
    provenance: Vec<ProvenanceEntry>,
}

impl MemoryManager {
    /// Create with default blocks (persona, human, working)
    pub fn new(kb: Arc<KnowledgeBase>, llm: Box<dyn LlmProvider>) -> Self {
        Self {
            core_blocks: vec![
                MemoryBlock::new("persona", "{ \"name\": \"NeoTrix\" }", 2000, true),
                MemoryBlock::new("human", "{ \"name\": \"User\" }", 2000, true),
                MemoryBlock::new("working", "{}", 4000, true),
            ],
            kb,
            total_budget: 8000,
            current_tokens: 0,
            llm,
            search_cache: LruCache::new(NonZeroUsize::new(50).unwrap()),
            provenance: Vec::new(),
        }
    }

    // === STEP 1: Inject memory into system prompt ===

    /// Serialize core memory blocks as JSON and append to the system prompt.
    /// This is called before every LLM reasoning call.
    pub fn inject_memory(&self, prompt: &mut String) {
        let memory_section = self.render_memory_section();
        // Inject at a memory marker, or append before tool definitions
        if let Some(pos) = prompt.find("<!-- MEMORY -->") {
            prompt.insert_str(pos, &memory_section);
        } else {
            prompt.push_str(&memory_section);
        }
    }

    fn render_memory_section(&self) -> String {
        let mut section = String::from("\n\n<memory>\n");
        for block in &self.core_blocks {
            let json = serde_json::to_string_pretty(&block.content).unwrap_or_default();
            section.push_str(&format!(
                "  <block name=\"{}\" editable=\"{}\" tokens=\"{}\">\n",
                block.name, block.is_editable, block.token_count
            ));
            section.push_str(&format!("    {}\n", json));
            section.push_str("  </block>\n");
        }
        section.push_str(&format!(
            "  <usage total_tokens=\"{}\" budget=\"{}\" />\n",
            self.current_tokens, self.total_budget
        ));
        section.push_str("</memory>\n\n");
        section.push_str(
            "Available memory tools:\n\
             - memory_read(name): read a memory block\n\
             - memory_write(name, content, append): write to a memory block\n\
             - memory_consolidate(name): compress memory to fit limit\n\
             - memory_forget(name, fact_ids): remove specific facts\n\
             - memory_search(query, k): search archival memory\n\
             - memory_insert(content, tags, importance): store new knowledge\n"
        );
        section
    }

    // === STEP 2: Process tool calls from LLM response ===

    /// Parse and execute memory tool calls from the agent's response.
    /// Returns events for GWT broadcast.
    pub fn process_tool(&mut self, tool: &MemoryTool) -> Result<MemoryEvent, MemoryError> {
        match tool {
            MemoryTool::MemoryRead { name } => self.handle_read(name),
            MemoryTool::MemoryWrite { name, content, append } => {
                self.handle_write(name, content, *append)
            }
            MemoryTool::MemoryConsolidate { name, target_tokens } => {
                self.handle_consolidate(name, *target_tokens)
            }
            MemoryTool::MemoryForget { name, fact_ids } => self.handle_forget(name, fact_ids),
            MemoryTool::MemorySearch { query, k, scope } => self.handle_search(query, *k, scope),
            MemoryTool::MemoryInsert { content, tags, importance } => {
                self.handle_insert(content, tags, *importance)
            }
        }
    }

    fn handle_read(&self, name: &str) -> Result<MemoryEvent, MemoryError> {
        let block = self.core_blocks.iter()
            .find(|b| b.name == name)
            .ok_or(MemoryError::BlockNotFound(name.to_string()))?;
        Ok(MemoryEvent::Read {
            name: name.to_string(),
            content: block.content.clone(),
        })
    }

    fn handle_write(&mut self, name: &str, content: &str, append: bool) -> Result<MemoryEvent, MemoryError> {
        let block = self.core_blocks.iter_mut()
            .find(|b| b.name == name)
            .ok_or(MemoryError::BlockNotFound(name.to_string()))?;

        if !block.is_editable {
            return Err(MemoryError::BlockNotEditable(name.to_string()));
        }

        let old_content = block.content.clone();
        if append {
            block.content.push_str("\n");
            block.content.push_str(content);
        } else {
            block.content = content.to_string();
        }
        block.token_count = estimate_tokens(&block.content);
        block.version += 1;
        block.last_modified = SystemTime::now();

        self.provenance.push(ProvenanceEntry {
            action: "write".into(),
            block: name.to_string(),
            timestamp: SystemTime::now(),
            summary: truncate(content, 100),
        });

        // If approaching limit, schedule consolidation
        let needs_consolidation = block.needs_consolidation();

        Ok(MemoryEvent::Write {
            name: name.to_string(),
            old_content,
            new_content: content.to_string(),
            version: block.version,
            triggered_consolidation: needs_consolidation,
        })
    }

    // === STEP 3: Auto-consolidation ===

    /// Run consolidation on blocks that exceed their token limit.
    /// Uses an LLM call (cheap model) to summarize the content.
    pub fn auto_consolidate(&mut self) -> Result<Vec<ConsolidationReport>, MemoryError> {
        let mut reports = Vec::new();

        for block in &mut self.core_blocks {
            if !block.needs_consolidation() {
                continue;
            }

            // LLM summarizes the block content to fit within limit
            let summary = self.summarize_block(block)?;
            let report = ConsolidationReport {
                block: block.name.clone(),
                before_tokens: block.token_count,
                after_tokens: summary.token_count,
                facts_preserved: summary.fact_count,
                facts_dropped: summary.dropped_count,
            };

            block.content = summary.content;
            block.token_count = summary.token_count;
            block.version += 1;
            reports.push(report);
        }

        Ok(reports)
    }

    fn summarize_block(&self, block: &MemoryBlock) -> Result<SummaryResult, MemoryError> {
        let prompt = format!(
            "You are a memory consolidation agent. \
             Summarize the following content to fit within {} tokens. \
             Preserve all key facts, relationships, and preferences. \
             Drop redundant or outdated information. \
             Output as condensed structured text.\n\n{}",
            block.limit, block.content
        );

        let response = self.llm.complete(&LlmRequest {
            model: "gpt-4o-mini".into(), // cheap model for consolidation
            prompt,
            max_tokens: block.limit,
            temperature: 0.3, // deterministic
            ..Default::default()
        }).map_err(|e| MemoryError::ConsolidationFailed(e.to_string()))?;

        Ok(SummaryResult {
            content: response.text,
            token_count: estimate_tokens(&response.text),
            fact_count: estimate_facts(&response.text), // heuristic
            dropped_count: 0, // cannot measure precisely
        })
    }

    // === STEP 4: Background archival sync ===

    /// Move old working memory content to archival storage.
    /// Triggered when working block exceeds 70% of its limit.
    pub fn background_sync(&self) -> Option<BackgroundTask> {
        let working = self.core_blocks.iter()
            .find(|b| b.name == "working")?;

        if working.token_count < (working.limit as f64 * 0.7) as usize {
            return None; // no sync needed
        }

        Some(BackgroundTask {
            block: working.name.clone(),
            action: "archival_sync".into(),
            estimated_duration: Duration::from_secs(2),
        })
    }
}
```

---

## 2. Injection Format (System Prompt)

The memory blocks are injected into the system prompt as structured JSON with a clear tool interface. This mirrors Letta's approach of "memory blocks in context window, agent edits via tools."

```
<memory>
  <block name="persona" editable="true" tokens="187">
    {
      "name": "NeoTrix",
      "expertise": ["Rust", "distributed systems", "AI architecture"],
      "current_goal": "Implement graph orchestration",
      "constraints": ["no unsafe code", "80% coverage required"]
    }
  </block>
  <block name="human" editable="true" tokens="42">
    {
      "name": "User",
      "preferences": { "verbosity": "concise", "format": "markdown" },
      "known_topics": ["E8", "VSA", "HyperCube"]
    }
  </block>
  <block name="working" editable="true" tokens="1532">
    {
      "current_task": "Graph orchestration design",
      "intermediate_findings": [
        "LangGraph uses superstep model",
        "Reducer pattern enables deterministic recovery",
        "SEAL migration requires handoff pattern"
      ],
      "active_attempts": [],
      "deadline": null
    }
  </block>
  <usage total_tokens="1761" budget="8000" />
</memory>

Available memory tools:
- memory_read(name): read a memory block
- memory_write(name, content, append): write to a memory block
- memory_consolidate(name): compress memory to fit limit
- memory_forget(name, fact_ids): remove specific facts
- memory_search(query, k): search archival memory
- memory_insert(content, tags, importance): store new knowledge
```

---

## 3. Consolidation Algorithm

When a memory block exceeds its `limit`, the `auto_consolidate()` method triggers an LLM-based compression. The algorithm:

### 3.1 Trigger Conditions

1. **Synchronous trigger**: After `MemoryWrite` if `block.token_count > block.limit`
2. **Pre-call trigger**: Before `inject_memory()` if any block exceeds limit (lazy)
3. **Periodic trigger**: cron-like check every N reasoning steps

### 3.2 Consolidation Prompt Design

```
System: You are a memory consolidation agent. Your task is to compress the
following memory block to fit within {limit} tokens. Follow these rules:

1. PRESERVE all factual claims (names, dates, preferences, decisions)
2. PRESERVE all relationships between entities
3. PRESERVE all active goals and pending tasks
4. CONDENSE verbose explanations to key points
5. DROP redundant statements
6. DROP outdated information (marked with timestamp older than 7 days)
7. MERGE related facts into structured entries

Output format: condense to the most information-dense representation.
Use JSON arrays/lists instead of prose where possible.

Block name: {name}
Current size: {token_count} tokens
Target: ≤ {limit} tokens

Content:
{content}

Condensed output:
```

### 3.3 Conflict Detection (Mem0 Pattern)

When new information contradicts existing memory, the consolidation LLM marks both with a `confidence` score rather than deleting:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFact {
    pub id: String,
    pub content: String,
    pub confidence: f64,        // 0.0–1.0
    pub source_episode: String, // provenance: what action created this
    pub created: SystemTime,
    pub last_confirmed: SystemTime, // when was this fact last corroborated
    pub is_active: bool,        // soft-delete: mark inactive, never delete
    pub conflicting_with: Vec<String>, // IDs of facts that contradict this
}
```

On conflict:
1. Both facts are preserved with `is_active: true`
2. Each gets a `conflicting_with` reference to the other
3. The newer fact gets `confidence = 0.6`, older drops to `0.4`
4. On retrieval, facts with `confidence < 0.3` are filtered out by default
5. The agent can explicitly query low-confidence facts via `MemorySearch { include_low_confidence: true }`

---

## 4. Provenance Chain

Every memory write records the agent action that produced it. This enables audit trails and the "turkey scientist" observer to detect patterns.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub action: String,          // "write", "consolidate", "forget", "insert"
    pub block: String,
    pub timestamp: SystemTime,
    pub summary: String,         // truncate(content, 100)
}

/// Stored as a KnowledgeNode with type KnowledgeNodeType::ProvenanceRecord
impl MemoryManager {
    pub fn flush_provenance(&self) -> Result<usize, MemoryError> {
        let mut count = 0;
        for entry in &self.provenance {
            let node = KnowledgeNode {
                id: format!("prov-{}", uuid::Uuid::new_v4()),
                node_type: KnowledgeNodeType::ProvenanceRecord,
                title: format!("memory:{}:{}", entry.action, entry.block),
                content: serde_json::to_string(&entry).unwrap_or_default(),
                url: None,
                source: "agent_memory".into(),
                timestamp: entry.timestamp
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
                tags: vec!["provenance".into(), entry.action.clone(), entry.block.clone()],
                metadata: HashMap::new(),
                embedding: None,
                source_episode: String::new(),
                confidence: 1.0,
                access_count: 0,
            };
            self.kb.insert_node(&node).map_err(MemoryError::KbError)?;
            count += 1;
        }
        Ok(count)
    }
}
```

---

## 5. E8 Integration

### 5.1 Memory-Aware Reasoning Loop

```rust
// Wrapper around ReasoningEngine::reason() that injects and processes memory
pub fn reason_with_memory(
    engine: &mut ReasoningEngine,
    memory: &mut MemoryManager,
    task: &str,
) -> NeoTrixResult<String> {
    // 1. Pre-call memory management
    let consolidation_reports = memory.auto_consolidate()?;
    for report in &consolidation_reports {
        // Broadcast consolidation events to GWT
        engine.gwt.broadcast(GwtEvent::MemoryConsolidated {
            block: report.block.clone(),
            before: report.before_tokens,
            after: report.after_tokens,
        });
    }

    // 2. Check background sync
    if let Some(task) = memory.background_sync() {
        // Spawn async archival sync (don't block reasoning)
        tokio::spawn(async move {
            // sync working memory to archival store
            memory.sync_to_archival(&task.block).await;
        });
    }

    // 3. Build prompt with memory injection
    let mut prompt = String::new();
    memory.inject_memory(&mut prompt);
    prompt.push_str(&format!("\n\nUser task: {}\n", task));

    // 4. Run E8 reasoning
    let response = engine.reason(&prompt)?;

    // 5. Parse response for memory tool calls
    // Tool calls appear in the response as structured JSON blocks
    for tool in extract_memory_tools(&response) {
        match memory.process_tool(&tool) {
            Ok(event) => {
                engine.gwt.broadcast(GwtEvent::MemoryToolExecuted {
                    tool: format!("{:?}", tool),
                    event: format!("{:?}", event),
                });
            }
            Err(e) => {
                // Log but don't fail the reasoning cycle
                log::warn!("Memory tool failed: {:?}", e);
            }
        }
    }

    // 6. Flush provenance periodically
    if engine.traces_since_distill % 5 == 0 {
        memory.flush_provenance()?;
    }

    Ok(response)
}
```

### 5.2 Tool Call Extraction

The LLM returns tool calls as structured JSON in its response. We extract them using regex or structured output parsing:

```rust
/// Extract MemoryTool calls from LLM response text.
/// Tools are embedded as: <memory_tool>{"type":"MemoryWrite","name":"human","content":"..."}</memory_tool>
fn extract_memory_tools(response: &str) -> Vec<MemoryTool> {
    let mut tools = Vec::new();
    let re = Regex::new(r"<memory_tool>(.*?)</memory_tool>").unwrap();

    for cap in re.captures_iter(response) {
        if let Ok(tool) = serde_json::from_str::<MemoryTool>(&cap[1]) {
            tools.push(tool);
        }
    }

    tools
}

/// Alternative: the LLM response can be a structured JSON with a tools array.
/// This is preferred when using JSON mode.
fn extract_memory_tools_structured(response: &str) -> Vec<MemoryTool> {
    // Try parsing the entire response as: { "reasoning": "...", "memory_tools": [...] }
    #[derive(Deserialize)]
    struct ToolResponse {
        #[serde(default)]
        memory_tools: Vec<MemoryTool>,
    }

    if let Ok(parsed) = serde_json::from_str::<ToolResponse>(response) {
        return parsed.memory_tools;
    }
    Vec::new()
}
```

---

## 6. GWT Integration

Memory events trigger GWT resonance — the workspace broadcasts memory changes so other specialists can react.

```rust
// GwtEvent variants for memory
pub enum MemoryGwtEvent {
    /// A memory block was written
    MemoryWritten {
        block: String,
        version: u64,
        summary: String,
    },
    /// A memory block was consolidated
    MemoryConsolidated {
        block: String,
        before_tokens: usize,
        after_tokens: usize,
    },
    /// Facts were inserted into archival memory
    FactsInserted {
        count: usize,
        tags: Vec<String>,
    },
    /// Memory was searched (for observability)
    MemorySearched {
        query: String,
        results_count: usize,
    },
    /// Token budget pressure warning
    MemoryPressure {
        usage_pct: f64,  // 0.0–1.0
        blocks_above_limit: Vec<String>,
    },
}
```

---

## 7. Mem0-Inspired Extraction Pipeline

Beyond the Letta-style self-editing blocks, we add a Mem0-style background extraction pipeline that automatically distills conversation into structured memories.

```
┌─────────────┐     ┌──────────────────┐     ┌────────────────┐
│ Conversation  │────▶ Extraction Phase  │────▶ Update Phase    │
│ (user+agent)  │     │ (LLM extracts   │     │ (CRUD decisions │
│               │     │  salient facts) │     │  via LLM)       │
└─────────────┘     └──────────────────┘     └────────────────┘
                           │                           │
                           ▼                           ▼
                    ┌──────────────────┐     ┌────────────────┐
                    │ Embedding        │     │ Conflict       │
                    │ + Similarity     │     │ Detection      │
                    │ Search (top-5)   │     │ (LLM tool call)│
                    └──────────────────┘     └────────────────┘
                                                      │
                                                      ▼
                                               ┌────────────────┐
                                               │ Store in       │
                                               │ nt_memory_kb   │
                                               │ as KnowledgeNode│
                                               └────────────────┘
```

```rust
pub struct ExtractionPipeline {
    kb: Arc<KnowledgeBase>,
    llm: Box<dyn LlmProvider>,
    summary_cache: LruCache<String, String>, // conversation_id → summary
}

impl ExtractionPipeline {
    /// Process a single conversation turn:
    /// 1. Extract salient facts
    /// 2. Embed and find similar existing facts
    /// 3. LLM decides ADD / UPDATE / DELETE / NOOP
    /// 4. Execute on KB
    pub fn process_turn(&mut self, turn: &ConversationTurn) -> Result<ExtractionReport, MemoryError> {
        // Step 1: Build context with recent messages + global summary
        let summary = self.get_or_refresh_summary(&turn.conversation_id)?;
        let recent = self.get_recent_messages(&turn.conversation_id, 5)?;

        let context = format!(
            "Conversation Summary: {}\n\nRecent Messages:\n{}\n\nCurrent Turn:\nUser: {}\nAgent: {}",
            summary,
            recent.join("\n"),
            turn.user_message,
            turn.agent_message,
        );

        // Step 2: LLM extracts facts
        let facts = self.extract_facts(&context)?;

        // Step 3: For each fact, find similar and classify
        let mut report = ExtractionReport::default();
        for fact in &facts {
            let similar = self.find_similar(fact, 5)?;
            let operation = self.classify_operation(fact, &similar)?;
            self.execute_operation(&operation, fact)?;
            report.record_operation(operation);
        }

        Ok(report)
    }

    fn extract_facts(&self, context: &str) -> Result<Vec<ExtractedFact>, MemoryError> {
        let prompt = format!(
            "Extract salient facts from this conversation turn. \
             Each fact should be a single piece of information about the user, \
             the agent, their relationship, or the task.\n\
             Output as JSON array: [{{\"fact\": \"...\", \"category\": \"...\", \
             \"importance\": 0.0..1.0}}]\n\n{}",
            context
        );

        let response = self.llm.complete(&LlmRequest {
            model: "gpt-4o-mini".into(),
            prompt,
            max_tokens: 1000,
            temperature: 0.2,
            ..Default::default()
        })?;

        serde_json::from_str(&response.text)
            .map_err(|e| MemoryError::ParseError(e.to_string()))
    }

    fn find_similar(&self, fact: &ExtractedFact, k: usize) -> Result<Vec<SimilarFact>, MemoryError> {
        let results = self.kb.search(&kb::SearchQuery {
            text: &fact.fact,
            limit: k,
            search_mode: SearchMode::Hybrid,
            ..Default::default()
        })?;

        Ok(results.into_iter().map(|r| SimilarFact {
            id: r.id,
            content: r.content,
            similarity: r.score,
        }).collect())
    }

    fn classify_operation(&self, new_fact: &ExtractedFact, similar: &[SimilarFact])
        -> Result<MemoryOperation, MemoryError>
    {
        let similar_json = serde_json::to_string_pretty(similar).unwrap_or_default();
        let prompt = format!(
            "You are a memory update classifier. Given a new fact and similar existing facts,\
             decide the operation:\n\
             - ADD: no matching fact exists\n\
             - UPDATE: new fact complements/refines an existing fact\n\
             - DELETE: new fact contradicts (but we soft-delete instead)\n\
             - NOOP: already well-represented\n\n\
             New fact: {}\n\n\
             Similar existing facts:\n{}\n\n\
             Output: {{\"operation\": \"ADD|UPDATE|DELETE|NOOP\", \"target_id\": \"...\", \
             \"reason\": \"...\"}}",
            serde_json::to_string_pretty(new_fact).unwrap_or_default(),
            similar_json,
        );

        let response = self.llm.complete(&LlmRequest {
            model: "gpt-4o-mini".into(),
            prompt,
            max_tokens: 300,
            temperature: 0.1,
            ..Default::default()
        })?;

        serde_json::from_str(&response.text)
            .map_err(|e| MemoryError::ParseError(e.to_string()))
    }

    fn execute_operation(&self, op: &MemoryOperation, fact: &ExtractedFact) -> Result<(), MemoryError> {
        match op.operation.as_str() {
            "ADD" => {
                let node = KnowledgeNode {
                    id: uuid::Uuid::new_v4().to_string(),
                    node_type: KnowledgeNodeType::AgentMemory,
                    title: fact.fact.chars().take(100).collect(),
                    content: fact.fact.clone(),
                    url: None,
                    source: "agent_extraction".into(),
                    timestamp: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                    tags: vec!["agent_memory".into(), fact.category.clone()],
                    metadata: HashMap::from([
                        ("importance".into(), fact.importance.to_string()),
                        ("confidence".into(), "0.8".into()),
                    ]),
                    embedding: None,
                    source_episode: String::new(),
                    confidence: fact.importance,
                    access_count: 0,
                };
                self.kb.insert_node(&node)?;
            }
            "UPDATE" => {
                // Store as new version without deleting old (ADD-only + provenance)
                let mut node = self.kb.get_node(&op.target_id)?
                    .ok_or(MemoryError::FactNotFound(op.target_id.clone()))?;
                let old_id = node.id.clone();
                node.id = uuid::Uuid::new_v4().to_string();
                node.content = fact.fact.clone();
                node.metadata.insert("supersedes".into(), old_id);
                node.timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;
                node.confidence = fact.importance;
                self.kb.insert_node(&node)?;
            }
            "DELETE" => {
                // Soft delete — mark inactive
                if let Some(mut node) = self.kb.get_node(&op.target_id)? {
                    node.confidence = 0.0;
                    node.metadata.insert("status".into(), "archived".into());
                    self.kb.insert_node(&node)?;
                }
            }
            _ => {} // NOOP
        }
        Ok(())
    }
}
```

---

## 8. Integration Points

| Module | Integration |
|--------|------------|
| `nt_memory_kb` (L3) | Archival memory backend — stores `KnowledgeNode`s with `AgentMemory` type |
| `nt_core_e8` (L4) | `reason_with_memory()` wraps `engine.reason()` with memory injection + tool processing |
| `nt_core_gwt` (L5) | `MemoryGwtEvent` broadcasts for resonance — all specialists observe memory changes |
| `nt_mind_seal` (L8) | `ConversationDistillStage` feeds the extraction pipeline; `ProceduralMemoryStage` reads agent memories |
| `nt_io_provider` (L1) | Memory blocks injected into system prompt via `LlmRequest` |
| `nt_core_observer` (L9) | TurkeyScientist observes memory provenance for pattern detection |
| `nt_mcp` (L1) | Memory tools exposed as MCP tool definitions |
| `nt_core_policy` (L4) | Consolidation decisions affected by E8 mode — e.g., "debug" mode disables consolidation |

---

## 9. Implementation Plan

### Phase 1: MemoryBlock Types + Injection (1 day)
- `MemoryBlock` struct with serialization, `needs_consolidation()`, token estimation
- `MemoryManager` with `new()`, `inject_memory()`, `render_memory_section()`
- `MemoryTool` enum with `MemoryRead`, `MemoryWrite`
- `handle_read()`, `handle_write()` implementations
- Unit tests: block creation, token tracking, injection format rendering

### Phase 2: Tool Functions + Handler (2 days)
- `handle_consolidate()`, `handle_forget()`, `handle_search()`, `handle_insert()`
- `extract_memory_tools()` regex and structured parsers
- MCP tool definitions for all 6 memory tools
- Integration test: mock LLM returns tool calls → memory processes them correctly
- Integration test: agent writes to persona → next injection reflects changes

### Phase 3: Auto-Consolidation Algorithm (2 days)
- `auto_consolidate()` with LLM-based summarization
- `summarize_block()` with consolidation prompt
- Trigger conditions (synchronous, pre-call, periodic)
- `MemoryFact` with `confidence` and `conflicting_with` for conflict preservation
- `MemoryPressure` event when approaching token budget
- Tests: block exceeds limit → auto-consolidate → content preserved but shorter
- Tests: conflicting facts both retained with correct confidence

### Phase 4: Archival Sync + Provenance (2 days)
- `background_sync()` — moves working memory overflow to KB
- `ProvenanceEntry` and `flush_provenance()` — stores as `ProvenanceRecord` nodes
- `ExtractionPipeline` — Mem0-style extraction/update pipeline
- `extract_facts()`, `classify_operation()`, `execute_operation()`
- ADD-only archival with soft-delete
- GWT event integration (`MemoryGwtEvent`)
- Tests: provenance chain contains all writes; extraction pipeline produces valid KB inserts

---

## 10. Token Budget Management

```rust
impl MemoryManager {
    /// Calculate total token usage across all blocks
    pub fn total_tokens(&self) -> usize {
        self.core_blocks.iter().map(|b| b.token_count).sum()
    }

    /// Get pressure level as f64 0.0–1.0
    pub fn memory_pressure(&self) -> f64 {
        self.total_tokens() as f64 / self.total_budget as f64
    }

    /// Auto-trigger when pressure exceeds threshold
    pub fn check_pressure(&mut self) -> Option<MemoryPressure> {
        let pct = self.memory_pressure();
        if pct > 0.85 {
            let over_limit: Vec<String> = self.core_blocks.iter()
                .filter(|b| b.needs_consolidation())
                .map(|b| b.name.clone())
                .collect();
            Some(MemoryPressure {
                usage_pct: pct,
                blocks_above_limit: over_limit,
            })
        } else {
            None
        }
    }
}
```

---

## 11. Edge Cases & Safety

1. **Runaway writes**: Agent writes 1000 facts in one turn → `max_writes_per_turn = 20` limit enforced in `process_tool()`
2. **Consolidation loop**: Agent writes → consolidation triggers → writes again → infinite loop → `consolidation_cooldown = 3` reasoning steps before next consolidation
3. **Block name collision**: `name` field is unique per agent — `DuplicateBlock` error on creation
4. **Concurrent writes**: Shared memory blocks between agents → `MemoryWrite` with version check (optimistic locking). If `block.version` in write request < current `block.version`, reject with `VersionConflict`.
5. **Empty consolidation**: If all content is important, LLM may produce no compression → fall back to truncation (keep first N tokens, append `... (truncated)`)
6. **Provenance explosion**: Flush to KB every 5 turns, not every turn. `flush_provenance()` batches entries.
