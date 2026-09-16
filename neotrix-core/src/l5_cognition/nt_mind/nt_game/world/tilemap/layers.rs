use super::chunk::{ChunkManager, CHUNK_SIZE};
use super::tile::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapLayer {
    Terrain,
    Objects,
    Entities,
    Effects,
}

#[derive(Debug, Clone)]
pub struct LayeredTileMap {
    terrain: ChunkManager,
    objects: ChunkManager,
    entities: ChunkManager,
    effects: ChunkManager,
    width: usize,
    height: usize,
}

impl LayeredTileMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            terrain: ChunkManager::new(3),
            objects: ChunkManager::new(3),
            entities: ChunkManager::new(3),
            effects: ChunkManager::new(3),
            width,
            height,
        }
    }

    pub fn get_tile(&self, layer: MapLayer, x: usize, y: usize) -> Option<&Tile> {
        let (cx, cy) = ((x / CHUNK_SIZE) as i32, (y / CHUNK_SIZE) as i32);
        let (lx, ly) = (x % CHUNK_SIZE, y % CHUNK_SIZE);
        let manager = match layer {
            MapLayer::Terrain => &self.terrain,
            MapLayer::Objects => &self.objects,
            MapLayer::Entities => &self.entities,
            MapLayer::Effects => &self.effects,
        };
        manager.get_chunk(cx, cy).and_then(|c| c.get_tile(lx, ly))
    }

    pub fn set_tile(&mut self, layer: MapLayer, x: usize, y: usize, tile: Tile) {
        let (cx, cy) = ((x / CHUNK_SIZE) as i32, (y / CHUNK_SIZE) as i32);
        let (lx, ly) = (x % CHUNK_SIZE, y % CHUNK_SIZE);
        let manager = match layer {
            MapLayer::Terrain => &mut self.terrain,
            MapLayer::Objects => &mut self.objects,
            MapLayer::Entities => &mut self.entities,
            MapLayer::Effects => &mut self.effects,
        };
        manager.load_chunk(cx, cy).set_tile(lx, ly, tile);
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        self.get_tile(MapLayer::Terrain, x, y)
            .map_or(false, |t| t.walkable)
            && self
                .get_tile(MapLayer::Objects, x, y)
                .map_or(true, |t| t.walkable)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn loaded_chunks(&self) -> usize {
        self.terrain.loaded_chunks()
            + self.objects.loaded_chunks()
            + self.entities.loaded_chunks()
            + self.effects.loaded_chunks()
    }
}
