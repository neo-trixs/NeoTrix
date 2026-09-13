use rand::Rng;

use super::map::{CollisionType, MapLayer, Tile, TileMap};
use super::renderer::Vec2;

// ---------------------------------------------------------------------------
// Perlin-like Noise (value noise with smoothing)
// ---------------------------------------------------------------------------

/// Simple value noise generator (no external crate needed).
pub struct ValueNoise {
    perm: Vec<u8>,
    seed: u64,
}

impl ValueNoise {
    pub fn new(seed: u64) -> Self {
        let mut perm = vec![0u8; 512];
        let mut rng = rand::thread_rng();
        for i in 0..256 {
            perm[i] = i as u8;
        }
        // Fisher-Yates shuffle
        for i in (1..256).rev() {
            let j = rng.gen_range(0..=i);
            perm.swap(i, j);
        }
        for i in 0..256 {
            perm[i + 256] = perm[i];
        }
        Self { perm, seed }
    }

    fn hash(&self, x: i32, y: i32) -> usize {
        let h = (x as u64).wrapping_mul(374761393)
            .wrapping_add(y as u64).wrapping_mul(668265263)
            .wrapping_add(self.seed);
        (h & 0xFF) as usize
    }

    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// Sample 2D value noise at (x, y). Returns value in 0.0..1.0.
    pub fn noise2d(&self, x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - x.floor();
        let yf = y - y.floor();

        let u = Self::fade(xf);
        let v = Self::fade(yf);

        let a = self.perm[self.hash(xi, yi)] as f32 / 255.0;
        let b = self.perm[self.hash(xi + 1, yi)] as f32 / 255.0;
        let c = self.perm[self.hash(xi, yi + 1)] as f32 / 255.0;
        let d = self.perm[self.hash(xi + 1, yi + 1)] as f32 / 255.0;

        Self::lerp(
            Self::lerp(a, b, u),
            Self::lerp(c, d, u),
            v,
        )
    }

    /// Fractal Brownian Motion (octaves of noise).
    pub fn fbm(&self, x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            value += self.noise2d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= gain;
            frequency *= lacunarity;
        }

        value / max_value
    }
}

// ---------------------------------------------------------------------------
// Biome
// ---------------------------------------------------------------------------

/// Map biome types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Forest,
    Desert,
    Snow,
    Volcanic,
    Water,
    Plains,
    Swamp,
}

impl Biome {
    /// Base tile ID for this biome's ground tiles.
    pub fn ground_tile_id(&self) -> u32 {
        match self {
            Biome::Forest => 100,
            Biome::Desert => 200,
            Biome::Snow => 300,
            Biome::Volcanic => 400,
            Biome::Water => 500,
            Biome::Plains => 600,
            Biome::Swamp => 700,
        }
    }

    /// Wall tile ID for this biome.
    pub fn wall_tile_id(&self) -> u32 {
        self.ground_tile_id() + 10
    }

    /// Decoration tile IDs for this biome.
    pub fn deco_tile_ids(&self) -> Vec<u32> {
        match self {
            Biome::Forest => vec![120, 121, 122], // trees, bushes, flowers
            Biome::Desert => vec![220, 221],       // cactus, rocks
            Biome::Snow => vec![320, 321],         // snowman, pine
            Biome::Volcanic => vec![420, 421],     // lava, obsidian
            Biome::Water => vec![520],             // coral
            Biome::Plains => vec![620, 621],       // tall grass, rock
            Biome::Swamp => vec![720, 721],        // mushrooms, vines
        }
    }

    /// Collision type for ground tiles.
    pub fn ground_collision(&self) -> CollisionType {
        match self {
            Biome::Water => CollisionType::Water,
            _ => CollisionType::None,
        }
    }

    /// Speed modifier for this biome.
    pub fn ground_speed(&self) -> f32 {
        match self {
            Biome::Desert => 0.7,
            Biome::Snow => 0.8,
            Biome::Swamp => 0.6,
            Biome::Water => 0.4,
            _ => 1.0,
        }
    }

    /// Damage per step for this biome (e.g. volcanic).
    pub fn ground_damage(&self) -> i32 {
        match self {
            Biome::Volcanic => 5,
            _ => 0,
        }
    }

    /// Palette color for minimap rendering.
    pub fn palette_color(&self) -> super::renderer::Color {
        use super::renderer::Color;
        match self {
            Biome::Forest => Color::rgba(0.1, 0.5, 0.1, 1.0),
            Biome::Desert => Color::rgba(0.8, 0.7, 0.4, 1.0),
            Biome::Snow => Color::rgba(0.9, 0.9, 0.95, 1.0),
            Biome::Volcanic => Color::rgba(0.6, 0.1, 0.0, 1.0),
            Biome::Water => Color::rgba(0.1, 0.3, 0.8, 1.0),
            Biome::Plains => Color::rgba(0.3, 0.6, 0.2, 1.0),
            Biome::Swamp => Color::rgba(0.2, 0.35, 0.15, 1.0),
        }
    }
}

/// Select a biome from noise values (height, moisture).
pub fn select_biome(height: f32, moisture: f32) -> Biome {
    if height < 0.2 {
        Biome::Water
    } else if height < 0.35 {
        if moisture > 0.6 { Biome::Swamp } else { Biome::Plains }
    } else if height < 0.65 {
        if moisture > 0.6 { Biome::Forest }
        else if moisture > 0.3 { Biome::Plains }
        else { Biome::Desert }
    } else if height < 0.8 {
        if moisture > 0.5 { Biome::Snow } else { Biome::Desert }
    } else {
        Biome::Volcanic
    }
}

// ---------------------------------------------------------------------------
// Region Template
// ---------------------------------------------------------------------------

/// A pre-defined region layout that overrides biome generation in an area.
#[derive(Debug, Clone)]
pub struct RegionTemplate {
    pub name: String,
    pub biome: Biome,
    /// Relative position (0.0..1.0) within the map.
    pub position: Vec2,
    /// Size in tiles.
    pub size: (u32, u32),
    /// Spawn points for entities within this region.
    pub spawn_points: Vec<Vec2>,
    /// NPC definitions: (npc_name, relative_position within region).
    pub npcs: Vec<(String, Vec2)>,
}

impl RegionTemplate {
    pub fn forest(center: Vec2) -> Self {
        Self {
            name: "Forest Region".to_string(),
            biome: Biome::Forest,
            position: center,
            size: (16, 16),
            spawn_points: vec![Vec2::new(0.5, 0.5)],
            npcs: vec![
                ("Lumberjack".to_string(), Vec2::new(0.3, 0.7)),
                ("Herbalist".to_string(), Vec2::new(0.7, 0.3)),
            ],
        }
    }

    pub fn desert(center: Vec2) -> Self {
        Self {
            name: "Desert Region".to_string(),
            biome: Biome::Desert,
            position: center,
            size: (20, 20),
            spawn_points: vec![Vec2::new(0.5, 0.5)],
            npcs: vec![
                ("Merchant".to_string(), Vec2::new(0.5, 0.5)),
            ],
        }
    }

    pub fn snow(center: Vec2) -> Self {
        Self {
            name: "Snow Region".to_string(),
            biome: Biome::Snow,
            position: center,
            size: (18, 18),
            spawn_points: vec![Vec2::new(0.5, 0.5)],
            npcs: vec![
                ("Ice Mage".to_string(), Vec2::new(0.6, 0.4)),
            ],
        }
    }

    pub fn volcanic(center: Vec2) -> Self {
        Self {
            name: "Volcanic Region".to_string(),
            biome: Biome::Volcanic,
            position: center,
            size: (14, 14),
            spawn_points: vec![Vec2::new(0.5, 0.5)],
            npcs: vec![
                ("Fire Giant".to_string(), Vec2::new(0.5, 0.5)),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// MapGenerator
// ---------------------------------------------------------------------------

/// Configuration for map generation.
#[derive(Debug, Clone)]
pub struct MapGenConfig {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub seed: u64,
    pub height_octaves: u32,
    pub moisture_octaves: u32,
    pub height_scale: f32,
    pub moisture_scale: f32,
    pub wall_threshold: f32,
    pub decoration_density: f32,
    pub region_templates: Vec<RegionTemplate>,
}

impl Default for MapGenConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            tile_size: 16.0,
            seed: 42,
            height_octaves: 4,
            moisture_octaves: 3,
            height_scale: 0.05,
            moisture_scale: 0.08,
            wall_threshold: 0.85,
            decoration_density: 0.05,
            region_templates: Vec::new(),
        }
    }
}

/// Generated map output.
#[derive(Debug)]
pub struct GeneratedMap {
    pub tilemap: TileMap,
    pub biome_grid: Vec<Vec<Biome>>,
    pub spawn_points: Vec<Vec2>,
    pub npc_placements: Vec<(String, Vec2)>,
}

/// Procedural map generator using value noise.
pub struct MapGenerator;

impl MapGenerator {
    /// Generate a complete TileMap from config.
    pub fn generate(config: &MapGenConfig) -> GeneratedMap {
        let height_noise = ValueNoise::new(config.seed);
        let moisture_noise = ValueNoise::new(config.seed.wrapping_add(1));
        let detail_noise = ValueNoise::new(config.seed.wrapping_add(2));

        let mut biome_grid = vec![vec![Biome::Plains; config.width]; config.height];
        let mut ground_layer = MapLayer::new("ground", config.width, config.height);
        let mut wall_layer = MapLayer::new("walls", config.width, config.height);
        wall_layer.z_index = 1;

        let mut spawn_points = Vec::new();
        let mut npc_placements: Vec<(String, Vec2)> = Vec::new();

        // Phase 1: Biome selection via noise
        for y in 0..config.height {
            for x in 0..config.width {
                let nx = x as f32 * config.height_scale;
                let ny = y as f32 * config.height_scale;

                let height = height_noise.fbm(nx, ny, config.height_octaves, 2.0, 0.5);
                let moisture = moisture_noise.fbm(
                    nx * config.moisture_scale / config.height_scale,
                    ny * config.moisture_scale / config.height_scale,
                    config.moisture_octaves,
                    2.0,
                    0.5,
                );

                let biome = select_biome(height, moisture);
                biome_grid[y][x] = biome;

                // Create ground tile
                let mut tile = Tile::new(biome.ground_tile_id())
                    .with_collision(biome.ground_collision())
                    .with_speed(biome.ground_speed())
                    .with_damage(biome.ground_damage());
                tile.properties.insert("biome".to_string(), super::map::TileProperty::String(format!("{:?}", biome)));

                ground_layer.set_tile(x, y, tile);

                // Add walls for high elevation
                if height > config.wall_threshold {
                    let wall = Tile::new(biome.wall_tile_id())
                        .with_collision(CollisionType::Solid);
                    wall_layer.set_tile(x, y, wall);
                }

                // Add decorations with probability
                let deco_threshold = 1.0 - config.decoration_density;
                let deco_noise = detail_noise.noise2d(x as f32 * 0.3, y as f32 * 0.3);
                if deco_noise > deco_threshold && height < config.wall_threshold {
                    let deco_ids = biome.deco_tile_ids();
                    if !deco_ids.is_empty() {
                        let idx = (deco_noise * deco_ids.len() as f32) as usize % deco_ids.len();
                        let deco_tile = Tile::new(deco_ids[idx])
                            .with_collision(CollisionType::Solid);
                        wall_layer.set_tile(x, y, deco_tile);
                    }
                }
            }
        }

        // Phase 2: Apply region templates
        for region in &config.region_templates {
            let origin_x = (region.position.x * config.width as f32) as usize;
            let origin_y = (region.position.y * config.height as f32) as usize;
            let (rw, rh) = region.size;

            for dy in 0..rh {
                for dx in 0..rw {
                    let tx = origin_x + dx as usize;
                    let ty = origin_y + dy as usize;
                    if tx < config.width && ty < config.height {
                        biome_grid[ty][tx] = region.biome;

                        let tile = Tile::new(region.biome.ground_tile_id())
                            .with_collision(region.biome.ground_collision())
                            .with_speed(region.biome.ground_speed())
                            .with_damage(region.biome.ground_damage());
                        ground_layer.set_tile(tx, ty, tile);
                    }
                }
            }

            // Place region spawn points
            for sp in &region.spawn_points {
                let wx = (origin_x as f32 + sp.x * rw as f32) * config.tile_size;
                let wy = (origin_y as f32 + sp.y * rh as f32) * config.tile_size;
                spawn_points.push(Vec2::new(wx, wy));
            }

            // Place region NPCs
            for (name, pos) in &region.npcs {
                let wx = (origin_x as f32 + pos.x * rw as f32) * config.tile_size;
                let wy = (origin_y as f32 + pos.y * rh as f32) * config.tile_size;
                npc_placements.push((name.clone(), Vec2::new(wx, wy)));
            }
        }

        // Phase 3: Place default spawn at center if none exist
        if spawn_points.is_empty() {
            let cx = config.width as f32 * config.tile_size * 0.5;
            let cy = config.height as f32 * config.tile_size * 0.5;
            spawn_points.push(Vec2::new(cx, cy));
        }

        // Phase 4: Build TileMap
        let mut tilemap = TileMap::new(config.width, config.height, config.tile_size);
        tilemap.add_layer(ground_layer);
        tilemap.add_layer(wall_layer);

        // Set palette colors from biomes
        for biome in &[Biome::Forest, Biome::Desert, Biome::Snow, Biome::Volcanic, Biome::Water, Biome::Plains, Biome::Swamp] {
            tilemap.set_palette_color(biome.ground_tile_id(), biome.palette_color());
        }

        GeneratedMap {
            tilemap,
            biome_grid,
            spawn_points,
            npc_placements,
        }
    }

    /// Generate a simple dungeon-style map with rooms and corridors.
    pub fn generate_dungeon(config: &MapGenConfig, num_rooms: u32, room_min: u32, room_max: u32) -> GeneratedMap {
        let mut rng = rand::thread_rng();
        let mut ground_layer = MapLayer::new("ground", config.width, config.height);
        let mut wall_layer = MapLayer::new("walls", config.width, config.height);
        wall_layer.z_index = 1;

        let mut rooms: Vec<(usize, usize, u32, u32)> = Vec::new();
        let mut spawn_points = Vec::new();
        let mut npc_placements = Vec::new();

        // Fill with walls
        for y in 0..config.height {
            for x in 0..config.width {
                let wall = Tile::new(410).with_collision(CollisionType::Solid);
                wall_layer.set_tile(x, y, wall);
            }
        }

        // Place rooms
        for _ in 0..num_rooms {
            let w = rng.gen_range(room_min..=room_max);
            let h = rng.gen_range(room_min..=room_max);
            let x = rng.gen_range(1..config.width.saturating_sub(w as usize + 1));
            let y = rng.gen_range(1..config.height.saturating_sub(h as usize + 1));

            // Check overlap
            let overlaps = rooms.iter().any(|&(rx, ry, rw, rh)| {
                x < (rx + rw as usize + 1) && x + w as usize + 1 > rx
                    && y < (ry + rh as usize + 1) && y + h as usize + 1 > ry
            });
            if overlaps {
                continue;
            }

            // Carve room
            for dy in 0..h {
                for dx in 0..w {
                    let tx = x + dx as usize;
                    let ty = y + dy as usize;
                    if tx < config.width && ty < config.height {
                        let floor = Tile::new(400).with_collision(CollisionType::None).with_speed(1.0);
                        ground_layer.set_tile(tx, ty, floor);
                        wall_layer.set_tile(tx, ty, Tile::new(0));
                    }
                }
            }

            // Room center as spawn
            let cx = (x as f32 + w as f32 * 0.5) * config.tile_size;
            let cy = (y as f32 + h as f32 * 0.5) * config.tile_size;
            spawn_points.push(Vec2::new(cx, cy));
            rooms.push((x, y, w, h));
        }

        // Connect rooms with corridors
        for i in 1..rooms.len() {
            let (ax, ay, aw, ah) = rooms[i - 1];
            let (bx, by, _bw, _bh) = rooms[i];
            let x1 = ax + aw as usize / 2;
            let y1 = ay + ah as usize / 2;
            let x2 = bx + _bw as usize / 2;
            let y2 = by + _bh as usize / 2;

            // Horizontal then vertical
            let min_x = x1.min(x2);
            let max_x = x1.max(x2);
            for x in min_x..=max_x {
                if x < config.width && y1 < config.height {
                    let floor = Tile::new(400).with_collision(CollisionType::None);
                    ground_layer.set_tile(x, y1, floor);
                    wall_layer.set_tile(x, y1, Tile::new(0));
                }
            }
            let min_y = y1.min(y2);
            let max_y = y1.max(y2);
            for y in min_y..=max_y {
                if x2 < config.width && y < config.height {
                    let floor = Tile::new(400).with_collision(CollisionType::None);
                    ground_layer.set_tile(x2, y, floor);
                    wall_layer.set_tile(x2, y, Tile::new(0));
                }
            }
        }

        // Place NPCs in rooms (skip first room which is spawn)
        let npc_names = ["Guard", "Merchant", "Healer", "Blacksmith", "Sage"];
        for (i, &(rx, ry, rw, rh)) in rooms.iter().enumerate().skip(1) {
            if i - 1 >= npc_names.len() { break; }
            let nx = (rx as f32 + rw as f32 * 0.5) * config.tile_size;
            let ny = (ry as f32 + rh as f32 * 0.3) * config.tile_size;
            npc_placements.push((npc_names[i - 1].to_string(), Vec2::new(nx, ny)));
        }

        // Build TileMap
        let mut tilemap = TileMap::new(config.width, config.height, config.tile_size);
        tilemap.add_layer(ground_layer);
        tilemap.add_layer(wall_layer);

        // Set dungeon palette
        tilemap.set_palette_color(400, super::renderer::Color::rgba(0.3, 0.3, 0.35, 1.0));
        tilemap.set_palette_color(410, super::renderer::Color::rgba(0.15, 0.15, 0.2, 1.0));

        GeneratedMap {
            tilemap,
            biome_grid: vec![vec![Biome::Plains; config.width]; config.height],
            spawn_points,
            npc_placements,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_noise_range() {
        let noise = ValueNoise::new(42);
        for i in 0..100 {
            let x = i as f32 * 0.1;
            let v = noise.noise2d(x, 0.0);
            assert!(v >= 0.0 && v <= 1.0, "noise value {v} out of range");
        }
    }

    #[test]
    fn test_fbm_range() {
        let noise = ValueNoise::new(42);
        for i in 0..100 {
            let x = i as f32 * 0.1;
            let v = noise.fbm(x, 0.0, 4, 2.0, 0.5);
            assert!(v >= 0.0 && v <= 1.0, "fbm value {v} out of range");
        }
    }

    #[test]
    fn test_biome_selection() {
        assert_eq!(select_biome(0.1, 0.5), Biome::Water);
        assert_eq!(select_biome(0.5, 0.8), Biome::Forest);
        assert_eq!(select_biome(0.5, 0.1), Biome::Desert);
        assert_eq!(select_biome(0.9, 0.5), Biome::Volcanic);
    }

    #[test]
    fn test_generate_map() {
        let config = MapGenConfig {
            width: 16,
            height: 16,
            tile_size: 16.0,
            seed: 42,
            ..Default::default()
        };
        let result = MapGenerator::generate(&config);
        assert_eq!(result.tilemap.width, 16);
        assert_eq!(result.tilemap.height, 16);
        assert_eq!(result.tilemap.layers.len(), 2);
        assert!(!result.spawn_points.is_empty());
    }

    #[test]
    fn test_generate_dungeon() {
        let config = MapGenConfig {
            width: 32,
            height: 32,
            tile_size: 16.0,
            ..Default::default()
        };
        let result = MapGenerator::generate_dungeon(&config, 6, 4, 8);
        assert_eq!(result.tilemap.layers.len(), 2);
        assert!(!result.spawn_points.is_empty());
    }

    #[test]
    fn test_region_templates() {
        let mut config = MapGenConfig {
            width: 32,
            height: 32,
            ..Default::default()
        };
        config.region_templates.push(RegionTemplate::forest(Vec2::new(0.5, 0.5)));
        let result = MapGenerator::generate(&config);
        assert!(!result.npc_placements.is_empty());
    }

    #[test]
    fn test_biome_properties() {
        assert_eq!(Biome::Water.ground_collision(), CollisionType::Water);
        assert!(Biome::Desert.ground_speed() < 1.0);
        assert!(Biome::Volcanic.ground_damage() > 0);
        assert_eq!(Biome::Plains.ground_damage(), 0);
    }
}
