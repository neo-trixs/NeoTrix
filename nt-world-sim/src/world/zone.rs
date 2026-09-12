#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Farm,
    Town,
    Mine,
    Forest,
    Lake,
    Beach,
    Cave,
    Special,
}

#[derive(Debug, Clone)]
pub struct ZoneConnection {
    pub target_zone: String,
    pub entry_point: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub zone_type: ZoneType,
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub name: String,
    pub connections: Vec<ZoneConnection>,
    pub music_track: Option<String>,
    pub ambient_sound: Option<String>,
    pub npc_spots: Vec<(u32, u32)>,
    pub resource_nodes: Vec<(u32, u32, String)>,
}

impl Zone {
    pub fn new(zone_type: ZoneType, position: (u32, u32), size: (u32, u32), name: &str) -> Self {
        Self {
            zone_type,
            position,
            size,
            name: name.to_string(),
            connections: Vec::new(),
            music_track: None,
            ambient_sound: None,
            npc_spots: Vec::new(),
            resource_nodes: Vec::new(),
        }
    }

    pub fn with_music(mut self, track: &str) -> Self {
        self.music_track = Some(track.to_string());
        self
    }

    pub fn with_ambient(mut self, sound: &str) -> Self {
        self.ambient_sound = Some(sound.to_string());
        self
    }

    pub fn with_npc(mut self, x: u32, y: u32) -> Self {
        self.npc_spots.push((x, y));
        self
    }

    pub fn with_resource(mut self, x: u32, y: u32, resource: &str) -> Self {
        self.resource_nodes.push((x, y, resource.to_string()));
        self
    }

    pub fn thought_meadow() -> Self {
        Self::new(ZoneType::Farm, (10, 10), (20, 15), "Thought Meadow")
            .with_music("meadow_theme")
            .with_ambient("birds_chirping")
            .with_npc(15, 12)
            .with_npc(18, 14)
            .with_resource(12, 11, "seeds")
            .with_resource(20, 13, "water_well")
    }

    pub fn neural_hub() -> Self {
        Self::new(ZoneType::Town, (50, 30), (20, 20), "Neural Hub")
            .with_music("town_theme")
            .with_ambient("crowd_murmur")
            .with_npc(55, 35)
            .with_npc(60, 35)
            .with_npc(65, 35)
            .with_npc(58, 40)
            .with_resource(52, 32, "shop")
            .with_resource(68, 45, "blacksmith")
    }

    pub fn knowledge_mines() -> Self {
        Self::new(ZoneType::Mine, (70, 10), (15, 15), "Knowledge Mines")
            .with_music("mine_theme")
            .with_ambient("pickaxe_echo")
            .with_npc(75, 15)
            .with_resource(72, 12, "crystal_ore")
            .with_resource(78, 18, "ancient_tablet")
            .with_resource(82, 14, "deep_mineral")
    }

    pub fn memory_forest() -> Self {
        Self::new(ZoneType::Forest, (10, 50), (25, 25), "Memory Forest")
            .with_music("forest_theme")
            .with_ambient("rustling_leaves")
            .with_npc(20, 55)
            .with_npc(25, 60)
            .with_resource(15, 52, "ancient_wood")
            .with_resource(22, 58, "herbs")
            .with_resource(30, 65, "rare_mushroom")
    }

    pub fn dream_lake() -> Self {
        Self::new(ZoneType::Lake, (60, 60), (20, 15), "Dream Lake")
            .with_music("lake_theme")
            .with_ambient("water_lapping")
            .with_npc(65, 65)
            .with_resource(62, 62, "fish")
            .with_resource(70, 68, "pearl")
            .with_resource(75, 63, "water_crystal")
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.position.0
            && x < self.position.0 + self.size.0
            && y >= self.position.1
            && y < self.position.1 + self.size.1
    }

    pub fn center(&self) -> (u32, u32) {
        (
            self.position.0 + self.size.0 / 2,
            self.position.1 + self.size.1 / 2,
        )
    }

    pub fn connect_to(&mut self, target: &str, entry: (u32, u32)) {
        self.connections.push(ZoneConnection {
            target_zone: target.to_string(),
            entry_point: entry,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_creation() {
        let zone = Zone::new(ZoneType::Farm, (10, 10), (20, 15), "Test Farm");
        assert_eq!(zone.name, "Test Farm");
        assert_eq!(zone.zone_type, ZoneType::Farm);
        assert_eq!(zone.position, (10, 10));
        assert_eq!(zone.size, (20, 15));
    }

    #[test]
    fn test_zone_contains() {
        let zone = Zone::new(ZoneType::Town, (10, 10), (20, 15), "Town");
        assert!(zone.contains(10, 10));
        assert!(zone.contains(29, 24));
        assert!(!zone.contains(30, 24));
        assert!(!zone.contains(9, 10));
    }

    #[test]
    fn test_zone_center() {
        let zone = Zone::new(ZoneType::Forest, (10, 10), (20, 20), "Forest");
        assert_eq!(zone.center(), (20, 20));
    }

    #[test]
    fn test_zone_connections() {
        let mut zone = Zone::new(ZoneType::Mine, (0, 0), (10, 10), "Mine");
        zone.connect_to("Town", (5, 5));
        zone.connect_to("Forest", (0, 5));
        assert_eq!(zone.connections.len(), 2);
        assert_eq!(zone.connections[0].target_zone, "Town");
        assert_eq!(zone.connections[1].entry_point, (0, 5));
    }
}
