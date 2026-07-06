#!/usr/bin/env python3
"""Absorb all NeoTrix documentation into SQLite knowledge base."""
import sqlite3
import json
import os
import re
import time
import glob
import hashlib

DB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
DOCS_DIR = os.environ.get("NEOTRIX_DOCS_DIR", os.path.expanduser("~/.neotrix/docs"))
NOW = int(time.time())

# Domain importance mapping
DOMAIN_IMPORTANCE = {
    "0-ARCHITECTURE": 0.9,
    "1-DESIGN": 0.7,
    "2-PLANS": 0.6,
    "3-API": 0.7,
    "4-GUIDES": 0.8,
    "5-LEARNING": 0.9,
    "6-REFERENCE": 0.7,
    "root": 0.5,
}

DOMAIN_NODE_TYPE = {
    "0-ARCHITECTURE": "Article",
    "1-DESIGN": "Article",
    "2-PLANS": "Paper",
    "3-API": "Source",
    "4-GUIDES": "Guide",
    "5-LEARNING": "Guide",
    "6-REFERENCE": "Reference",
    "root": "Article",
}

# Architecture concept definitions
ARCH_CONCEPTS = [
    {
        "id": "arch-e8-engine",
        "node_type": "Framework",
        "title": "E8 推理引擎 (E8 Hexagram Engine)",
        "summary": "64-state deterministic reasoning engine based on 6-axis binary hexagrams. E8 forms the core inference mechanism of NeoTrix, encoding all cognitive states as 6-bit values (0-63) mapped to I-Ching hexagram symbols.",
        "content": """# E8 推理引擎 (E8 Hexagram Engine)

The E8 Hexagram Engine is NeoTrix's core 64-state deterministic reasoning engine. It uses a 6-axis binary encoding scheme where each axis represents a fundamental cognitive dimension:

| Bit | Axis | Description |
|-----|------|-------------|
| 0 | 行动/静观 (Action/Contemplation) | Active vs passive stance |
| 1 | 发散/收敛 (Divergence/Convergence) | Exploratory vs focused thinking |
| 2 | 抽象/具体 (Abstract/Concrete) | High-level vs grounded reasoning |
| 3 | 分析/直觉 (Analysis/Intuition) | Step-by-step vs holistic processing |
| 4 | 批判/接纳 (Critique/Acceptance) | Critical evaluation vs open reception |
| 5 | 自我/世界 (Self/World) | Internal reflection vs external attention |

Key components:
- **E8Policy**: RL-based policy with TD learning and epsilon-greedy exploration across all 64 modes
- **E8TransitionMatrix**: 64×64 transition probability matrix for mode transitions
- **E8RootSystem**: Lie algebra E8 root system visualization (240 roots in 8D space)
- **Trajectory Compression**: Trace compression and trajectory analysis with PRM scoring
- **E8 State Predictor**: Predictive model for next-state anticipation

The E8 engine integrates with PRM (Process Reward Model) for step-level scoring, SAE (Sparse Autoencoder) for feature extraction, and VSA HyperCube for knowledge state embeddings.""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.95,
    },
    {
        "id": "arch-seal-pipeline",
        "node_type": "Framework",
        "title": "SEAL 自进化管道 (Self-Evolving Architecture Loop)",
        "summary": "34-stage self-iteration pipeline enabling NeoTrix to autonomously evolve its own code, knowledge, and capabilities through a continuous loop of monitoring, analysis, editing, and validation.",
        "content": """# SEAL 自进化管道 (Self-Evolving Architecture Loop)

SEAL (Self-Evolving Architecture Loop) is NeoTrix's 34-stage autonomous evolution pipeline. It runs continuously, allowing the system to self-improve through experience.

## Pipeline Stages (ordered)

1. **SnapshotStage** — System state snapshot
2. **AutonomyGateStage** — Autonomy boundary check
3. **MemoryRetrievalStage** — Relevant KB retrieval
4. **GapAnalysisStage** — Capability gap identification
5. **SSMUpdateStage** — State space model update
6. **OpenSourceCompareStage** — External comparison
7. **SelfEditGenStage** — Edit proposal generation
8. **BoundedEditStage** — Constrained self-editing
9. **ApplyEditsStage** — Edit application
10. **RewardCalcStage** — Reward computation
11. **ValidationGateStage** — Cargo check validation
12. **GWTAbsorbStage** — GWT broadcast absorption
13. **StatsSignificanceStage** — Statistical validation
14. **HarnessAdaptStage** — Environment adaptation
15. **TaskAffinityStage** — Task affinity analysis
16. **KnowledgeQualityStage** — Knowledge quality assessment
17. **RollbackDecisionStage** — Rollback decision
18. **RejectedFeedbackStage** — Feedback from rejections
19. **ChampionCompareStage** — Champion model comparison
20. **BankStorageStage** — Inference bank storage
21. **HyperCubeOptimizeStage** — VSA optimization
22. **E8ExperimentStage** — E8 mode experimentation
23. **EpochSlowUpdateStage** — Slow weight updates
24. **SecurityScanStage** — Secret scanning
25. **SessionDistillStage** — Session distillation
26. **ConversationDistillStage** — Conversation pattern extraction
27. **AgingDiagnosisStage** — System aging detection
28. **MetaEvolveStage** — Hyper-meta agent evolution
29. **DGMMetaEvolveStage** — Diffusion-based meta editing
30. **ExternalKnowledgeAbsorbStage** — External knowledge ingestion
31. **CacheCleanupStage** — Build cache cleanup
32. **PRMTrainingStage** — Process reward model training
33. **ProceduralMemoryStage** — Skill proceduralization
34. **ConstitutionalSelfCritiqueStage** — Self-critique alignment""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.95,
    },
    {
        "id": "arch-hypercube-vsa",
        "node_type": "Theory",
        "title": "HyperCube VSA 知识表示 (HyperCube Vector-Symbolic Architecture)",
        "summary": "Knowledge representation using Vector-Symbolic Architectures (VSA) with FHRR, GHRR, and qFHRR encoding schemes operating on 2048/4096-dimensional vectors with MAP-BSC algebra for binding and bundling.",
        "content": """# HyperCube VSA 知识表示

HyperCube is NeoTrix's knowledge representation system based on Vector-Symbolic Architectures (VSA). It encodes knowledge as high-dimensional vectors that support algebraic operations.

## Encoding Schemes

- **FHRR (Fourier Holographic Reduced Representation)**: Phase-based encoding using complex-valued vectors. D=2048 dimensions, rotation-based binding.
- **GHRR (Generalized Holographic Reduced Representation)**: Extended FHRR with configurable dimension and fractional power encoding.
- **qFHRR (Quantized Fourier Holographic Reduced Representation)**: Compressed FHRR with 8-bit integer quantization, reducing memory by 75%.

## Key Properties
- **MAP-BSC Algebra**: Multiplication (binding), Addition (bundling), Permutation (shifting)
- **Similarity Search**: Cosine similarity for nearest-neighbor retrieval
- **Diffusion Process**: Iterative refinement through VSA diffusion for concept blending
- **Rolling Buffer**: Sliding window mechanism for temporal memory
- **Incremental Mean**: Online learning for prototype vectors

## Dimensions
- Primary: 2048 dimensions (FHRR/HyperCube)
- Legacy: 4096 dimensions (KnowledgeHyperCube)
- Quantized: 2048 dimensions × 8-bit integers""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.95,
    },
    {
        "id": "arch-gwt-workspace",
        "node_type": "Theory",
        "title": "GWT 全局工作空间 (Global Workspace Theory Workspace)",
        "summary": "Attention routing system inspired by Global Workspace Theory (GWT) of consciousness. 13 specialist modules compete for access to a global workspace with resonance-based binding via Kuramoto oscillators.",
        "content": """# GWT 全局工作空间 (Global Workspace Theory Workspace)

NeoTrix's GWT (Global Workspace) implements Baars' Global Workspace Theory of consciousness. It provides an attention-routing mechanism where specialist modules compete for access to a shared workspace.

## Architecture
- **13 Specialist Modules**: Each module detects specific patterns (code, math, security, social, etc.)
- **Competition Gate**: Modules compete via dynamic expert balancing with MoE router
- **Resonance Binding**: Kuramoto oscillator network synchronizes specialist outputs
- **Context Engine**: Hierarchical context management with 5-layer compression pipeline

## Key Components
- **GlobalWorkspace**: Central workspace with competition, resonance, and broadcast
- **MoERouter**: Mixture-of-Experts routing with DynamicExpertBalancer
- **CompetitionGate**: Multi-headed attention-based competition
- **ResonanceReport**: Oscillator synchronization metrics
- **ContextBudget**: Token budget allocation with compression rules
- **AuditBlock**: Attention audit trail for transparency

## Compression Pipeline (5-layer)
1. Budget → 2. Trim → 3. Compress → 4. Fold → 5. Auto-compress""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.95,
    },
    {
        "id": "arch-prm",
        "node_type": "Algorithm",
        "title": "PRM 过程奖励模型 (Process Reward Model)",
        "summary": "Step-level process reward model for scoring intermediate reasoning steps. Implements Monte Carlo PRM, Math-Shepherd, and GRPO variants for fine-grained reward assignment.",
        "content": """# PRM 过程奖励模型 (Process Reward Model)

PRM (Process Reward Model) provides step-level scoring for the E8 reasoning engine's inference trajectories, enabling fine-grained reward signals beyond outcome-only rewards.

## Variants
- **MC-PRM**: Monte Carlo Process Reward Model — scores each step via rollouts
- **Math-Shepherd**: Step-level PRM with automatic label generation
- **WS-GRPO**: Group Relative Policy Optimization with weighted sampling
- **ConcreteGRPO**: Quantized GRPO with concrete distributions
- **PRM+GRPO**: Combined PRM and GRPO training loop

## Key Components
- **ProcessRewardLearner**: Learning with pairwise preference comparisons
- **ProcessScorer**: Step-level scoring with heuristic and learned modes
- **HeuristicCoach**: Provides initial training signal without human labels
- **TrajectoryBuffer**: Stores reasoning trajectories with step rewards

## Integration
PRM is wired into the L6 Self model via engine_core's ProcessRewardLearner field. It scores each intermediate E8 state transition, feeding into the SEAL pipeline's RewardCalcStage.""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.85,
    },
    {
        "id": "arch-sae",
        "node_type": "Algorithm",
        "title": "SAE 稀疏自编码器 (Sparse Autoencoder)",
        "summary": "Sparse autoencoder for feature extraction from E8 latent representations. Supports TopK sparsification and feature steering for interpretable control of reasoning.",
        "content": """# SAE 稀疏自编码器 (Sparse Autoencoder)

SAE (Sparse Autoencoder) extracts interpretable features from the E8 engine's latent representations. Inspired by Anthropic's research on monosemanticity, it enables feature-level steering of the reasoning process.

## Key Components
- **SparseAutoencoder**: Core encoder/decoder with TopK sparsification (keep top 10%)
- **FeatureSteeringController**: Inject feature activations to guide reasoning direction
- **SAEBridge**: Connects SAE to E8 state vectors during reasoning

## Features
- TopK activation sparsification
- Feature steering via latent injection
- SAE feature extraction during every E8 transition
- Integration with GWT for feature-level attention audit

## Purpose
Enables interpretable and steerable AI by decomposing the E8 state space into human-understandable features.""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.85,
    },
    {
        "id": "arch-shield",
        "node_type": "Framework",
        "title": "NT-SHIELD 安全系统",
        "summary": "Two-layer security sandbox system with SandboxEnforcer (disabled/read-only/docker modes) and CloudSandbox providers. Includes vault, permission system, guardrails, and secret scanning.",
        "content": """# NT-SHIELD 安全系统

NT-SHIELD provides comprehensive security for NeoTrix across multiple layers.

## Core Components
- **SandboxEnforcer**: Global mutex-backed mode toggler (disabled / read-only / docker)
- **CloudSandbox**: Multi-runtime sandbox (Python3, Node18, Rust, Go, Linux) via Docker
- **Key Vault**: Secure credential storage with OS keychain integration
- **Permission System**: 22 security rules (SEC-001 to SEC-022) for tool call inspection
- **Guardrails**: Constitutional AI safety constraints
- **Secret Scanner**: 13 regex patterns for credential detection
- **Prompt Injection Protection**: Input sanitization and attack detection
- **Tool Inspection Stack**: 5-layer tool call security pipeline

## Security Layers
1. **L1 — Physical/Network**: Proxy routing, port scanning, HTTP interception
2. **L2 — Execution**: Docker sandbox, resource limits (512MB memory, 1 CPU)
3. **L3 — Permission**: 22-rule permission system with chain-based approval
4. **L4 — Content**: Prompt injection, secret scanning, browser security
5. **L5 — Audit**: Port scan audit, security audit trails""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.85,
    },
    {
        "id": "arch-act",
        "node_type": "Framework",
        "title": "NT-ACT 行动系统",
        "summary": "Action tool system for interacting with the physical/digital world. Includes crypto finance engine, social media connectors, voice interaction, file sync, code generation, and goal management.",
        "content": """# NT-ACT 行动系统

NT-ACT provides NeoTrix with the ability to act upon the world — from financial transactions to social media posting to code generation.

## Subsystems
- **nt_act_crypto**: Crypto finance engine — wallet management, DEX swaps, cross-chain bridging, yield farming, portfolio tracking, cost/budget management
- **nt_act_social**: Social connectors — Twitter/X, Reddit, YouTube, TikTok posting and monitoring
- **nt_act_sync**: File sync and backup across devices
- **nt_act_code**: Self code generation and AST manipulation
- **nt_act_goal**: Self goal management with conflict resolution
- **nt_act_autonomy**: Autonomous decision engine with cross-session memory
- **nt_act_voice**: Voice interaction with TTS/STT
- **nt_act_spear**: SPEAR protocol for secure agent communication
- **nt_act_gram**: NeoGram messaging
- **nt_act_earn**: Yield/earn engine
- **nt_act_worktree**: Git worktree management with E8 metadata""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.8,
    },
    {
        "id": "arch-world",
        "node_type": "Framework",
        "title": "NT-WORLD 感知系统",
        "summary": "World perception system providing browser automation, web crawling, search, scraping, and sensory input processing. Implements the Free Energy Principle for active inference.",
        "content": """# NT-WORLD 感知系统

NT-WORLD is NeoTrix's sensory interface to the physical and digital world. It provides comprehensive perception capabilities.

## Subsystems
- **nt_world_browse**: Browser automation with anti-detection and human-like interaction patterns
- **nt_world_crawl**: Web crawling with rate limiting and politeness policies
- **nt_world_search**: Web search via multiple search engines
- **nt_world_scrape**: HTML content extraction and parsing
- **nt_world_sense**: Sensory input processing (vision, audio)
- **nt_world_model**: World model with E8 and JEPA variants
- **nt_world_jepa**: JEPA world model (encoder/predictor/loss)
- **nt_world_infer**: Active inference based on Free Energy Principle
- **nt_world_pred**: HyperCube-based world prediction
- **nt_world_exploration_engine**: Continuous multi-source exploration (GitHub, ArXiv, Web)
- **nt_world_github_absorber**: Full GitHub repository absorption pipeline
- **nt_world_absorber**: Unified external source orchestrator""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.8,
    },
    {
        "id": "arch-io",
        "node_type": "Framework",
        "title": "NT-IO 人机界面",
        "summary": "Human-computer interface layer providing CLI (27 command modules), TUI (Ratatui), HTTP/WS/WebRTC server, desktop app (Tauri), avatar system, and session management.",
        "content": """# NT-IO 人机界面

NT-IO is NeoTrix's interface layer for human interaction, providing multiple interaction modalities.

## Subsystems
- **nt_io_cli**: Command-line interface with 27 command modules (smart help, aggregation, categorization)
- **nt_io_tui**: Terminal UI built with Ratatui
- **nt_io_server**: Server with HTTP, WebSocket, and WebRTC support
- **nt_io_proxy**: Proxy daemon for remote connectivity
- **nt_io_boot**: Boot entry points (desktop, headless, server)
- **nt_io_web**: Web UI interface
- **nt_io_notify**: Cross-platform notification system
- **nt_io_avatar**: 3D avatar system with emotional expressions
- **nt_io_session_recovery**: Git-backed session snapshot and recovery
- **nt_io_agents_md**: AGENTS.md/CLAUDE.md/.cursorrules discovery and parsing
- **nt_io_logging**: Tracing and logging infrastructure
- **nt_io_lsp**: Language server protocol integration""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.8,
    },
    {
        "id": "arch-memory",
        "node_type": "Framework",
        "title": "NT-MEMORY 知识库 (Knowledge Base)",
        "summary": "SQLite-based persistent knowledge base with FTS5 full-text search, BM25 ranking, embedding-based similarity search, and graph traversal. 22 node types, 19 relation types, hierarchical memory with cortex dimensions.",
        "content": """# NT-MEMORY 知识库

NT-MEMORY provides NeoTrix with persistent long-term memory using SQLite as the storage backend.

## Key Capabilities
- **22 Node Types**: Concept, Paper, Repository, Person, Event, Source, Tool, Framework, Algorithm, Theory, Method, Dataset, Benchmark, Organization, Book, Course, Article, CodeSnippet, Idea, Question, Insight, Image
- **19 Relation Types**: Related, PartOf, Implements, Extends, References, Uses, DependsOn, SimilarTo, OppositeTo, Impacts, Supports, Contradicts, EvolvesInto, Precedes, Follows, Causes, Requires, AssociatedWith, TranslatedTo
- **FTS5 Search**: Full-text search on node title and content (~0.16ms)
- **BM25 Fallback**: Okapi BM25 ranking (~0.33ms)
- **Hybrid Search**: FTS5 → embedding cosine rerank → top-N (0.3×FTS + 0.7×cos)
- **Graph Traversal**: BFS and subgraph queries
- **Embedding API**: OpenAI-compatible batch embedding
- **Memory Cortex**: Dimensional tagging for multi-modal memory
- **Conversation Records**: Stores user↔AI interactions for evolution feedback
- **Evolution Records**: Knowledge patterns extracted from conversation analysis""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.85,
    },
    {
        "id": "arch-mind",
        "node_type": "Framework",
        "title": "NT-MIND 自我进化系统",
        "summary": "Self-evolution system with SEAL 34-stage pipeline, skill engine, aging diagnosis, cleanup management, knowledge absorption, and self-diagnosis capabilities.",
        "content": """# NT-MIND 自我进化系统

NT-MIND manages NeoTrix's autonomous self-evolution — the system's ability to monitor, diagnose, and improve its own codebase and capabilities.

## Subsystems
- **nt_mind_seal**: 34-stage SEAL self-iteration pipeline (core evolution mechanism)
- **nt_mind_brain**: NeoTrix thinking core (coordination of cognitive subsystems)
- **nt_mind_strat**: Self-edit strategy (Conservative/Aggressive/DGM modes)
- **nt_mind_skill**: Skill optimization (BoundedEdit, ValidationGate, EpochSlowUpdate)
- **nt_mind_adapt**: Environment adapter (cross-model transfer, harness adapt)
- **nt_mind_age**: Aging diagnosis (4-indicator aging detection)
- **nt_mind_scan**: Secret scanning (13 regex patterns, GWT alerts)
- **nt_mind_valid**: Validation gate (cargo check gating)
- **nt_mind_sia**: Self-improving agent loop
- **nt_mind_hmeta**: Hyper-meta agent (self-modification proposals)
- **nt_mind_edit**: Self-edit operations
- **nt_mind_sleep**: Offline memory consolidation
- **nt_mind_dgm**: DGM diffusion-based self-editing
- **nt_mind_skill_engine**: Skills auto-invocation system (YAML front-matter scanning, E8 pattern matching, GWT broadcast)
- **nt_mind_hook**: 25+ lifecycle hook events
- **nt_mind_cleanup**: Automatic cleanup/archive system
- **nt_mind_automation**: Event-driven automation (5 trigger types, 4 action types)
- **nt_mind_self_diagnose**: System self-diagnosis with issue detection
- **nt_mind_evolution_loop**: Continuous evolution loop
- **nt_mind_background_loop**: Background processing loop""",
        "domain": "0-ARCHITECTURE",
        "importance": 0.9,
    },
]

def extract_title(content, filepath):
    """Extract title from first heading or filename."""
    lines = content.split('\n')
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('# ') and not stripped.startswith('# '):
            # Check it's not a markdown header anchor
            return stripped[2:].strip()
        if stripped.startswith('# '):
            return stripped[2:].strip()
    # Fallback: use filename without extension
    basename = os.path.basename(filepath)
    name = os.path.splitext(basename)[0]
    # Make it human-readable
    name = name.replace('-', ' ').replace('_', ' ').title()
    return name


def extract_summary(content):
    """Extract first meaningful paragraph as summary (max 200 chars)."""
    lines = content.split('\n')
    in_frontmatter = False
    for line in lines:
        stripped = line.strip()
        if stripped == '---':
            in_frontmatter = not in_frontmatter
            continue
        if in_frontmatter:
            continue
        if stripped.startswith('#'):
            continue
        if stripped.startswith('|') and not any(c.isalpha() for c in stripped):
            continue
        if stripped and len(stripped) > 20 and not stripped.startswith('!['):
            # Found first real paragraph
            # Clean markdown formatting
            summary = stripped
            # Remove bold/italic markers
            summary = summary.replace('**', '').replace('__', '').replace('*', '').replace('_', '')
            # Remove links keeping text
            summary = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', summary)
            if len(summary) > 200:
                summary = summary[:197] + '...'
            return summary
    return ''


def escape_sql(val):
    """Escape a value for SQLite single-quoted string."""
    if val is None:
        return 'NULL'
    s = str(val)
    s = s.replace("'", "''")
    return f"'{s}'"


def insert_node(cursor, node_id, node_type, title, summary, content, url, domain, importance, metadata=None):
    """Insert a knowledge node."""
    cursor.execute("""
        INSERT OR REPLACE INTO nodes
        (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'en', 0.9, ?, ?, ?, ?)
    """, (
        node_id, node_type, title, summary, content, url, domain,
        importance, NOW, NOW,
        json.dumps(metadata) if metadata else None
    ))


def insert_edge(cursor, source_id, target_id, relation_type, weight=1.0, metadata=None):
    """Insert a knowledge edge."""
    edge_id = hashlib.sha256(f"{source_id}:{target_id}:{relation_type}".encode()).hexdigest()[:32]
    cursor.execute("""
        INSERT OR IGNORE INTO edges
        (id, source_id, target_id, relation_type, weight, created_at, metadata)
        VALUES (?, ?, ?, ?, ?, ?, ?)
    """, (
        edge_id, source_id, target_id, relation_type, weight, NOW,
        json.dumps(metadata) if metadata else None
    ))


def main():
    conn = sqlite3.connect(DB)
    cursor = conn.cursor()
    
    doc_count = 0
    edge_count = 0
    errors = []
    
    # Phase 1: Insert architecture concept nodes
    print("=== Phase 1: Architecture Concept Nodes ===")
    arch_ids = []
    for concept in ARCH_CONCEPTS:
        try:
            insert_node(
                cursor,
                concept["id"],
                concept["node_type"],
                concept["title"],
                concept["summary"],
                concept["content"],
                f"file://docs/0-ARCHITECTURE/{concept['id'].replace('arch-', '')}.md",
                concept["domain"],
                concept.get("importance", 0.9),
                {"source": "architecture_absorb", "type": "architecture_concept", "lines": len(concept["content"].split('\n'))}
            )
            arch_ids.append(concept["id"])
            print(f"  ✓ {concept['id']}")
        except Exception as e:
            errors.append(f"Arch concept {concept['id']}: {e}")
            print(f"  ✗ {concept['id']}: {e}")
    
    doc_count += len(ARCH_CONCEPTS)
    
    # Phase 2: Scan and insert all doc files
    print("\n=== Phase 2: Document Nodes ===")
    md_files = sorted(glob.glob(os.path.join(DOCS_DIR, '**', '*.md'), recursive=True))
    
    # Map of node_id -> domain for edge creation
    doc_nodes = {}
    
    for filepath in md_files:
        rel_path = os.path.relpath(filepath, DOCS_DIR)
        parts = rel_path.split(os.sep)
        
        # Determine domain
        domain = parts[0] if len(parts) > 1 else "root"
        
        # Create node ID — preserve original filename case for uniqueness
        raw_slug = os.path.splitext(parts[-1])[0]
        safe_slug = re.sub(r'[^a-zA-Z0-9]', '-', raw_slug)
        safe_slug = re.sub(r'-+', '-', safe_slug).strip('-')
        node_id = f"doc-{domain}-{safe_slug}" if domain != "root" else f"doc-root-{safe_slug}"
        
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
            
            title = extract_title(content, filepath)
            summary = extract_summary(content)
            node_type = DOMAIN_NODE_TYPE.get(domain, "Article")
            importance = DOMAIN_IMPORTANCE.get(domain, 0.5)
            
            lines = content.count('\n') + 1
            size_bytes = len(content.encode('utf-8'))
            
            insert_node(
                cursor,
                node_id, node_type, title, summary, content,
                f"file://docs/{rel_path}",
                domain, importance,
                {"file_type": "markdown", "lines": lines, "size_bytes": size_bytes}
            )
            
            doc_nodes[node_id] = domain
            doc_count += 1
            print(f"  ✓ {rel_path} ({lines} lines)")
            
        except Exception as e:
            errors.append(f"{rel_path}: {e}")
            print(f"  ✗ {rel_path}: {e}")
    
    # Phase 3: Create relationship edges (docs to their domain concept)
    print("\n=== Phase 3: Relationship Edges ===")
    
    domain_to_arch = {
        "0-ARCHITECTURE": "arch-e8-engine",
        "1-DESIGN": "arch-mind",
        "2-PLANS": "arch-seal-pipeline",
        "3-API": "arch-io",
        "4-GUIDES": "arch-io",
        "5-LEARNING": "arch-mind",
        "6-REFERENCE": "arch-shield",
    }
    
    for node_id, domain in doc_nodes.items():
        try:
            arch_target = domain_to_arch.get(domain)
            if arch_target:
                insert_edge(cursor, node_id, arch_target, "Related", 0.8, {"mapping": "doc_to_architecture"})
                edge_count += 1
        except Exception as e:
            errors.append(f"Edge {node_id} → {arch_target}: {e}")
    
    # Cross-reference edges between architecture concepts
    arch_pairs = [
        ("arch-e8-engine", "arch-prm", "References"),
        ("arch-e8-engine", "arch-sae", "References"),
        ("arch-e8-engine", "arch-hypercube-vsa", "Implements"),
        ("arch-seal-pipeline", "arch-mind", "PartOf"),
        ("arch-gwt-workspace", "arch-e8-engine", "References"),
        ("arch-hypercube-vsa", "arch-gwt-workspace", "Implements"),
        ("arch-prm", "arch-seal-pipeline", "PartOf"),
        ("arch-shield", "arch-mind", "Related"),
        ("arch-act", "arch-e8-engine", "Uses"),
        ("arch-world", "arch-e8-engine", "Uses"),
        ("arch-io", "arch-gwt-workspace", "Uses"),
        ("arch-memory", "arch-hypercube-vsa", "Implements"),
    ]
    
    for src, tgt, rel in arch_pairs:
        try:
            insert_edge(cursor, src, tgt, rel, 1.0, {"source": "architecture_absorb"})
            edge_count += 1
        except Exception as e:
            errors.append(f"Arch edge {src} → {tgt}: {e}")
    
    # Phase 4: Create knowledge architecture index in kv_store
    print("\n=== Phase 4: Knowledge Architecture Index ===")
    
    index_data = {
        "architecture_concepts": {c["id"]: {"title": c["title"], "type": c["node_type"]} for c in ARCH_CONCEPTS},
        "domains": {
            domain: {
                "importance": DOMAIN_IMPORTANCE[domain],
                "docs": [nid for nid, d in doc_nodes.items() if d == domain]
            }
            for domain in set(doc_nodes.values())
        },
        "total_docs": len(md_files),
        "total_arch_concepts": len(ARCH_CONCEPTS),
        "total_edges": edge_count,
        "absorbed_at": NOW,
    }
    
    try:
        cursor.execute("""
            INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at)
            VALUES (?, ?, ?, ?)
        """, ("knowledge_architecture", "index", json.dumps(index_data, ensure_ascii=False), NOW))
        print("  ✓ knowledge_architecture/index")
    except Exception as e:
        errors.append(f"kv_store index: {e}")
        print(f"  ✗ kv_store index: {e}")
    
    conn.commit()
    conn.close()
    
    print(f"\n{'='*60}")
    print(f"Absorption Complete!")
    print(f"  Nodes created: {doc_count}")
    print(f"  Edges created: {edge_count}")
    print(f"  Errors: {len(errors)}")
    if errors:
        print(f"\nErrors:")
        for e in errors:
            print(f"  - {e}")
    
    return doc_count, edge_count, errors


if __name__ == "__main__":
    main()
