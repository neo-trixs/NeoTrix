use crate::core::{Resource, UniversalWorld};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CoreTheme {
    pub name: String,
    pub states: HashMap<String, String>,
    pub scale: f32,
    pub eye_tracking: bool,
}

impl Resource for CoreTheme {}

impl Default for CoreTheme {
    fn default() -> Self {
        let mut states = HashMap::new();
        states.insert("idle".to_string(), "idle.gif".to_string());
        states.insert("thinking".to_string(), "thinking.gif".to_string());
        states.insert("typing".to_string(), "typing.gif".to_string());
        states.insert("building".to_string(), "building.gif".to_string());
        states.insert("groove".to_string(), "groove.gif".to_string());
        states.insert("juggling".to_string(), "juggling.gif".to_string());
        states.insert("error".to_string(), "error.gif".to_string());
        states.insert("happy".to_string(), "happy.gif".to_string());
        states.insert("notification".to_string(), "notification.gif".to_string());
        states.insert("sweeping".to_string(), "sweeping.gif".to_string());
        states.insert("carrying".to_string(), "carrying.gif".to_string());
        states.insert("sleeping".to_string(), "sleeping.gif".to_string());

        Self {
            name: "Default".to_string(),
            states,
            scale: 1.0,
            eye_tracking: true,
        }
    }
}

impl CoreTheme {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            states: HashMap::new(),
            scale: 1.0,
            eye_tracking: true,
        }
    }

    pub fn with_state(mut self, state: &str, path: &str) -> Self {
        self.states.insert(state.to_string(), path.to_string());
        self
    }

    pub fn get_animation_path(&self, state: &str) -> Option<&str> {
        self.states.get(state).map(|s| s.as_str())
    }
}

pub struct CoreThemeManager;

impl CoreThemeManager {
    pub fn load_default_theme(world: &mut UniversalWorld) {
        let theme = CoreTheme::default();
        world.insert_resource(theme);
    }

    pub fn load_theme(world: &mut UniversalWorld, name: &str, states: Vec<(&str, &str)>) {
        let mut theme = CoreTheme::new(name);
        for (state, path) in states {
            theme = theme.with_state(state, path);
        }
        world.insert_resource(theme);
    }

    pub fn get_animation_path(world: &UniversalWorld, state: &str) -> Option<String> {
        world
            .get_resource::<CoreTheme>()
            .and_then(|theme| theme.get_animation_path(state).map(|s| s.to_string()))
    }

    pub fn replace_theme(world: &mut UniversalWorld, theme: CoreTheme) {
        world.insert_resource(theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::UniversalWorld;

    #[test]
    fn test_default_theme() {
        let theme = CoreTheme::default();
        assert_eq!(theme.name, "Default");
        assert_eq!(theme.states.len(), 12);
        assert!(theme.eye_tracking);
        assert_eq!(theme.scale, 1.0);
    }

    #[test]
    fn test_theme_new_with_states() {
        let theme = CoreTheme::new("Custom")
            .with_state("idle", "custom_idle.gif")
            .with_state("thinking", "custom_think.gif");

        assert_eq!(theme.name, "Custom");
        assert_eq!(theme.states.len(), 2);
        assert_eq!(theme.get_animation_path("idle"), Some("custom_idle.gif"));
        assert_eq!(theme.get_animation_path("missing"), None);
    }

    #[test]
    fn test_load_default_theme() {
        let mut world = UniversalWorld::new();
        CoreThemeManager::load_default_theme(&mut world);

        let theme = world.get_resource::<CoreTheme>().unwrap();
        assert_eq!(theme.name, "Default");
    }

    #[test]
    fn test_get_animation_path() {
        let mut world = UniversalWorld::new();
        CoreThemeManager::load_default_theme(&mut world);

        let path = CoreThemeManager::get_animation_path(&world, "thinking");
        assert_eq!(path, Some("thinking.gif".to_string()));

        let missing = CoreThemeManager::get_animation_path(&world, "nonexistent");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_replace_theme() {
        let mut world = UniversalWorld::new();
        CoreThemeManager::load_default_theme(&mut world);

        let custom = CoreTheme::new("Dark").with_state("idle", "dark_idle.gif");
        CoreThemeManager::replace_theme(&mut world, custom);

        let theme = world.get_resource::<CoreTheme>().unwrap();
        assert_eq!(theme.name, "Dark");
        assert_eq!(theme.states.len(), 1);
    }

    #[test]
    fn test_load_custom_theme() {
        let mut world = UniversalWorld::new();
        CoreThemeManager::load_theme(
            &mut world,
            "Minimal",
            vec![("idle", "min_idle.gif"), ("sleeping", "min_sleep.gif")],
        );

        let theme = world.get_resource::<CoreTheme>().unwrap();
        assert_eq!(theme.name, "Minimal");
        assert_eq!(theme.states.len(), 2);
    }
}
