# NeoTrix Native App — osaurus 1:1 Replication in Rust

## Overview

Replicate [osaurus-ai/osaurus](https://github.com/osaurus-ai/osaurus) 1:1 in NeoTrix's Rust architecture, then integrate with NeoTrix backend APIs.

**osaurus Description**: "Native macOS harness for AI agents — any model, persistent memory, autonomous execution, cryptographic identity. Built in Swift. Fully offline. Open source."

## Target Architecture

NeoTrix 6-Layer Architecture:
```
L6 Meta-Cognition (元认知层)  →  nt_meta + nt_repair + nt_nexus
L5 Cognition (认知层)         →  nt_core + nt_mind
L4 Emotion (情感层)           →  nt_feel (core emotion engine)
L3 Embodiment (具身层)        →  nt_physical + nt_shield + nt_feel
L2 Perception (感知层)        →  nt_world + nt_sense
L1 Action (行动层)            →  nt_act + nt_io + nt_memory
```

## Feature Parity Matrix

| osaurus Feature | NeoTrix Implementation | Layer |
|-----------------|------------------------|-------|
| **Any Model Support** | Multi-provider LLM gateway (Ollama, OpenAI, Anthropic, local) | L1 nt_io |
| **Persistent Memory** | KB (SQLite + FTS5 + embeddings) + ConsciousnessTree cross-session | L1 nt_memory + L6 nt_nexus |
| **Autonomous Execution** | AgentLoop + SEAL pipeline + CapabilityTree auto-discovery | L5 nt_mind + L1 nt_act |
| **Cryptographic Identity** | Ed25519 keypair + DID + signing/verification | L3 nt_shield |
| **Fully Offline** | Local-first: Ollama, embedded models, KB local | All layers |
| **Native App** | Tauri-free native Rust binary + optional Tauri wrapper | L1 nt_io |

## Module Structure

```
neotrix-core/src/neotrix/nt_native_app/
├── mod.rs                    # Public API
├── types.rs                  # Core types
├── model_registry.rs         # Any model support
├── memory.rs                 # Persistent memory (KB integration)
├── agent_loop.rs             # Autonomous execution
├── crypto_identity.rs        # Cryptographic identity
├── offline.rs                # Offline-first capabilities
├── cli.rs                    # CLI commands
└── integration.rs            # Backend API integration
```

## Core Types

### Model Registry
```rust
pub struct ModelRegistry {
    providers: HashMap<String, Box<dyn ModelProvider>>,
    active_model: Option<ModelId>,
    config: ModelConfig,
}

pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &str;
    fn models(&self) -> Vec<ModelInfo>;
    fn complete(&self, req: CompletionRequest) -> impl Stream<Item = Result<CompletionChunk>>;
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
```

### Persistent Memory
```rust
pub struct NativeMemory {
    kb: KnowledgeBase,
    consciousness_tree: ConsciousnessTree,
    session_store: SessionStore,
}

pub struct Session {
    id: SessionId,
    messages: VecDeque<ChatMessage>,
    context: SessionContext,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### Agent Loop (Autonomous Execution)
```rust
pub struct AgentLoop {
    model_registry: Arc<ModelRegistry>,
    memory: Arc<NativeMemory>,
    capabilities: CapabilityRegistry,
    tools: ToolRegistry,
    config: AgentConfig,
}

pub struct AgentConfig {
    pub max_turns: usize,
    pub approval_mode: ApprovalMode,
    pub sandbox_mode: SandboxMode,
    pub budget_usd: Option<f64>,
}
```

### Cryptographic Identity
```rust
pub struct CryptoIdentity {
    keypair: Ed25519KeyPair,
    did: DidDocument,
    key_store: KeyStore,
}

pub struct DidDocument {
    id: String,
    verification_method: Vec<VerificationMethod>,
    authentication: Vec<String>,
    created: DateTime<Utc>,
}
```

## Integration Points

### NeoTrix Backend APIs

| Component | Integration |
|-----------|-------------|
| **E8 Hexagram** | Reasoning engine for agent decisions |
| **GWT** | Attention routing for multi-agent coordination |
| **VSA HyperCube** | Knowledge representation & analogical reasoning |
| **ConsciousnessTree** | Meta-cognition, self-evolution tracking |
| **KB** | Persistent storage, search, embeddings |
| **SEAL Pipeline** | Self-evolution loops |
| **CapabilityTree** | Dynamic capability discovery |
| **nt_shield** | Security, sandboxing, crypto |

### CLI Commands

```bash
neotrix native-app [COMMAND]

Commands:
  run              Run native app (interactive)
  model list       List available models
  model select     Select active model
  memory query     Query persistent memory
  memory export    Export memory
  identity show    Show DID/keypair
  identity rotate  Rotate keys
  agent start      Start autonomous agent
  agent stop       Stop agent
  offline sync     Sync offline changes
```

## Implementation Phases

### Phase 1: Core Types & Model Registry (Week 1)
- [ ] ModelRegistry with provider trait
- [ ] Ollama, OpenAI, Anthropic providers
- [ ] Local model detection (llama.cpp, ollama)
- [ ] Model selection & switching

### Phase 2: Persistent Memory (Week 1)
- [ ] KB integration (SQLite + FTS5)
- [ ] Session management
- [ ] Cross-session continuity (nt_nexus)
- [ ] Memory search & retrieval

### Phase 3: Autonomous Agent Loop (Week 2)
- [ ] AgentLoop with tool calling
- [ ] Approval modes (Suggest/AutoEdit/FullAuto)
- [ ] Sandbox integration
- [ ] Budget tracking

### Phase 4: Cryptographic Identity (Week 2)
- [ ] Ed25519 keypair generation
- [ ] DID document creation
- [ ] Key rotation & backup
- [ ] Signing/verification

### Phase 5: Offline-First (Week 2)
- [ ] Local model management
- [ ] Embedded KB
- [ ] Offline queue & sync
- [ ] Air-gap support

### Phase 6: Backend Integration (Week 3)
- [ ] E8 reasoning for decisions
- [ ] GWT attention routing
- [ ] ConsciousnessTree meta-cognition
- [ ] SEAL self-evolution

### Phase 7: CLI & Polish (Week 3)
- [ ] CLI commands
- [ ] TUI (ratatui-based)
- [ ] Config management
- [ ] Tests & benchmarks

## Configuration

```toml
# ~/.config/neotrix/native-app.toml
[model]
active = "ollama/llama3.1"
providers = ["ollama", "openai", "anthropic", "local"]

[memory]
kb_path = "~/.neotrix/kb.sqlite"
embedding_model = "nomic-embed-text"
max_sessions = 1000

[agent]
max_turns = 50
approval_mode = "suggest"
sandbox_mode = "read-only"
budget_usd = 10.0

[identity]
auto_generate = true
key_path = "~/.neotrix/identity.ed25519"
did_method = "key"

[offline]
enabled = true
sync_interval = 300
queue_path = "~/.neotrix/offline_queue"
```

## Success Criteria

1. **Functional Parity**: All osaurus features work in NeoTrix
2. **Performance**: < 100ms cold start, < 50ms model switch
3. **Offline**: Full functionality without network
4. **Integration**: Seamless NeoTrix backend API usage
5. **Tests**: > 90% coverage, all integration tests pass
6. **Binary Size**: < 50MB stripped

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Swift-specific APIs | Use cross-platform Rust crates (tauri not required) |
| Model provider changes | Abstract provider trait, versioned APIs |
| KB schema migration | Versioned migrations, backward compat |
| Crypto key loss | Shamir secret sharing backup, hardware key support |

## Dependencies

```toml
# Core
neotrix-core = { path = ".." }
neotrix-unified = { path = "../unified" }

# Crypto
ed25519-dalek = "2.0"
didkit = "0.4"
x25519-dalek = "2.0"

# Models
ollama-rs = "0.9"
async-openai = "0.20"
anthropic-rs = "0.5"
llama-cpp-2 = { version = "0.10", features = ["metal"] }

# Storage
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "chrono", "uuid"] }
sled = "0.34"

# Async
tokio = { version = "1.40", features = ["full"] }
futures = "0.3"

# CLI
clap = { version = "4.5", features = ["derive", "env"] }
ratatui = "0.28"
crossterm = "0.28"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
```

---

**Next Step**: Implement Phase 1 - Core Types & Model Registry