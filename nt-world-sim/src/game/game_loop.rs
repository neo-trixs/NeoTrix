use std::collections::HashMap;
use crate::core::{UniversalWorld, UniversalEntity, EntityId};
use crate::core::scheduler::{ParallelScheduler, SystemDependency};
use crate::engine::physics::{SimplePhysicsWorld, PhysicsWorld};
use crate::engine::camera::Camera2D;
use crate::engine::scene::{SceneGraph, SceneNode};
use crate::engine::input::KeyCode as InputKeyCode;
use crate::engine::event_bus::TypedEventBus;
use crate::engine::audio::{AudioManager, StubAudioBackend};
use crate::engine::renderer::{Vec2 as RendererVec2, ParticleSystem};
use super::time::{GameTime, TimeSystem};
use super::weather::{Weather, WeatherSystem};
use super::inventory::Inventory;
use super::farming::FarmPlot;
use super::npc::{Npc, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    Playing,
    Paused,
    Dialogue,
    Inventory,
    Crafting,
    Mining,
    GameOver,
}

pub struct GameLoop {
    pub state: GameState,
    pub world: UniversalWorld,
    pub scheduler: ParallelScheduler,
    pub player_entity: Option<UniversalEntity>,
    pub tick_count: u64,
    pub target_fps: u32,
    pub accumulator: f32,
    pub fixed_timestep: f32,
    pub physics: SimplePhysicsWorld,
    pub camera: Camera2D,
    pub scene: SceneGraph,
    pub input_map: HashMap<InputKeyCode, GameAction>,
    pub event_bus: TypedEventBus,
    pub audio: AudioManager,
    pub particles: ParticleSystem,
}

fn create_default_input_map() -> HashMap<InputKeyCode, GameAction> {
    let mut map = HashMap::new();
    map.insert(InputKeyCode::W, GameAction::Move { dx: 0, dy: -1 });
    map.insert(InputKeyCode::S, GameAction::Move { dx: 0, dy: 1 });
    map.insert(InputKeyCode::A, GameAction::Move { dx: -1, dy: 0 });
    map.insert(InputKeyCode::D, GameAction::Move { dx: 1, dy: 0 });
    map.insert(InputKeyCode::Num1, GameAction::UseTool(1));
    map.insert(InputKeyCode::Num2, GameAction::UseTool(2));
    map.insert(InputKeyCode::Num3, GameAction::UseTool(3));
    map.insert(InputKeyCode::Num4, GameAction::UseTool(4));
    map.insert(InputKeyCode::Num5, GameAction::UseTool(5));
    map.insert(InputKeyCode::E, GameAction::Interact);
    map.insert(InputKeyCode::I, GameAction::ToggleInventory);
    map.insert(InputKeyCode::C, GameAction::ToggleCrafting);
    map.insert(InputKeyCode::Enter, GameAction::AdvanceDialogue);
    map.insert(InputKeyCode::F5, GameAction::Save);
    map.insert(InputKeyCode::F9, GameAction::Load);
    map
}

impl GameLoop {
    pub fn new() -> Self {
        let mut world = UniversalWorld::new();
        let mut scheduler = ParallelScheduler::new();
        let mut audio = AudioManager::new();
        audio.add_backend(Box::new(StubAudioBackend));

        scheduler.add_system(Box::new(TimeSystem), SystemDependency::new());
        scheduler.add_system(Box::new(WeatherSystem), SystemDependency::new());

        world.insert_resource(GameTime::new());
        world.insert_resource(Weather::new());
        world.insert_resource(Inventory::new(20, 12));

        let root_entity = UniversalEntity::new(EntityId(0), 0);

        Self {
            state: GameState::Title,
            world,
            scheduler,
            player_entity: None,
            tick_count: 0,
            target_fps: 60,
            accumulator: 0.0,
            fixed_timestep: 1.0 / 60.0,
            physics: SimplePhysicsWorld::new(),
            camera: Camera2D::new(800.0, 600.0),
            scene: SceneGraph::new(root_entity),
            input_map: create_default_input_map(),
            event_bus: TypedEventBus::new(),
            audio,
            particles: ParticleSystem::new(1024),
        }
    }

    pub fn start_game(&mut self) {
        self.state = GameState::Playing;

        let player = self.world.spawn();
        self.world.insert_component(player, Position::new(50, 30));
        self.player_entity = Some(player);

        let player_node = SceneNode::new(player)
            .with_parent(self.scene.root())
            .with_tag("player");
        self.scene.add_node(player_node);

        self.generate_initial_farm();
        self.spawn_npcs();

        self.tick_count = 0;
    }

    pub fn update(&mut self, dt: f32) {
        match self.state {
            GameState::Playing => {
                self.accumulator += dt;
                while self.accumulator >= self.fixed_timestep {
                    self.fixed_update(self.fixed_timestep);
                    self.accumulator -= self.fixed_timestep;
                }
                self.render(dt);
            }
            GameState::Dialogue | GameState::Inventory | GameState::Crafting => {
                self.accumulator += dt;
                while self.accumulator >= self.fixed_timestep {
                    self.fixed_update(self.fixed_timestep);
                    self.accumulator -= self.fixed_timestep;
                }
            }
            GameState::Paused | GameState::Title | GameState::GameOver => {}
            _ => {}
        }
    }

    fn fixed_update(&mut self, dt: f32) {
        self.tick_count += 1;

        self.scheduler.run(&mut self.world, dt);

        self.event_bus.process_all();

        self.physics.step(dt);

        if let Some(player) = self.player_entity {
            if let Some(pos) = self.world.get_component::<Position>(player) {
                let player_vec2 = RendererVec2 { x: pos.x as f32 * 16.0, y: pos.y as f32 * 16.0 };
                self.camera.follow(player_vec2);
            }
        }

        self.particles.update(dt);
        self.camera.update_shake(dt);

        self.check_day_transition();
        self.update_npc_schedules();
        self.update_crops();
        self.update_weather();
    }

    fn check_day_transition(&mut self) {
        let is_new_day = self.world.get_resource::<GameTime>()
            .map(|t| t.hour == 6 && t.minute == 0 && t.tick_counter == 0)
            .unwrap_or(false);

        if is_new_day {
            self.on_new_day();
        }
    }

    fn on_new_day(&mut self) {
        if let Some(time) = self.world.get_resource::<GameTime>() {
            let season = time.season;
            let season_mod = match season {
                super::time::Season::Clarity => 1.0,
                super::time::Season::Flow => 1.3,
                super::time::Season::Reflection => 0.8,
                super::time::Season::Stillness => 0.5,
            };
            println!("New day! Season modifier: {}", season_mod);
        }
    }

    fn update_npc_schedules(&mut self) {
        if let Some(time) = self.world.get_resource::<GameTime>() {
            let _hour = time.hour;
        }
    }

    fn update_crops(&mut self) {}

    fn update_weather(&mut self) {
        if let Some(time) = self.world.get_resource::<GameTime>() {
            if time.tick_counter % 100 == 0 {
                if let Some(weather) = self.world.get_resource_mut::<Weather>() {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .subsec_nanos();
                    let idx = nanos % 5;
                    weather.current = match idx {
                        0 => super::weather::WeatherType::Clear,
                        1 => super::weather::WeatherType::Rain,
                        2 => super::weather::WeatherType::Storm,
                        3 => super::weather::WeatherType::Snow,
                        _ => super::weather::WeatherType::Clear,
                    };
                }
            }
        }
    }

    fn generate_initial_farm(&mut self) {
        for y in 0..8u32 {
            for x in 0..10u32 {
                let farm_plot = FarmPlot::new(x, y);
                let entity = self.world.spawn();
                self.world.insert_component(entity, farm_plot);
            }
        }
    }

    fn spawn_npcs(&mut self) {
        use super::npc::NpcDefinition;

        for npc_def in NpcDefinition::all_npcs() {
            let entity = self.world.spawn();
            self.world.insert_component(entity, Position::new(npc_def.position.0, npc_def.position.1));

            let npc = Npc::new(
                &npc_def.name,
                super::npc::NpcRole::Companion,
            );
            self.world.insert_component(entity, npc);
        }
    }

    pub fn handle_input(&mut self, action: GameAction) {
        match action {
            GameAction::Move { dx, dy } => {
                if self.state == GameState::Playing {
                    self.move_player(dx, dy);
                }
            }
            GameAction::UseTool(tool_index) => {
                if self.state == GameState::Playing {
                    self.use_tool(tool_index);
                }
            }
            GameAction::Interact => {
                if self.state == GameState::Playing {
                    self.interact();
                }
            }
            GameAction::ToggleInventory => {
                self.state = match self.state {
                    GameState::Playing => GameState::Inventory,
                    GameState::Inventory => GameState::Playing,
                    _ => self.state,
                };
            }
            GameAction::ToggleCrafting => {
                self.state = match self.state {
                    GameState::Playing => GameState::Crafting,
                    GameState::Crafting => GameState::Playing,
                    _ => self.state,
                };
            }
            GameAction::AdvanceDialogue => {
                if self.state == GameState::Dialogue {
                    self.state = GameState::Playing;
                }
            }
            GameAction::Sleep => {
                if self.state == GameState::Playing {
                    self.sleep();
                }
            }
            GameAction::Save => {
                let _ = self.save_game();
            }
            GameAction::Load => {
                let _ = self.load_game();
            }
        }
    }

    fn move_player(&mut self, dx: i32, dy: i32) {
        if let Some(player) = self.player_entity {
            if let Some(pos) = self.world.get_component_mut::<Position>(player) {
                let new_x = (pos.x as i32 + dx).max(0) as u32;
                let new_y = (pos.y as i32 + dy).max(0) as u32;
                pos.x = new_x;
                pos.y = new_y;
            }
        }
    }

    fn use_tool(&mut self, tool_index: u32) {
        let tool_type = match tool_index {
            1 => super::inventory::ToolType::Hoe,
            2 => super::inventory::ToolType::WateringCan,
            3 => super::inventory::ToolType::Pickaxe,
            4 => super::inventory::ToolType::Axe,
            5 => super::inventory::ToolType::FishingRod,
            _ => return,
        };

        if let Some(player) = self.player_entity {
            if let Some(pos) = self.world.get_component::<Position>(player) {
                let target_x = pos.x;
                let target_y = pos.y + 1;

                match tool_type {
                    super::inventory::ToolType::Hoe => {
                        println!("Hoeing tile at ({}, {})", target_x, target_y);
                        self.camera.shake(2.0);
                    }
                    super::inventory::ToolType::WateringCan => {
                        println!("Watering tile at ({}, {})", target_x, target_y);
                        self.camera.shake(1.0);
                    }
                    super::inventory::ToolType::Pickaxe => {
                        self.camera.shake(4.0);
                    }
                    super::inventory::ToolType::Axe => {
                        self.camera.shake(3.0);
                    }
                    _ => {}
                }
            }
        }
    }

    fn interact(&mut self) {
        println!("Interacting with nearby object...");
    }

    fn sleep(&mut self) {
        if let Some(time) = self.world.get_resource_mut::<GameTime>() {
            time.hour = 6;
            time.minute = 0;
            time.day += 1;
            if time.day > 28 {
                time.day = 1;
                time.season = time.season.next();
            }
            println!("Slept until next day. Day {}, {}", time.day, time.season_name());
        }
        self.camera.shake(5.0);
    }

    fn render(&self, _dt: f32) {
        if self.tick_count % 600 == 0 {
            if let Some(time) = self.world.get_resource::<GameTime>() {
                println!("Tick {}: Day {} {:02}:{:02} {}",
                    self.tick_count, time.day, time.hour, time.minute, time.season_name());
            }
        }
    }

    pub fn save_game(&self) -> crate::error::GameResult<()> {
        let manager = crate::save::SaveManager::default_dir();
        let time = self.world.get_resource::<super::time::GameTime>().cloned().unwrap_or_default();
        let energy = self.world.get_resource::<super::energy::Energy>().cloned().unwrap_or_default();
        let inventory = self.world.get_resource::<Inventory>().cloned().unwrap_or(Inventory::new(0, 0));
        let save_data = crate::save::SaveData::from_game_state(
            "Player",
            &time,
            &energy,
            &inventory,
            &[],
            &[],
            &crate::world::zone::WorldMap::new(),
            self.tick_count as f64,
        );
        manager.save(0, &save_data)
    }

    pub fn load_game(&mut self) -> crate::error::GameResult<()> {
        let manager = crate::save::SaveManager::default_dir();
        let data = manager.load(0)?;
        self.tick_count = data.tick_count;
        Ok(())
    }

    pub fn process_raw_input(&mut self, pressed_keys: &[InputKeyCode]) {
        for key in pressed_keys {
            if let Some(action) = self.input_map.get(key) {
                self.handle_input(action.clone());
            }
        }
    }

    pub fn update_camera(&mut self) {
        if let Some(player) = self.player_entity {
            if let Some(pos) = self.world.get_component::<Position>(player) {
                let target = RendererVec2 {
                    x: pos.x as f32 * 16.0 + 8.0,
                    y: pos.y as f32 * 16.0 + 8.0,
                };
                self.camera.follow(target);
            }
        }
    }

    pub fn get_state_summary(&self) -> String {
        let time_str = self.world.get_resource::<GameTime>()
            .map(|t| format!("Day {} {:02}:{:02} {}", t.day, t.hour, t.minute, t.season_name()))
            .unwrap_or_else(|| "No time".to_string());

        format!("State: {:?} | {} | Tick: {}", self.state, time_str, self.tick_count)
    }
}

#[derive(Debug, Clone)]
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

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_loop_creation() {
        let game = GameLoop::new();
        assert_eq!(game.state, GameState::Title);
        assert_eq!(game.tick_count, 0);
    }

    #[test]
    fn test_game_loop_start() {
        let mut game = GameLoop::new();
        game.start_game();
        assert_eq!(game.state, GameState::Playing);
        assert!(game.player_entity.is_some());
    }

    #[test]
    fn test_game_loop_update() {
        let mut game = GameLoop::new();
        game.start_game();
        game.update(1.0 / 60.0);
        assert!(game.tick_count > 0);
    }

    #[test]
    fn test_game_action_inventory() {
        let mut game = GameLoop::new();
        game.start_game();
        game.handle_input(GameAction::ToggleInventory);
        assert_eq!(game.state, GameState::Inventory);
        game.handle_input(GameAction::ToggleInventory);
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn test_game_sleep() {
        let mut game = GameLoop::new();
        game.start_game();
        game.handle_input(GameAction::Sleep);
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn test_player_movement() {
        let mut game = GameLoop::new();
        game.start_game();
        game.handle_input(GameAction::Move { dx: 1, dy: 0 });
        if let Some(player) = game.player_entity {
            let pos = game.world.get_component::<Position>(player).unwrap();
            assert_eq!(pos.x, 51);
            assert_eq!(pos.y, 30);
        }
    }

    #[test]
    fn test_state_summary() {
        let game = GameLoop::new();
        let summary = game.get_state_summary();
        assert!(summary.contains("Title"));
    }
}
