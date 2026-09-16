use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type TileId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Empty,
    Ground,
    Water,
    Wall,
    Door,
    Tree,
    Rock,
    Grass,
    Sand,
    Snow,
    Lava,
    Custom(TileId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub id: TileId,
    pub tile_type: TileType,
    pub walkable: bool,
    pub transparent: bool,
    pub metadata: HashMap<String, String>,
}

impl Tile {
    pub fn new(id: TileId, tile_type: TileType) -> Self {
        Self {
            id,
            tile_type,
            walkable: true,
            transparent: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_walkable(mut self, walkable: bool) -> Self {
        self.walkable = walkable;
        self
    }

    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(0, TileType::Empty)
    }
}
