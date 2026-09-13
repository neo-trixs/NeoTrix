pub mod core;
pub mod renderer;
pub mod input;
pub mod event_bus;
pub mod audio;
pub mod scene;
pub mod camera;
pub mod asset;
pub mod sprite_batch;
pub mod particle;
pub mod input_map;
pub mod debug_overlay;
pub mod ecs;
pub mod map;
pub mod collision;
pub mod mapgen;
pub mod dialogue;
pub mod quest;
pub mod npc;
pub mod resources;
pub mod inventory;
pub mod combat;
pub mod skill;
pub mod physics;
pub mod components;
pub mod systems;
pub mod ui;
pub mod effects;

pub use renderer::{Color, Vec2, Rect, Transform, Sprite, TileDef, TileMap, Camera, Renderer, CanvasRenderer};
pub use renderer::{SpriteBatch, TextureAtlas, TilemapRenderer, ParticleSystem, DebugRenderer, ScreenEffects, GameRenderer};
pub use renderer::{FrameTimer, DrawCallBatcher, PerformanceMetrics};
pub use physics::{PhysicsEntity, BodyType, RigidBody, Collider as PhysicsCollider, CollisionInfo, PhysicsWorld, SimplePhysicsWorld};
pub use input::{KeyCode, MouseButton, GamepadAxis, GamepadButton, InputState, InputProvider, SimpleInputProvider};
pub use camera::{Camera2D, CameraBounds};
pub use asset::{AssetServer, AssetHandle, TextureData, SoundData, FontData, GlyphData};

pub use core::{GameState, StateStack, StackEntry, CoreEvent, CoreEventBus, LoopConfig, LoopTimer, GameEngine, GameLoopCallbacks};
pub use ecs::{Entity, Component, World, System, SystemRunner, CollisionEvent};

// ECS components (game-level)
pub use components::{
    Transform as EcsTransform, Velocity, GameSprite, Health, Collider,
    PlayerMarker, NpcMarker, MonsterMarker,
    GameCamera, TimeState, RenderCommandBuffer,
};

// ECS systems
pub use systems::{
    MovementSystem, EcsCollisionSystem, CameraSystem, HealthSystem,
    RenderSystem, PlayerInfoSystem, NpcAiSystem, MonsterAiSystem,
};
pub use map::{Tile, TileProperty, CollisionType, MapLayer, TileMap as GameTileMap, AutoTileSystem, AutoTileFlags, FogOfWar, FogState, MinimapRenderer};
pub use collision::{AABB, TileCollisionSystem, TileCollisionResult};
pub use mapgen::{ValueNoise, Biome, RegionTemplate, MapGenConfig, MapGenerator, GeneratedMap};

pub use dialogue::{
    DialogueNode, DialogueChoice, DialogueCondition, DialogueEffect, DialoguePortrait,
    DialogueTree, DialogueRunner, DialogueNodeDisplay, ChoiceDisplay,
    GameStateContext, QuestConditionState, PortraitPosition,
};

pub use quest::{
    Quest, QuestState, Objective, ObjectiveType, Reward, QuestManager, JournalEntry,
};

pub use npc::{
    NPC, NPCState, NPCBehavior, NPCSchedule, ScheduleEntry,
    NPCRelationship, RelationTier, NPCManager, BehaviorTransition, NPCEvent,
};

pub use inventory::{
    Item, ItemType, EquipSlot, Rarity, ItemUseEffect,
    InventorySlot, Inventory,
};

pub use combat::{
    CombatEntity, CombatState, CombatManager, AutoCombatAI,
    StatusEffect, DamageInput, DamageResult,
    SkillCast, CombatLogEntry, AIDecision, LootTable, LootEntry, LootDrop,
    StatusEvent, calculate_damage, calc_combat_damage,
};

pub use skill::{
    Skill, SkillType, SkillTarget, SkillAffix, AffixType, AffixBonus,
    SkillEffect, SkillCooldowns, SkillManager,
};

pub use resources::{ResourceHandle, ResourceManager, TypedResourceManager, ResourceLoader};

// UI
pub use ui::{UiTheme, Panel, Button, ButtonState, TextLabel, Bar, BarKind, InventoryGrid, UiMinimap, UIRenderer};
pub use ui::InventorySlot as UiInventorySlot;

// Effects
pub use effects::{FloatingNumber, FloatingKind, FloatingNumberManager, SkillEffect as EffectSkillSkill, SkillEffectKind, SkillEffectManager, ScreenShake, EffectsRenderer};
