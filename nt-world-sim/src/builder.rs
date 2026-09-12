use crate::core::UniversalWorld;
use crate::core::scheduler::{ParallelScheduler, UniversalSystem, SystemDependency};
use crate::engine::renderer::{CanvasRenderer, Camera, Color, Renderer};
use crate::engine::physics::{SimplePhysicsWorld, PhysicsWorld};
use crate::engine::input::SimpleInputProvider;

/// Pre-configured game template
pub enum GameTemplate {
    StardewValley,
    Civilization,
    TowerDefense,
    TopDownRPG,
    Platformer,
    Custom,
}

/// Game builder with fluent API
pub struct GameBuilder {
    name: String,
    width: u32,
    height: u32,
    background_color: Color,
    template: GameTemplate,
    systems: Vec<(Box<dyn UniversalSystem>, SystemDependency)>,
    title: String,
}

impl GameBuilder {
    /// Create a new game builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            width: 800,
            height: 600,
            background_color: Color { r: 0.1, g: 0.1, b: 0.15, a: 1.0 },
            template: GameTemplate::Custom,
            systems: Vec::new(),
            title: name.to_string(),
        }
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set background color
    pub fn with_background(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background_color = Color { r, g, b, a: 1.0 };
        self
    }

    /// Set window title
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Use a pre-configured template
    pub fn with_template(mut self, template: GameTemplate) -> Self {
        self.template = template;
        self
    }

    /// Add a system
    pub fn with_system(mut self, system: Box<dyn UniversalSystem>, deps: SystemDependency) -> Self {
        self.systems.push((system, deps));
        self
    }

    /// Build the game engine (consumes builder)
    pub fn build(self) -> BuiltGame {
        let world = UniversalWorld::new();
        let mut renderer = CanvasRenderer::new(self.width as f32, self.height as f32);
        renderer.clear(self.background_color);

        let mut scheduler = ParallelScheduler::new();
        for (system, deps) in self.systems {
            scheduler.add_system(system, deps);
        }
        scheduler.build_schedule();

        let camera = Camera::new(self.width as f32, self.height as f32);

        BuiltGame {
            name: self.name.clone(),
            title: self.title,
            width: self.width,
            height: self.height,
            world,
            scheduler,
            renderer,
            physics: SimplePhysicsWorld::new(),
            input: SimpleInputProvider::new(),
            camera,
            background_color: self.background_color,
            frame: 0,
            running: true,
        }
    }
}

/// A fully constructed game ready to run
pub struct BuiltGame {
    pub name: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub world: UniversalWorld,
    pub scheduler: ParallelScheduler,
    pub renderer: CanvasRenderer,
    pub physics: SimplePhysicsWorld,
    pub input: SimpleInputProvider,
    pub camera: Camera,
    pub background_color: Color,
    pub frame: u64,
    pub running: bool,
}

impl BuiltGame {
    /// Run one simulation tick
    pub fn tick(&mut self, dt: f32) {
        self.physics.step(dt);
        self.scheduler.run(&mut self.world, dt);
        self.frame += 1;
    }

    /// Check if game should keep running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the game
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get current frame number
    pub fn frame(&self) -> u64 {
        self.frame
    }
}

/// Template presets
impl GameBuilder {
    pub fn stardew_valley() -> Self {
        Self::new("Stardew Valley Clone")
            .with_size(1200, 800)
            .with_background(0.2, 0.3, 0.15)
            .with_template(GameTemplate::StardewValley)
            .with_title("NeoTrix Valley")
    }

    pub fn civilization() -> Self {
        Self::new("Civilization Clone")
            .with_size(1024, 768)
            .with_background(0.15, 0.2, 0.1)
            .with_template(GameTemplate::Civilization)
            .with_title("NeoTrix Civ")
    }

    pub fn tower_defense() -> Self {
        Self::new("Tower Defense")
            .with_size(960, 640)
            .with_background(0.1, 0.1, 0.2)
            .with_template(GameTemplate::TowerDefense)
            .with_title("NeoTrix TD")
    }

    pub fn top_down_rpg() -> Self {
        Self::new("Top-Down RPG")
            .with_size(800, 600)
            .with_background(0.05, 0.05, 0.1)
            .with_template(GameTemplate::TopDownRPG)
            .with_title("NeoTrix RPG")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_build() {
        let game = GameBuilder::new("Test")
            .with_size(640, 480)
            .with_background(0.0, 0.0, 0.0)
            .build();

        assert_eq!(game.name, "Test");
        assert_eq!(game.width, 640);
        assert_eq!(game.height, 480);
        assert!(game.is_running());
    }

    #[test]
    fn test_template_builders() {
        let game = GameBuilder::stardew_valley().build();
        assert_eq!(game.name, "Stardew Valley Clone");
        assert_eq!(game.width, 1200);

        let game = GameBuilder::civilization().build();
        assert_eq!(game.name, "Civilization Clone");

        let game = GameBuilder::tower_defense().build();
        assert_eq!(game.name, "Tower Defense");
    }

    #[test]
    fn test_tick_increments() {
        let mut game = GameBuilder::new("Test").build();
        assert_eq!(game.frame(), 0);
        game.tick(0.016);
        assert_eq!(game.frame(), 1);
        game.tick(0.016);
        assert_eq!(game.frame(), 2);
    }

    #[test]
    fn test_stop_game() {
        let mut game = GameBuilder::new("Test").build();
        assert!(game.is_running());
        game.stop();
        assert!(!game.is_running());
    }

    #[test]
    fn test_custom_game() {
        let game = GameBuilder::new("Custom")
            .with_size(1920, 1080)
            .with_background(0.5, 0.5, 0.5)
            .with_title("My Game")
            .build();

        assert_eq!(game.width, 1920);
        assert_eq!(game.title, "My Game");
    }
}
