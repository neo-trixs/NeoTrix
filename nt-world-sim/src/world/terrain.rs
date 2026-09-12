use super::tile::{TileMap, Tile, TileType};

pub struct TerrainGenerator {
    seed: u64,
}

impl TerrainGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Simple noise function (hash-based)
    fn noise(&self, x: i32, y: i32) -> f32 {
        let n = (x as u64).wrapping_mul(374761393)
            .wrapping_add((y as u64).wrapping_mul(668265263))
            .wrapping_add(self.seed);
        let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        (n & 0xFFFF) as f32 / 65535.0
    }

    /// Fractal Brownian Motion
    fn fbm(&self, x: f32, y: f32, octaves: u32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        for _ in 0..octaves {
            let ix = (x * frequency) as i32;
            let iy = (y * frequency) as i32;
            value += amplitude * self.noise(ix, iy);
            amplitude *= 0.5;
            frequency *= 2.0;
        }
        value
    }

    pub fn generate_farm(&self, map: &mut TileMap) {
        // Farm layout: grass field with dirt path and water
        for y in 0..map.height {
            for x in 0..map.width {
                let elevation = self.fbm(x as f32 * 0.05, y as f32 * 0.05, 4);
                let moisture = self.fbm(x as f32 * 0.08 + 100.0, y as f32 * 0.08 + 100.0, 3);

                let tile_type = if elevation < 0.3 {
                    TileType::Water
                } else if elevation < 0.4 {
                    TileType::Sand
                } else if moisture > 0.7 && elevation > 0.6 {
                    TileType::Tree
                } else if x == 0 || x == map.width - 1 || y == 0 || y == map.height - 1 {
                    TileType::Wall
                } else {
                    TileType::Grass
                };

                map.set(x, y, Tile::new(tile_type));
            }
        }

        // Add paths
        for x in 1..map.width - 1 {
            map.set(x, map.height / 2, Tile::new(TileType::Path));
        }
        for y in 1..map.height - 1 {
            map.set(map.width / 2, y, Tile::new(TileType::Path));
        }
    }

    pub fn generate_mine_floor(&self, map: &mut TileMap, floor: u32) {
        let density = 0.3 + (floor as f32 * 0.01).min(0.5);

        for y in 0..map.height {
            for x in 0..map.width {
                let n = self.noise(x as i32 + floor as i32 * 100, y as i32);
                let tile_type = if n < density {
                    TileType::Stone
                } else if x == 0 || x == map.width - 1 || y == 0 || y == map.height - 1 {
                    TileType::Wall
                } else {
                    TileType::Dirt
                };
                map.set(x, y, Tile::new(tile_type));
            }
        }

        // Add entrance at top-center
        map.set(map.width / 2, 0, Tile::new(TileType::Path));
        map.set(map.width / 2, 1, Tile::new(TileType::Path));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_farm_generation() {
        let gen = TerrainGenerator::new(42);
        let mut map = TileMap::new(20, 20, 16.0);
        gen.generate_farm(&mut map);

        // Should have some water, grass, paths
        let mut has_water = false;
        let mut has_grass = false;
        let mut has_path = false;
        for tile in &map.tiles {
            match tile.tile_type {
                TileType::Water => has_water = true,
                TileType::Grass => has_grass = true,
                TileType::Path => has_path = true,
                _ => {}
            }
        }
        assert!(has_water || has_grass); // At least one terrain type
        assert!(has_path); // Path always generated
    }

    #[test]
    fn test_mine_generation() {
        let gen = TerrainGenerator::new(42);
        let mut map = TileMap::new(16, 16, 16.0);
        gen.generate_mine_floor(&mut map, 5);

        // Entrance should be walkable
        assert!(map.is_walkable(8, 0));
        assert!(map.is_walkable(8, 1));
    }
}
