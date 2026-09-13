use std::time::Instant;

use crate::engine::core::{GameEngine, LoopConfig, GameState, GameLoopCallbacks};
use crate::engine::ecs::{World, SystemRunner, Entity};
use crate::engine::renderer::{DrawCommand, Color, Vec2, Rect, CanvasRenderer, Renderer};
use crate::engine::input::SimpleInputProvider;
use crate::engine::event_bus::TypedEventBus;
use crate::engine::audio::AudioManager;
use crate::engine::save::{EngineSaveManager, SaveBackend, SaveGameState};
use crate::engine::ui::{UIRenderer, MenuSystem, DialogueBox, InventoryGrid};
use crate::engine::perf::{PerfAggregator, SpatialHashGrid};
use crate::engine::particle::ParticlePool;
use crate::engine::effects::EffectsRenderer;
use crate::engine::components::{
    Transform as EcsTransform, Velocity, GameSprite, Health, Collider,
    PlayerMarker, GameCamera, TimeState, RenderCommandBuffer,
};
use crate::engine::systems::{
    MovementSystem, EcsCollisionSystem, CameraSystem, HealthSystem, RenderSystem,
};

// ---------------------------------------------------------------------------
// System Ordering Constants
// ---------------------------------------------------------------------------

pub const PRIORITY_INPUT: i32 = -200;
pub const PRIORITY_AI: i32 = -100;
pub const PRIORITY_PHYSICS: i32 = -50;
pub const PRIORITY_GAME_LOGIC: i32 = 0;
pub const PRIORITY_COLLISION: i32 = 10;
pub const PRIORITY_CAMERA: i32 = 20;
pub const PRIORITY_EFFECTS: i32 = 50;
pub const PRIORITY_RENDER: i32 = 100;
pub const PRIORITY_UI: i32 = 200;

// ---------------------------------------------------------------------------
// Game Configuration
// ---------------------------------------------------------------------------

#[derive(Clone)]
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

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "NeoTrix Game".into(),
            width: 800.0,
            height: 600.0,
            tick_rate: 60.0,
            max_frame_skip: 5,
            auto_save_interval_secs: 300.0,
            save_dir: "saves".into(),
            particle_pool_size: 2048,
            spatial_cell_size: 64.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Game State Flags
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Init,
    Loading,
    MainMenu,
    Playing,
    Paused,
    Dialogue,
    Combat,
    Cutscene,
    GameOver,
}

// ---------------------------------------------------------------------------
// Game — ties all systems together
// ---------------------------------------------------------------------------

pub struct Game {
    // Core engine
    pub engine: GameEngine,
    pub config: GameConfig,

    // ECS
    pub world: World,
    pub system_runner: SystemRunner,

    // Rendering
    pub renderer: CanvasRenderer,
    pub ui_renderer: UIRenderer,
    pub draw_commands: Vec<DrawCommand>,

    // Input
    pub input: SimpleInputProvider,

    // Audio
    pub audio: AudioManager,

    // Save/Load
    pub save_manager: Option<EngineSaveManager>,
    pub current_save: Option<SaveGameState>,
    pub current_save_slot: u32,

    // UI
    pub menu: MenuSystem,
    pub dialogue: DialogueBox,
    pub inventory_grid: InventoryGrid,

    // Performance
    pub perf: PerfAggregator,
    pub particle_pool: ParticlePool,
    pub effects: EffectsRenderer,

    // Event bus
    pub event_bus: TypedEventBus,

    // State
    pub phase: GamePhase,
    pub tick_count: u64,
    pub play_time: f64,
    pub needs_save: bool,

    // Timing
    #[allow(dead_code)]
    last_frame: Instant,
    pub fps: f64,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let loop_config = LoopConfig {
            tick_rate: config.tick_rate,
            max_frame_skip: config.max_frame_skip,
            headless: false,
        };

        let engine = GameEngine::new(loop_config);
        let mut world = World::new();

        // Insert default resources
        world.insert_resource(GameCamera::new(config.width, config.height));
        world.insert_resource(RenderCommandBuffer::new());
        world.insert_resource(TimeState::default());

        let mut system_runner = SystemRunner::new();
        system_runner.add_system(Box::new(MovementSystem));
        system_runner.add_system(Box::new(EcsCollisionSystem::new()));
        system_runner.add_system(Box::new(CameraSystem));
        system_runner.add_system(Box::new(HealthSystem));
        system_runner.add_system(Box::new(RenderSystem));

        let mut perf = PerfAggregator::new();
        perf.spatial_grid = SpatialHashGrid::new(config.spatial_cell_size);

        Self {
            engine,
            config: config.clone(),
            world,
            system_runner,
            renderer: CanvasRenderer::new(config.width, config.height),
            ui_renderer: UIRenderer::new(),
            draw_commands: Vec::new(),
            input: SimpleInputProvider::new(),
            audio: AudioManager::new(),
            save_manager: None,
            current_save: None,
            current_save_slot: 0,
            menu: MenuSystem::new(),
            dialogue: DialogueBox::new(10.0, config.height - 200.0, config.width - 20.0, 180.0),
            inventory_grid: InventoryGrid::new(config.width - 320.0, 50.0, 8, 4, 36.0),
            perf,
            particle_pool: ParticlePool::new(config.particle_pool_size),
            effects: EffectsRenderer::new(),
            event_bus: TypedEventBus::new(),
            phase: GamePhase::Init,
            tick_count: 0,
            play_time: 0.0,
            needs_save: false,
            last_frame: Instant::now(),
            fps: 0.0,
        }
    }

    /// Initialize the game with a save backend
    pub fn init_with_save(&mut self, backend: Box<dyn SaveBackend>) {
        self.save_manager = Some(EngineSaveManager::with_auto_save(
            backend,
            self.config.auto_save_interval_secs,
        ));
        self.phase = GamePhase::MainMenu;
    }

    /// Start a new game
    pub fn new_game(&mut self, slot: u32, player_name: &str) {
        let state = EngineSaveManager::new_game(slot, player_name);
        self.current_save = Some(state);
        self.current_save_slot = slot;
        self.phase = GamePhase::Playing;
        self.tick_count = 0;
        self.play_time = 0.0;

        // Spawn player entity
        let player = self.world.spawn();
        self.world.insert(player, EcsTransform::from_position(0.0, 0.0));
        self.world.insert(player, Velocity::zero());
        self.world.insert(player, GameSprite::new("player.png").with_z_index(10));
        self.world.insert(player, Health::new(100.0));
        self.world.insert(player, Collider::aabb(16.0, 16.0));
        self.world.insert(player, PlayerMarker);

        // Set camera target
        if let Some(cam) = self.world.get_resource_mut::<GameCamera>() {
            cam.target = Some(player);
        }
    }

    /// Load a saved game
    pub fn load_game(&mut self, slot: u32) -> Result<(), String> {
        let mgr = self.save_manager.as_ref().ok_or("No save manager")?;
        let state = mgr.load(slot)?;
        self.current_save = Some(state);
        self.current_save_slot = slot;
        self.phase = GamePhase::Playing;
        Ok(())
    }

    /// Save the current game
    pub fn save_game(&mut self) -> Result<(), String> {
        if let (Some(ref mut mgr), Some(ref state)) = (&mut self.save_manager, &self.current_save) {
            mgr.save(self.current_save_slot, state)?;
            self.needs_save = false;
            Ok(())
        } else {
            Err("No save manager or active save".into())
        }
    }

    /// Main tick — runs one fixed timestep
    pub fn tick(&mut self, dt: f64) {
        let dt_f32 = dt as f32;
        self.tick_count += 1;

        // Update time resource
        if let Some(time) = self.world.get_resource_mut::<TimeState>() {
            time.dt = dt_f32;
            time.elapsed += dt;
            time.frame = self.tick_count;
        }

        // Update input
        self.input.update();

        // Run ECS systems based on phase
        match self.phase {
            GamePhase::Playing => {
                self.system_runner.run_all(&mut self.world, dt_f32);

                // Update effects
                self.effects.update(dt_f32, &mut self.particle_pool);

                // Auto-save check
                if let Some(ref mut mgr) = self.save_manager {
                    if let Some(ref state) = self.current_save {
                        if mgr.auto_save_tick(dt, Some(state)) {
                            self.needs_save = false;
                        }
                    }
                }
            }
            GamePhase::Dialogue => {
                self.dialogue.update(dt_f32);
            }
            GamePhase::Paused => {
                // No updates
            }
            _ => {}
        }

        // Performance tracking
        self.perf.tick(dt);

        // Update play time
        if self.phase == GamePhase::Playing {
            self.play_time += dt;
        }
    }

    /// Main render — produces draw commands
    pub fn render(&mut self, _alpha: f64) {
        self.renderer.clear(Color::rgb(0.1, 0.1, 0.15));

        match self.phase {
            GamePhase::Playing | GamePhase::Paused | GamePhase::Dialogue => {
                // Collect ECS render commands
                if let Some(cmds) = self.world.get_resource::<RenderCommandBuffer>() {
                    self.draw_commands.extend(cmds.commands.iter().cloned());
                }

                // Particle effects
                let pv = self.particle_pool.render(&self.get_camera());
                if !pv.is_empty() {
                    self.draw_commands.push(DrawCommand::DrawParticles { particles: pv });
                }

                // Floating numbers
                self.draw_commands.extend(self.effects.render(&self.get_camera()));

                // HUD
                self.render_hud();

                // UI overlays
                match self.phase {
                    GamePhase::Paused => self.render_pause_menu(),
                    GamePhase::Dialogue => {
                        self.draw_commands.extend(self.ui_renderer.render_dialogue(&self.dialogue));
                    }
                    _ => {}
                }

                if self.menu.visible {
                    self.render_menu_screen();
                }

                // Perf overlay
                let overlay = self.perf.format_overlay();
                self.draw_commands.push(DrawCommand::DrawText {
                    text: overlay,
                    position: Vec2::new(8.0, 8.0),
                    color: Color::rgba(0.8, 0.8, 0.8, 0.7),
                    size: 10.0,
                });
            }
            GamePhase::MainMenu => {
                self.render_main_menu();
            }
            _ => {}
        }

        self.renderer.present();
    }

    fn get_camera(&self) -> crate::engine::renderer::Camera {
        let (vw, vh) = (self.config.width, self.config.height);
        match self.world.get_resource::<GameCamera>() {
            Some(cam) => crate::engine::renderer::Camera {
                position: cam.position,
                zoom: cam.zoom,
                viewport_width: vw,
                viewport_height: vh,
            },
            None => crate::engine::renderer::Camera::new(vw, vh),
        }
    }

    fn render_hud(&mut self) {
        // HP bar
        let hp_bar = crate::engine::ui::Bar::new(
            10.0, self.config.height - 40.0, 200.0, 20.0,
            crate::engine::ui::BarKind::Hp, 100.0, 100.0,
        );
        self.draw_commands.extend(self.ui_renderer.render_bar(&hp_bar));

        // Minimap placeholder
        let minimap = crate::engine::ui::UiMinimap::new(
            self.config.width - 120.0, 10.0, 10, 10, 10.0,
        );
        let tile_colors: Vec<Color> = (0..100).map(|_| Color::rgba(0.2, 0.3, 0.2, 1.0)).collect();
        self.draw_commands.extend(
            self.ui_renderer.render_minimap(&minimap, &tile_colors, 10, None, 16.0),
        );
    }

    fn render_main_menu(&mut self) {
        let cx = self.config.width / 2.0;
        let cy = self.config.height / 2.0;

        // Title
        self.draw_commands.push(DrawCommand::DrawText {
            text: self.config.title.clone(),
            position: Vec2::new(cx - 100.0, cy - 120.0),
            color: Color::rgba(1.0, 0.9, 0.4, 1.0),
            size: 32.0,
        });

        // Menu buttons
        let buttons = ["New Game", "Load Game", "Settings", "Quit"];
        for (i, label) in buttons.iter().enumerate() {
            let by = cy - 40.0 + i as f32 * 50.0;
            let btn = crate::engine::ui::Button::new(cx - 100.0, by, 200.0, 40.0, label);
            self.draw_commands.extend(self.ui_renderer.render_button(&btn));
        }
    }

    fn render_pause_menu(&mut self) {
        // Overlay
        self.draw_commands.push(DrawCommand::DrawRect {
            rect: Rect::new(0.0, 0.0, self.config.width, self.config.height),
            color: Color::rgba(0.0, 0.0, 0.0, 0.6),
        });

        let cx = self.config.width / 2.0;
        let cy = self.config.height / 2.0;

        self.draw_commands.push(DrawCommand::DrawText {
            text: "PAUSED".into(),
            position: Vec2::new(cx - 40.0, cy - 100.0),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            size: 28.0,
        });

        let buttons = ["Resume", "Settings", "Save Game", "Main Menu"];
        for (i, label) in buttons.iter().enumerate() {
            let by = cy - 40.0 + i as f32 * 50.0;
            let btn = crate::engine::ui::Button::new(cx - 100.0, by, 200.0, 40.0, label);
            self.draw_commands.extend(self.ui_renderer.render_button(&btn));
        }
    }

    fn render_menu_screen(&mut self) {
        match self.menu.current_screen {
            crate::engine::ui::MenuScreen::Settings => {
                let screen = crate::engine::ui::SettingsScreen {
                    volume_master: self.audio.volumes().0,
                    volume_music: self.audio.volumes().1,
                    volume_sfx: self.audio.volumes().2,
                    ..crate::engine::ui::SettingsScreen::new()
                };
                self.draw_commands.extend(self.ui_renderer.render_settings_screen(
                    &screen, 100.0, 50.0, 600.0, 400.0,
                ));
            }
            crate::engine::ui::MenuScreen::Inventory => {
                self.draw_commands.extend(self.ui_renderer.render_inventory(&self.inventory_grid));
            }
            _ => {}
        }
    }

    /// Handle input events (call from platform layer)
    pub fn on_key_down(&mut self, key: crate::engine::input::KeyCode) {
        self.input.key_down(key);
    }

    pub fn on_key_up(&mut self, key: crate::engine::input::KeyCode) {
        self.input.key_up(key);
    }

    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        self.input.set_mouse_position(Vec2::new(x, y));
    }

    pub fn on_mouse_button(&mut self, button: crate::engine::input::MouseButton, pressed: bool) {
        if pressed {
            self.input.mouse_button_down(button);
        } else {
            self.input.mouse_button_up(button);
        }
    }

    /// Handle pause toggle
    pub fn toggle_pause(&mut self) {
        match self.phase {
            GamePhase::Playing => {
                self.phase = GamePhase::Paused;
                self.menu.open_pause();
            }
            GamePhase::Paused => {
                self.phase = GamePhase::Playing;
                self.menu.close();
            }
            _ => {}
        }
    }

    /// Start dialogue
    pub fn start_dialogue(&mut self, speaker: &str, text: &str) {
        self.dialogue.show(speaker, text);
        self.phase = GamePhase::Dialogue;
    }

    /// End dialogue
    pub fn end_dialogue(&mut self) {
        self.dialogue.visible = false;
        self.phase = GamePhase::Playing;
    }

    /// Process queued draw commands (for platform backends)
    pub fn drain_draw_commands(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.draw_commands)
    }

    /// Spawn an entity with standard components
    pub fn spawn_entity(&mut self, x: f32, y: f32, texture: &str) -> Entity {
        let e = self.world.spawn();
        self.world.insert(e, EcsTransform::from_position(x, y));
        self.world.insert(e, Velocity::zero());
        self.world.insert(e, GameSprite::new(texture));
        e
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.world.entity_count()
    }
}

// ---------------------------------------------------------------------------
// GameCallbacks — implements GameLoopCallbacks for Game
// ---------------------------------------------------------------------------

pub struct GameCallbacks<'a> {
    pub game: &'a mut Game,
}

impl<'a> GameLoopCallbacks for GameCallbacks<'a> {
    fn on_init(&mut self) {
        self.game.phase = GamePhase::MainMenu;
    }

    fn on_tick(&mut self, dt: f64) {
        self.game.tick(dt);
    }

    fn on_render(&mut self, alpha: f64) {
        self.game.render(alpha);
    }

    fn on_state_change(&mut self, from: GameState, to: GameState) {
        let _ = (from, to);
    }

    fn on_shutdown(&mut self) {
        // Auto-save on shutdown
        let _ = self.game.save_game();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game::new(GameConfig::default());
        assert_eq!(game.phase, GamePhase::Init);
        assert_eq!(game.tick_count, 0);
    }

    #[test]
    fn test_game_new_game() {
        let mut game = Game::new(GameConfig::default());
        game.new_game(0, "Hero");
        assert_eq!(game.phase, GamePhase::Playing);
        assert!(game.current_save.is_some());
        assert_eq!(game.entity_count(), 1);
    }

    #[test]
    fn test_game_toggle_pause() {
        let mut game = Game::new(GameConfig::default());
        game.new_game(0, "Hero");
        game.toggle_pause();
        assert_eq!(game.phase, GamePhase::Paused);
        game.toggle_pause();
        assert_eq!(game.phase, GamePhase::Playing);
    }

    #[test]
    fn test_game_dialogue() {
        let mut game = Game::new(GameConfig::default());
        game.new_game(0, "Hero");
        game.start_dialogue("NPC", "Hello!");
        assert_eq!(game.phase, GamePhase::Dialogue);
        game.end_dialogue();
        assert_eq!(game.phase, GamePhase::Playing);
    }

    #[test]
    fn test_game_tick() {
        let mut game = Game::new(GameConfig::default());
        game.new_game(0, "Hero");
        game.tick(1.0 / 60.0);
        assert_eq!(game.tick_count, 1);
    }

    #[test]
    fn test_game_spawn_entity() {
        let mut game = Game::new(GameConfig::default());
        game.new_game(0, "Hero");
        let e = game.spawn_entity(100.0, 200.0, "npc.png");
        assert!(game.world.is_alive(e));
        assert_eq!(game.entity_count(), 2);
    }

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.width, 800.0);
        assert_eq!(config.height, 600.0);
        assert_eq!(config.tick_rate, 60.0);
    }

    #[test]
    fn test_system_priorities() {
        assert!(PRIORITY_INPUT < PRIORITY_AI);
        assert!(PRIORITY_AI < PRIORITY_PHYSICS);
        assert!(PRIORITY_PHYSICS < PRIORITY_GAME_LOGIC);
        assert!(PRIORITY_GAME_LOGIC < PRIORITY_COLLISION);
        assert!(PRIORITY_COLLISION < PRIORITY_CAMERA);
        assert!(PRIORITY_CAMERA < PRIORITY_RENDER);
        assert!(PRIORITY_RENDER < PRIORITY_UI);
    }
}
