use super::ReasoningBank;
use crate::core::nt_core_knowledge::TaskType;
use crate::core::nt_core_bank::ReasoningMemory;

impl ReasoningBank {
    pub fn initialize_with_design_knowledge(&mut self) {
        if !self.memories.is_empty() { return; }
        let design_knowledge = vec![
            ("OpenCodeX 18-parameter layout system: messageSeparation, messagePaddingTop/Bottom, containerPaddingTop/Bottom, containerGap, toolMarginTop, agentInfoMarginTop, containerPaddingLeft/Right, messagePaddingLeft, textIndent, toolIndent, showHeader, showFooter, forceSidebarHidden, showInputAgentInfo, showInputBorder, inputAgentInfoPaddingTop, inputBoxPaddingTop/Bottom", TaskType::UIDesign, 0.95),
            ("macOS design: translucent background (rgba), backdrop-filter: blur(20px), border-radius: 10px, subtle borders (rgba(0,0,0,0.08)), smooth transitions (0.2s cubic-bezier), font: Inter/system-ui", TaskType::UIDesign, 0.92),
            ("Three-column layout: left session list (240px), center chat (flex), right file manager (280px). Gap: 8px, padding: 8px. Each panel: rounded corners, subtle shadow, backdrop blur", TaskType::UIDesign, 0.90),
            ("OpenCodeX design philosophy: JSON-configurable layout parameters, 18 spatial parameters (spacing, visibility, behavior). Users can switch between default/dense layouts via /layout command. Custom layouts in ~/.config/opencode/layout/", TaskType::UIDesign, 0.88),
            ("Session management: unique IDs, timestamp tracking, active state, localStorage persistence, create/switch/delete operations. Display: session name, relative time (2 min ago), active highlight", TaskType::UIDesign, 0.85),
            ("File tree interaction: folder toggle, nested children (16px indent), click to expand/collapse, active state highlighting, file icons, hover effects", TaskType::UIDesign, 0.83),
            ("Chat UI: user/assistant/system message types, fade-in animation, thinking state (pulse/spin animation), status dot indicator, message counting, SEAL badge display", TaskType::UIDesign, 0.87),
        ];
        for (desc, task_type, reward) in design_knowledge {
            self.store(ReasoningMemory::new(desc, task_type, &[], reward));
        }
    }

    pub fn initialize_with_coding_knowledge(&mut self) {
        let code_knowledge = vec![
            ("Rust error handling best practices: use Result<T, E> instead of unwrap(), prefer ? operator for propagation, use thiserror for library errors, use anyhow for application errors", TaskType::CodeReview, 0.92),
            ("Memory safety in Rust: ownership rules, borrowing, lifetime annotations, Rc<RefCell<T>> for shared mutability, Arc<Mutex<T>> for thread-safe shared state", TaskType::CodeReview, 0.90),
            ("Concurrent programming patterns: use tokio for async runtime, async/await for I/O bound tasks, tokio::spawn for CPU-bound tasks, channels for message passing, Mutex/RwLock for shared state", TaskType::CodeGeneration, 0.88),
            ("Security audit checklist: check for command injection, path traversal, SQL injection, XSS, hardcoded secrets, unsafe deserialization, DoS vectors. Use cargo audit regularly", TaskType::Security, 0.95),
            ("API design principles: consistent naming, RESTful URLs, proper HTTP methods, status codes, error body format, pagination, rate limiting headers, API versioning", TaskType::CodeGeneration, 0.87),
            ("Testing strategy: unit tests for pure functions, integration tests for API, property-based testing for edge cases, snapshot testing for UI/output, benchmark tests for performance, doc tests for examples", TaskType::CodeReview, 0.85),
            ("Performance optimization: profile before optimizing, focus on O(n) improvements, use BTreeMap/HashMap wisely, avoid unnecessary allocations, use iterators instead of loops when clearer, batch DB queries", TaskType::CodeGeneration, 0.84),
            ("Database optimization: index columns used in WHERE/JOIN/ORDER BY, use EXPLAIN ANALYZE to check query plans, avoid N+1 queries, connection pooling, migration versioning, transactions for atomic operations, prepared statements for repeated queries", TaskType::CodeGeneration, 0.86),
            ("React component patterns: composition over inheritance, custom hooks for shared logic, useMemo/useCallback for performance, context + useReducer for complex state, error boundaries for crash recovery, lazy loading + code splitting for large apps, TypeScript for type safety", TaskType::UIDesign, 0.89),
            ("CI/CD pipeline: lint, type check, unit tests, integration tests, build, artifact publishing, semantic versioning, changelog generation", TaskType::Planning, 0.82),
        ];
        for (desc, task_type, reward) in code_knowledge {
            self.store(ReasoningMemory::new(desc, task_type, &[], reward));
        }
    }

    pub fn initialize_with_everos_knowledge(&mut self) {
        let everos_knowledge = vec![
            ("EverOS hypergraph memory architecture: three-layer hierarchy — topic layer (broad themes), event layer (specific episodes), fact layer (atomic details). Hyperedges capture high-order associations between concepts. Retrieval follows coarse-to-fine: topic → event → fact. Each hyperedge connects multiple nodes forming higher-order relationships beyond pairwise associations.", TaskType::Research, 0.93),
            ("Biological memory imprinting pattern: memories are consolidated based on surprise/salience, not recency. Important memories have higher consolidation priority. The system maintains a working-to-long-term memory consolidation pipeline with importance threshold gating. This prevents catastrophic forgetting by prioritizing structurally significant experiences.", TaskType::Learning, 0.91),
            ("Self-evolution evaluation via EvoAgentBench methodology: longitudinal growth curves (measure capability trajectory over successive iterations), transfer efficiency (quantify how well knowledge acquired for one task type transfers to new domains), error avoidance (track reduction of repeated mistakes via contrastive reflection), and skill-hit quality (precision of capability vector targeting measured by cosine similarity to ideal profile).", TaskType::Reflection, 0.88),
            ("Memory extraction from unstructured conversation: detect entity relationships, temporal context, user preferences, task outcomes, and action items. Store as typed memory records with embedding for semantic retrieval. Uses hybrid search (BM25 + vector embedding) with RRF fusion for cross-session recall. Importance-weighted ranking ensures salient memories surface first.", TaskType::CodeAnalysis, 0.85),
            ("Multi-session context continuity via persistent memory bank: each session appends new memories; old memories decay via time-weighted importance but remain retrievable. Cross-session recall uses RRF fusion of BM25 + vector embedding search. Memory lifecycle management includes TTL-based expiration, importance-based consolidation, and periodic pruning of low-value traces.", TaskType::Planning, 0.87),
        ];
        for (desc, task_type, reward) in everos_knowledge {
            self.store(ReasoningMemory::new(desc, task_type, &[], reward));
        }
    }

    pub fn initialize_with_explainers(&mut self) {
        let explainers = vec![
            ("EXPLAINER: Rust borrow checker patterns — 为什么 RwLock 比 Mutex 更适合读多写少场景？\
              RwLock 允许多个读线程并行访问（共享读），但写线程独占（排他写）。\
              Mutex 无论读写都要独占，读操作多时产生不必要的竞争。\
              选择依据：读:写比例 > 80:20 → RwLock，否则 → Mutex。\
              核心原则：用正确的锁避免性能陷阱，而非盲目用 Mutex", TaskType::MetaCognition, 0.95),
            ("EXPLAINER: 知识吸收策略 — 为什么微编辑优于大改动？\
              借鉴 gstack 矩阵分解理论：大变换的条件数大，数值不稳定，回滚成本高。\
              微编辑（MicroEdit）序列将变化分解为小步，每步条件数接近 1，支持单步回滚。\
              类比：做手术时用精确的小切口而非大切口。\
              实践：每次调整 ≤3 个维度，完成后立即 normalize 防止维度膨胀", TaskType::MetaCognition, 0.93),
            ("EXPLAINER: 能力向量归一化 — 为什么必须有界维度？\
              无界向量会导致梯度爆炸：某些维度因历史积累过高，支配其他维度，丧失区分度。\
              归一化将向量约束到单位球面，保证所有维度在公平的尺度上竞争。\
              类比：高考标准分 vs 原始分——原始分差距过大时单科决定全局。\
              策略：每次 absorb/self-edit 后立即 normalize，使用 L2 范数", TaskType::MetaCognition, 0.92),
            ("EXPLAINER: 任务分解 — 为什么自上而下规划脱离上下文会失败？\
              柏拉图式理想（先全貌后分解）假设我们知道所有约束。\
              实践中约束在分解过程中浮现，自上而下遗漏局部异常。\
              正确模式：分层递进（top-down 框架 + bottom-up 反馈），每层完成后调整上层计划。\
              类比：写文章先有大纲没错，但每段写完都要重审大纲。\
              SEAL 循环中的 CriticNode 正是这种反馈机制的实现", TaskType::MetaCognition, 0.94),
            ("EXPLAINER: 自迭代循环 — 为什么外部验证不可或缺？\
              内部自评（self-evaluation）有确认偏误：系统倾向于给自己打高分。\
              外部验证（编译检查、测试、用户反馈、Playwright）提供独立于系统的奖励信号（External reward）。\
              无外部验证的 RL 奖励会陷入自我欺骗的局部最优。\
              类比：考试自己批改 vs 老师批改——自己批改永远发现不了盲点。\
              SEAL 原则：外部信号可用时优先使用，降级到内部评估时标记为 RewardSource::Internal", TaskType::MetaCognition, 0.95),
            ("EXPLAINER: 模块组织 — 为什么小文件优于大文件？\
              单文件 >800 行时认知负荷过载：需要同时追踪的上下文超过工作记忆容量(≈7±2 块)。\
              小文件让每个模块有单一职责、清晰的导入导出边界、更易测试。\
              但过度拆分也有代价：文件间跳转增加导航成本。\
              黄金分割：每个文件 150-400 行，按功能而非按类型拆分。\
              类比：图书的章节——太长读不下去，太短缺乏连贯性", TaskType::MetaCognition, 0.90),
        ];
        for (desc, task_type, reward) in explainers {
            self.store(ReasoningMemory::new(desc, task_type, &[], reward));
        }
    }
}
