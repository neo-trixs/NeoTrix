use super::tile::{Tile, TileType, WorldMap};
use super::zone::{Zone, ZoneType};

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub seed: u64,
    pub farm_position: (u32, u32),
    pub farm_size: (u32, u32),
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            width: 100,
            height: 80,
            tile_size: 16,
            seed: 42,
            farm_position: (10, 10),
            farm_size: (20, 15),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Meadow,
    Forest,
    Mountain,
    Lake,
    Beach,
    Cave,
}

pub struct WorldGenerator;

impl WorldGenerator {
    pub fn generate(config: &GeneratorConfig) -> WorldMap {
        let mut map = WorldMap::new(config.width, config.height, config.tile_size, 3);

        Self::generate_terrain(&mut map, config);
        Self::place_farm(&mut map, config);
        Self::generate_zones(&mut map, config);
        Self::generate_paths(&mut map);
        Self::place_decorations(&mut map, config);
        Self::place_special_tiles(&mut map, config);

        map
    }

    fn hash_pos(x: u32, y: u32, seed: u64) -> u32 {
        let n = (x as u64)
            .wrapping_mul(374761393)
            .wrapping_add((y as u64).wrapping_mul(668265263))
            .wrapping_add(seed);
        let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
        (n & 0xFFFF) as u32
    }

    fn generate_terrain(map: &mut WorldMap, config: &GeneratorConfig) {
        for y in 0..config.height {
            for x in 0..config.width {
                let hash = Self::hash_pos(x, y, config.seed) % 100;
                let tile_type = if hash < 5 {
                    TileType::Water
                } else if hash < 8 {
                    TileType::DeepWater
                } else if hash < 18 {
                    TileType::Tree
                } else if hash < 23 {
                    TileType::Rock
                } else if hash < 28 {
                    TileType::Bush
                } else if hash < 30 {
                    TileType::Mushroom
                } else if hash < 32 {
                    TileType::Crystal
                } else {
                    TileType::Grass
                };

                map.set_tile(0, x, y, Tile::new(tile_type));
            }
        }
    }

    fn place_farm(map: &mut WorldMap, config: &GeneratorConfig) {
        let (fx, fy) = config.farm_position;
        let (fw, fh) = config.farm_size;
        for y in fy..fy + fh {
            for x in fx..fx + fw {
                map.set_tile(0, x, y, Tile::new(TileType::Dirt));
                map.set_tile(1, x, y, Tile::new(TileType::Void));
            }
        }

        // Farm path
        let mid_y = fy + fh / 2;
        for x in fx..fx + fw {
            map.set_tile(0, x, mid_y, Tile::new(TileType::Path));
        }

        // Placeholder — spawn point placed after paths in place_special_tiles
    }

    fn generate_zones(map: &mut WorldMap, config: &GeneratorConfig) {
        let zones = vec![
            Zone::new(
                ZoneType::Farm,
                config.farm_position,
                config.farm_size,
                "Thought Meadow",
            ),
            Zone::new(ZoneType::Town, (50, 30), (20, 20), "Neural Hub"),
            Zone::new(ZoneType::Mine, (70, 10), (15, 15), "Knowledge Mines"),
            Zone::new(ZoneType::Forest, (10, 50), (25, 25), "Memory Forest"),
            Zone::new(ZoneType::Lake, (60, 60), (20, 15), "Dream Lake"),
        ];
        map.zones = zones;
    }

    fn generate_paths(map: &mut WorldMap) {
        // Connect zones with straight paths on the ground layer
        let zone_positions: Vec<(u32, u32)> = map.zones.iter().map(|z| z.center()).collect();

        for i in 0..zone_positions.len() {
            for j in (i + 1)..zone_positions.len() {
                let (x1, y1) = zone_positions[i];
                let (x2, y2) = zone_positions[j];

                // Horizontal then vertical path
                let min_x = x1.min(x2);
                let max_x = x1.max(x2);
                for x in min_x..=max_x {
                    if x < map.width {
                        map.set_tile(0, x, y1, Tile::new(TileType::Path));
                    }
                }
                let min_y = y1.min(y2);
                let max_y = y1.max(y2);
                for y in min_y..=max_y {
                    if y < map.height {
                        map.set_tile(0, x2, y, Tile::new(TileType::Path));
                    }
                }
            }
        }
    }

    fn place_decorations(map: &mut WorldMap, _config: &GeneratorConfig) {
        // Clear decoration layer to Void, then add some flowers
        for y in 0..map.height {
            for x in 0..map.width {
                let ground = map.get_tile(0, x, y);
                if let Some(tile) = ground {
                    if tile.tile_type == TileType::Grass {
                        let hash = Self::hash_pos(x, y, 99);
                        if hash % 20 == 0 {
                            map.set_tile(1, x, y, Tile::new(TileType::Mushroom));
                        }
                    }
                }
            }
        }
    }

    fn place_special_tiles(map: &mut WorldMap, config: &GeneratorConfig) {
        // Spawn point at farm center (placed last so paths don't overwrite it)
        let (fx, fy) = config.farm_position;
        let (fw, fh) = config.farm_size;
        let cx = fx + fw / 2;
        let cy = fy + fh / 2;
        map.set_tile(0, cx, cy, Tile::new(TileType::SpawnPoint));

        // Exit points at zone edges — collect positions first to avoid borrow conflict
        let zone_positions: Vec<((u32, u32), (u32, u32))> = map
            .zones
            .iter()
            .map(|z| (z.position, z.size))
            .collect();
        for ((zx, zy), (zw, zh)) in zone_positions {
            if zx + zw < map.width {
                map.set_tile(0, zx + zw, zy + zh / 2, Tile::new(TileType::ExitPoint));
            }
        }
    }

    pub fn biome_at(config: &GeneratorConfig, x: u32, y: u32) -> BiomeType {
        let hash = Self::hash_pos(x, y, config.seed);
        match hash % 6 {
            0 => BiomeType::Meadow,
            1 => BiomeType::Forest,
            2 => BiomeType::Mountain,
            3 => BiomeType::Lake,
            4 => BiomeType::Beach,
            _ => BiomeType::Cave,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default() {
        let config = GeneratorConfig::default();
        let map = WorldGenerator::generate(&config);
        assert_eq!(map.width, 100);
        assert_eq!(map.height, 80);
        assert_eq!(map.zones.len(), 5);
    }

    #[test]
    fn test_farm_placed() {
        let config = GeneratorConfig::default();
        let map = WorldGenerator::generate(&config);
        let (fx, fy) = config.farm_position;
        let (fw, fh) = config.farm_size;

        // Farm area should be Dirt
        assert_eq!(
            map.get_tile(0, fx, fy).unwrap().tile_type,
            TileType::Dirt
        );
        assert_eq!(
            map.get_tile(0, fx + fw - 1, fy + fh - 1)
                .unwrap()
                .tile_type,
            TileType::Dirt
        );
    }

    #[test]
    fn test_spawn_point_exists() {
        let config = GeneratorConfig::default();
        let map = WorldGenerator::generate(&config);
        let (fx, fy) = config.farm_position;
        let (fw, fh) = config.farm_size;
        let cx = fx + fw / 2;
        let cy = fy + fh / 2;
        assert_eq!(
            map.get_tile(0, cx, cy).unwrap().tile_type,
            TileType::SpawnPoint
        );
    }

    #[test]
    fn test_paths_exist() {
        let config = GeneratorConfig::default();
        let map = WorldGenerator::generate(&config);
        let has_path = map.layers[0]
            .iter()
            .any(|t| t.tile_type == TileType::Path);
        assert!(has_path);
    }

    #[test]
    fn test_hash_deterministic() {
        let a = WorldGenerator::hash_pos(5, 10, 42);
        let b = WorldGenerator::hash_pos(5, 10, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn test_biome_variety() {
        let config = GeneratorConfig::default();
        let mut biomes = std::collections::HashSet::new();
        for y in 0..20 {
            for x in 0..20 {
                biomes.insert(WorldGenerator::biome_at(&config, x, y));
            }
        }
        assert!(biomes.len() > 1);
    }
}
