use super::tile::{Tile, TileType};
use std::collections::HashMap;

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub tiles: Vec<Tile>,
    pub dirty: bool,
}

impl Chunk {
    pub fn new(x: i32, y: i32) -> Self {
        let tiles = vec![Tile::new(0, TileType::Empty); CHUNK_SIZE * CHUNK_SIZE];
        Self {
            x,
            y,
            tiles,
            dirty: false,
        }
    }

    pub fn get_tile(&self, local_x: usize, local_y: usize) -> Option<&Tile> {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            self.tiles.get(local_y * CHUNK_SIZE + local_x)
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, local_x: usize, local_y: usize, tile: Tile) {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            self.tiles[local_y * CHUNK_SIZE + local_x] = tile;
            self.dirty = true;
        }
    }

    pub fn fill(&mut self, tile: Tile) {
        for t in &mut self.tiles {
            *t = tile.clone();
        }
        self.dirty = true;
    }

    pub fn count_type(&self, tile_type: TileType) -> usize {
        self.tiles
            .iter()
            .filter(|t| t.tile_type == tile_type)
            .count()
    }
}

#[derive(Debug, Clone)]
pub struct ChunkManager {
    chunks: HashMap<(i32, i32), Chunk>,
    view_radius: i32,
}

impl ChunkManager {
    pub fn new(view_radius: i32) -> Self {
        Self {
            chunks: HashMap::new(),
            view_radius,
        }
    }

    pub fn get_chunk(&self, cx: i32, cy: i32) -> Option<&Chunk> {
        self.chunks.get(&(cx, cy))
    }

    pub fn get_chunk_mut(&mut self, cx: i32, cy: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(cx, cy))
    }

    pub fn load_chunk(&mut self, cx: i32, cy: i32) -> &mut Chunk {
        self.chunks
            .entry((cx, cy))
            .or_insert_with(|| Chunk::new(cx, cy))
    }

    pub fn unload_chunk(&mut self, cx: i32, cy: i32) -> bool {
        self.chunks.remove(&(cx, cy)).is_some()
    }

    pub fn world_to_chunk(&self, wx: f64, wy: f64) -> (i32, i32) {
        (
            (wx / CHUNK_SIZE as f64).floor() as i32,
            (wy / CHUNK_SIZE as f64).floor() as i32,
        )
    }

    pub fn world_to_local(&self, wx: f64, wy: f64) -> (usize, usize) {
        ((wx as usize) % CHUNK_SIZE, (wy as usize) % CHUNK_SIZE)
    }

    pub fn loaded_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn visible_chunks(&self, center_x: i32, center_y: i32) -> Vec<(i32, i32)> {
        let mut visible = Vec::new();
        for dy in -self.view_radius..=self.view_radius {
            for dx in -self.view_radius..=self.view_radius {
                visible.push((center_x + dx, center_y + dy));
            }
        }
        visible
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new(3)
    }
}
