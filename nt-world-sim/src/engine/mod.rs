pub mod renderer;
pub mod physics;
pub mod input;
pub mod events;
pub mod pet_state;
pub mod hook_system;
pub mod theme_system;

pub use renderer::{Color, Vec2, Rect, Transform, Sprite, TileDef, TileMap, Camera, Renderer, CanvasRenderer};
pub use physics::{BodyType, RigidBody, Collider, CollisionInfo, PhysicsWorld, SimplePhysicsWorld};
pub use input::{KeyCode, MouseButton, GamepadAxis, GamepadButton, InputState, InputProvider, SimpleInputProvider};
pub use events::{Event, EventHandler, EventBus, MoveEvent, CollisionEvent, InputEvent, GameEvent};
pub use pet_state::{
    PetState, PetStateComponent, PetAnimation, EyeTracking, PermissionBubble, 
    PermissionAction, SessionInfo, SubagentInfo, ZzzParticle,
    PetStateSystem, EyeTrackingSystem, PermissionBubbleSystem, SessionSystem,
};
pub use hook_system::{
    HookEvent, HookConfig, HookManager, PermissionMode, PermissionRequest,
    PermissionBubbleLayout, PermissionHotkeys, hook_event_to_pet_state,
};
pub use theme_system::{
    ThemeConfig, AnimationDef, ShadowDef, ThemeManager, ThemeVariant,
    SpriteSheet, AnimationPlayer,
};
