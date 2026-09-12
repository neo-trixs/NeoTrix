use std::collections::HashMap;

use crate::engine::renderer::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    // Ground
    Grass,
    Dirt,
    Sand,
    Stone,
    Water,
    DeepWater,
    // Farmable
    TilledSoil,
    WateredSoil,
    EnrichedSoil,
    // Objects
    Tree,
    Bush,
    Rock,
    Crystal,
    Mushroom,
    // Structures
    Path,
    Bridge,
    Fence,
    Wall,
    Door,
    // Special
    SpawnPoint,
    ExitPoint,
    NPCSpot,
    ShopTile,
    Void,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        matches!(
            self,
            TileType::Grass
                | TileType::Dirt
                | TileType::Sand
                | TileType::Stone
                | TileType::TilledSoil
                | TileType::WateredSoil
                | TileType::EnrichedSoil
                | TileType::Path
                | TileType::Bridge
                | TileType::Door
                | TileType::SpawnPoint
                | TileType::ExitPoint
                | TileType::NPCSpot
                | TileType::ShopTile
        )
    }

    pub fn is_farmable(&self) -> bool {
        matches!(self, TileType::Dirt | TileType::Grass)
    }

    pub fn is_water_source(&self) -> bool {
        matches!(self, TileType::Water | TileType::DeepWater)
    }

    pub fn color(&self) -> Color {
        match self {
            TileType::Grass => Color::rgb(0.3, 0.6, 0.2),
            TileType::Dirt => Color::rgb(0.4, 0.3, 0.2),
            TileType::Sand => Color::rgb(0.9, 0.85, 0.6),
            TileType::Stone => Color::rgb(0.5, 0.5, 0.5),
            TileType::Water => Color::rgb(0.2, 0.4, 0.8),
            TileType::DeepWater => Color::rgb(0.1, 0.2, 0.6),
            TileType::TilledSoil => Color::rgb(0.35, 0.25, 0.15),
            TileType::WateredSoil => Color::rgb(0.25, 0.2, 0.12),
            TileType::EnrichedSoil => Color::rgb(0.45, 0.3, 0.15),
            TileType::Tree => Color::rgb(0.2, 0.5, 0.1),
            TileType::Bush => Color::rgb(0.25, 0.55, 0.15),
            TileType::Rock => Color::rgb(0.45, 0.45, 0.45),
            TileType::Crystal => Color::rgb(0.6, 0.4, 0.9),
            TileType::Mushroom => Color::rgb(0.7, 0.3, 0.2),
            TileType::Path => Color::rgb(0.6, 0.5, 0.3),
            TileType::Bridge => Color::rgb(0.55, 0.4, 0.25),
            TileType::Fence => Color::rgb(0.5, 0.35, 0.2),
            TileType::Wall => Color::rgb(0.4, 0.4, 0.4),
            TileType::Door => Color::rgb(0.45, 0.3, 0.15),
            TileType::SpawnPoint => Color::rgb(0.0, 1.0, 0.5),
            TileType::ExitPoint => Color::rgb(1.0, 0.5, 0.0),
            TileType::NPCSpot => Color::rgb(0.8, 0.8, 0.0),
            TileType::ShopTile => Color::rgb(0.9, 0.6, 0.1),
            TileType::Void => Color::rgb(0.05, 0.05, 0.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileLayer {
    Ground,
    Decoration,
    Object,
    Water,
    Effect,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub tile_type: TileType,
    pub walkable: bool,
    pub interactable: bool,
    pub animated: bool,
    pub frame: u32,
    pub metadata: HashMap<String, String>,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        let walkable = tile_type.is_walkable();
        let interactable = matches!(
            tile_type,
            TileType::TilledSoil
                | TileType::WateredSoil
                | TileType::EnrichedSoil
                | TileType::Tree
                | TileType::Bush
                | TileType::Rock
                | TileType::Crystal
                | TileType::Mushroom
                | TileType::Door
                | TileType::NPCSpot
                | TileType::ShopTile
                | TileType::Fence
        );
        Self {
            tile_type,
            walkable,
            interactable,
            animated: false,
            frame: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct WorldMap {
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub layers: Vec<Vec<Tile>>,
    pub zones: Vec<super::zone::Zone>,
}

impl WorldMap {
    pub fn new(width: u32, height: u32, tile_size: u32, layer_count: usize) -> Self {
        let total = (width * height) as usize;
        let layers = (0..layer_count)
            .map(|_| vec![Tile::new(TileType::Grass); total])
            .collect();
        Self {
            width,
            height,
            tile_size,
            layers,
            zones: Vec::new(),
        }
    }

    pub fn get_tile(&self, layer: usize, x: u32, y: u32) -> Option<&Tile> {
        if layer < self.layers.len() && x < self.width && y < self.height {
            Some(&self.layers[layer][(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, layer: usize, x: u32, y: u32, tile: Tile) {
        if layer < self.layers.len() && x < self.width && y < self.height {
            self.layers[layer][(y * self.width + x) as usize] = tile;
        }
    }

    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.layers
            .iter()
            .all(|layer| {
                let idx = (y * self.width + x) as usize;
                idx < layer.len() && layer[idx].walkable
            })
    }

    pub fn world_to_tile(&self, wx: f32, wy: f32) -> (u32, u32) {
        (
            (wx / self.tile_size as f32) as u32,
            (wy / self.tile_size as f32) as u32,
        )
    }

    pub fn tile_to_world(&self, tx: u32, ty: u32) -> (f32, f32) {
        (
            tx as f32 * self.tile_size as f32,
            ty as f32 * self.tile_size as f32,
        )
    }

    pub fn set_rect(&mut self, layer: usize, x: u32, y: u32, w: u32, h: u32, tile_type: TileType) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_tile(layer, x + dx, y + dy, Tile::new(tile_type));
            }
        }
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn total_tiles(&self) -> usize {
        (self.width * self.height) as usize
    }
}

impl Default for WorldMap {
    fn default() -> Self {
        Self::new(100, 80, 16, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_type_walkability() {
        assert!(TileType::Grass.is_walkable());
        assert!(TileType::Dirt.is_walkable());
        assert!(!TileType::Water.is_walkable());
        assert!(!TileType::Wall.is_walkable());
        assert!(!TileType::DeepWater.is_walkable());
        assert!(TileType::Path.is_walkable());
        assert!(TileType::Door.is_walkable());
    }

    #[test]
    fn test_tile_type_farmable() {
        assert!(TileType::Dirt.is_farmable());
        assert!(TileType::Grass.is_farmable());
        assert!(!TileType::Water.is_farmable());
        assert!(!TileType::Stone.is_farmable());
    }

    #[test]
    fn test_tile_type_water_source() {
        assert!(TileType::Water.is_water_source());
        assert!(TileType::DeepWater.is_water_source());
        assert!(!TileType::Grass.is_water_source());
    }

    #[test]
    fn test_tile_creation() {
        let tile = Tile::new(TileType::Tree);
        assert!(!tile.walkable);
        assert!(tile.interactable);

        let tile = Tile::new(TileType::Grass);
        assert!(tile.walkable);
        assert!(!tile.interactable);

        let tile = Tile::new(TileType::TilledSoil);
        assert!(tile.walkable);
        assert!(tile.interactable);
    }

    #[test]
    fn test_tile_with_metadata() {
        let tile = Tile::new(TileType::Tree)
            .with_metadata("species", "oak")
            .with_metadata("age", "5");
        assert_eq!(tile.metadata.get("species").unwrap(), "oak");
        assert_eq!(tile.metadata.get("age").unwrap(), "5");
    }

    #[test]
    fn test_world_map_create() {
        let map = WorldMap::new(10, 10, 16, 3);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.layer_count(), 3);
        assert_eq!(map.total_tiles(), 100);
    }

    #[test]
    fn test_world_map_get_set() {
        let mut map = WorldMap::new(5, 5, 16, 3);
        map.set_tile(0, 2, 3, Tile::new(TileType::Water));
        assert_eq!(map.get_tile(0, 2, 3).unwrap().tile_type, TileType::Water);
        assert_eq!(map.get_tile(0, 0, 0).unwrap().tile_type, TileType::Grass);
    }

    #[test]
    fn test_world_map_walkability() {
        let mut map = WorldMap::new(5, 5, 16, 3);
        assert!(map.is_walkable(0, 0));
        map.set_tile(0, 1, 1, Tile::new(TileType::Wall));
        assert!(!map.is_walkable(1, 1));
        map.set_tile(0, 2, 2, Tile::new(TileType::Water));
        assert!(!map.is_walkable(2, 2));
    }

    #[test]
    fn test_world_map_set_rect() {
        let mut map = WorldMap::new(10, 10, 16, 3);
        map.set_rect(0, 2, 2, 3, 3, TileType::Dirt);
        assert_eq!(map.get_tile(0, 2, 2).unwrap().tile_type, TileType::Dirt);
        assert_eq!(map.get_tile(0, 4, 4).unwrap().tile_type, TileType::Dirt);
        assert_eq!(map.get_tile(0, 5, 5).unwrap().tile_type, TileType::Grass);
    }

    #[test]
    fn test_world_map_coordinate_conversion() {
        let map = WorldMap::new(10, 10, 16, 3);
        let (tx, ty) = map.world_to_tile(32.0, 48.0);
        assert_eq!((tx, ty), (2, 3));
        let (wx, wy) = map.tile_to_world(2, 3);
        assert!((wx - 32.0).abs() < 0.01);
        assert!((wy - 48.0).abs() < 0.01);
    }

    #[test]
    fn test_world_map_out_of_bounds() {
        let map = WorldMap::new(5, 5, 16, 3);
        assert!(map.get_tile(0, 10, 10).is_none());
        assert!(map.get_tile(5, 0, 0).is_none()); // layer 5 doesn't exist
    }
}
