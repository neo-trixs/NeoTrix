use super::tile::TileMap;
use super::terrain::TerrainGenerator;

#[derive(Debug, Clone, PartialEq)]
pub enum ZoneType {
    Farm,
    Town,
    Mine,
    Forest,
    Lake,
    Hub,
    Subconscious,
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub id: u32,
    pub name: String,
    pub zone_type: ZoneType,
    pub width: u32,
    pub height: u32,
    pub tile_map: TileMap,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub connections: Vec<u32>,
}

impl Zone {
    pub fn new(id: u32, name: &str, zone_type: ZoneType, width: u32, height: u32) -> Self {
        let mut tile_map = TileMap::new(width, height, 16.0);
        let gen = TerrainGenerator::new(id as u64);

        match zone_type {
            ZoneType::Farm => gen.generate_farm(&mut tile_map),
            ZoneType::Mine => gen.generate_mine_floor(&mut tile_map, 1),
            _ => gen.generate_farm(&mut tile_map),
        }

        Self {
            id,
            name: name.to_string(),
            zone_type,
            width,
            height,
            tile_map,
            spawn_x: width as f32 * 8.0,
            spawn_y: height as f32 * 8.0,
            connections: Vec::new(),
        }
    }

    pub fn connect_to(&mut self, other_id: u32) {
        if !self.connections.contains(&other_id) {
            self.connections.push(other_id);
        }
    }
}

pub struct WorldMap {
    pub zones: Vec<Zone>,
    pub current_zone: u32,
}

impl WorldMap {
    pub fn new() -> Self {
        Self {
            zones: Vec::new(),
            current_zone: 0,
        }
    }

    pub fn add_zone(&mut self, zone: Zone) -> u32 {
        let id = zone.id;
        self.zones.push(zone);
        id
    }

    pub fn get_zone(&self, id: u32) -> Option<&Zone> {
        self.zones.iter().find(|z| z.id == id)
    }

    pub fn current(&self) -> &Zone {
        &self.zones[self.current_zone as usize]
    }

    pub fn current_mut(&mut self) -> &mut Zone {
        &mut self.zones[self.current_zone as usize]
    }

    pub fn travel_to(&mut self, zone_id: u32) -> bool {
        if self.get_zone(zone_id).is_some() {
            self.current_zone = zone_id;
            true
        } else {
            false
        }
    }
}

impl Default for WorldMap {
    fn default() -> Self {
        let mut map = Self::new();

        // Create starter zones
        let farm = Zone::new(0, "Thought Meadow", ZoneType::Farm, 40, 40);
        let town = Zone::new(1, "Neural Hub", ZoneType::Hub, 30, 30);
        let mine = Zone::new(2, "Knowledge Mines", ZoneType::Mine, 20, 20);
        let forest = Zone::new(3, "Memory Forest", ZoneType::Forest, 35, 35);
        let lake = Zone::new(4, "Dream Lake", ZoneType::Lake, 25, 25);

        map.add_zone(farm);
        map.add_zone(town);
        map.add_zone(mine);
        map.add_zone(forest);
        map.add_zone(lake);

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_creation() {
        let zone = Zone::new(0, "Farm", ZoneType::Farm, 20, 20);
        assert_eq!(zone.name, "Farm");
        assert_eq!(zone.width, 20);
    }

    #[test]
    fn test_world_map() {
        let mut map = WorldMap::default();
        assert!(map.get_zone(0).is_some());
        assert!(map.get_zone(5).is_none());

        assert!(map.travel_to(1));
        assert_eq!(map.current().name, "Neural Hub");
    }

    #[test]
    fn test_zone_connections() {
        let mut farm = Zone::new(0, "Farm", ZoneType::Farm, 20, 20);
        farm.connect_to(1);
        farm.connect_to(2);
        assert_eq!(farm.connections.len(), 2);
    }
}
