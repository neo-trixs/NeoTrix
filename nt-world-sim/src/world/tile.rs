use crate::engine::renderer::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    Grass,
    Dirt,
    Water,
    Stone,
    Sand,
    Snow,
    Tree,
    Path,
    Wall,
    Crop { crop_id: u32, growth_stage: u8 },
    Building,
    Furniture,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        matches!(self, TileType::Grass | TileType::Dirt | TileType::Sand | TileType::Path | TileType::Snow)
    }

    pub fn is_farmable(&self) -> bool {
        matches!(self, TileType::Dirt | TileType::Grass)
    }

    pub fn color(&self) -> Color {
        match self {
            TileType::Grass => Color { r: 0.3, g: 0.6, b: 0.2, a: 1.0 },
            TileType::Dirt => Color { r: 0.4, g: 0.3, b: 0.2, a: 1.0 },
            TileType::Water => Color { r: 0.2, g: 0.4, b: 0.8, a: 1.0 },
            TileType::Stone => Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            TileType::Sand => Color { r: 0.9, g: 0.85, b: 0.6, a: 1.0 },
            TileType::Snow => Color { r: 0.9, g: 0.95, b: 1.0, a: 1.0 },
            TileType::Tree => Color { r: 0.2, g: 0.5, b: 0.1, a: 1.0 },
            TileType::Path => Color { r: 0.6, g: 0.5, b: 0.3, a: 1.0 },
            TileType::Wall => Color { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
            TileType::Crop { .. } => Color { r: 0.1, g: 0.7, b: 0.1, a: 1.0 },
            TileType::Building => Color { r: 0.5, g: 0.35, b: 0.2, a: 1.0 },
            TileType::Furniture => Color { r: 0.6, g: 0.4, b: 0.2, a: 1.0 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub tile_type: TileType,
    pub variant: u32,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            variant: 0,
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    pub tiles: Vec<Tile>,
}

impl TileMap {
    pub fn new(width: u32, height: u32, tile_size: f32) -> Self {
        let tiles = vec![Tile::new(TileType::Grass); (width * height) as usize];
        Self { width, height, tile_size, tiles }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<&Tile> {
        if x < self.width && y < self.height {
            self.tiles.get((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x < self.width && y < self.height {
            self.tiles.get_mut((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn set(&mut self, x: u32, y: u32, tile: Tile) {
        if let Some(slot) = self.get_mut(x, y) {
            *slot = tile;
        }
    }

    pub fn set_rect(&mut self, x: u32, y: u32, w: u32, h: u32, tile_type: TileType) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy, Tile::new(tile_type));
            }
        }
    }

    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.get(x, y).map(|t| t.tile_type.is_walkable()).unwrap_or(false)
    }

    pub fn world_to_tile(&self, wx: f32, wy: f32) -> (u32, u32) {
        ((wx / self.tile_size) as u32, (wy / self.tile_size) as u32)
    }

    pub fn tile_to_world(&self, tx: u32, ty: u32) -> (f32, f32) {
        (tx as f32 * self.tile_size + self.tile_size * 0.5, ty as f32 * self.tile_size + self.tile_size * 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_map_create() {
        let map = TileMap::new(10, 10, 16.0);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.tiles.len(), 100);
    }

    #[test]
    fn test_tile_get_set() {
        let mut map = TileMap::new(5, 5, 16.0);
        map.set(2, 3, Tile::new(TileType::Water));
        assert_eq!(map.get(2, 3).unwrap().tile_type, TileType::Water);
        assert_eq!(map.get(0, 0).unwrap().tile_type, TileType::Grass);
    }

    #[test]
    fn test_walkability() {
        let mut map = TileMap::new(5, 5, 16.0);
        assert!(map.is_walkable(0, 0)); // Grass
        map.set(1, 1, Tile::new(TileType::Wall));
        assert!(!map.is_walkable(1, 1));
        map.set(2, 2, Tile::new(TileType::Water));
        assert!(!map.is_walkable(2, 2));
    }

    #[test]
    fn test_world_tile_conversion() {
        let map = TileMap::new(10, 10, 16.0);
        let (tx, ty) = map.world_to_tile(32.0, 48.0);
        assert_eq!((tx, ty), (2, 3));
        let (wx, wy) = map.tile_to_world(2, 3);
        assert!((wx - 40.0).abs() < 0.01);
        assert!((wy - 56.0).abs() < 0.01);
    }

    #[test]
    fn test_set_rect() {
        let mut map = TileMap::new(10, 10, 16.0);
        map.set_rect(2, 2, 3, 3, TileType::Dirt);
        assert_eq!(map.get(2, 2).unwrap().tile_type, TileType::Dirt);
        assert_eq!(map.get(4, 4).unwrap().tile_type, TileType::Dirt);
        assert_eq!(map.get(5, 5).unwrap().tile_type, TileType::Grass);
    }
}
