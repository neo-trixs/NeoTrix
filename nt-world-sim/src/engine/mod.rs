//! # NeoTrix Game Engine
//!
//! A modular 2D game engine with ECS, physics, rendering, and audio.
//!
//! ## Architecture
//!
//! The engine is organized into layered subsystems:
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │  Game (top-level orchestrator)               │
//! ├─────────────────────────────────────────────┤
//! │  ECS Layer: World + Components + Systems     │
//! ├─────────────────────────────────────────────┤
//! │  Core Layer: StateStack, EventBuses, Loop    │
//! ├─────────────────────────────────────────────┤
//! │  Subsystems: Physics, Audio, Map, Combat...  │
//! ├─────────────────────────────────────────────┤
//! │  Math primitives (Vec2, Rect, Color)         │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nt_world_sim::engine::{
//!     Game, GameConfig, GamePhase,
//!     SimpleInputProvider, KeyCode,
//! };
//!
//! let config = GameConfig::default();
//! let mut game = Game::new(config);
//! game.new_game(0, "Hero");
//! game.tick(1.0 / 60.0);
//! ```

// ===========================================================================
// Module declarations
// ===========================================================================

/// Math primitives: `Vec2`, `Rect`, `Color`, `Transform` — single fact source.
pub mod core;

/// Renderer: sprites, tilemaps, particles, debug overlay, screen effects.
pub mod renderer;

/// Input: keyboard, mouse, gamepad state and provider trait.
pub mod input;

/// Typed event bus for decoupled inter-system communication.
pub mod event_bus;

/// Audio: backends, spatial audio, crossfader, triggers, manager.
pub mod audio;

/// Scene graph: hierarchical transforms, dirty flags, queries.
pub mod scene;

/// Camera: full-featured 2D camera with follow, shake, bounds.
pub mod camera;

/// Asset server: textures, sounds, fonts with handles.
pub mod asset;

/// Sprite batching for efficient draw calls.
pub mod sprite_batch;

/// Particle system: emit, update, render.
pub mod particle;

/// Key mapping / input mapping utilities.
pub mod input_map;

/// Debug overlay: grid, collision boxes, FPS, text overlays.
pub mod debug_overlay;

/// ECS: entities, components, world, systems, system runner.
pub mod ecs;

/// Tile map: multi-layer maps, auto-tiling, fog of war, minimap.
pub mod map;

/// Tile collision: AABB overlap, raycast, movement resolution.
pub mod collision;

/// Procedural map generation: noise, biomes, regions.
pub mod mapgen;

/// Dialogue system: trees, choices, conditions, effects.
pub mod dialogue;

/// Quest system: objectives, rewards, journal.
pub mod quest;

/// NPC system: AI, schedules, relationships, managers.
pub mod npc;

/// Generic resource manager with reference counting.
pub mod resources;

/// Inventory system: items, slots, equipment.
pub mod inventory;

/// Combat system: damage, status effects, AI, loot.
pub mod combat;

/// Skill system: skills, affixes, cooldowns.
pub mod skill;

/// Physics: rigid bodies, colliders, collision detection and resolution.
pub mod physics;

/// ECS components: Transform, Velocity, Health, Collider, markers.
pub mod components;

/// ECS systems: Movement, Collision, Camera, Health, Render, AI.
pub mod systems;

/// UI widgets: panels, buttons, bars, inventory grid, minimap.
pub mod ui;

/// Visual effects: floating numbers, skill effects, screen shake.
pub mod effects;

/// Save/load: backends, auto-save, game state serialization.
pub mod save;

/// Performance: spatial hashing, viewport culling, object pooling.
pub mod perf;

/// Game: top-level orchestrator tying all subsystems together.
pub mod game;

// ===========================================================================
// Re-exports — Math primitives (from crate::core)
// ===========================================================================

pub use crate::core::{Vec2, Rect, Color, Transform};

// ===========================================================================
// Re-exports — Renderer
// ===========================================================================

pub use renderer::{
    Sprite, TileDef,
    Renderer, CanvasRenderer,
    SpriteBatch, TextureAtlas, TilemapRenderer,
    ParticleSystem, DebugRenderer, ScreenEffects,
    GameRenderer,
    FrameTimer, DrawCallBatcher, PerformanceMetrics,
};

// ===========================================================================
// Re-exports — Physics
//
// Note: `Collider` is renamed to `PhysicsCollider` to avoid conflict with
// the ECS `components::Collider` (AABB component).
// ===========================================================================

pub use physics::{
    PhysicsEntity, BodyType, RigidBody,
    Collider as PhysicsCollider,
    CollisionInfo,
    PhysicsWorld, SimplePhysicsWorld,
};

// ===========================================================================
// Re-exports — Input
// ===========================================================================

pub use input::{
    KeyCode, MouseButton, GamepadAxis, GamepadButton,
    InputState, InputProvider, SimpleInputProvider,
};

// ===========================================================================
// Re-exports — Camera
// ===========================================================================

pub use camera::{Camera2D, CameraBounds};

// ===========================================================================
// Re-exports — Asset
// ===========================================================================

pub use asset::{AssetServer, AssetHandle, TextureData, SoundData, FontData, GlyphData};

// ===========================================================================
// Re-exports — Core (game loop / state machine)
// ===========================================================================

pub use core::{
    GameState, StateStack, StackEntry,
    CoreEvent, CoreEventBus,
    LoopConfig, LoopTimer,
    GameEngine, GameLoopCallbacks,
};

// ===========================================================================
// Re-exports — ECS
// ===========================================================================

pub use ecs::{Entity, Component, World, System, SystemRunner, CollisionEvent};

// ===========================================================================
// Re-exports — ECS Components
//
// `Transform` is renamed to `EcsTransform` to avoid conflict with the math
// `core::Transform`. `Collider` is already disambiguated via `PhysicsCollider`
// for the physics version.
// ===========================================================================

pub use components::{
    Transform as EcsTransform, Velocity, GameSprite, Health, Collider,
    PlayerMarker, NpcMarker, MonsterMarker,
    GameCamera, TimeState, RenderCommandBuffer,
};

// ===========================================================================
// Re-exports — ECS Systems
// ===========================================================================

pub use systems::{
    MovementSystem, EcsCollisionSystem, CameraSystem, HealthSystem,
    RenderSystem, PlayerInfoSystem, NpcAiSystem, MonsterAiSystem,
};

// ===========================================================================
// Re-exports — Map
//
// `TileMap` is renamed to `GameTileMap` to avoid conflict with the simple
// `renderer::TileMap`. Prefer `GameTileMap` for multi-layer maps.
// ===========================================================================

pub use map::{
    Tile, TileProperty, CollisionType, MapLayer,
    TileMap as GameTileMap,
    AutoTileSystem, AutoTileFlags,
    FogOfWar, FogState, MinimapRenderer,
};

// ===========================================================================
// Re-exports — Collision
// ===========================================================================

pub use collision::{AABB, TileCollisionSystem, TileCollisionResult};

// ===========================================================================
// Re-exports — Map Generation
// ===========================================================================

pub use mapgen::{ValueNoise, Biome, RegionTemplate, MapGenConfig, MapGenerator, GeneratedMap};

// ===========================================================================
// Re-exports — Dialogue
// ===========================================================================

pub use dialogue::{
    DialogueNode, DialogueChoice, DialogueCondition, DialogueEffect, DialoguePortrait,
    DialogueTree, DialogueRunner, DialogueNodeDisplay, ChoiceDisplay,
    GameStateContext, QuestConditionState, PortraitPosition,
};

// ===========================================================================
// Re-exports — Quest
// ===========================================================================

pub use quest::{
    Quest, QuestState, Objective, ObjectiveType, Reward, QuestManager, JournalEntry,
};

// ===========================================================================
// Re-exports — NPC
// ===========================================================================

pub use npc::{
    NPC, NPCState, NPCBehavior, NPCSchedule, ScheduleEntry,
    NPCRelationship, RelationTier, NPCManager, BehaviorTransition, NPCEvent,
};

// ===========================================================================
// Re-exports — Inventory
// ===========================================================================

pub use inventory::{
    Item, ItemType, EquipSlot, Rarity, ItemUseEffect,
    InventorySlot, Inventory,
};

// ===========================================================================
// Re-exports — Combat
// ===========================================================================

pub use combat::{
    CombatEntity, CombatState, CombatManager, AutoCombatAI,
    StatusEffect, DamageInput, DamageResult,
    SkillCast, CombatLogEntry, AIDecision, LootTable, LootEntry, LootDrop,
    StatusEvent, calculate_damage, calc_combat_damage,
};

// ===========================================================================
// Re-exports — Skill
// ===========================================================================

pub use skill::{
    Skill, SkillType, SkillTarget, SkillAffix, AffixType, AffixBonus,
    SkillEffect, SkillCooldowns, SkillManager,
};

// ===========================================================================
// Re-exports — Resources
// ===========================================================================

pub use resources::{ResourceHandle, ResourceManager, TypedResourceManager, ResourceLoader};

// ===========================================================================
// Re-exports — UI
// ===========================================================================

pub use ui::{
    UiTheme, Panel, Button, ButtonState, TextLabel,
    Bar, BarKind, InventoryGrid, UiMinimap, UIRenderer,
};
pub use ui::InventorySlot as UiInventorySlot;

// ===========================================================================
// Re-exports — Effects
//
// `SkillEffect` is renamed to `EffectSkillEffect` to avoid conflict with
// the `skill::SkillEffect`.
// ===========================================================================

pub use effects::{
    FloatingNumber, FloatingKind, FloatingNumberManager,
    SkillEffect as EffectSkillEffect, SkillEffectKind,
    SkillEffectManager, ScreenShake, EffectsRenderer,
};

// ===========================================================================
// Re-exports — Save/Load
// ===========================================================================

pub use save::{
    SaveBackend, FileSaveBackend, MemorySaveBackend,
    AutoSaveManager, EngineSaveManager,
    SaveGameState, SaveMetadata, PlayerSave, Vec2Save,
};

// ===========================================================================
// Re-exports — Performance
// ===========================================================================

pub use perf::{
    SpatialHashGrid, ViewportCuller, ObjectPool,
    FrameTimeMonitor, MemoryTracker,
    PerfAggregator, PerfSnapshot,
};

// ===========================================================================
// Re-exports — Game (top-level)
// ===========================================================================

pub use game::{Game, GameConfig, GamePhase, GameCallbacks};
