# NT-WORLD-SIM Architecture

## Overview

NT-WORLD-SIM is a consciousness evolution simulation engine inspired by Stardew Valley. It implements NeoTrix's 6-layer consciousness architecture as a playable game engine with ECS, physics, rendering, and complete game mechanics.

## 6-Layer Architecture

### L6 Meta-Cognition
- **Quest system** — 27 quests (Main/Side/Daily/Special/Achievement) with objectives, rewards, and prerequisites
- **Event calendar** — 8 seasonal events (Flower Festival, Fishing Contest, Harvest Festival, etc.)
- **Perfection tracker** — Endgame tracker across 8 categories (farming, fishing, mining, combat, relationships, quests, recipes, shipping)
- **Achievement system** — Milestone tracking integrated with quest rewards

### L5 Cognition
- **Game loop** — Fixed-timestep game loop with state machine (Title/Playing/Paused/Dialogue/Inventory/Crafting/Mining/GameOver)
- **State machine** — `GameState` enum with transitions driven by player input and game events
- **Skill progression** — 5 skills (Awareness, Focus, Creativity, Empathy, Logic) × 10 levels with XP curves and profession choices at levels 5/10
- **Error handling** — `GameError` enum with Io/Serde/Game/Asset/Physics/Audio/Save/Load/InvalidState variants

### L4 Emotion
- **NPC resonance** — Relationship system with 8 NPCs (Awareness, Focus, Creativity, Empathy, Memory, Logic, Wisdom, Dreams)
- **Dialogue trees** — Branching dialogue with conditional responses based on resonance and quest state
- **Gift system** — Liked/disliked/loved item preferences per NPC, weekly gift limits
- **Relationship system** — Stages from Stranger → Acquaintance → Friend → Close Friend → Best Friend → Soulmate, with marriage and children

### L3 Embodiment
- **Combat system** — Turn-based combat with Attack/Defend/UseItem/Flee actions, 4 enemy types (Knowledge Goblin, Confusion Slime, Doubt Bat, Fear Spider)
- **Mining system** — Procedural mine floors with 8 mineral types, enemies, ladders, chests
- **Fishing system** — 10 fish types with season/time/depth requirements, quality tiers (Normal/Silver/Gold/Iridium)
- **Foraging system** — 6 forage items with spawn rates, seasonal/zone restrictions
- **Cooking system** — 30 cooking recipes with ingredient requirements and energy restore
- **Artisan machines** — Process raw materials into refined goods (juice, wine, oil, etc.)

### L2 Perception
- **World generation** — Procedural world generation with biome types
- **Tile system** — Multi-layer tile map with 25+ tile types
- **Pathfinding** — A* pathfinding algorithm
- **Camera system** — 2D camera with follow, bounds, and shake effects
- **Particle system** — Particle effects for visual feedback

### L1 Action
- **Inventory system** — Stackable items, hotbar (12 slots) + main inventory (20 slots), tool upgrades
- **Crafting system** — 30 crafting recipes with skill level requirements
- **Economy system** — Gold tracking, daily earnings, spending history
- **Input mapping** — Keyboard, mouse, and gamepad input with configurable mappings
- **Rendering system** — Canvas-based 2D renderer with sprite batching, tile rendering, and screen effects

## Core Modules

### core/
- **Entity-Component system** — `UniversalEntity` with archetype-based storage, `Component` and `Resource` traits
- **World (ECS container)** — `UniversalWorld` managing entities, components, and resources
- **Scheduler** — `ParallelScheduler` with dependency-aware wave-parallel execution
- **Math** — `Vec2`, `Rect`, `Color`, `Transform` types
- **Change detection** — `Changed<T>` wrapper for dirty-flag propagation

### engine/
- **Renderer** — `CanvasRenderer` implementing `Renderer` trait with Color, Vec2, Rect, Transform, Sprite primitives
- **Physics** — `SimplePhysicsWorld` with AABB collision detection/response, RigidBody (static/dynamic/kinematic), Collider
- **Input** — `SimpleInputProvider` with KeyCode, MouseButton, GamepadAxis, GamepadButton support
- **Audio** — `AudioManager` with `StubAudioBackend` (ready for real backend integration)
- **Scene** — `SceneGraph` with hierarchical transforms and `SceneNode` tree
- **Camera** — `Camera2D` with follow, bounds clamping, and screen shake
- **Asset** — `AssetServer` with texture/font/sound management and async loading
- **Particles** — `ParticleSystem` for visual effects
- **Debug overlay** — Runtime debug information display
- **Event bus** — `TypedEventBus` for decoupled event dispatch
- **Sprite batch** — `SpriteBatch` for efficient batched rendering

### game/
- **Time** — Day/night cycle (6AM-2AM), 4 seasons (Clarity/Flow/Reflection/Stillness), year tracking
- **Weather** — Weather system (Clear/Rain/Storm/Snow) with seasonal effects
- **Farming** — Crop planting, growth stages (Empty → Tilled → Seeded → Growing → Ready → Withered), watering, harvesting
- **Inventory** — Item/Tool management, stacking, hotbar selection, tool upgrade paths
- **Crafting** — Recipe registry with ingredient checking, skill level requirements
- **Skills** — 5 skills with XP curves, level-up events, profession choices
- **Mining** — Procedural mine floors, mineral extraction, enemy encounters, ladder progression
- **Combat** — Turn-based encounters with 4 action types, damage calculation, loot drops
- **Fishing** — Season/time-based fish spawning, catch minigame, quality determination
- **Foraging** — World-spawned collectible items with seasonal/zone restrictions
- **Cooking** — Recipe-based food preparation with energy restoration
- **Economy** — Gold management, earning/spending tracking, daily earnings log
- **NPC** — 8 NPCs with roles (Merchant/Guide/Companion/Mentor/Challenge), schedules, gift preferences
- **Relationship** — Resonance-based progression, marriage system, children
- **Quests** — 27 quests with objectives, rewards, prerequisites, and deadlines
- **Events** — 8 seasonal events with special activities and rewards
- **Perfection** — Endgame tracker across 8 categories toward 100% completion

### world/
- **Tiles** — Multi-layer tile system with 25+ tile types (grass, water, path, farmable, etc.)
- **Generator** — Procedural world generation with biome configuration
- **Zones** — 5 zones (Farm, Forest, Mountain, Lake, Mine) with metadata and connections
- **Pathfinding** — A* algorithm for NPC/player navigation

### ui/
- **Theme** — Stardew Valley wooden aesthetic with UiDrawCommand rendering
- **Widgets** — Button, Slider, ProgressBar base widgets
- **HUD** — Time display, energy bar, gold counter
- **Inventory UI** — Grid-based inventory display with item tooltips
- **Dialogue box** — Branching dialogue display with choice selection
- **Minimap** — World overview with player/NPC markers
- **Quest tracker** — Active quest display with objective progress
- **Game UI manager** — Coordinates all UI layers, notifications, tooltips

### adapters/
- **Bevy** — Bevy ECS adapter for component/system mapping
- **Unity** — Unity DOTS adapter for IComponentData/ISystem mapping
- **Godot** — Godot 4 adapter for node/signal mapping
- **Unreal** — Unreal Engine 5 adapter for GAS attribute mapping

### codegen/
- **Game definition** — `GameDefinition` struct with entities, systems, engine config
- **Parser** — YAML/JSON game definition parsing
- **Generator** — Code generation for Bevy/Unity/Godot targets

### save/
- **SaveManager** — Slot-based save/load with JSON serialization
- **SaveData** — Complete game state serialization (time, skills, inventory, farm, NPCs, world)

## Data Flow

```
Input → GameLoop → Systems → World → Renderer → Screen
         ↓
    EventBus → Event Handlers
         ↓
    Save/Load → Disk
```

## Performance Targets
- 60 FPS target with fixed-timestep accumulator
- ECS archetype layout for cache-friendly component access
- Wave-parallel system scheduling via `ParallelScheduler`
- Dirty flag propagation via `Changed<T>` change detection
- Sprite batching for efficient draw call reduction
