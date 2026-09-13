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
pub mod components;
pub mod systems;
pub mod map;
pub mod dialogue;
pub mod quest;
pub mod npc;
pub mod resources;
pub mod physics;

pub use renderer::{Color, Vec2, Rect, Transform, Sprite, TileDef, TileMap, Camera, Renderer, CanvasRenderer};
pub use renderer::{SpriteBatch, TilemapRenderer, ParticleSystem, DebugRenderer, ScreenEffects, GameRenderer};
pub use renderer::{FrameTimer, DrawCallBatcher, PerformanceMetrics};
pub use physics::{PhysicsEntity, BodyType, RigidBody, Collider as PhysicsCollider, CollisionInfo, PhysicsWorld, SimplePhysicsWorld};
pub use input::{KeyCode, MouseButton, GamepadAxis, GamepadButton, InputState, InputProvider, SimpleInputProvider};
pub use camera::{Camera2D, CameraBounds};
pub use asset::{AssetServer, AssetHandle, TextureData, SoundData, FontData, GlyphData};

// Core game loop
pub use core::{GameState, StateStack, StackEntry, CoreEvent, CoreEventBus, LoopConfig, LoopTimer, GameEngine, GameLoopCallbacks};

// ECS
pub use ecs::{Entity, Component, World, System, SystemRunner, CollisionEvent};

// Components
pub use components::{
    Transform as EcsTransform, Velocity, GameSprite, Health, Collider,
    PlayerMarker, NpcMarker, MonsterMarker,
    GameCamera, TimeState, RenderCommands, RenderCmd,
};

// Systems
pub use systems::{
    MovementSystem, CollisionSystem, CameraSystem, HealthSystem,
    RenderSystem, PlayerInfoSystem, NpcAiSystem, MonsterAiSystem,
};

// Map
pub use map::{Tile, TileProperty, MapLayer, TileMap as GameTileMap, AutoTileSystem, AutoTileFlags, FogOfWar, FogState, MinimapRenderer};

// Dialogue
pub use dialogue::{DialogueNode, DialogueChoice, DialogueCondition, DialogueTree, DialogueRunner, DialogueNodeDisplay, ChoiceDisplay};

// Quest
pub use quest::{Quest, QuestState, Objective, ObjectiveType, Reward, QuestManager, JournalEntry};

// NPC
pub use npc::{NPC, NPCState, NPCBehavior, NPCSchedule, ScheduleEntry, NPCRelationship, RelationTier, NPCManager};

// Resources
pub use resources::{ResourceHandle, ResourceManager, TypedResourceManager, ResourceLoader};
