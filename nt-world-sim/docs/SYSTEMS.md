# NT-WORLD-SIM Systems Reference

Complete reference for all game systems, their data structures, key methods, and integration points.

---

## Game Loop (`game/game_loop.rs`)

**Purpose**: Central orchestrator managing game state, system execution, and player interaction.

**Data Structures**:
```rust
pub enum GameState { Title, Playing, Paused, Dialogue, Inventory, Crafting, Mining, GameOver }

pub struct GameLoop {
    pub state: GameState,
    pub world: UniversalWorld,
    pub scheduler: ParallelScheduler,
    pub player_entity: Option<UniversalEntity>,
    pub tick_count: u64,
    pub physics: SimplePhysicsWorld,
    pub camera: Camera2D,
    pub scene: SceneGraph,
    pub event_bus: TypedEventBus,
    pub audio: AudioManager,
    pub particles: ParticleSystem,
    pub relationships: RelationshipSystem,
    pub perfection: PerfectionTracker,
    pub cooking: CookingSystem,
    pub fishing: FishingSystem,
    pub foraging: ForagingSystem,
    pub skills: SkillSystem,
    pub mine_state: MineState,
    pub economy: Economy,
    pub quest_db: QuestDatabase,
    pub calendar: EventCalendar,
}

pub enum GameAction {
    Move { dx: i32, dy: i32 },
    UseTool(u32),
    Interact,
    ToggleInventory,
    ToggleCrafting,
    AdvanceDialogue,
    Sleep,
    Save,
    Load,
}
```

**Key Methods**:
- `GameLoop::new()` — Initializes all subsystems
- `GameLoop::start_game()` — Transitions to Playing, spawns player, initializes world
- `GameLoop::update(dt)` — Fixed-timestep update loop
- `GameLoop::handle_input(action)` — Processes player actions with state-dependent routing
- `GameLoop::get_state_summary()` — Returns formatted state string

**Integration**: Owns all subsystems; routes input to appropriate handlers based on `GameState`.

---

## Time System (`game/time.rs`)

**Purpose**: Day/night cycle, season progression, and time tracking.

**Data Structures**:
```rust
pub enum Season { Clarity, Flow, Reflection, Stillness }

pub struct GameTime {
    pub day: u32,
    pub season: Season,
    pub year: u32,
    pub hour: u32,
    pub minute: u32,
    pub time_speed: f32,
}
```

**Key Methods**:
- `GameTime::advance(dt)` — Advances time by delta
- `GameTime::season_name()` — Returns season as string
- `GameTime::is_day()` — True if hour 6-20
- `GameTime::day_of_season()` — Day number within current season (1-28)

**Integration**: Drives weather changes, crop growth, NPC schedules, fish/forage spawning.

---

## Weather System (`game/weather.rs`)

**Purpose**: Dynamic weather with seasonal effects.

**Data Structures**:
```rust
pub enum WeatherType { Clear, Rain, Storm, Snow }

pub struct Weather {
    pub current: WeatherType,
    pub intensity: f32,
    pub duration_remaining: f32,
}
```

**Key Methods**:
- `WeatherSystem::update(dt, season)` — Advances weather, rolls transitions
- `Weather::name()` — Returns weather type as string

**Integration**: Affects farming (auto-water on rain), visual effects, fish availability.

---

## Inventory System (`game/inventory.rs`)

**Purpose**: Item storage, tool management, and hotbar selection.

**Data Structures**:
```rust
pub enum ToolType { Hoe, WateringCan, Pickaxe, Axe, Scythe, FishingRod }

pub struct Tool { pub tool_type: ToolType, pub level: u8, pub energy_cost: u32 }

pub enum ItemCategory { Seed, Crop, Tool, Material, Food, Forage, Mineral, Fish, Crafted }

pub struct Item {
    pub id: u32, pub name: String, pub description: String,
    pub stack: u32, pub max_stack: u32, pub base_value: u32,
    pub category: ItemCategory, pub quality: ItemQuality,
}

pub struct InventorySlot { pub item_id: Option<u32>, pub quantity: u32 }

pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub hotbar: Vec<InventorySlot>,
    pub selected_hotbar: usize,
    pub gold: i32,
}
```

**Key Methods**:
- `Inventory::add_item(id, qty)` — Adds item, returns overflow quantity
- `Inventory::remove_item(id, qty)` — Removes item, returns actual removed
- `Inventory::has_items(id, qty)` — Checks item availability
- `Inventory::select_hotbar(index)` — Selects hotbar slot
- `Tool::upgrade_cost()` — Returns gold cost for next upgrade level

**Integration**: Used by farming (seed planting), crafting (ingredient consumption), cooking, economy (gold).

---

## Crafting System (`game/crafting.rs`)

**Purpose**: Recipe-based item creation with skill requirements.

**Data Structures**:
```rust
pub struct CraftingRecipe {
    pub id: u32,
    pub name: String,
    pub ingredients: Vec<(u32, u32)>,  // (item_id, quantity)
    pub result: (u32, u32),            // (item_id, quantity)
    pub category: String,
    pub required_level: u32,
}

pub struct CraftingRegistry { recipes: HashMap<u32, CraftingRecipe> }
```

**Key Methods**:
- `CraftingRegistry::can_craft(recipe_id, inventory_counts, skill_level)` — Checks feasibility
- `CraftingRegistry::craft(recipe_id, inventory_counts, skill_level)` — Consumes ingredients, returns result
- `CraftingRegistry::get(id)` — Returns recipe reference

**Integration**: Requires inventory items, checks skill levels, produces items for inventory.

---

## Skill System (`game/skill.rs`)

**Purpose**: 5-skill progression with XP curves and profession specializations.

**Data Structures**:
```rust
pub enum SkillType { Awareness, Focus, Creativity, Empathy, Logic }

pub enum Profession {
    InnerPeace, DeepVision, Precision, Endurance, Artistry,
    Innovation, Healing, Communion, Analysis, Synthesis,
}

pub struct Skill {
    pub skill_type: SkillType,
    pub level: u32,       // 1-10
    pub xp: u32,
    pub xp_to_next: u32,
    pub profession: Option<Profession>,
}

pub enum SkillEvent {
    LevelUp { skill: SkillType, level: u32 },
    ProfessionChoice { skill: SkillType, level: u32 },
}
```

**Key Methods**:
- `Skill::add_xp(amount)` — Adds XP, triggers level-ups, returns events
- `Skill::xp_for_level(level)` — Returns XP threshold (100, 250, 500, ...)

**Integration**: Levels gate crafting recipes, affect combat damage, unlock professions at levels 5/10.

---

## Farming System (`game/farming.rs`)

**Purpose**: Crop lifecycle management from tilling to harvest.

**Data Structures**:
```rust
pub enum CropState {
    Empty, Tilled,
    Seeded { seed_id: u32, day_planted: u32 },
    Growing { seed_id: u32, day_planted: u32, growth: f32 },
    Ready { seed_id: u32 },
    Withered,
}

pub struct CropTile {
    pub state: CropState,
    pub watered: bool,
    pub fertilized: bool,
}

pub struct FarmPlot { pub x: u32, pub y: u32, pub tile: CropTile }
```

**Key Methods**:
- `FarmingSystem::till(x, y)` — Tills empty tile
- `FarmingSystem::plant(x, y, seed_id)` — Plants seed on tilled tile
- `FarmingSystem::water(x, y)` — Waters crop tile
- `FarmingSystem::harvest(x, y, inventory)` — Harvests ready crop, adds to inventory
- `FarmingSystem::advance_day(season)` — Grows crops, checks wither conditions

**Integration**: Uses inventory for seeds/harvest, time system for growth, weather for auto-water.

---

## Mining System (`game/mining.rs`)

**Purpose**: Procedural mine exploration with mineral extraction and enemy encounters.

**Data Structures**:
```rust
pub enum MineralType {
    ThoughtFragment, CuriosityShard, LogicCrystal, EmpathyGem,
    CreativityOrb, FocusStone, WisdomOre, VoidEssence,
}

pub enum MineTile { Empty, Mineral(MineralType), Ladder, Staircase, Enemy(EnemyType), Chest }

pub enum EnemyType { KnowledgeGoblin, ConfusionSlime, DoubtBat, FearSpider }

pub struct MineFloor {
    pub depth: u32,
    pub tiles: Vec<Vec<MineTile>>,
    pub width: u32, pub height: u32,
    pub cleared: bool, pub has_ladder: bool,
}

pub struct MineState {
    pub current_depth: u32,
    pub floors: Vec<MineFloor>,
    pub player_x: u32, pub player_y: u32,
}
```

**Key Methods**:
- `MineState::generate_floor(depth)` — Procedurally generates mine floor
- `MineState::mine_tile(x, y)` — Extracts mineral, returns drops
- `MineState::descend()` — Goes to next floor depth

**Integration**: Drops feed into inventory/crafting, enemies trigger combat, minerals used for artisan recipes.

---

## Combat System (`game/combat.rs`)

**Purpose**: Turn-based combat encounters with 4 action types.

**Data Structures**:
```rust
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub hp: u32, pub max_hp: u32,
    pub attack: u32, pub defense: u32,
    pub xp_reward: u32,
    pub drop_id: Option<u32>,
    pub x: u32, pub y: u32,
}

pub enum CombatAction { Attack, Defend, UseItem(u32), Flee }

pub struct CombatResult {
    pub player_damage_taken: u32,
    pub enemy_damage_taken: u32,
    pub enemy_killed: bool,
    pub xp_gained: u32,
    pub drops: Vec<u32>,
    pub player_fled: bool,
}
```

**Key Methods**:
- `Enemy::new(type, depth)` — Scales stats with mine depth
- `Enemy::take_damage(damage)` — Applies damage, returns true if killed
- `CombatSystem::resolve(player_action, enemy)` — Resolves turn, returns result

**Integration**: Scales with mine depth, drops go to inventory, XP feeds skill system.

---

## Fishing System (`game/fishing.rs`)

**Purpose**: Season/time-based fishing with catch quality determination.

**Data Structures**:
```rust
pub struct Fish {
    pub id: u32, pub name: String,
    pub difficulty: u32, pub value: u32, pub xp: u32,
    pub seasons: Vec<Season>,
    pub times: Vec<u32>,
    pub min_depth: u32,
}

pub enum FishQuality { Normal, Silver, Gold, Iridium }

pub struct FishingSystem {
    pub fish_caught: Vec<(Fish, FishQuality)>,
    pub total_caught: u32,
    pub current_cast: Option<CastLine>,
}
```

**Key Methods**:
- `Fish::available_at(season, hour)` — Checks spawn conditions
- `FishingSystem::cast(season, hour, skill_level)` — Determines available fish, starts minigame
- `FishingSystem::catch(quality)` — Completes catch, adds to fish_caught

**Integration**: Season/hour from time system, skill level affects quality odds, fish used for cooking/economy.

---

## Foraging System (`game/foraging.rs`)

**Purpose**: World-spawned collectible items with seasonal/zone restrictions.

**Data Structures**:
```rust
pub struct ForageItem {
    pub id: u32, pub name: String,
    pub spawn_rate: f32,
    pub seasons: Vec<Season>,
    pub zones: Vec<String>,
    pub value: u32, pub xp: u32,
}

pub struct ForagingSystem {
    pub items_collected: HashMap<u32, u32>,
    pub total_collected: u32,
    pub spawn_points: Vec<(u32, u32, u32)>,  // (x, y, item_id)
}
```

**Key Methods**:
- `ForageItem::available_in_season(season)` — Checks seasonal availability
- `ForagingSystem::spawn_items(width, height, season)` — Populates world with forageables
- `ForagingSystem::collect(x, y)` — Picks up item at position

**Integration**: Season from time system, zones from world map, items used for cooking/gifting/selling.

---

## Cooking System (`game/cooking.rs`)

**Purpose**: Recipe-based food preparation with energy restoration.

**Data Structures**:
```rust
pub struct CookingRecipe {
    pub id: u32, pub name: String,
    pub ingredients: Vec<(u32, u32)>,
    pub result: (u32, u32),
    pub energy_restore: u32,
    pub skill_bonus: Option<(SkillType, u32)>,
}

pub struct CookingSystem {
    pub recipes_discovered: Vec<u32>,
    pub meals_cooked: u32,
}
```

**Key Methods**:
- `CookingSystem::can_cook(recipe_id, inventory)` — Checks ingredient availability
- `CookingSystem::cook(recipe_id, inventory, energy)` — Consumes ingredients, restores energy
- `CookingSystem::discover(recipe_id)` — Unlocks recipe for use

**Integration**: Ingredients from inventory/farming/foraging/fishing, restores player energy.

---

## Economy System (`game/economy.rs`)

**Purpose**: Gold management with earning/spending tracking.

**Data Structures**:
```rust
pub struct DailyEarning { pub day: u32, pub amount: i32, pub source: String }

pub struct Economy {
    pub gold: i32,
    pub total_earned: i32,
    pub total_spent: i32,
    pub daily_earnings: Vec<DailyEarning>,
}
```

**Key Methods**:
- `Economy::earn(amount, source, day)` — Adds gold, logs earning
- `Economy::spend(amount, source)` — Deducts gold, returns success
- `Economy::can_afford(amount)` — Checks gold availability

**Integration**: Gold used for shop purchases, tool upgrades, crafting, artisan machines.

---

## NPC System (`game/npc.rs`)

**Purpose**: 8 NPCs with roles, schedules, gift preferences, and dialogue.

**Data Structures**:
```rust
pub enum NpcRole { Merchant, Guide, Companion, Mentor, Challenge }

pub enum NpcType { Awareness, Focus, Creativity, Empathy, Memory, Logic, Wisdom, Dreams }

pub struct Npc {
    pub name: String,
    pub role: NpcRole,
    pub resonance: u32, pub max_resonance: u32,
    pub gifts_this_week: u32,
    pub liked_items: Vec<u32>,
    pub disliked_items: Vec<u32>,
    pub loved_items: Vec<u32>,
    pub schedule: Vec<NpcScheduleEntry>,
}

pub struct NpcScheduleEntry { pub hour: u32, pub x: u32, pub y: u32 }
```

**Key Methods**:
- `Npc::give_gift(item_id)` — Adjusts resonance based on preference
- `Npc::get_dialogue(resonance, quest_state)` — Returns appropriate dialogue tree
- `Npc::get_schedule(hour)` — Returns position for given hour

**Integration**: Resonance drives relationship progression, dialogue integrates with quest system.

---

## Relationship System (`game/relationship.rs`)

**Purpose**: Resonance-based NPC relationships with marriage and children.

**Data Structures**:
```rust
pub enum RelationshipStage {
    Stranger, Acquaintance, Friend, CloseFriend, BestFriend, Soulmate,
}

pub struct Relationship {
    pub npc_name: String,
    pub resonance: u32,
    pub stage: RelationshipStage,
    pub days_known: u32,
}

pub struct Child {
    pub name: String,
    pub age: u32,
    pub inherited_skill: SkillType,
}

pub struct RelationshipSystem {
    pub relationships: HashMap<String, Relationship>,
    pub married_to: Option<String>,
    pub children: Vec<Child>,
}
```

**Key Methods**:
- `RelationshipSystem::increase_resonance(npc, amount)` — Increases resonance, checks stage transitions
- `RelationshipSystem::can_marry(npc)` — Checks Best Friend stage + requirements
- `RelationshipSystem::marry(npc)` — Sets spouse, triggers marriage event

**Integration**: Resonance from gift-giving/dialogue, unlocks marriage at Best Friend, children inherit skills.

---

## Quest System (`game/quest.rs`)

**Purpose**: 27 quests with objectives, rewards, prerequisites, and deadlines.

**Data Structures**:
```rust
pub enum QuestStatus { Available, Active, Completed, Failed }
pub enum QuestType { Main, Side, Daily, Special, Achievement }

pub struct Quest {
    pub id: u32, pub name: String, pub description: String,
    pub quest_type: QuestType, pub giver: String,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<QuestReward>,
    pub status: QuestStatus,
    pub day_given: u32, pub day_deadline: Option<u32>,
    pub prerequisite_quests: Vec<u32>,
}

pub struct QuestObjective {
    pub description: String,
    pub target_id: Option<u32>,
    pub target_count: u32, pub current_count: u32,
    pub completed: bool,
}

pub enum QuestReward {
    Gold(u32), Item { item_id: u32, quantity: u32 },
    XP { skill: String, amount: u32 },
    Resonance { npc: String, amount: u32 },
    Unlock(String),
}
```

**Key Methods**:
- `QuestDatabase::get_available(player_quests)` — Returns quests meeting prerequisites
- `QuestDatabase::accept(quest_id)` — Activates quest
- `QuestDatabase::update_progress(quest_id, objective_idx, count)` — Advances objective
- `QuestDatabase::complete(quest_id, rewards)` — Marks complete, grants rewards

**Integration**: Prerequisites chain quests, rewards feed economy/skills/relationships, deadlines use time system.

---

## Events System (`game/events.rs`)

**Purpose**: 8 seasonal events with special activities and rewards.

**Data Structures**:
```rust
pub enum SeasonalEvent {
    FlowerFestival, FishingContest, ConcertInthePark,
    MineExplorationEvent, HarvestFestival, CookingContest,
    StargazingNight, MeditationRetreat,
}

pub struct EventCalendar {
    pub current_event: Option<SeasonalEvent>,
    pub days_until_event: u32,
    pub events_attended: Vec<SeasonalEvent>,
}
```

**Key Methods**:
- `SeasonalEvent::all()` — Returns all event types
- `SeasonalEvent::name()` / `description()` — Returns event metadata
- `EventCalendar::advance_day(season)` — Checks if event triggers today
- `EventCalendar::attend(event)` — Marks attendance, grants rewards

**Integration**: Events trigger on specific season/day combinations, grant unique rewards, unlock quest objectives.

---

## Perfection System (`game/perfection.rs`)

**Purpose**: Endgame tracker measuring completion across 8 categories.

**Data Structures**:
```rust
pub enum PerfectionCategory {
    Farming, Fishing, Mining, Combat,
    Relationships, Quests, Recipes, Shipping,
}

pub struct PerfectionTracker {
    pub categories: HashMap<PerfectionCategory, f32>,
    pub total_percent: f32,
}
```

**Key Methods**:
- `PerfectionTracker::update(category, value)` — Updates category progress
- `PerfectionTracker::calculate_total()` — Recalculates overall percentage
- `PerfectionTracker::is_complete()` — True when total >= 100%

**Integration**: Aggregates progress from all other systems (farm tiles, fish caught, quests completed, etc.).

---

## World System (`world/`)

### Tile System (`world/tile.rs`)
```rust
pub enum TileType { Grass, Water, Path, Farmable, Stone, Sand, Snow, ... }
pub struct Tile { pub tile_type: TileType, pub walkable: bool, pub layers: Vec<TileLayer> }
pub struct WorldMap { pub width: u32, pub height: u32, pub tiles: Vec<Vec<Tile>> }
```

### Generator (`world/generator.rs`)
```rust
pub enum BiomeType { Plains, Forest, Mountain, Lake, Desert }
pub struct GeneratorConfig { pub width: u32, pub height: u32, pub seed: u64, pub biomes: Vec<BiomeType> }
pub struct WorldGenerator { pub config: GeneratorConfig }
```

### Zones (`world/zone.rs`)
```rust
pub enum ZoneType { Farm, Forest, Mountain, Lake, Mine }
pub struct Zone { pub id: u32, pub name: String, pub zone_type: ZoneType, pub bounds: Rect }
pub struct ZoneConnection { pub from: u32, pub to: u32, pub position: Vec2 }
```

### Pathfinding (`world/pathfinding.rs`)
```rust
pub struct PathNode { pub x: i32, pub y: i32, pub g: f32, pub h: f32, pub parent: Option<usize> }
pub fn astar(start: Vec2, goal: Vec2, world: &WorldMap) -> Option<Vec<Vec2>>
```

---

## Engine Subsystems (`engine/`)

### Renderer (`engine/renderer.rs`)
- `CanvasRenderer` — 2D canvas rendering with color, shapes, sprites
- `SpriteBatch` — Batched sprite rendering for draw call optimization
- `TilemapRenderer` — Efficient tile rendering with culling
- `ParticleSystem` — Particle effect rendering
- `DebugRenderer` — Debug overlay rendering
- `ScreenEffects` — Post-processing effects
- `GameRenderer` — Top-level renderer coordinating all sub-renderers

### Physics (`engine/physics.rs`)
- `SimplePhysicsWorld` — AABB collision detection/response
- `RigidBody` — Static/Dynamic/Kinematic body types
- `Collider` — AABB/Circle collider shapes
- `PhysicsWorld` trait — Interface for physics implementations

### Input (`engine/input.rs`)
- `SimpleInputProvider` — Keyboard/mouse/gamepad state tracking
- `InputState` — Current frame input snapshot
- `KeyCode` — Keyboard key enumeration
- `MouseButton` / `GamepadAxis` / `GamepadButton` — Input device types

### Camera (`engine/camera.rs`)
- `Camera2D` — 2D camera with position, zoom, rotation
- `CameraBounds` — Viewport clamping constraints

### Asset (`engine/asset.rs`)
- `AssetServer` — Texture/font/sound loading and caching
- `AssetHandle<T>` — Reference-counted asset handles
- `TextureData` / `SoundData` / `FontData` — Asset data types

### Scene (`engine/scene.rs`)
- `SceneGraph` — Hierarchical transform tree
- `SceneNode` — Node with transform, children, and optional entity binding

### Event Bus (`engine/event_bus.rs`)
- `TypedEventBus` — Type-erased event dispatch with handler registration
- `GameEventHandler` trait — Event listener interface

---

## Save System (`save/`)

**Purpose**: Complete game state serialization with slot-based management.

**Data Structures**:
```rust
pub struct SaveData {
    pub version: u32, pub slot: u32,
    pub player_name: String, pub play_time_seconds: f64,
    pub day: u32, pub season: String, pub year: u32,
    pub hour: u32, pub minute: u32,
    pub energy: f32, pub max_energy: f32,
    pub skills: HashMap<String, u32>,
    pub inventory: Vec<InventorySlotData>,
    pub farm_tiles: Vec<FarmTileData>,
    pub npcs: Vec<NpcData>,
    pub current_zone: u32, pub unlocked_zones: Vec<u32>,
}

pub struct SaveManager { save_dir: PathBuf }
```

**Key Methods**:
- `SaveManager::save(slot, data)` — Serializes to JSON file
- `SaveManager::load(slot)` — Deserializes from JSON file
- `SaveManager::save_from_world(slot, world, tick)` — Extracts state from ECS world
- `SaveManager::list_saves()` — Lists all save slot numbers
