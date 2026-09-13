# NeoTrix Engine — Public API Reference

Complete API reference for all public types, traits, and functions in `nt_world_sim::engine`.

---

## Table of Contents

- [Math Primitives](#math-primitives)
- [ECS](#ecs)
- [Renderer](#renderer)
- [Physics](#physics)
- [Input](#input)
- [Camera](#camera)
- [Audio](#audio)
- [Map](#map)
- [Collision](#collision)
- [Combat](#combat)
- [Dialogue](#dialogue)
- [Quest](#quest)
- [NPC](#npc)
- [Inventory](#inventory)
- [Skill](#skill)
- [UI](#ui)
- [Effects](#effects)
- [Save/Load](#saveload)
- [Performance](#performance)
- [Resources](#resources)
- [Game](#game)

---

## Math Primitives

### `Vec2`

2D vector with `x: f32` and `y: f32`.

```rust
pub struct Vec2 { pub x: f32, pub y: f32 }

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self;
    pub fn zero() -> Self;
    pub fn one() -> Self;
    pub fn length(self) -> f32;
    pub fn normalize(self) -> Vec2;
}

impl std::ops::Add for Vec2 { ... }
impl std::ops::Sub for Vec2 { ... }
impl std::ops::Mul<f32> for Vec2 { ... }
```

### `Rect`

Axis-aligned rectangle.

```rust
pub struct Rect {
    pub x: f32, pub y: f32,
    pub width: f32, pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self;
    pub fn right(&self) -> f32;
    pub fn bottom(&self) -> f32;
    pub fn center(&self) -> Vec2;
    pub fn contains(&self, point: &Vec2) -> bool;
    pub fn intersects(&self, other: &Rect) -> bool;
}
```

### `Color`

RGBA color with `r, g, b, a: f32` (0.0–1.0).

```rust
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self;
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self;
    pub fn white() -> Self;
    pub fn black() -> Self;
    pub fn with_alpha(self, alpha: f32) -> Color;
}
```

### `Transform`

Spatial transform (re-exported from `core`).

```rust
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}
```

---

## ECS

### `Entity`

Generational arena ID.

```rust
pub struct Entity {
    pub id: u64,
    pub generation: u32,
}

impl Entity {
    pub fn new(id: u64, generation: u32) -> Self;
    pub fn id(self) -> u64;
}
```

### `Component` (trait)

Marker trait for ECS components.

```rust
pub trait Component: Any + Send + Sync + 'static {}
```

### `World`

Manages entities, components, and resources.

```rust
pub struct World { ... }

impl World {
    pub fn new() -> Self;
    pub fn spawn(&mut self) -> Entity;
    pub fn spawn_tagged(&mut self, tag: &str) -> Entity;
    pub fn find_by_tag(&self, tag: &str) -> Option<Entity>;
    pub fn despawn(&mut self, entity: Entity);
    pub fn is_alive(&self, entity: Entity) -> bool;
    pub fn entity_count(&self) -> usize;
    pub fn entities(&self) -> Vec<Entity>;

    // Component ops
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T);
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T>;
    pub fn has<T: Component>(&self, entity: Entity) -> bool;
    pub fn remove<T: Component>(&mut self, entity: Entity) -> bool;
    pub fn query<T: Component>(&self) -> Vec<Entity>;
    pub fn query2<A: Component, B: Component>(&self) -> Vec<Entity>;
    pub fn query3<A: Component, B: Component, C: Component>(&self) -> Vec<Entity>;

    // Resource ops
    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, resource: T);
    pub fn get_resource<T: 'static>(&self) -> Option<&T>;
    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T>;
}
```

### `System` (trait)

```rust
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self, world: &mut World, dt: f32);
    fn priority(&self) -> i32 { 0 }
}
```

### `SystemRunner`

Runs systems in priority order.

```rust
pub struct SystemRunner { ... }

impl SystemRunner {
    pub fn new() -> Self;
    pub fn add_system(&mut self, system: Box<dyn System>);
    pub fn run_all(&mut self, world: &mut World, dt: f32);
    pub fn system_count(&self) -> usize;
    pub fn system_names(&self) -> Vec<&str>;
}
```

---

## Renderer

### `Sprite`

Renderable sprite unit.

```rust
pub struct Sprite {
    pub x: f32, pub y: f32,
    pub width: f32, pub height: f32,
    pub texture_id: String,
    pub frame: u32,
    pub flip_x: bool, pub flip_y: bool,
    pub color: Color,
    pub alpha: f32,
    pub rotation: f32,
    pub z_index: i32,
}

impl Sprite {
    pub fn new(texture_id: &str) -> Self;
    pub fn from_rect(texture_id: &str, x: f32, y: f32, w: f32, h: f32) -> Self;
    pub fn colored(color: Color, w: f32, h: f32) -> Self;
    pub fn with_position(self, x: f32, y: f32) -> Self;
    pub fn with_frame(self, frame: u32) -> Self;
    pub fn with_alpha(self, alpha: f32) -> Self;
    pub fn with_rotation(self, rotation: f32) -> Self;
    pub fn with_z_index(self, z: i32) -> Self;
}
```

### `TileMap` (renderer)

Simple single-layer tile map for basic rendering.

```rust
pub struct TileMap {
    pub tiles: Vec<Vec<u32>>,
    pub tile_size: Vec2,
    pub palette: HashMap<u32, TileDef>,
}

impl TileMap {
    pub fn new(width: usize, height: usize, tile_size: Vec2) -> Self;
    pub fn get_tile(&self, x: usize, y: usize) -> Option<u32>;
    pub fn set_tile(&mut self, x: usize, y: usize, tile_id: u32);
    pub fn is_walkable(&self, x: usize, y: usize) -> bool;
}
```

### `DrawCommand`

Draw command enum for the rendering backend.

```rust
pub enum DrawCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawCircle { center: Vec2, radius: f32, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, width: f32 },
    DrawText { text: String, position: Vec2, color: Color, size: f32 },
    DrawSprite { texture: String, dest: Rect, src_rect: Option<Rect>, color: Color, alpha: f32, flip_x: bool, flip_y: bool, rotation: f32, z_index: i32 },
    DrawTilemap { tile_colors: Vec<(Rect, Color)>, z_index: i32 },
    DrawQuad { dest: Rect, color: Color, rotation: f32, z_index: i32 },
    DrawParticles { particles: Vec<ParticleDrawVertex> },
    Present,
}
```

### `Renderer` (trait)

```rust
pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform);
    fn draw_tilemap(&mut self, tilemap: &TileMap, camera: &Camera);
    fn draw_text(&mut self, text: &str, position: Vec2, color: Color, size: f32);
    fn draw_rect(&mut self, rect: &Rect, color: Color);
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color);
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32);
    fn present(&mut self);
    fn viewport_size(&self) -> Vec2;
}
```

### `CanvasRenderer`

Records `DrawCommand`s for a platform backend.

```rust
pub struct CanvasRenderer { ... }

impl CanvasRenderer {
    pub fn new(width: f32, height: f32) -> Self;
    pub fn drain_commands(&mut self) -> Vec<DrawCommand>;
    pub fn commands(&self) -> &[DrawCommand];
    pub fn frame_count(&self) -> u64;
}
```

### `GameRenderer`

Combined renderer holding all sub-renderers.

```rust
pub struct GameRenderer {
    pub sprite_batch: SpriteBatch,
    pub tilemap_renderer: TilemapRenderer,
    pub particle_system: ParticleSystem,
    pub debug_renderer: DebugRenderer,
    pub effects: ScreenEffects,
    pub camera: Camera,
    pub frame_timer: FrameTimer,
    pub draw_call_batcher: DrawCallBatcher,
    pub metrics: PerformanceMetrics,
}

impl GameRenderer {
    pub fn new(width: f32, height: f32) -> Self;
    pub fn begin_frame(&mut self);
    pub fn submit_tilemap(&mut self, tilemap: &TileMap);
    pub fn submit_sprite(&mut self, texture: &str, dest: Rect, color: Color, z_index: i32);
    pub fn end_frame(&mut self, dt: f32) -> Vec<DrawCommand>;
}
```

---

## Physics

### `PhysicsEntity`

Type alias for entity IDs in the physics system.

```rust
pub type PhysicsEntity = crate::core::entity::EntityId;
```

### `BodyType`

```rust
pub enum BodyType { Static, Dynamic, Kinematic }
```

### `RigidBody`

```rust
pub struct RigidBody {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub mass: f32,
    pub body_type: BodyType,
    pub gravity_scale: f32,
    pub friction: f32,
    pub restitution: f32,
}

impl RigidBody {
    pub fn new(body_type: BodyType) -> Self;
    pub fn dynamic() -> Self;
    pub fn static_body() -> Self;
    pub fn kinematic() -> Self;
}
```

### `Collider` (physics)

```rust
pub enum Collider {
    AABB { half_extents: Vec2 },
    Circle { radius: f32 },
    Polygon { vertices: Vec<Vec2> },
}

impl Collider {
    pub fn aabb(half_extents: Vec2) -> Self;
    pub fn circle(radius: f32) -> Self;
    pub fn polygon(vertices: Vec<Vec2>) -> Self;
    pub fn bounding_box(&self, position: Vec2) -> Rect;
}
```

### `PhysicsWorld` (trait)

```rust
pub trait PhysicsWorld {
    fn add_body(&mut self, entity: PhysicsEntity, body: RigidBody);
    fn remove_body(&mut self, entity: PhysicsEntity);
    fn add_collider(&mut self, entity: PhysicsEntity, collider: Collider);
    fn remove_collider(&mut self, entity: PhysicsEntity);
    fn get_body(&self, entity: PhysicsEntity) -> Option<&RigidBody>;
    fn get_body_mut(&mut self, entity: PhysicsEntity) -> Option<&mut RigidBody>;
    fn apply_force(&mut self, entity: PhysicsEntity, force: Vec2);
    fn apply_impulse(&mut self, entity: PhysicsEntity, impulse: Vec2);
    fn step(&mut self, dt: f32);
    fn query_point(&self, point: Vec2) -> Vec<PhysicsEntity>;
    fn query_rect(&self, rect: Rect) -> Vec<PhysicsEntity>;
    fn collisions(&self) -> &[CollisionInfo];
}
```

### `SimplePhysicsWorld`

```rust
pub struct SimplePhysicsWorld { ... }

impl SimplePhysicsWorld {
    pub fn new() -> Self;
    pub fn set_gravity(&mut self, gravity: Vec2);
}
impl PhysicsWorld for SimplePhysicsWorld { ... }
```

---

## Input

### Key/Button Enums

```rust
pub enum KeyCode { A, B, C, ..., Z, Num0, ..., Num9, F1, ..., F12, Space, Enter, Escape, ... }
pub enum MouseButton { Left, Right, Middle, X1, X2 }
pub enum GamepadAxis { LeftX, LeftY, RightX, RightY, LeftTrigger, RightTrigger }
pub enum GamepadButton { A, B, X, Y, LeftBumper, RightBumper, ... }
```

### `InputState`

```rust
pub struct InputState {
    pub keys_down: HashSet<KeyCode>,
    pub keys_just_pressed: HashSet<KeyCode>,
    pub keys_just_released: HashSet<KeyCode>,
    pub mouse_position: Vec2,
    pub mouse_delta: Vec2,
    pub mouse_scroll: f32,
    // ... gamepad state
}
```

### `InputProvider` (trait)

```rust
pub trait InputProvider {
    fn is_key_pressed(&self, key: KeyCode) -> bool;
    fn is_key_just_pressed(&self, key: KeyCode) -> bool;
    fn is_key_just_released(&self, key: KeyCode) -> bool;
    fn get_mouse_position(&self) -> Vec2;
    fn get_mouse_delta(&self) -> Vec2;
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;
    // ... gamepad methods
}
```

### `SimpleInputProvider`

```rust
pub struct SimpleInputProvider { ... }

impl SimpleInputProvider {
    pub fn new() -> Self;
    pub fn key_down(&mut self, key: KeyCode);
    pub fn key_up(&mut self, key: KeyCode);
    pub fn set_mouse_position(&mut self, position: Vec2);
    pub fn mouse_button_down(&mut self, button: MouseButton);
    pub fn mouse_button_up(&mut self, button: MouseButton);
}
impl InputProvider for SimpleInputProvider { ... }
```

---

## Camera

### `Camera2D`

Full-featured 2D camera with follow, shake, bounds, fade.

```rust
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub follow_smoothing: f32,
    pub bounds: Option<CameraBounds>,
    pub shake_intensity: f32,
    pub shake_offset: Vec2,
    pub screen_fade: f32,
}

impl Camera2D {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self;
    pub fn follow(&mut self, target_pos: Vec2);
    pub fn snap_to(&mut self, target_pos: Vec2);
    pub fn shake(&mut self, intensity: f32);
    pub fn update_shake(&mut self, dt: f32) -> Vec2;
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Vec2;
    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> Vec2;
    pub fn visible_world_rect(&self) -> Rect;
    pub fn is_visible(&self, rect: &Rect) -> bool;
    pub fn fade_to_black(&mut self, dt: f32) -> bool;
    pub fn fade_from_black(&mut self, dt: f32) -> bool;
    pub fn effective_position(&self) -> Vec2;
}
```

### `CameraBounds`

```rust
pub struct CameraBounds { pub min_x: f32, pub min_y: f32, pub max_x: f32, pub max_y: f32 }
```

---

## Audio

### `AudioManager`

```rust
pub struct AudioManager { ... }

impl AudioManager {
    pub fn new() -> Self;
    pub fn add_backend(&mut self, backend: Box<dyn AudioBackend>);
    pub fn load(&mut self, path: &str) -> Result<AudioHandle, String>;
    pub fn play(&mut self, handle: AudioHandle) -> Result<(), String>;
    pub fn play_categorized(&mut self, handle: AudioHandle, category: SoundCategory) -> Result<(), String>;
    pub fn play_spatial(&mut self, handle: AudioHandle, category: SoundCategory, position: [f32; 2]) -> Result<(), String>;
    pub fn stop(&mut self, handle: AudioHandle) -> Result<(), String>;
    pub fn set_master_volume(&mut self, volume: f32);
    pub fn set_category_volume(&mut self, category: SoundCategory, volume: f32);
    pub fn crossfade_music(&mut self, track: &str);
    pub fn set_listener_position(&mut self, pos: [f32; 2]);
    pub fn register_trigger(&mut self, trigger: AudioTrigger, sound_ids: Vec<String>);
    pub fn fire_trigger(&mut self, trigger: &AudioTrigger) -> Vec<Result<(), String>>;
    pub fn update(&mut self, dt: f32);
}
```

### `AudioBackend` (trait)

```rust
pub trait AudioBackend: Send + Sync {
    fn load_sound(&mut self, path: &str) -> Result<AudioHandle, String>;
    fn play_sound(&mut self, handle: AudioHandle, volume: f32) -> Result<(), String>;
    fn stop_sound(&mut self, handle: AudioHandle) -> Result<(), String>;
    fn set_volume(&mut self, handle: AudioHandle, volume: f32) -> Result<(), String>;
    fn set_position(&mut self, handle: AudioHandle, position: [f32; 2]) -> Result<(), String>;
    fn is_playing(&self, handle: AudioHandle) -> bool;
}
```

### `SoundCategory`

```rust
pub enum SoundCategory { Music, Sfx, Voice, Ambient }
```

### `AudioTrigger`

```rust
pub enum AudioTrigger {
    CombatStart, CombatEnd, PlayerHit, EnemyHit, PlayerDeath,
    LevelUp, QuestComplete, ItemPickup, MenuOpen, MenuClose,
    DialogueStart, DialogueEnd, Exploration,
    EnvironmentChange(String), Custom(String),
}
```

---

## Map

### `TileMap` (game)

Multi-layer tile map with collision, speed, damage per tile.

```rust
pub struct TileMap {
    pub layers: Vec<MapLayer>,
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub palette: HashMap<u32, Color>,
}

impl TileMap {
    pub fn new(width: usize, height: usize, tile_size: f32) -> Self;
    pub fn add_layer(&mut self, layer: MapLayer);
    pub fn remove_layer(&mut self, index: usize) -> Option<MapLayer>;
    pub fn get_layer(&self, name: &str) -> Option<&MapLayer>;
    pub fn tile_at(&self, layer: usize, x: usize, y: usize) -> Option<&Tile>;
    pub fn set_tile_at(&mut self, layer: usize, x: usize, y: usize, tile: Tile);
    pub fn is_walkable(&self, x: usize, y: usize) -> bool;
    pub fn is_walkable_world(&self, world_x: f32, world_y: f32) -> bool;
    pub fn get_speed_multiplier(&self, world_x: f32, world_y: f32) -> f32;
    pub fn get_damage(&self, world_x: f32, world_y: f32) -> i32;
    pub fn world_size(&self) -> Vec2;
    pub fn world_to_tile(&self, world_x: f32, world_y: f32) -> (usize, usize);
    pub fn tile_to_world(&self, x: usize, y: usize) -> Vec2;
}
```

### `Tile`

```rust
pub struct Tile {
    pub id: u32,
    pub properties: HashMap<String, TileProperty>,
    pub collision: CollisionType,
    pub damage: i32,
    pub speed: f32,
}
```

### `CollisionType`

```rust
pub enum CollisionType { None, Solid, OneWay, Water }
```

### `MapLayer`

```rust
pub struct MapLayer {
    pub name: String,
    pub tiles: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
    pub visible: bool,
    pub opacity: f32,
    pub parallax: (f32, f32),
    pub z_index: i32,
    pub collision_enabled: bool,
}
```

### `FogOfWar`

```rust
pub struct FogOfWar {
    pub states: Vec<Vec<FogState>>,
    pub width: usize,
    pub height: usize,
    pub visibility_radius: i32,
}

impl FogOfWar {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn reveal_circle(&mut self, tx: i32, ty: i32, radius: i32);
    pub fn reveal_around(&mut self, tx: usize, ty: usize);
    pub fn fade_visible(&mut self);
    pub fn is_visible(&self, x: usize, y: usize) -> bool;
    pub fn is_explored(&self, x: usize, y: usize) -> bool;
    pub fn reset(&mut self);
}
```

### `AutoTileSystem`

```rust
pub struct AutoTileSystem {
    pub active_tile_id: u32,
    pub base_tile_id: u32,
}

impl AutoTileSystem {
    pub fn new(active_tile_id: u32, base_tile_id: u32) -> Self;
    pub fn compute_flags(&self, layer: &MapLayer, x: usize, y: usize) -> AutoTileFlags;
    pub fn apply_to_layer(&self, layer: &mut MapLayer);
    pub fn apply_at(&self, layer: &mut MapLayer, x: usize, y: usize);
    pub fn smooth_boundary(&self, layer: &mut MapLayer, terrain_b_id: u32);
}
```

---

## Collision

### `AABB`

```rust
pub struct AABB { pub min: Vec2, pub max: Vec2 }

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> Self;
    pub fn from_center_half(center: Vec2, half: Vec2) -> Self;
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self;
    pub fn overlaps(&self, other: &AABB) -> bool;
    pub fn contains(&self, point: Vec2) -> bool;
    pub fn overlap_displacement(&self, other: &AABB) -> Option<Vec2>;
}
```

### `TileCollisionSystem`

```rust
pub struct TileCollisionSystem { ... }

impl TileCollisionSystem {
    pub fn new() -> Self;
    pub fn tile_aabb(tilemap: &TileMap, tx: usize, ty: usize) -> AABB;
    pub fn overlapping_tiles(tilemap: &TileMap, aabb: &AABB) -> Vec<(usize, usize)>;
    pub fn check_entity_collisions(&self, tilemap: &TileMap, entity_aabb: &AABB) -> Vec<TileCollisionResult>;
    pub fn resolve_movement(&self, tilemap: &TileMap, current_pos: Vec2, desired_pos: Vec2, entity_size: Vec2) -> (Vec2, Vec<TileCollisionResult>);
    pub fn push_out_of_solids(&self, tilemap: &TileMap, pos: Vec2, entity_size: Vec2) -> Vec2;
    pub fn raycast_tilemap(&self, tilemap: &TileMap, start: Vec2, end: Vec2) -> Option<(Vec2, f32, (usize, usize))>;
}
```

---

## Combat

### `CombatManager`

```rust
pub struct CombatManager { ... }

impl CombatManager {
    pub fn new() -> Self;
    pub fn add_entity(&mut self, entity: CombatEntity);
    pub fn remove_entity(&mut self, name: &str);
    pub fn get_entity(&self, name: &str) -> Option<&CombatEntity>;
    pub fn calculate_damage(attacker: &CombatEntity, defender: &CombatEntity, skill_multiplier: f32) -> DamageResult;
    pub fn apply_status_effect(&mut self, entity_name: &str, effect: StatusEffect);
    pub fn tick(&mut self, dt: f32);
}
```

### `CombatEntity`

```rust
pub struct CombatEntity {
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
    pub position: Vec2,
    pub state: CombatState,
    pub status_effects: Vec<StatusEffect>,
}
```

### `DamageResult`

```rust
pub struct DamageResult {
    pub damage: f32,
    pub is_crit: bool,
    pub blocked: bool,
}
```

---

## Dialogue

### `DialogueTree`

```rust
pub struct DialogueTree {
    pub nodes: HashMap<String, DialogueNode>,
    pub start_node: String,
}
```

### `DialogueNode`

```rust
pub struct DialogueNode {
    pub speaker: String,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
    pub conditions: Vec<DialogueCondition>,
    pub effects: Vec<DialogueEffect>,
    pub portrait: Option<DialoguePortrait>,
}
```

### `DialogueRunner`

```rust
pub struct DialogueRunner { ... }

impl DialogueRunner {
    pub fn new(tree: DialogueTree) -> Self;
    pub fn with_game_state(self, state: GameStateContext) -> Self;
    pub fn advance(&mut self) -> bool;
    pub fn select_choice(&mut self, index: usize) -> bool;
    pub fn current_node(&self) -> Option<&DialogueNode>;
    pub fn is_finished(&self) -> bool;
}
```

---

## Quest

### `QuestManager`

```rust
pub struct QuestManager { ... }

impl QuestManager {
    pub fn new() -> Self;
    pub fn add_quest(&mut self, quest: Quest);
    pub fn accept_quest(&mut self, quest_id: &str) -> bool;
    pub fn complete_objective(&mut self, quest_id: &str, objective_id: &str) -> bool;
    pub fn complete_quest(&mut self, quest_id: &str) -> bool;
    pub fn get_quest(&self, quest_id: &str) -> Option<&Quest>;
    pub fn active_quests(&self) -> Vec<&Quest>;
    pub fn completed_quests(&self) -> Vec<&Quest>;
}
```

### `Quest`

```rust
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub state: QuestState,
    pub objectives: Vec<Objective>,
    pub rewards: Vec<Reward>,
    pub prerequisites: Vec<String>,
    pub repeatable: bool,
}
```

---

## NPC

### `NPCManager`

```rust
pub struct NPCManager { ... }

impl NPCManager {
    pub fn new() -> Self;
    pub fn add_npc(&mut self, npc: NPC);
    pub fn remove_npc(&mut self, id: &str);
    pub fn get_npc(&self, id: &str) -> Option<&NPC>;
    pub fn update(&mut self, dt: f32, player_pos: Vec2);
    pub fn get_nearby_npcs(&self, pos: Vec2, radius: f32) -> Vec<&NPC>;
}
```

### `NPC`

```rust
pub struct NPC {
    pub id: String,
    pub name: String,
    pub state: NPCState,
    pub behavior: NPCBehavior,
    pub schedule: NPCSchedule,
    pub position: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub loot_table: Vec<LootEntry>,
    pub xp_reward: u32,
    pub aggro_radius: f32,
    pub deaggro_radius: f32,
}
```

---

## Inventory

### `Inventory`

```rust
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub max_slots: usize,
}

impl Inventory {
    pub fn new(max_slots: usize) -> Self;
    pub fn add_item(&mut self, item: Item, quantity: u32) -> bool;
    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> bool;
    pub fn count(&self, item_id: &str) -> u32;
    pub fn is_empty(&self) -> bool;
    pub fn slots(&self) -> &[InventorySlot];
}
```

### `Item`

```rust
pub struct Item {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub rarity: Rarity,
    pub stackable: bool,
    pub max_stack: u32,
    pub weight: f32,
    pub use_effect: Option<ItemUseEffect>,
}
```

### `ItemType`

```rust
pub enum ItemType { Consumable, Equipment, Material, Quest, Misc }
```

### `Rarity`

```rust
pub enum Rarity { Common, Uncommon, Rare, Epic, Legendary }
```

---

## Skill

### `SkillManager`

```rust
pub struct SkillManager { ... }

impl SkillManager {
    pub fn new() -> Self;
    pub fn add_skill(&mut self, skill: Skill);
    pub fn use_skill(&mut self, skill_id: &str) -> bool;
    pub fn tick_cooldowns(&mut self, dt: f32);
    pub fn get_skill(&self, skill_id: &str) -> Option<&Skill>;
    pub fn skills_by_type(&self, skill_type: SkillType) -> Vec<&Skill>;
}
```

### `Skill`

```rust
pub struct Skill {
    pub id: String,
    pub name: String,
    pub skill_type: SkillType,
    pub target: SkillTarget,
    pub base_damage: f32,
    pub mana_cost: f32,
    pub cooldown: f32,
    pub level: u32,
    pub max_level: u32,
    pub affixes: Vec<SkillAffix>,
}
```

---

## UI

### `UIRenderer`

```rust
pub struct UIRenderer { ... }

impl UIRenderer {
    pub fn new() -> Self;
    pub fn render_panel(&self, panel: &Panel) -> Vec<DrawCommand>;
    pub fn render_button(&self, button: &Button) -> Vec<DrawCommand>;
    pub fn render_bar(&self, bar: &Bar) -> Vec<DrawCommand>;
    pub fn render_text_label(&self, label: &TextLabel) -> Vec<DrawCommand>;
    pub fn render_inventory(&self, grid: &InventoryGrid) -> Vec<DrawCommand>;
    pub fn render_dialogue(&self, dialogue: &DialogueBox) -> Vec<DrawCommand>;
    pub fn render_minimap(&self, minimap: &UiMinimap, colors: &[Color], cols: usize, player_pos: Option<Vec2>, tile_size: f32) -> Vec<DrawCommand>;
}
```

### Widget Types

```rust
pub struct Panel { pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub style: UiTheme }
pub struct Button { pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub label: String, pub state: ButtonState }
pub struct Bar { pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub kind: BarKind, pub value: f32, pub max: f32 }
pub struct TextLabel { pub x: f32, pub y: f32, pub text: String, pub color: Color, pub size: f32 }
```

---

## Effects

### `EffectsRenderer`

```rust
pub struct EffectsRenderer { ... }

impl EffectsRenderer {
    pub fn new() -> Self;
    pub fn update(&mut self, dt: f32, particle_pool: &mut ParticlePool);
    pub fn render(&self, camera: &Camera) -> Vec<DrawCommand>;
    pub fn spawn_floating_number(&mut self, pos: Vec2, value: f32, kind: FloatingKind);
    pub fn spawn_skill_effect(&mut self, pos: Vec2, effect: SkillEffect);
}
```

### `FloatingKind`

```rust
pub enum FloatingKind { Damage, Heal, Critical, Miss, Experience }
```

### `ScreenShake`

```rust
pub struct ScreenShake { pub intensity: f32, pub decay: f32, pub offset: Vec2 }
```

---

## Save/Load

### `EngineSaveManager`

```rust
pub struct EngineSaveManager { ... }

impl EngineSaveManager {
    pub fn with_auto_save(backend: Box<dyn SaveBackend>, interval_secs: f64) -> Self;
    pub fn new_game(slot: u32, player_name: &str) -> SaveGameState;
    pub fn save(&mut self, slot: u32, state: &SaveGameState) -> Result<(), String>;
    pub fn load(&self, slot: u32) -> Result<SaveGameState, String>;
    pub fn auto_save_tick(&mut self, dt: f64, state: Option<&SaveGameState>) -> bool;
}
```

### `SaveBackend` (trait)

```rust
pub trait SaveBackend {
    fn save(&mut self, slot: u32, data: &[u8]) -> Result<(), String>;
    fn load(&self, slot: u32) -> Result<Vec<u8>, String>;
    fn exists(&self, slot: u32) -> bool;
    fn delete(&mut self, slot: u32) -> Result<(), String>;
}
```

### `FileSaveBackend`

```rust
pub struct FileSaveBackend { ... }
impl FileSaveBackend {
    pub fn new(dir: &str) -> Self;
}
impl SaveBackend for FileSaveBackend { ... }
```

### `MemorySaveBackend`

```rust
pub struct MemorySaveBackend { ... }
impl MemorySaveBackend {
    pub fn new() -> Self;
}
impl SaveBackend for MemorySaveBackend { ... }
```

---

## Performance

### `SpatialHashGrid`

```rust
pub struct SpatialHashGrid { ... }

impl SpatialHashGrid {
    pub fn new(cell_size: f32) -> Self;
    pub fn insert(&mut self, id: u64, rect: Rect);
    pub fn remove(&mut self, id: u64);
    pub fn query(&self, rect: Rect) -> Vec<u64>;
    pub fn potential_pairs(&self) -> Vec<(u64, u64)>;
}
```

### `PerfAggregator`

```rust
pub struct PerfAggregator { ... }

impl PerfAggregator {
    pub fn new() -> Self;
    pub fn tick(&mut self, dt: f32);
    pub fn format_overlay(&self) -> String;
    pub fn snapshot(&self) -> PerfSnapshot;
}
```

### `ObjectPool`

```rust
pub struct ObjectPool<T> { ... }

impl<T> ObjectPool<T> {
    pub fn new(create_fn: impl Fn() -> T, initial_size: usize) -> Self;
    pub fn acquire(&mut self) -> T;
    pub fn release(&mut self, item: T);
    pub fn available(&self) -> usize;
    pub fn in_use(&self) -> usize;
}
```

---

## Resources

### `ResourceManager`

```rust
pub struct ResourceManager { ... }

impl ResourceManager {
    pub fn new() -> Self;
    pub fn insert<T: Send + Sync + 'static>(&mut self, key: &str, data: T, size_bytes: usize);
    pub fn get<T: Send + Sync + 'static>(&self, key: &str) -> Option<ResourceHandle<T>>;
    pub fn get_ref<T: Send + Sync + 'static>(&self, handle: &ResourceHandle<T>) -> Option<&T>;
    pub fn get_mut<T: Send + Sync + 'static>(&mut self, handle: &ResourceHandle<T>) -> Option<&mut T>;
    pub fn contains<T: Send + Sync + 'static>(&self, key: &str) -> bool;
    pub fn remove<T: Send + Sync + 'static>(&mut self, key: &str) -> bool;
    pub fn total_count(&self) -> usize;
    pub fn total_memory_usage(&self) -> usize;
}
```

### `TypedResourceManager<T>`

```rust
pub struct TypedResourceManager<T: Send + Sync + 'static> { ... }

impl<T: Send + Sync + 'static> TypedResourceManager<T> {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: &str, resource: T);
    pub fn get(&self, key: &str) -> Option<Arc<T>>;
    pub fn contains(&self, key: &str) -> bool;
    pub fn remove(&mut self, key: &str) -> Option<Arc<T>>;
    pub fn count(&self) -> usize;
}
```

### `ResourceLoader` (trait)

```rust
pub trait ResourceLoader: Send + Sync {
    type Resource: Send + Sync + 'static;
    fn load(&self, path: &str) -> Result<Self::Resource, String>;
    fn name(&self) -> &str;
}
```

---

## Game

### `Game`

Top-level orchestrator tying all subsystems together.

```rust
pub struct Game {
    pub engine: GameEngine,
    pub config: GameConfig,
    pub world: World,
    pub system_runner: SystemRunner,
    pub renderer: CanvasRenderer,
    pub ui_renderer: UIRenderer,
    pub input: SimpleInputProvider,
    pub audio: AudioManager,
    pub save_manager: Option<EngineSaveManager>,
    pub menu: MenuSystem,
    pub dialogue: DialogueBox,
    pub inventory_grid: InventoryGrid,
    pub perf: PerfAggregator,
    pub particle_pool: ParticlePool,
    pub effects: EffectsRenderer,
    pub event_bus: TypedEventBus,
    pub phase: GamePhase,
    pub tick_count: u64,
    pub play_time: f64,
}

impl Game {
    pub fn new(config: GameConfig) -> Self;
    pub fn init_with_save(&mut self, backend: Box<dyn SaveBackend>);
    pub fn new_game(&mut self, slot: u32, player_name: &str);
    pub fn load_game(&mut self, slot: u32) -> Result<(), String>;
    pub fn save_game(&mut self) -> Result<(), String>;
    pub fn tick(&mut self, dt: f64);
    pub fn render(&mut self, alpha: f64);
    pub fn on_key_down(&mut self, key: KeyCode);
    pub fn on_key_up(&mut self, key: KeyCode);
    pub fn on_mouse_move(&mut self, x: f32, y: f32);
    pub fn on_mouse_button(&mut self, button: MouseButton, pressed: bool);
    pub fn toggle_pause(&mut self);
    pub fn start_dialogue(&mut self, speaker: &str, text: &str);
    pub fn end_dialogue(&mut self);
    pub fn drain_draw_commands(&mut self) -> Vec<DrawCommand>;
    pub fn spawn_entity(&mut self, x: f32, y: f32, texture: &str) -> Entity;
    pub fn entity_count(&self) -> usize;
}
```

### `GameConfig`

```rust
pub struct GameConfig {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub tick_rate: f64,
    pub max_frame_skip: u32,
    pub auto_save_interval_secs: f64,
    pub save_dir: String,
    pub particle_pool_size: usize,
    pub spatial_cell_size: f32,
}
```

### `GamePhase`

```rust
pub enum GamePhase {
    Init, Loading, MainMenu, Playing, Paused,
    Dialogue, Combat, Cutscene, GameOver,
}
```

### `GameCallbacks`

```rust
pub struct GameCallbacks<'a> { pub game: &'a mut Game }

impl<'a> GameLoopCallbacks for GameCallbacks<'a> {
    fn on_init(&mut self);
    fn on_tick(&mut self, dt: f64);
    fn on_render(&mut self, alpha: f64);
    fn on_state_change(&mut self, from: GameState, to: GameState);
    fn on_shutdown(&mut self);
}
```

---

## ECS Components

All ECS components implement the `Component` trait.

### Transform (ECS)

```rust
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}
```

### Velocity

```rust
pub struct Velocity { pub vx: f32, pub vy: f32 }
```

### Health

```rust
pub struct Health { pub current: f32, pub max: f32 }

impl Health {
    pub fn new(max: f32) -> Self;
    pub fn is_alive(&self) -> bool;
    pub fn is_full(&self) -> bool;
    pub fn ratio(&self) -> f32;
    pub fn take_damage(&mut self, amount: f32) -> f32;
    pub fn heal(&mut self, amount: f32) -> f32;
}
```

### Collider (ECS)

```rust
pub struct Collider {
    pub offset: Vec2,
    pub half_extents: Vec2,
    pub is_sensor: bool,
}

impl Collider {
    pub fn aabb(width: f32, height: f32) -> Self;
    pub fn sensor(width: f32, height: f32) -> Self;
    pub fn world_aabb(&self, transform: &Transform) -> Rect;
    pub fn overlaps(&self, a_tf: &Transform, other: &Collider, b_tf: &Transform) -> Option<Overlap>;
}
```

### Marker Components

```rust
pub struct PlayerMarker;
pub struct NpcMarker;
pub struct MonsterMarker;
```

### GameCamera (Resource)

```rust
pub struct GameCamera {
    pub position: Vec2,
    pub target: Option<Entity>,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub follow_speed: f32,
    pub dead_zone: Vec2,
}
```

### TimeState (Resource)

```rust
pub struct TimeState { pub dt: f32, pub elapsed: f64, pub frame: u64 }
```

### RenderCommandBuffer (Resource)

```rust
pub struct RenderCommandBuffer { pub commands: Vec<DrawCommand> }
```

---

## ECS Systems

All systems implement the `System` trait.

| System | Priority | Purpose |
|--------|----------|---------|
| `MovementSystem` | -100 | Integrates velocity into position |
| `EcsCollisionSystem` | 0 | AABB overlap detection between entities |
| `CameraSystem` | 10 | Smooth camera follow with dead zone |
| `HealthSystem` | 5 | Despawns entities with zero health |
| `RenderSystem` | 100 | Builds draw command buffer from sprites |
| `PlayerInfoSystem` | 200 | Reads player-specific state |
| `NpcAiSystem` | 50 | NPC behavior stub |
| `MonsterAiSystem` | 50 | Monster AI stub |
