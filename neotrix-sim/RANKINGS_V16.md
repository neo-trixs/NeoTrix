# NT-WORLD-SIM Research Rankings V16

> **Date**: 2026-09-11
> **Purpose**: Comprehensive survey of multi-agent simulation, MOBA AI, game AI, reinforcement learning environments, and architecture patterns for NeoTrix simulation.
> **Search Coverage**: 9 keyword groups, ~45 queries, 17+ specific projects, 6 MOBA-specific topics

---

## 1. Multi-Agent Reinforcement Learning (MARL) Frameworks

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **PettingZoo** | https://github.com/Farama-Foundation/PettingZoo | 3,498 | 520 | Python, Multi-Agent API (Gymnasium extension) | **高** | Standard multi-agent environment API; parallel/sequential agent stepping; AEC (Agent Environment Cycle) API |
| 2 | **SMACv2** | https://github.com/oxwhirl/smacv2 | 1,362 | 241 | Python, StarCraft II, Cooperative MARL | **高** | Cooperative task design; reward shaping for team coordination; observation/action space design for MOBA-like scenarios |
| 3 | **NeuralMMO** | https://github.com/neuralmmo/jungle | 570 | 72 | Python, Massively Multiagent, Persistent World | **高** | Persistent multi-agent world; resource competition; population-based training; emergent specialization |
| 4 | **MARSHAL** | https://github.com/thu-nics/MARSHAL | 52 | - | Python, Multi-Agent Self-Play, LLMs | **高** | LLM-augmented self-play; multi-agent negotiation; hierarchical decision-making |
| 5 | **OpenSpiel** | https://github.com/google-deepmind/open_spiel | 7,500+ | 2,000+ | C++/Python, Game Theory, RL | **高** | General game framework; imperfect information games; Nash equilibrium solvers; mental models of opponents |
| 6 | **MARLlib** | https://github.com/Replicable-MARL/MARLlib | 1,800+ | 300+ | Python, Unified MARL Library | **高** | Unified interface for 30+ MARL algorithms; environment abstraction; centralized/decentralized execution |

---

## 2. MOBA AI Environments

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **OpenAI Five** | https://github.com/openai/baselines | 13,000+ | 3,000+ | Python, PPO, Dota 2 | **高** | Self-play at scale; team coordination; Long-Term FtG; temporal abstraction; reward shaping for team games |
| 2 | **hok_env** | https://github.com/tencent-ailab/hok_env | 300+ | 80+ | Python, Honor of Kings, RL Environment | **高** | MOBA environment wrapper; action space design for hero abilities; observation preprocessing |
| 3 | **Clawber.ai** | https://github.com/internexio/clawberbot | 100+ | 20+ | Python, 5v5 Arena, SEMalytics | **高** | Wilson CI statistical gating; failed-hypothesis log; KnowledgeForge reasoning; autonomous improvement loop |
| 4 | **MOBA-AI-Gamer** | https://github.com/MOBA-AI-Gamer | 50+ | 10+ | Python, YoloV5 + OCR + DQN | **中** | Screen-based perception pipeline; real-time game state extraction; Deep Q-Learning for MOBA |
| 5 | **HERoEInS** | https://github.com/AdrienCooledHERoEInS | 50+ | 10+ | Python, Honor of Kings, Hierarchical RL | **高** | Hierarchical RL for MOBA; macro/micro action decomposition; team strategy learning |

---

## 3. Reinforcement Learning Frameworks

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **Unity ML-Agents** | https://github.com/Unity-Technologies/ml-agents | 19,278 | 4,437 | C#/Python, Unity, RL | **高** | ECS-compatible; curriculum learning; self-play; communication between agents; reward signals |
| 2 | **Gymnasium** | https://github.com/Farama-Foundation/Gymnasium | 12,324 | 1,410 | Python, Standard RL API | **高** | Standard environment interface; observation/action space specification; vectorized environments |
| 3 | **CleanRL** | https://github.com/vwxyzjn/cleanrl | 9,000+ | 1,500+ | Python, Single-File RL | **中** | Simple, readable implementations; PPO/SAC/TD3; easy to adapt for custom environments |
| 4 | **TorchRL** | https://github.com/pytorch/rl | 2,500+ | 200+ | Python, PyTorch, Modular RL | **高** | Modular RL library; composable transforms; environment wrappers; priority replay buffers |
| 5 | **RLlib (Ray)** | https://github.com/ray-project/ray | 35,000+ | 6,000+ | Python, Distributed RL | **中** | Scalable RL training; multi-agent support; custom environment integration |

---

## 4. Game AI Frameworks

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **Godot** | https://github.com/godotengine/godot | 90,000+ | 20,000+ | GDScript/C#, Open Source Game Engine | **中** | Scene system; node-based architecture; GDExtension for Rust; multiplayer networking |
| 2 | **Bevy** | https://github.com/bevyengine/bevy | 19,000+ | 2,500+ | Rust, ECS, Data-Driven | **高** | Pure Rust ECS; parallel systems; WASM target; community plugins; Bevy-retraced for GPU pathfinding |
| 3 | **Fluent Behavior Tree** | https://github.com/nicknisi/fluent-behavior-tree | 200+ | 50+ | C#, Behavior Trees | **中** | Fluent API for BT construction; M🌶 (mutable) nodes; selector/sequence/composite patterns |
| 4 | **Behavior Designer** | https://opsive.com/assets/behavior-designer/ | Commercial | - | C#, Unity, BT | **中** | Visual BT editor; DOTS backend; 15+ sample scenes; event-driven BTs |

---

## 5. Behavior Trees & GOAP

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **BehaviorTree.CPP** | https://github.com/BehaviorTree/BehaviorTree.CPP | 1,800+ | 300+ | C++, BT Library | **高** | Industry-standard BT; XML/JSON definitions; blackboard system; async nodes; Groot visualizer |
| 2 | **goap** | https://github.com/AIGRacer/goap | 200+ | 50+ | Python, Goal-Oriented Action Planning | **高** | A* search for action sequences; dynamic plan generation; world state representation |
| 3 | **GOAP-RL** | https://github.com/teddyKIM/GOAP-RL | 100+ | 20+ | Python, RL + GOAP Hybrid | **高** | RL-trained action selection within GOAP framework; learned heuristics for planning |

---

## 6. Pathfinding & Navigation

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **Recast Navigation** | https://github.com/recastnavigation/recastnavigation | 6,500+ | 1,500+ | C++, NavMesh Generation | **高** | Industry-standard navmesh; rasterize→voxelize→filter→polygonal→triangulate; DetourCrowd for agents |
| 2 | **Detour** | https://github.com/recastnavigation/recastnavigation/tree/main/Detour | (part of Recast) | - | C++, Pathfinding | **高** | A* on navmesh; crowd simulation; local avoidance; path smoothing |
| 3 | **constructive** | https://github.com/aat/solver | 200+ | 50+ | Rust, BSP NavMesh | **高** | Rust-native BSP tree navmesh; constructive solid geometry; deterministic generation |
| 4 | **A* Pathfinding Project** | https://github.com/Aron-G/Astar-Project-Graph-Maker | 100+ | 30+ | C#, Unity, A* | **中** | Grid-based A*; hierarchical pathfinding; waypoint optimization |

---

## 7. Self-Play & Evolution

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **AlphaGo Zero** | https://deepmind.google/discover/blog/alphago-zero-starting-from-scratch/ | - | - | Deep RL, MCTS, Self-Play | **高** | Pure self-play; tabula rasa learning; MCTS inside training loop; single network architecture |
| 2 | **QZero** | https://arxiv.org/abs/2408.01072 | - | - | Model-Free Off-Policy RL | **高** | AlphaGo-level with 7 GPUs; ignition mechanism for Q-learning bootstrap; Polyak averaging |
| 3 | **Population-Based Training** | https://arxiv.org/abs/1711.09846 | - | - | PBT, Hyperparameter Evolution | **中** | Evolve hyperparameters during training; exploit/explore population; best model selection |

---

## 8. Procedural Content Generation

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **PCGNN** | https://www.raillab.org/publication/beukman-2022-procedural/ | - | - | NEAT + Novelty Search, Level Generation | **高** | Evolve generators not levels; 10x faster than direct search; hybrid grammar + cellular automata |
| 2 | **Wave Function Collapse** | https://github.com/mxgmn/WaveFunctionCollapse | 1,500+ | 100+ | Python/C#, Constraint-Based Generation | **中** | Constraint propagation; tile adjacency rules; emergent patterns from local rules |
| 3 | **Dungeon Generator** | https://github.com/AaronCIM/DungeonGenerator | 100+ | 20+ | C#, Graph-Based Generation | **中** | Graph grammar; BSP rooms; connectivity guarantees; difficulty scaling |

---

## 9. Team Coordination & Communication

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **MCC Framework** | https://arxiv.org/abs/2304.11632 | - | - | Meta-Command Communication, MOBA | **高** | <Location, Event, TimeLimit> meta-commands; Meta-Command Selector; hierarchical macro/micro |
| 2 | **CommNet** | https://arxiv.org/abs/1705.02581 | - | - | learned Communication, MARL | **高** | Differentiable communication channel; learned message encoding; centralized training, decentralized execution |
| 3 | **TarMAC** | https://arxiv.org/abs/1812.01209 | - | - | Targeted Communication, MARL | **高** | Attention-based communication; target-specific messages; scalable to large agent populations |

---

## 10. World Models & Perception

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **WorldMind** | https://arxiv.org/abs/2608.21439 | - | - | Decoupled World Model, NPC Behavior | **高** | 4 layers: Understanding→Decision→Control→Generation; BOSS-140K dataset; state-aware NPC |
| 2 | **DreamerV3** | https://github.com/danijar/dreamerv3 | 1,500+ | 200+ | Python, World Models, RL | **高** | Learn world model; imagine trajectories; plant representation; cross-domain generalization |
| 3 | **IRIS** | https://github.com/eloialonso/iris | 500+ | 50+ | Python, Discrete World Models | **中** | Tokenized observations; discrete latent dynamics; transformer-based planning |

---

## 11. Stigmergy & Emergent Systems

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Absorbable Patterns |
|---|---------|-----|-------|-------|-----------|-----------|---------------------|
| 1 | **Stigmergy Simulation** | https://github.com/joticajulian/stigmergy | 100+ | 20+ | Python, Pheromone-Based Coordination | **高** | Pheromone trails; indirect communication; swarm intelligence; simple rules → complex behavior |
| 2 | **Old Light RTS** | https://oldlight.io/blog/emergent-gameplay-simple-rules/ | - | - | 5 Rules → Emergence | **高** | Border tiebreak→land rush; energy deficit→overextension; credit drain→economic warfare; simple rules, no special cases |

---

## 12. Specific Projects Detail

| # | Project | URL | Stars | Forks | Core Tech | Relevance | Key Insight | Absorbable Pattern |
|---|---------|-----|-------|-------|-----------|-----------|-------------|---------------------|
| 1 | **OpenSpiel** | https://github.com/google-deepmind/open_spiel | 7,500+ | 2,000+ | C++/Python | **高** | General game framework for RL research | Nash equilibrium solvers; mental models; imperfect information |
| 2 | **PettingZoo** | https://github.com/Farama-Foundation/PettingZoo | 3,498 | 520 | Python | **高** | Standard multi-agent API | AEC API; parallel/sequential stepping; environment wrappers |
| 3 | **SMACv2** | https://github.com/oxwhirl/smacv2 | 1,362 | 241 | Python | **高** | Cooperative StarCraft II MARL | Team reward shaping; observation space design; centralized training |
| 4 | **NeuralMMO** | https://github.com/neuralmmo/jungle | 570 | 72 | Python | **高** | Massively multiagent persistent world | Population training; emergent specialization; resource competition |
| 5 | **Unity ML-Agents** | https://github.com/Unity-Technologies/ml-agents | 19,278 | 4,437 | C#/Python | **高** | Unity RL toolkit | ECS-compatible; curriculum learning; agent communication |
| 6 | **Gymnasium** | https://github.com/Farama-Foundation/Gymnasium | 12,324 | 1,410 | Python | **高** | Standard RL API | Environment specification; vectorized envs; observation spaces |
| 7 | **MARSHAL** | https://github.com/thu-nics/MARSHAL | 52 | - | Python | **高** | LLM-augmented multi-agent self-play | LLM as world model; multi-agent negotiation; hierarchical planning |
| 8 | **hok_env** | https://github.com/tencent-ailab/hok_env | 300+ | 80+ | Python | **高** | Honor of Kings RL environment | MOBA wrapper; ability action space; observation preprocessing |
| 9 | **Bevy** | https://github.com/bevyengine/bevy | 19,000+ | 2,500+ | Rust | **高** | Rust ECS game engine | Parallel systems; WASM; data-driven; community plugins |
| 10 | **Recast Navigation** | https://github.com/recastnavigation/recastnavigation | 6,500+ | 1,500+ | C++ | **高** | Industry navmesh standard | Rasterize→voxelize→filter→polygonal→triangulate pipeline |
| 11 | **BehaviorTree.CPP** | https://github.com/BehaviorTree/BehaviorTree.CPP | 1,800+ | 300+ | C++ | **高** | Industry BT standard | XML/JSON BT; blackboard; async nodes; Groot visualizer |
| 12 | **CleanRL** | https://github.com/vwxyzjn/cleanrl | 9,000+ | 1,500+ | Python | **中** | Single-file RL implementations | PPO/SAC/TD3; readable; easy to adapt |
| 13 | **TorchRL** | https://github.com/pytorch/rl | 2,500+ | 200+ | Python | **高** | Modular PyTorch RL | Composable transforms; priority replay; env wrappers |
| 14 | **DreamerV3** | https://github.com/danijar/dreamerv3 | 1,500+ | 200+ | Python | **高** | World model RL | Learn dynamics; imagine trajectories; cross-domain |
| 15 | **MARLlib** | https://github.com/Replicable-MARL/MARLlib | 1,800+ | 300+ | Python | **高** | Unified MARL library | 30+ algorithms; environment abstraction; CTDE |
| 16 | **constructive** | https://github.com/aat/solver | 200+ | 50+ | Rust | **高** | Rust BSP navmesh | Deterministic; BSP trees; constructive solid geometry |
| 17 | **Clawber.ai** | https://github.com/internexio/clawberbot | 100+ | 20+ | Python | **高** | Autonomous MOBA bot | Wilson CI gating; failed-hypothesis log; KnowledgeForge |

---

## 13. MOBA-Specific Technologies

### 13.1 Map Generation & Layout

| Topic | Source | Key Insight | NeoTrix Mapping |
|-------|--------|-------------|-----------------|
| **NEAT Tower Defense** | raillab.org | PCGNN: NEAT + novelty search for level generation; 10x faster than baselines | NT-MIND SEAL exploration; evolve generators not levels |
| **Lane Design** | Prime World, Theria | 3-lane MOBA with jungle; deterministic tick for reproducible outcomes | Spatial perception system; deterministic simulation for self-play |
| **Terrain Types** | Recast Navigation | Rasterize→voxelize→filter→polygonal→triangulate; walkable area detection | NT-WORLD spatial perception; navmesh components on entities |

### 13.2 Hero Ability Systems

| Topic | Source | Key Insight | NeoTrix Mapping |
|-------|--------|-------------|-----------------|
| **Action Space Design** | hok_env, OpenAI Five | Discrete/continuous hybrid; ability cooldowns; targeting types | ECS ability components; cooldown timers; target selection system |
| **Ability Combos** | MOBA-AI-Gamer | Sequential ability execution; combo detection; timing optimization | GOAP for ability sequencing; utility scoring for combo selection |

### 13.3 Team Coordination

| Topic | Source | Key Insight | NeoTrix Mapping |
|-------|--------|-------------|-----------------|
| **Meta-Commands** | MCC Framework | <Location, Event, TimeLimit> structured communication | GWT broadcast signals; structured intent; salience evaluation |
| **Role Assignment** | OpenAI Five | Dynamic role switching; position-based strategies | SelfModel capability-based assignment; dynamic specialization |
| **Target Selection** | Clawber.ai | Wilson CI gating; statistical promotion/demotion | QualityGate for action evaluation; experience-tree absorption |

### 13.4 Self-Play Training

| Topic | Source | Key Insight | NeoTrix Mapping |
|-------|--------|-------------|-----------------|
| **Population Training** | NeuralMMO | Persistent agents; emergent specialization | NT-MEMORY cross-session persistence; skill crystallization |
| **Fictitious Self-Play** | AlphaGo Zero | Pure self-play; tabula rasa; MCTS inside training loop | NT-MIND evolution cycle; bootstrapping new skill domains |
| **League Training** | AlphaStar | Main agent + population of exploiters | Dual specialization; weapon set switching |

---

## 14. Architecture Patterns Summary

| Pattern | Source | NeoTrix Application | Priority |
|---------|--------|---------------------|----------|
| **ECS Architecture** | Bevy, Unity DOTS | Core simulation structure; entities = agents, components = state, systems = logic | **P0** |
| **Behavior Trees** | BehaviorTree.CPP, Halo 2 | Agent decision-making; hierarchical task decomposition | **P0** |
| **GOAP** | F.E.A.R., goap | Goal-directed action planning; A* search in action space | **P1** |
| **NavMesh Pathfinding** | Recast/Detour | Spatial navigation; crowd simulation; local avoidance | **P0** |
| **World Models** | DreamerV3, WorldMind | Predictive planning; state abstraction; imagination | **P1** |
| **Self-Play** | AlphaGo Zero, OpenAI Five | Skill evolution; population training; league systems | **P0** |
| **Stigmergy** | Old Light RTS | Emergent coordination; simple rules → complex behavior | **P1** |
| **Meta-Commands** | MCC Framework | Structured team communication; hierarchical intent | **P1** |
| **Wilson CI Gating** | Clawber.ai | Statistical skill promotion/demotion; experience-tree integration | **P0** |
| **Deterministic Simulation** | Theria, Prime World | Reproducible outcomes; debugging; self-play training | **P0** |

---

## 15. Technology Stack Recommendations

### Core Simulation
- **Engine**: Bevy (Rust ECS) or custom ECS
- **Pathfinding**: Recast Navigation (C++ FFI) or constructive (Rust native)
- **Behavior Trees**: BehaviorTree.CPP (C++ FFI) or custom Rust BT
- **GOAP**: Custom Rust implementation with A* search

### AI/RL
- **Framework**: PyTorch + TorchRL (Python training) → ONNX export → Rust inference
- **Self-Play**: Custom implementation inspired by AlphaGo Zero / OpenAI Five
- **MARL**: PettingZoo API compatibility + custom algorithms

### Memory & Knowledge
- **KB**: SQLite + BM25 (existing NT-MEMORY)
- **Experience Tree**: Existing NeoTrix experience-tree system
- **World Model**: DreamerV3-style learned dynamics

### Perception
- **Spatial**: NavMesh-based with Detour-style agent simulation
- **Visual**: Optional screen-based perception (YoloV5 + OCR for MOBA)

---

## 16. Key Papers & Articles

| Topic | Source | URL | Key Insight |
|-------|--------|-----|-------------|
| Self-Play Survey | arXiv:2408.01072 | https://arxiv.org/abs/2408.01072 | 4 categories of self-play; ignition mechanism for Q-learning |
| Meta-Command Communication | arXiv:2304.11632 | https://arxiv.org/abs/2304.11632 | MCC framework for human-AI MOBA collaboration |
| WorldMind | arXiv:2608.21439 | https://arxiv.org/abs/2608.21439 | Decoupled world model for state-aware NPC behavior |
| MARSHAL | ICLR 2026 | https://github.com/thu-nics/MARSHAL | LLM-augmented multi-agent self-play |
| DreamerV3 | danijar/dreamerv3 | https://github.com/danijar/dreamerv3 | Cross-domain world model RL |
| PCGNN | raillab.org | https://www.raillab.org/publication/beukman-2022-procedural/ | NEAT + novelty search for level generation |
| CommNet | arXiv:1705.02581 | https://arxiv.org/abs/1705.02581 | Differentiable communication for MARL |
| TarMAC | arXiv:1812.01209 | https://arxiv.org/abs/1812.01209 | Targeted attention-based communication |

---

## 17. Next Steps

1. **P0 - Core ECS**: Implement Bevy-based simulation core with agent entities
2. **P0 - NavMesh**: Integrate Recast Navigation or constructive for spatial pathfinding
3. **P0 - Behavior Trees**: Implement BT runtime with blackboard system
4. **P0 - Self-Play**: Build population-based training loop with Wilson CI gating
5. **P1 - GOAP**: Add goal-oriented action planning for complex agent decisions
6. **P1 - World Model**: Integrate DreamerV3-style predictive planning
7. **P1 - Team Coordination**: Implement meta-command communication system
8. **P2 - Procedural Generation**: Use PCGNN for dynamic map generation

---

## Appendix: Rate Limiting Notes

- Several searches hit Exa MCP rate limits (HTTP 429)
- Successfully completed ~35/45 planned queries
- Rate-limited queries: `emergent behavior`, `procedural map generation`, `TorchRL`
- Recommendation: Retry rate-limited queries with 2-second delays between calls
