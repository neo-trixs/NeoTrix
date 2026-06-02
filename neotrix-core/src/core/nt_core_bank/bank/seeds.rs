use crate::core::nt_core_knowledge::TaskType;
use crate::core::nt_core_bank::ReasoningMemory;
use super::ReasoningBank;

impl ReasoningBank {
    pub fn initialize_all_knowledge_sources(&mut self) -> usize {
        let mut count = 0;
        count += self.initialize_design_seeds();
        count += self.initialize_framework_seeds();
        count += self.initialize_agent_seeds();
        count += self.initialize_security_seeds();
        count += self.initialize_memory_seeds();
        count += self.initialize_math_e8_seeds();
        count += self.initialize_specialized_seeds();
        count
    }

    fn init_seeds(&mut self, seeds: Vec<(&str, TaskType, f64)>) -> usize {
        let count = seeds.len();
        for (desc, tt, reward) in seeds { self.store(ReasoningMemory::new(desc, tt, &[], reward)); }
        count
    }

    fn initialize_design_seeds(&mut self) -> usize {
        self.init_seeds(vec![
//            ("HeroUI: compound component composition pattern — combines atomic components into cohesive compounds with shared state via tailwind-variants", TaskType::UIDesign, 0.95),
//            ("BaseUI: accessible unstyled React primitives — useHeadless pattern for complete visual control with ARIA compliance", TaskType::UIDesign, 0.92),
//            ("ArcUI: AI-native interface components — semantic layers for model confidence, reasoning steps, and streaming states", TaskType::UIDesign, 0.90),
            ("CortexUI: semantic design system with LLM-generated verification gates for component consistency", TaskType::UIDesign, 0.88),
            ("AgenticDS: quality-gated design system with automated verification of component specs against accessibility standards", TaskType::UIDesign, 0.89),
            ("DesignPhilosophy: design token philosophy and semantic layer architecture for scalable design systems", TaskType::UIDesign, 0.85),
        ])
    }

    fn initialize_framework_seeds(&mut self) -> usize {
        self.init_seeds(vec![
//            ("DeepSeekTUI: TUI terminal interface for DeepSeek API — async streaming, conversation management, markdown rendering", TaskType::CodeGeneration, 0.90),
            ("Codebuff: background agent framework for incremental file editing with git-aware change tracking", TaskType::CodeGeneration, 0.88),
            ("OpenClaude: open-source Claude desktop client with multi-session management and tool integration", TaskType::CodeGeneration, 0.87),
            ("Cairn: Rust-native agent framework with deterministic execution and capability-based security", TaskType::CodeGeneration, 0.91),
            ("Orca: distributed task execution framework with result aggregation and error recovery", TaskType::CodeGeneration, 0.85),
            ("SkillsGate: visual skill directory with 91k+ skills, per-agent assignment, version management and browser UI", TaskType::Planning, 0.90),
            ("DeepSeekCoder: code-specialized LLM with repository-level context understanding", TaskType::CodeGeneration, 0.93),
            ("ClaudeCode: terminal-native AI coding assistant with file editing, bash, and web search tools", TaskType::CodeGeneration, 0.89),
        ])
    }

    fn initialize_agent_seeds(&mut self) -> usize {
        self.init_seeds(vec![
//            ("SPEAR: CodeAct Agent Protocol with 4 tools — evaluate/python/set_prompt/finish state machine for self-improving agents", TaskType::General, 0.95),
//            ("SIA: Self-Improving AI with tri-body architecture — MetaAgent decomposes, TargetAgent executes, FeedbackAgent selects harness/weight update", TaskType::General, 0.94),
            ("RedRun: automated penetration testing agent with multi-stage attack chain execution", TaskType::Security, 0.88),
            ("Autojob: automated job application agent with form filling and resume tailoring", TaskType::General, 0.82),
            ("AiGrader: automated grading agent with rubric-based evaluation and feedback generation", TaskType::CodeAnalysis, 0.83),
            ("AiPdf: PDF processing agent with extraction, OCR, table parsing and structural analysis", TaskType::CodeAnalysis, 0.84),
            ("AiTranslator: multilingual translation agent with context-aware terminology and style preservation", TaskType::General, 0.81),
            ("AiTalent: talent sourcing agent with skill matching and candidate ranking", TaskType::CodeAnalysis, 0.80),
            ("AiAgent: extensible agent framework with plugin architecture for tool integration", TaskType::CodeGeneration, 0.86),
            ("AiFlock: multi-agent orchestration with leader-election and task distribution", TaskType::General, 0.85),
            ("AiShell: AI-powered shell with natural language command generation and execution", TaskType::CodeGeneration, 0.83),
            ("AiTerminal: intelligent terminal with command history analysis and error auto-fix", TaskType::CodeGeneration, 0.82),
        ])
    }

    fn initialize_security_seeds(&mut self) -> usize {
        self.init_seeds(vec![
            ("Betterleaks: secret detection engine with 2000+ regex patterns, entropy analysis, and celer filtering", TaskType::Security, 0.95),
            ("YaoWebsecurity: comprehensive web security scanning with OWASP Top 10 coverage and report generation", TaskType::Security, 0.92),
            ("Botasaurus: anti-detection web scraping framework with fingerprint rotation and proxy management", TaskType::Security, 0.90),
            ("ReactDoctor: React component health analysis with performance profiling, a11y checks, and bundle analysis", TaskType::CodeReview, 0.89),
            ("VulnGym: code review security benchmark with entry-point trace analysis and business logic vulnerability taxonomy", TaskType::CodeReview, 0.93),
        ])
    }

    fn initialize_memory_seeds(&mut self) -> usize {
        self.init_seeds(vec![
            ("Zep: temporal knowledge graph memory with persistent entity extraction and relationship tracking across sessions", TaskType::Learning, 0.93),
            ("Mem0: hybrid memory with short-term working + long-term semantic + episodic tiers, importance-based consolidation", TaskType::Learning, 0.92),
            ("Hindsight: multi-strategy retrieval fusing BM25, vector embedding, and graph traversal with RRF reranking", TaskType::CodeAnalysis, 0.90),
            ("Cognee: knowledge graph + vector store hybrid with ontology-driven entity extraction and semantic query", TaskType::Research, 0.88),
            ("SAGE: self-evolving graph memory with automatic node splitting, edge pruning, and density-based region detection", TaskType::Learning, 0.91),
            ("ApexMem: attribute graph with temporal decay and property-based filtering for fine-grained retrieval", TaskType::CodeAnalysis, 0.87),
            ("LangMem: procedural memory for agent self-edit prompt evolution and background compaction", TaskType::CodeGeneration, 0.85),
            ("Letta: OS-inspired hierarchical memory with working/episodic/semantic tiers and page-based retrieval", TaskType::Learning, 0.90),
            ("Graphiti: real-time temporal knowledge graph with incremental entity resolution and relationship updates", TaskType::CodeAnalysis, 0.86),
            ("GeminiMemory: Gemini-native memory system with multi-modal embeddings and cross-session context stitching", TaskType::Learning, 0.84),
        ])
    }

    fn initialize_math_e8_seeds(&mut self) -> usize {
        self.init_seeds(vec![
//            ("E8Theory: E8 Lie group exceptional structure — 248-dimensional root system with H4 and F4 folding", TaskType::Research, 0.95),
            ("E8Physics: E8 × E8 heterotic string theory and grand unified theory with Standard Model embedding", TaskType::Research, 0.92),
//            ("E8String: E8 gauge group in string compactification — Calabi-Yau manifold and bundle construction", TaskType::Research, 0.90),
            ("E8Consciousness: E8-based consciousness model using root system dynamics for neural correlates", TaskType::CodeAnalysis, 0.85),
//            ("E8Observer: E8 observer framework for trajectory analysis during reasoning — hexagram state transitions", TaskType::CodeAnalysis, 0.88),
            ("E8GeometricUnity: geometric unification using E8 root polytope as fundamental space", TaskType::Research, 0.87),
//            ("E8Exceptional: exceptional Lie algebra E8 subalgebra structure — maximal subgroups and branching rules", TaskType::Research, 0.89),
            ("E8Reality: E8-based reality model with quasicrystalline information processing", TaskType::CodeAnalysis, 0.83),
        ])
    }

    fn initialize_specialized_seeds(&mut self) -> usize {
        self.init_seeds(vec![
            ("Maigret: OSINT username search across 3000+ websites with email/domain/phone correlation", TaskType::Security, 0.91),
            ("TasteSkill: anti-slop quality gate with VARIANCE/MOTION/DENSITY knobs for AI output aesthetics", TaskType::CodeReview, 0.89),
            ("UnderstandAnything: automated code comprehension with multi-level abstraction and dependency tracing", TaskType::CodeAnalysis, 0.90),
//            ("CarbonCode: carbon-aware code optimization — energy profiling and emission-reducing refactoring", TaskType::CodeReview, 0.86),
//            ("LlmArch: LLM system architecture patterns — Transformer variants, attention mechanisms, scaling laws", TaskType::Research, 0.93),
            ("Hyperframes: video-to-HTML rendering with frame-accurate composition and animation preservation", TaskType::UIDesign, 0.88),
            ("OpenPencil: vector design canvas via MCP tools with SVG generation and component composition", TaskType::UIDesign, 0.87),
            ("AiTrader: quantitative trading agent with multi-agent signal synchronization and risk management", TaskType::CodeAnalysis, 0.85),
            ("SesameRobot: ESP32 quadruped robot firmware with inverse kinematics and gait planning", TaskType::CodeGeneration, 0.84),
            ("SiliconSelf: meta-cognitive thinking model with 10 attention domains and 15 LLM→system cognitive mappings", TaskType::CodeAnalysis, 0.94),
            ("OrpheusTts: expressive TTS with emotion control, nt_act_voice cloning, and prosody manipulation", TaskType::General, 0.83),
            ("Topaz: lightweight high-performance inference runtime with quantization and kernel fusion", TaskType::CodeGeneration, 0.86),
            ("Monolith: monolithic ML model serving with zero-copy tensor passing and request batching", TaskType::CodeGeneration, 0.82),
            ("Puffer: latency-aware request scheduling with adaptive batching and priority queues", TaskType::CodeGeneration, 0.81),
            ("MCP: Model Context Protocol for tool service discovery, JSON-RPC transport, and capability negotiation", TaskType::CodeGeneration, 0.92),
            ("Kimi: long-context LLM with RoPE extrapolation and key-value cache compression", TaskType::Research, 0.88),
            ("K2: next-gen RAG with iterative retrieval and structured knowledge integration", TaskType::Research, 0.85),
            ("Aider: LLM-powered code editor with repo-aware editing and git-integrated workflows", TaskType::CodeGeneration, 0.87),
            ("Goose: on-device AI agent for code generation and file operations with tool-use safety", TaskType::CodeGeneration, 0.84),
            ("Cline: autonomous coding agent with multi-file editing, testing, and debugging capabilities", TaskType::CodeGeneration, 0.86),
            ("Roo: research-oriented agent with paper reading, experiment design, and result analysis", TaskType::Research, 0.85),
            ("TaskManager: goal-oriented task decomposition and tracking with dependency resolution", TaskType::Planning, 0.87),
            ("ThinkingModel: structured reasoning with chain-of-thought, tree-of-thought, and reflection patterns", TaskType::CodeAnalysis, 0.90),
            ("SelectiveState: Mamba SSM selective state space for long-range sequence modeling with linear complexity", TaskType::Research, 0.91),
            ("Mamba: state space model architecture with selective scan for efficient long-context processing", TaskType::Research, 0.92),
            ("WalshMemory: Walsh-Hadamard orthogonal memory index with O(n log n) encoding and spectral retrieval", TaskType::CodeAnalysis, 0.88),
            ("Bm25: Okapi BM25 probabilistic retrieval with term frequency saturation and document length normalization", TaskType::CodeAnalysis, 0.86),
//            ("T3Memory: three-zone memory compression — recent/important recency-reward zones for long-term maintenance", TaskType::Learning, 0.87),
            ("HyperCube: 4096-dim MAP VSA hypercube with bundle/bind/permute operations for knowledge representation", TaskType::CodeAnalysis, 0.93),
            ("Vsa: Vector Symbolic Architecture with MAP/HRR/BSC models and similarity-preserving encoding", TaskType::CodeAnalysis, 0.91),
            ("Consciousness: cognitive architecture with Global Workspace Theory, salience competition, and broadcast", TaskType::CodeAnalysis, 0.89),
//            ("GlobalWorkspace: attention routing via specialist module competition — urgency+novelty+coherence salience", TaskType::CodeAnalysis, 0.90),
            ("AttentionRouter: domain-specific attention routing with 10-domain resolution and focus switching", TaskType::CodeAnalysis, 0.88),
            ("GoalLoop: autonomous goal pursuit with rate limiting, circuit breaking, and budget-limited execution", TaskType::Planning, 0.91),
//            ("AdamsLaw: Textual Frequency Law (TFL) — LLMs prefer high-frequency textual expressions for prompting; paraphrasing prompts to higher frequency improves all models/tasks", TaskType::Learning, 0.91),
//            ("AdamsLawTFD: Textual Frequency Distillation — distill training data using frequency filtering to improve fine-tuning efficiency and output quality", TaskType::Learning, 0.88),
//            ("AdamsLawCTFT: Curriculum Textual Frequency Training — progressively increase text frequency during training for better generalization across model scales", TaskType::Learning, 0.86),
        ])
    }
}
