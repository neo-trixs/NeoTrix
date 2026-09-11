# RESEARCH_MOBA_V2.md — MOBA AI & microduck_rl Deep Dive for NT-WORLD-SIM

> Generated: 2026-09-11 | Sources: 25 searches/fetches | For: NT-WORLD-SIM MOBA simulation

---

## 1. microduck_rl — Code Structure Analysis

**URL**: https://github.com/pollen-robotics/microduck_rl (2.0k★, Apache 2.0)

### Code Structure

```
src/mjlab_microduck/
├── robot/
│   ├── microduck/                    # MJCF exports, export configs, scenes
│   └── microduck_constants.py        # robot cfgs, HOME frame, BAM actuator cfg
├── actuator/friction_dr_bam.py       # BAM + friction DR + backlash encoder feedback
├── tasks/
│   ├── __init__.py                   # task registration (base + backlash variants)
│   ├── mdp.py                        # rewards, events, observations, custom classes
│   ├── backlash.py                   # make_backlash_variant() env-cfg wrapper
│   └── microduck_*_env_cfg.py        # one cfg module per task family
├── train_cli.py                      # `train` script
├── train_hook.py                     # intercepts `train ... --hf-jobs`
└── hf_jobs.py                        # Hugging Face Jobs submission
```

### Key Insight
- **Shared 61-dim observation contract** across ALL policies enables runtime hot-swapping — any policy can take over the robot at any moment
- 14 servo joints (0-4 left leg, 5-8 neck/head, 9-13 right leg)
- ONNX export bakes observation normalizer into graph — runtime sees normalized obs
- 4096 parallel envs, ~1-2h training for usable gait on single GPU

### NeoTrix Mapping
| microduck_rl Pattern | NT-WORLD-SIM Equivalent |
|---|---|
| Shared obs contract (61-dim) | **NT-SIM Agent Observation Protocol** — standardized observation space for all MOBA agents |
| BAM actuator model (voltage control, back-EMF, friction) | **NT-SIM Agent Physics Model** — ability-specific action dynamics with cooldowns, cast times, resource costs |
| Domain randomization (per-env DR) | **NT-SIM Match Variability** — agent skill variance, meta shifts, ping simulation |
| Backlash variants (passive joints) | **NT-SIM Latency Model** — input delay, packet loss as "backlash" in agent response |
| Runtime policy hot-swapping | **NT-SIM Strategy Switching** — agents dynamically switch between lane/teamfight/objective strategies |
| Task registration system | **NT-SIM Agent Registry** — composable agent capabilities registered at startup |

### Code Pattern: Task Registration
```python
# microduck_rl pattern: each task registered as env-cfg module
# NT-WORLD-SIM mapping: each MOBA behavior as composable module
TASK_REGISTRY = {
    "velocity_flat": MicroduckVelocityFlatEnvCfg,
    "velocity_rough": MicroduckVelocityRoughEnvCfg,
    "standup": MicroduckStandUpEnvCfg,
    ...
}
```

### Code Pattern: Domain Randomization Toggles
```python
# microduck_rl pattern: ENABLE_* booleans at top of env cfg
ENABLE_FRICTION_DR = True
ENABLE_VOLTAGE_DR = True
ENABLE_DELAY_DR = True
# NT-WORLD-SIM: ENABLE_LANE_DR, ENABLE_DRAFT_DR, ENABLE_PING_DR
```

---

## 2. microduck_rl Training Environment

**URL**: https://github.com/pollen-robotics/microduck_rl

### Key Insight
- Built on **mjlab** (MuJoCo Warp) with **PPO** (rsl_rl)
- Policies trained at **50 Hz** — same frequency as real robot
- Domain randomization covers: battery voltage, voltage sag, command delay, friction magnitude, terrain
- Terrain variants: Flat/Rough for each task
- 13 registered tasks (Velocity, VelStand, StandUp, SitStand, GroundPick, BallKick, Roulade, Roller variants, Spin)

### NeoTrix Mapping
- **NT-SIM Training Loop**: Use PPO with similar DR approach for MOBA agent training
- **Multi-task training**: Each MOBA behavior (laning, teamfight, objective control) as separate task with shared backbone
- **Hz matching**: Train at same tick rate as deployment (e.g., 10 Hz game tick)

---

## 3. microduck_rl BAM Actuator Model

**URL**: https://github.com/Rhoban/bam

### Key Insight
- BAM M6 model for Dynamixel XL330 servo
- Models: **voltage control law, back-EMF, Coulomb/Stribeck/load-dependent friction**
- `FrictionDRBamActuator` — per-env domain randomization on all actuator parameters
- At tiny scale (~800g biped), **actuator fidelity IS the sim2real gap**
- `BacklashEncoderBamActuator` — reads through backlash like real hardware

### NeoTrix Mapping
| BAM Concept | NT-WORLD-SIM Equivalent |
|---|---|
| Voltage control law | **Ability resource cost model** — mana/energy cost per ability with regen |
| Back-EMF (resistive force) | **Cooldown dynamics** — abilities create temporary vulnerability windows |
| Coulomb/Stribeck friction | **Cast time variability** — animation canceling, input buffering |
| Load-dependent friction | **Weight-dependent mechanics** — items affecting movement/attack speed |
| Per-env DR on voltage | **Per-agent skill variance** — mechanical skill, decision quality |

---

## 4. microduck_rl Sim2Real Transfer

**URL**: https://github.com/pollen-robotics/microduck_rl/blob/develop/AGENTS.md

### Key Insight
- **AGENTS.md is the distilled playbook** — environment-building workflow + reward-design lessons
- Sim2real gap closed by: BAM physics + DR + backlash modeling + reward shaping
- Reward design is the hardest part — actuator fidelity alone isn't enough
- ONNX export via single safe path (`scripts/export.py`) — never hand-convert
- `infer_policy.py` rehearses runtime hot-swapping of multiple policies

### NeoTrix Mapping
- **NT-SIM Reward Engineering**: Complex, multi-objective reward for MOBA behaviors (CS + positioning + objective + kills)
- **Single export path**: Agents must go through validated export pipeline before deployment
- **Policy composition**: Multiple specialized policies (lane, teamfight, objective) composed at runtime

---

## 5. OpenAI Five Architecture

**URL**: https://cdn.openai.com/research-covers/openai-five/network-architecture.pdf | https://arxiv.org/abs/1912.06680

### Key Insight
- **5 LSTMs** — one per hero, each controlling a DotA 2 character
- **Max-pooling** over units (allied/enemy heroes, non-heroes, neutrals)
- Input encoding: hero stats, nearby terrain (8x8 grid), abilities (embedding+FC), items (embedding+FC), modifiers (embedding+FC)
- **Action space**: discrete (ability selection) + continuous (target position X/Y, teleport destination, delay)
- **Unit attention**: dot-product attention over all units for targeting
- Discount factor γ ≈ 0.999841 (1 - 1/6300) — heavily weights future rewards
- **Backprop through only 16 timesteps** (2.1 seconds of game time) despite hour-long games

### Architecture Diagram (from paper)
```
Hero Obs → LSTM(1024) → Action Heads:
  ├── Available Actions → Softmax → Selected Action
  ├── Offset X/Y → Softmax → Target Position
  ├── Move X/Y → Softmax → Move Direction
  ├── Teleport Destination
  ├── Delay
  └── Unit Attention → Target Unit
```

### NeoTrix Mapping
| OpenAI Five | NT-WORLD-SIM |
|---|---|
| Per-hero LSTM | **Per-agent policy network** — separate LSTM/Transformer per agent role |
| Max-pool over units | **Spatial attention pool** — aggregate nearby agent info |
| Terrain 8x8 grid | **Lane/map state encoding** — minimap grid with vision |
| Ability/Item/Modifier embeddings | **Ability/item state vectors** — game state encoding |
| Unit attention (dot product) | **Target selection attention** — choose who to attack/ability target |
| γ ≈ 0.999841 | **Long-horizon discount** — objectives (towers, nexus) heavily weighted |
| 16-step backprop | **BPTT window** — balance credit assignment vs compute |

### Code Pattern: Hierarchical LSTM
```rust
// NT-WORLD-SIM equivalent of OpenAI Five's architecture
pub struct AgentPolicy {
    // Per-agent observation encoder
    obs_encoder: Linear(obs_dim, hidden),  // hero stats + abilities
    // Spatial attention over nearby entities
    spatial_pool: AttentionPool(hidden),
    // LSTM for temporal memory
    lstm: LSTM(hidden, hidden),
    // Action heads
    action_head: Linear(hidden, num_actions),
    position_head: Linear(hidden, 2),  // x, y target
    target_head: Linear(hidden, num_units),  // who to target
}
```

---

## 6. OpenAI Five Reward Shaping

**URL**: https://arxiv.org/abs/1912.06680

### Key Insight
- **80% of reward is kill-related**: kill enemy hero (+X), deny allied hero, hero survival
- **10% structure**: ability accuracy, laning (near creeps but not too close), movement efficiency
- **10% game outcome**: win/loss, tower advantage
- **Surgery** technique: when game patches change mechanics, interpolate between old and new reward functions
- Training: 128,000 cores, 256 GPUs, ~10 months, ~45,000 years of game time

### NeoTrix Mapping
- **Multi-objective reward decomposition**:
  - 60% team objectives (towers, inhibitors, nexus)
  - 25% combat performance (kills, deaths, assists)
  - 10% economic efficiency (CS, gold/xp differential)
  - 5% positioning (map control, vision)
- **Reward surgery**: When meta changes, smoothly interpolate reward functions
- **Distributed training**: 128K cores → scale to multiple GPUs for NT-SIM

---

## 7. OpenAI Five Self-Play

**URL**: https://arxiv.org/abs/1912.06680

### Key Insight
- **Continuous self-play** against current and past versions of the model
- **Pool of opponents**: randomly sample from historical checkpoints
- Prevents "strategy cycling" — model must be robust to all historical strategies
- **Win-rate based matchmaking**: opponents selected based on similar skill level
- 10 months of continuous self-play training

### NeoTrix Mapping
| Self-Play Pattern | NT-WORLD-SIM |
|---|---|
| Historical checkpoint pool | **Agent archive** — store checkpoints, sample opponents |
| Win-rate matchmaking | **ELO-based matching** — pair similar-skilled agents |
| Anti-cycling | **Diverse opponent pool** — prevent meta collapse |
| Continuous training | **Online learning loop** — agents improve continuously |

### Code Pattern: Self-Play Pool
```rust
pub struct SelfPlayPool {
    agents: Vec<AgentCheckpoint>,  // historical checkpoints
    current: AgentCheckpoint,
    elo_ratings: Vec<f32>,
}

impl SelfPlayPool {
    pub fn sample_opponent(&self, skill_range: f32) -> &AgentCheckpoint {
        // Sample from historical checkpoints within skill range
        // Prevents strategy cycling
    }
}
```

---

## 8. OpenAI Five Hierarchical LSTM

**URL**: https://arxiv.org/pdf/1912.06721

### Key Insight
- **LSTM memory acts as implicit plan** — contains info about future goals
- Hidden State Decoders trained to predict: future gold, net worth rank, tower destruction timing
- **No explicit macro-actions** — hierarchy emerges from learned representations
- Plans detected via similarity analysis of distributed representations
- Sub-goals (reach map location, destroy tower) learned without explicit supervision

### NeoTrix Mapping
| Hierarchical LSTM | NT-WORLD-SIM |
|---|---|
| Implicit planning in LSTM | **Temporal attention** — agent memory encodes current strategic plan |
| Hidden State Decoders | **Plan introspection** — extract current objective from agent hidden state |
| Sub-goal detection | **Behavior classification** — detect laning/teamfighting/objective-taking from hidden state |
| No macro-actions needed | **End-to-end learning** — don't hardcode strategy phases |

### Code Pattern: Plan Introspection
```rust
// Extract what the agent is currently planning from hidden state
pub struct PlanDecoder {
    // Predicts: "will this agent attack dragon in next 30s?"
    objective_predictor: Linear(hidden, num_objectives),
    // Predicts: "how much gold will this agent have in 60s?"
    resource_predictor: Linear(hidden, 1),
}

// Used for:
// 1. Commentary/spectating
// 2. Opponent modeling
// 3. Strategy-aware reward shaping
```

---

## 9. PyMARL QMIX Algorithm

**URL**: https://github.com/oxwhirl/pymarl (2.2k★) | https://arxiv.org/abs/1803.11485

### Key Insight
- **QMIX**: Centralized training, decentralized execution (CTDE)
- Learns joint Q-function as **non-linear monotonic mixing** of per-agent utilities
- `∂Q_tot/∂Q_a ≥ 0` — monotonic constraint enables tractable decentralization
- **Hypernetwork** conditions mixing weights on global state → state-dependent coordination
- Significantly outperforms VDN and IQL on SMAC benchmarks
- Per-agent Q-networks condition only on **local observations** — enables decentralized execution

### Architecture
```
Global State → HyperNet → Mixing Weights (per layer)
Agent_i obs → Agent_i Q-net → Q_i(s, u_i)
Q_tot = MixNet(Q_1, Q_2, ..., Q_n; state)
```

### NeoTrix Mapping
| QMIX Component | NT-WORLD-SIM |
|---|---|
| Per-agent Q-network | **Per-agent value function** — estimates value of each action given local observation |
| Hypernetwork (state-conditioned mixing) | **Team coordination network** — global state informs how individual actions compose |
| Monotonic constraint | **Non-negative coordination** — team value increases when any agent improves |
| CTDE paradigm | **Training with full map info, executing with local vision** — matches MOBA fog-of-war |

### Code Pattern: QMIX-style Value Decomposition
```rust
pub struct TeamQMIX {
    agent_q_nets: Vec<AgentQNet>,  // one per agent
    hyper_net: HyperNetwork,  // generates mixing weights from global state
}

impl TeamQMIX {
    pub fn q_tot(&self, agent_actions: &[QValues], global_state: &State) -> QValue {
        let weights = self.hyper_net.forward(global_state);
        // Monotonic mixing: Q_tot = Σ w_i * Q_i (simplified)
        monotonic_mix(agent_actions, weights)
    }
}
```

---

## 10. PyMARL VDN Value Decomposition

**URL**: https://arxiv.org/abs/1706.05296

### Key Insight
- **VDN**: Simplest value decomposition — `Q_tot = Σ Q_i(u_i, obs_i)`
- **No state conditioning** — each agent's Q-function depends only on local observations
- **Fully disconnected coordination graph** — no explicit coordination during training
- Limitation: cannot represent value functions where agent's optimal action depends on other agents' actions
- Serves as surprisingly strong baseline even in competitive settings

### NeoTrix Mapping
| VDN | NT-WORLD-SIM |
|---|---|
| Sum decomposition | **Independent agent training** — each agent optimizes locally |
| No state info | **Fog-of-war training** — agents only see their local area |
| Strong baseline | **Start with VDN, upgrade to QMIX** — progressive complexity |

---

## 11. SMAC StarCraft Multi-Agent Challenge

**URL**: https://github.com/oxwhirl/smac

### Key Insight
- **Standard benchmark** for cooperative MARL
- Scenarios: 3m (3 marines) → 1c3s5z (colossus + stalkers + zealots)
- **Perfect and imperfect information** variants
- Maps test different coordination challenges: focus fire, kiting, flanking
- Win rate reported as primary metric
- Used by QMIX, VDN, COMA, IQL, QTRAN papers

### NeoTrix Mapping
| SMAC Scenario | NT-WORLD-SIM Equivalent |
|---|---|
| 3m (simple combat) | **2v2 skirmish** — basic teamfight coordination |
| 2s3z (mixed units) | **5v5 teamfight** — mixed roles (tank/dps/support) |
| 1c3s5z (complex) | **Full teamfight with objectives** — dragon/baron fights |
| Focus fire scenarios | **Target selection** — who to focus in teamfight |
| Kiting scenarios | **Spacing/positioning** — kiting, zoning |

---

## 12. OpenSpiel Game Abstraction

**URL**: https://github.com/google-deepmind/open_spiel (5.5k★)

### Key Insight
- **C++ core** with Python bindings — procedural extensive-form games
- Supports: n-player, zero-sum, cooperative, general-sum, one-shot, sequential, simultaneous-move, perfect/imperfect information
- **Game as first-class object** — standardized API for all game types
- Algorithms: CFR, MCTS, DQN, Policy Gradient, Deep CFR
- Analysis tools: exploitability, NashConv, visit distributions

### NeoTrix Mapping
| OpenSpiel Concept | NT-WORLD-SIM |
|---|---|
| Game abstraction | **MOBA match abstraction** — formalize match as extensive-form game |
| Player API | **Agent API** — standardized interface for all agents |
| Information set | **Fog-of-war information set** — what each agent can observe |
| Chance nodes | **RNG elements** — critical strikes, skill shots |
| Simultaneous moves | **Real-time actions** — all agents act simultaneously |
| Analysis tools | **Match analysis** — exploitability, Nash equilibrium approximation |

### Code Pattern: Game Abstraction
```rust
// NT-WORLD-SIM: Abstract MOBA match as OpenSpiel-compatible game
pub struct MobaGame {
    teams: [Team; 2],
    map: MapState,
    time: GameTime,
}

impl Game for MobaGame {
    type State = MatchState;
    type Action = AgentAction;
    
    fn legal_actions(&self, player: PlayerId) -> Vec<Action> {
        // Returns actions legal given fog-of-war, cooldowns, mana
    }
    
    fn apply_action(&mut self, action: Action) -> Outcome {
        // Simulate one game tick with all agents' simultaneous actions
    }
}
```

---

## 13. TorchRL / TensorDict

**URL**: https://github.com/pytorch/rl

### Key Insight
- **TensorDict**: Dict-like container for tensors with shared batch dimensions
- **TorchRL**: Modular RL library built on TensorDict
- Supports: PPO, SAC, TD3, DQN, A2C, REINFORCE
- **Environment wrappers**: Gym-compatible, transforms, parallel execution
- **Data collection**: replay buffers, on-policy collectors
- **Modular design**: environment, collector, loss module, actor-critic are independent

### NeoTrix Mapping
| TorchRL Concept | NT-WORLD-SIM |
|---|---|
| TensorDict | **Agent state dict** — standardized container for agent observations/actions/values |
| Environment wrapper | **Match environment** — wraps MOBA match as RL environment |
| Parallel execution | **Batch match simulation** — run multiple matches in parallel |
| Replay buffer | **Experience replay** — store and sample past match experiences |
| Modular loss | **Reward decomposition** — separate loss terms for different objectives |

---

## 14. MOBA Lane Phase AI

### Key Insight
- **Laning phase**: 0-15 minutes, heroes stay in assigned lanes (top/mid/bot)
- **CS (creep score)**: Last-hit minions for gold — core economic mechanic
- **Trading**: Short trades with opponent, managing health/mana resources
- **Wave management**: Freeze (keep wave near tower), slow push (build big wave), fast push (shove quickly)
- **Recall timing**: Return to base when low HP/mana or after kill

### Key Behaviors to Model
1. **Last-hitting**: Timing attacks to kill minions at low HP
2. **Trading stance**: Positioning to punish opponent's CS attempts
3. **Wave manipulation**: Freezing, slow pushing, fast pushing
4. **Vision control**: Placing/de-clearing wards
5. **Recall timing**: When to base for items/health

### NeoTrix Mapping
```
LanePhaseAI {
    state: LaneState {
        ally_minions: Vec<Minion>,
        enemy_minions: Vec<Minion>,
        enemy_hero: HeroState,
        my_hp: f32,
        my_mana: f32,
        wave_position: f32,  // distance from own tower
    }
    actions: LaneAction {
        last_hit: Option<MinionId>,
        trade: Option<HeroId>,
        position: Vec2,
        recall: bool,
        ward: Option<Vec2>,
    }
}
```

---

## 15. MOBA Teamfight Positioning

### Key Insight
- **Frontline** (tanks/bruisers): Absorb damage, create space, engage/disengage
- **Backline** (ADC/mage): Deal damage from safe distance, protect carries
- **Support**: Vision, peel, engage, sustain
- **Positioning principles**:
  - Don't stack (AoE vulnerability)
  - Stay near cover (walls, brushes)
  - Protect carries at all costs
  - Target priority: low-HP > high-threat > nearest

### Key Metrics
- **Effective HP**: Total damage before death (HP + armor + MR + shields)
- **DPS uptime**: Percentage of teamfight spent dealing damage
- **Positioning error**: Distance from optimal position

### NeoTrix Mapping
```
TeamfightPositioning {
    roles: {
        frontline: PositioningRule { range: 200-400, target: "nearest_threat" },
        backline: PositioningRule { range: 500-700, target: "highest_dps" },
        support: PositioningRule { range: 300-500, target: "ally_lowest_hp" },
    }
    formation: {
        ideal_spacing: 300,  // minimum distance between allies
        max_cluster: 3,      // max allies in AoE radius
        escape_vectors: Vec<Vec2>,  // pre-computed escape routes
    }
}
```

---

## 16. MOBA Objective Control AI

### Key Insight
- **Objectives**: Dragon (stacking buff), Baron (team siege buff), Rift Herald (tower push), Towers (map control), Inhibitors (super minions), Nexus (win condition)
- **Objective value changes** over game time:
  - Early: Dragon stacks, first tower bonus
  - Mid: Baron, tower control
  - Late: Baron, Elder Dragon, Nexus
- **Zoning**: Control area around objective before starting
- **Smite fight**: 50/50 steal attempts — high risk, high reward
- **Cross-map trades**: Give up dragon for tower, give up tower for Baron

### Key Decision Framework
```
Should we take this objective?
├── Can we take it without contest? → Take it
├── Can we win the teamfight? → Force fight, then take
├── Can we trade for something better? → Trade
├── Is it too risky? → Give it up, farm
```

### NeoTrix Mapping
| Objective Concept | NT-WORLD-SIM |
|---|---|
| Objective value scaling | **Dynamic objective valuation** — value changes with game state |
| Zoning | **Area control** — claim territory before objective |
| Smite timing | **Burst timing** — coordinate burst damage for secure |
| Cross-map trade | **Strategic tradeoff** — model as multi-objective optimization |

---

## 17. MOBA Vision Control AI

### Key Insight
- **Fog of War**: Limited vision creates information asymmetry
- **Vision types**: Minions (lane vision), towers (area vision), wards (placed vision), champion abilities (temporary vision)
- **Vision denial**: Clearing enemy wards (Oracle Lens, Control Wards)
- **Deep wards**: Vision in enemy jungle → track jungler movement
- **Vision评分**: How much of the map you control vs enemy

### Key Behaviors
1. **Ward placement**: Strategic locations (bushes, jungle entrances, objectives)
2. **Ward clearing**: Oracle Lens sweeps, Control Ward denial
3. **Vision tracking**: Predict enemy positions from visible info
4. **Fog abuse**: Use fog to ambush, zone, or escape

### NeoTrix Mapping
```
VisionControlAI {
    ward_budget: 3,  // max wards per player
    priority_locations: {
        objective: [dragon_pit, baron_pit],
        jungle_entrances: [blue_buff_entrance, red_buff_entrance],
        lane_bushes: [top_bush, mid_bush, bot_bush],
    }
    vision_value: Map<Vector2, f32>,  // vision importance per tile
    enemy_ward_tracker: WardTracker,  // track enemy ward placements
}
```

---

## 18. MOBA Ward Placement AI

### Key Insight
- **Ward types**: Stealth Ward (invisible, 90s), Control Ward (visible, reveals stealth), Farsight Alteration (long range, fragile)
- **Optimal ward locations** depend on game state:
  - Laning: River bushes, tri-bush
  - Mid-game: Jungle entrances, objective areas
  - Late-game: Baron/Dragon, base gates
- **Ward efficiency**: Maximum vision coverage with minimum wards
- **Predictive placement**: Ward where enemy WILL go, not where they are

### NeoTrix Mapping
```
WardPlacementPolicy {
    game_phase: GamePhase,
    objective_upcoming: Option<Objective>,
    enemy_jungler_last_seen: Option<(Position, Time)>,
    
    fn place_ward(&self, state: &GameState) -> WardPlacement {
        // Score candidate locations based on:
        // 1. Vision coverage (new tiles revealed)
        // 2. Enemy path prediction
        // 3. Objective proximity
        // 4. Safety of placement (not in enemy vision)
    }
}
```

---

## 19. MOBA Gank Detection AI

### Key Insight
- **Gank**: Jungler + laner coordinate to kill an enemy laner
- **Detection signals**:
  - Enemy laner behavior change (aggressive positioning)
  - Missing minimap info (enemy mid laner disappeared)
  - Jungle camp clear patterns
  - Ward vision gaps
- **Response**: Retreat to tower, place defensive ward, call for counter-gank

### Key Detection Heuristics
1. **Aggressive signal**: Enemy laner suddenly positions aggressively → likely gank incoming
2. **Missing signal**: Enemy last seen on minimap X seconds ago → could be roaming
3. **Path prediction**: Enemy jungler started bot → will path to top by 3:30

### NeoTrix Mapping
```
GankDetectionAI {
    threat_map: ThreatMap,  // per-tile threat level
    
    fn update_threat(&mut self, state: &GameState) {
        // Decrease threat in visible areas
        // Increase threat in fog near enemy positions
        // Factor in enemy jungler clear speed and pathing
    }
    
    fn should_retreat(&self, lane_state: &LaneState) -> bool {
        // Retreat if threat exceeds threshold
        self.threat_map.get(lane_state.position) > RETREAT_THRESHOLD
    }
}
```

---

## 20. MOBA Dragon/Baron AI

### Key Insight
- **Dragon Soul**: 4 dragons → permanent team buff (elemental types)
- **Elder Dragon**: Ultra-powerful buff (execute low-HP enemies)
- **Baron Nashor**: Buff for pushing lanes (empowered minions)
- **Timing windows**:
  - Dragon spawns 5:00, respawns 5:00 after kill
  - Baron spawns 20:00, respawns 7:00 after kill
- **Team decision**: When to force, when to give, when to contest

### NeoTrix Mapping
| Dragon/Baron | NT-WORLD-SIM |
|---|---|
| Spawn timing | **Timer-based events** — predict spawn windows |
| Stacking value | **Diminishing/growing returns** — 4th dragon worth more than 1st |
| Baron buff | **Siege state** — team gets empowered push capability |
| Elder Dragon | **Win condition** — execute threshold changes teamfight math |

---

## 21. MOBA Tower Dive AI

### Key Insight
- **Tower dive**: Attack enemy under their tower (tower deals heavy damage)
- **Aggro mechanics**: Tower targets first enemy to damage allied hero
- **Reset**: Tower aggro resets when no enemy hero in range
- **Juggling**: Multiple allies take tower aggro turns
- **Prerequisites**: Minion wave under tower (towers prioritize minions first)

### Key Decision Factors
1. Enemy HP below execute threshold
2. Minion wave available to tank initial tower shots
3. Ally available to juggle aggro
4. Enemy cooldowns blown (can't CC under tower)

### NeoTrix Mapping
```
TowerDiveDecision {
    enemy_hp: f32,
    my_burst: f32,  // max damage in one rotation
    minion_wave: bool,
    ally_available: bool,
    enemy_cc_available: bool,
    
    fn should_dive(&self) -> DiveDecision {
        if self.enemy_hp < self.my_burst && self.minion_wave {
            DiveDecision::Dive { juggle_order: self.plan_juggle() }
        } else {
            DiveDecision::PokeAndRetreat
        }
    }
}
```

---

## 22. MOBA Split Push AI

### Key Insight
- **Split push**: One hero pushes a lane alone while team applies pressure elsewhere
- **Types**: 1-3-1 (three groups), 1-4 (one solo, four grouped), 4-1 (four pushing, one split)
- **Requirements**: Strong 1v1 champion, teleport for joins, map awareness
- **Risk**: Getting collapsed on by multiple enemies
- **Value**: Forces enemy response → creates numbers advantage elsewhere

### NeoTrix Mapping
```
SplitPushAI {
    champion_strength: SplitPushScore,  // 1v1, waveclear, escape
    team_state: TeamState,
    map_pressure: MapPressureMap,  // which lanes have pressure
    
    fn should_split_push(&self, state: &GameState) -> bool {
        // Split push if:
        // 1. Can win 1v1 against likely defender
        // 2. Team can disengage if 4v5
        // 3. No major objective spawning soon
        // 4. Teleport available for joins
    }
}
```

---

## 23. MOBA Team Composition AI

### Key Insight
- **Team composition** determines strategy:
  - **Early game comp**: Win lanes, snowball (e.g., Lee Sin, Renekton)
  - **Late game comp**: Scale, teamfight (e.g., Jax, Kog'Maw)
  - **Split push comp**: 1-3-1 pressure (e.g., Fiora, Twisted Fate)
  - **Wombo combo**: AoE synergy (e.g., Malphite + Yasuo)
- **Role distribution**: 1 top, 1 mid, 1 jungle, 1 ADC, 1 support
- **Synergy scoring**: How well champions work together

### NeoTrix Mapping
```
CompositionScorer {
    fn score_composition(&self, team: &[Champion]) -> CompositionScore {
        CompositionScore {
            early_power: self.early_game_strength(team),
            late_power: self.late_game_strength(team),
            teamfight: self.teamfight_synergy(team),
            split_push: self.split_push_potential(team),
            engage: self.engage_tools(team),
            peel: self.peel_tools(team),
        }
    }
}
```

---

## 24. MOBA Counter Pick AI

### Key Insight
- **Counter pick**: Select champion that advantages against opponent's pick
- **Counter logic**:
  - Range vs melee advantage
  - CC vs mobile champion
  - Sustained damage vs burst
  - Magic damage vs armor stack
- **Counter database**: Historical win rates of matchups
- **Team-aware countering**: Counter pick considering team composition

### NeoTrix Mapping
```
CounterPickAI {
    matchup_database: MatchupWinrates,  // champion A vs champion B winrates
    
    fn suggest_counter(&self, enemy_pick: Champion, team_comp: &[Champion]) -> Vec<Champion> {
        // Score each champion by:
        // 1. Head-to-head winrate vs enemy_pick
        // 2. Synergy with team_comp
        // 3. Meta strength
        // 4. Player proficiency
    }
}
```

---

## 25. MOBA Draft Phase AI

### Key Insight
- **Ban phase**: Each team bans champions they don't want to face
- **Pick phase**: Alternating picks (1-2-2-2-2-1 format in LoL)
- **First pick advantage**: Strong meta champion available
- **Last pick advantage**: Counter pick opportunity
- **Draft strategy**:
  - **Blind pick**: Pick safe, flexible champions early
  - **Counter pick**: Save last pick for counter
  - **Flex pick**: Champions playable in multiple roles
  - **Priority**: Which role gets the "strong" pick

### Key Draft Concepts
1. **Ban priority**: Remove strongest meta champions
2. **First rotation**: Pick flexible, high-priority champions
3. **Second rotation**: Fill remaining roles, consider counters
4. **Last pick**: Counter pick or flex pick

### NeoTrix Mapping
```
DraftAI {
    meta_tier: MetaTierList,  // champion strength rankings
    matchup_database: MatchupDatabase,
    
    fn draft_move(&self, state: &DraftState) -> DraftAction {
        match state.phase {
            BanPhase => self.suggest_ban(state),
            PickPhase => self.suggest_pick(state),
        }
    }
    
    fn suggest_ban(&self, state: &DraftState) -> Champion {
        // Ban highest tier champion not yet banned
        // Consider: what enemy team likely wants
    }
    
    fn suggest_pick(&self, state: &DraftState) -> Champion {
        if state.is_first_pick {
            // Pick highest tier available
        } else {
            // Counter pick against enemy's composition
        }
    }
}
```

---

## Cross-Cutting Patterns: microduck_rl → NT-WORLD-SIM

### Pattern 1: Shared Observation Contract
**microduck**: 61-dim shared across all policies → hot-swappable at runtime
**NT-WORLD-SIM**: Standardized observation space for all MOBA agents → strategy switching mid-game

### Pattern 2: Actuator Fidelity = Sim2Real Gap
**microduck**: BAM voltage control model IS the sim2real gap
**NT-WORLD-SIM**: Accurate ability model (cooldowns, cast times, resource costs) IS the behavior gap

### Pattern 3: Domain Randomization for Robustness
**microduck**: Per-env DR on voltage, friction, delay
**NT-WORLD-SIM**: Per-match DR on agent skill, meta shifts, ping

### Pattern 4: Multi-Task Training
**microduck**: 13 tasks sharing observation contract
**NT-WORLD-SIM**: Lane/teamfight/objective sub-policies sharing backbone

### Pattern 5: Policy Composition
**microduck**: Hot-swap walk/recover/trick policies
**NT-WORLD-SIM**: Dynamic strategy selection (laning → teamfighting → siege)

---

## Cross-Cutting Patterns: OpenAI Five → NT-WORLD-SIM

### Pattern 1: Per-Agent LSTM with Shared Architecture
**OpenAI Five**: 5 LSTMs, one per hero, same architecture
**NT-WORLD-SIM**: Per-agent policy networks, role-specific heads

### Pattern 2: Attention over Entities
**OpenAI Five**: Max-pool + dot-product attention over all units
**NT-WORLD-SIM**: Spatial attention over nearby agents, minions, structures

### Pattern 3: Long-Horizon Discount
**OpenAI Five**: γ ≈ 0.999841, backprop 16 steps
**NT-WORLD-SIM**: High γ for objective rewards, limited BPTT window

### Pattern 4: Self-Play with Historical Pool
**OpenAI Five**: Continuous self-play against current + past checkpoints
**NT-WORLD-SIM**: Agent archive for training diversity

---

## Cross-Cutting Patterns: QMIX/PyMARL → NT-WORLD-SIM

### Pattern 1: CTDE (Centralized Training, Decentralized Execution)
**QMIX**: Train with global state, execute with local observations only
**NT-WORLD-SIM**: Train with full map info (cheat), deploy with fog-of-war

### Pattern 2: Value Decomposition
**QMIX**: Q_tot = monotonic_mix(Q_1, ..., Q_n)
**NT-WORLD-SIM**: Team value = composition of individual agent values

### Pattern 3: Hypernetwork for Coordination
**QMIX**: Hypernetwork conditions mixing weights on global state
**NT-WORLD-SIM**: Team coordination network takes global state → routes attention to relevant agents

---

## Priority Implementation Roadmap for NT-WORLD-SIM

### Phase 1: Foundation (Week 1-2)
1. **Shared observation protocol** (from microduck_rl shared obs contract)
2. **Agent physics model** (from BAM actuator model — abilities, cooldowns, resources)
3. **Match environment** (from OpenSpiel game abstraction)

### Phase 2: Single-Agent (Week 3-4)
4. **Lane phase AI** (last-hit, trading, wave management)
5. **Teamfight positioning** (role-based positioning rules)
6. **Vision control** (ward placement, gank detection)

### Phase 3: Multi-Agent Coordination (Week 5-6)
7. **QMIX-style value decomposition** (CTDE paradigm)
8. **Objective control** (dragon/baron decision framework)
9. **Self-play training loop** (from OpenAI Five pattern)

### Phase 4: Meta-AI (Week 7-8)
10. **Draft phase AI** (ban/pick strategy)
11. **Counter pick system** (matchup database)
12. **Team composition scoring** (synergy evaluation)

---

## Key Papers & References

| Paper | Key Contribution | NT-WORLD-SIM Relevance |
|---|---|---|
| OpenAI Five (1912.06680) | Large-scale self-play, PPO, hierarchical LSTM | Agent architecture, training infrastructure |
| OpenAI Five Planning (1912.06721) | LSTM as implicit planner, hidden state decoders | Plan introspection, opponent modeling |
| QMIX (1803.11485) | Monotonic value decomposition, CTDE | Multi-agent coordination |
| VDN (1706.05296) | Independent value decomposition | Baseline multi-agent |
| SMAC (1902.04043) | Cooperative MARL benchmark | Training scenarios |
| OpenSpiel (1908.09453) | Game abstraction framework | Match formalization |
| microduck_rl | BAM actuator, shared obs, sim2real | Agent physics, observation protocol |
| PyMARL | QMIX/VDN/COMA implementations | Algorithm reference |
