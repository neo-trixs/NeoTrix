/// NT-GAME 渲染模块 — 吸收 Godot 4 分层渲染架构
///
/// 本模块提供视觉游戏引擎的核心能力：
/// - 视觉组件（Transform2D, Sprite, Camera2D, Collider, etc.）
/// - 场景管理（Prefab + SceneInstance）
/// - 状态机（游戏状态切换）
/// - 输入系统（UE5 Enhanced Input + Godot InputMap）
/// - 渲染管线（macroquad 即时模式）
///
/// 设计参考：
/// - Godot 4: SceneTree + Node2D + Sprite2D + Camera2D
/// - Unity: ECS 组件模式 + BlendTree
/// - UE5: GAS 标签系统 + EnhancedInput

pub mod components;
pub mod hot_reload;
pub mod input;
pub mod physics;
pub mod save;
pub mod scene;
pub mod state_machine;
pub mod ui;

// Re-exports
pub use components::{
    AnimationClip, AnimationPlayer, BlendNode, BlendState, Camera2D, Collider, ColliderShape,
    Color, Health, Keyframe, Light2D, Movement, Name, Rect, ScreenShake, SpriteComponent,
    Timer, Transform2D, Vec2, Velocity,
};
pub use hot_reload::{ActiveCoroutine, CoroutineManager, CoroutineStep, HotReloader, TimerEntry, TimerManager};
pub use input::{
    gamepad, keys, InputAction, InputBinding, InputManager, InputMappingContext, InputState,
};
pub use physics::{CollisionSystem, EntityData, SpatialHash};
pub use save::{SaveError, SaveGame, SaveManager, SaveSlotInfo, CURRENT_SAVE_VERSION};
pub use scene::{Prefab, SceneManager, SceneInstance};
pub use state_machine::{
    GameState, MainMenuState, PausedState, PlayingState, StateContext, StateMachine,
};
pub use ui::{Screen, ScreenStack, UiState, Widget, WidgetContent, WidgetStyle};

/// 渲染配置
pub struct RenderConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
    pub target_fps: u32,
    pub clear_color: Color,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            window_title: "NeoTrix Game".to_string(),
            target_fps: 60,
            clear_color: Color::rgb(20, 20, 30),
        }
    }
}

/// 渲染世界 — 聚合所有渲染相关状态
pub struct RenderWorld {
    pub config: RenderConfig,
    pub scene_manager: SceneManager,
    pub state_machine: StateMachine,
    pub camera: Camera2D,
    pub camera_position: Vec2,
}

impl RenderWorld {
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            scene_manager: SceneManager::new(),
            state_machine: StateMachine::new(),
            camera: Camera2D::new(),
            camera_position: Vec2::ZERO,
        }
    }

    /// 初始化：注册预制体和状态
    pub fn initialize(&mut self) {
        // 注册预制体
        self.scene_manager
            .register_prefab(scene::create_player_prefab());
        self.scene_manager
            .register_prefab(scene::create_platform_prefab());

        // 注册游戏状态
        self.state_machine
            .add_state(Box::new(MainMenuState));
        self.state_machine
            .add_state(Box::new(PlayingState));
        self.state_machine
            .add_state(Box::new(PausedState));
    }

    /// 更新渲染世界
    pub fn update(&mut self, dt: f32) {
        let mut ctx = StateContext::new(dt);
        self.state_machine.update(&mut ctx);
    }
}

impl Default for RenderWorld {
    fn default() -> Self {
        Self::new(RenderConfig::default())
    }
}
