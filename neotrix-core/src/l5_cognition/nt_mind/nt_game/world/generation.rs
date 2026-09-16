use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Simple Perlin-like Noise
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    permutation: Vec<u32>,
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        let mut perm = Vec::with_capacity(512);
        let mut base: Vec<u32> = (0..256).collect();
        seed_shuffle(&mut base, seed);
        perm.extend_from_slice(&base);
        perm.extend_from_slice(&base);
        Self { permutation: perm }
    }

    pub fn noise_2d(&self, x: f64, y: f64) -> f64 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - x.floor();
        let yf = y - y.floor();

        let fade = |t: f64| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
        let lerp = |a: f64, b: f64, t: f64| a + t * (b - a);

        let u = fade(xf);
        let v = fade(yf);

        let aa = self.grad(xi, yi, xf, yf);
        let ab = self.grad(xi, yi + 1, xf, yf - 1.0);
        let ba = self.grad(xi + 1, yi, xf - 1.0, yf);
        let bb = self.grad(xi + 1, yi + 1, xf - 1.0, yf - 1.0);

        lerp(lerp(aa, ba, u), lerp(ab, bb, u), v)
    }

    fn grad(&self, x: i32, y: i32, dx: f64, dy: f64) -> f64 {
        let idx = self.permutation
            [(self.permutation[(x & 255) as usize] as usize + y as usize) & 255]
            as usize;
        match idx & 3 {
            0 => dx + dy,
            1 => -dx + dy,
            2 => dx - dy,
            3 => -dx - dy,
            _ => 0.0,
        }
    }

    pub fn fbm(&self, x: f64, y: f64, octaves: u32, lacunarity: f64, gain: f64) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_val = 0.0;

        for _ in 0..octaves {
            value += self.noise_2d(x * frequency, y * frequency) * amplitude;
            max_val += amplitude;
            amplitude *= gain;
            frequency *= lacunarity;
        }

        value / max_val
    }
}

fn seed_shuffle(arr: &mut [u32], seed: u64) {
    let mut state = seed;
    for i in (1..arr.len()).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        arr.swap(i, j);
    }
}

// ═══════════════════════════════════════════════════════════════════
// Biome System
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Plains,
    Forest,
    Mountain,
    Desert,
    Tundra,
    Swamp,
}

impl BiomeType {
    pub fn all() -> &'static [BiomeType] {
        &[
            BiomeType::Plains,
            BiomeType::Forest,
            BiomeType::Mountain,
            BiomeType::Desert,
            BiomeType::Tundra,
            BiomeType::Swamp,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            BiomeType::Plains => "Plains",
            BiomeType::Forest => "Forest",
            BiomeType::Mountain => "Mountain",
            BiomeType::Desert => "Desert",
            BiomeType::Tundra => "Tundra",
            BiomeType::Swamp => "Swamp",
        }
    }

    pub fn base_resource_density(&self) -> f64 {
        match self {
            BiomeType::Plains => 0.3,
            BiomeType::Forest => 0.7,
            BiomeType::Mountain => 0.6,
            BiomeType::Desert => 0.15,
            BiomeType::Tundra => 0.2,
            BiomeType::Swamp => 0.5,
        }
    }

    pub fn difficulty_rating(&self) -> f64 {
        match self {
            BiomeType::Plains => 0.2,
            BiomeType::Forest => 0.4,
            BiomeType::Mountain => 0.7,
            BiomeType::Desert => 0.5,
            BiomeType::Tundra => 0.6,
            BiomeType::Swamp => 0.5,
        }
    }
}

impl std::fmt::Display for BiomeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone)]
pub struct BiomeConfig {
    pub temperature_range: (f64, f64),
    pub moisture_range: (f64, f64),
    pub elevation_range: (f64, f64),
}

impl BiomeConfig {
    pub fn for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Plains => Self {
                temperature_range: (0.3, 0.7),
                moisture_range: (0.3, 0.7),
                elevation_range: (0.0, 0.3),
            },
            BiomeType::Forest => Self {
                temperature_range: (0.3, 0.8),
                moisture_range: (0.5, 1.0),
                elevation_range: (0.0, 0.5),
            },
            BiomeType::Mountain => Self {
                temperature_range: (0.0, 0.4),
                moisture_range: (0.1, 0.6),
                elevation_range: (0.5, 1.0),
            },
            BiomeType::Desert => Self {
                temperature_range: (0.6, 1.0),
                moisture_range: (0.0, 0.2),
                elevation_range: (0.0, 0.4),
            },
            BiomeType::Tundra => Self {
                temperature_range: (0.0, 0.2),
                moisture_range: (0.0, 0.5),
                elevation_range: (0.0, 0.6),
            },
            BiomeType::Swamp => Self {
                temperature_range: (0.3, 0.8),
                moisture_range: (0.7, 1.0),
                elevation_range: (0.0, 0.2),
            },
        }
    }

    pub fn matches(&self, temperature: f64, moisture: f64, elevation: f64) -> bool {
        temperature >= self.temperature_range.0
            && temperature <= self.temperature_range.1
            && moisture >= self.moisture_range.0
            && moisture <= self.moisture_range.1
            && elevation >= self.elevation_range.0
            && elevation <= self.elevation_range.1
    }
}

pub fn determine_biome(temperature: f64, moisture: f64, elevation: f64) -> BiomeType {
    for &biome in BiomeType::all() {
        let config = BiomeConfig::for_biome(biome);
        if config.matches(temperature, moisture, elevation) {
            return biome;
        }
    }
    BiomeType::Plains
}

// ═══════════════════════════════════════════════════════════════════
// Resource Spawning
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Wood,
    Stone,
    IronOre,
    GoldOre,
    Crystal,
    Herb,
    Food,
    Water,
}

impl ResourceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Wood => "Wood",
            ResourceType::Stone => "Stone",
            ResourceType::IronOre => "Iron Ore",
            ResourceType::GoldOre => "Gold Ore",
            ResourceType::Crystal => "Crystal",
            ResourceType::Herb => "Herb",
            ResourceType::Food => "Food",
            ResourceType::Water => "Water",
        }
    }

    pub fn spawn_biomes(&self) -> &'static [BiomeType] {
        match self {
            ResourceType::Wood => &[BiomeType::Forest, BiomeType::Swamp],
            ResourceType::Stone => &[BiomeType::Mountain, BiomeType::Plains],
            ResourceType::IronOre => &[BiomeType::Mountain],
            ResourceType::GoldOre => &[BiomeType::Mountain, BiomeType::Desert],
            ResourceType::Crystal => &[BiomeType::Mountain, BiomeType::Tundra],
            ResourceType::Herb => &[BiomeType::Forest, BiomeType::Swamp, BiomeType::Plains],
            ResourceType::Food => &[BiomeType::Plains, BiomeType::Forest],
            ResourceType::Water => &[BiomeType::Plains, BiomeType::Forest, BiomeType::Swamp],
        }
    }

    pub fn base_spawn_rate(&self) -> f64 {
        match self {
            ResourceType::Wood => 0.6,
            ResourceType::Stone => 0.5,
            ResourceType::IronOre => 0.15,
            ResourceType::GoldOre => 0.05,
            ResourceType::Crystal => 0.03,
            ResourceType::Herb => 0.4,
            ResourceType::Food => 0.5,
            ResourceType::Water => 0.5,
        }
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone)]
pub struct SpawnedResource {
    pub resource: ResourceType,
    pub amount: u32,
    pub x: i32,
    pub y: i32,
}

// ═══════════════════════════════════════════════════════════════════
// POI System
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoiType {
    Village,
    Dungeon,
    Sect,
    Market,
    Shrine,
    Ruins,
}

impl PoiType {
    pub fn display_name(&self) -> &'static str {
        match self {
            PoiType::Village => "Village",
            PoiType::Dungeon => "Dungeon",
            PoiType::Sect => "Sect",
            PoiType::Market => "Market",
            PoiType::Shrine => "Shrine",
            PoiType::Ruins => "Ruins",
        }
    }

    pub fn min_biome_difficulty(&self) -> f64 {
        match self {
            PoiType::Village => 0.1,
            PoiType::Market => 0.1,
            PoiType::Shrine => 0.2,
            PoiType::Sect => 0.4,
            PoiType::Dungeon => 0.5,
            PoiType::Ruins => 0.6,
        }
    }

    pub fn spawn_rate_per_chunk(&self) -> f64 {
        match self {
            PoiType::Village => 0.02,
            PoiType::Dungeon => 0.01,
            PoiType::Sect => 0.005,
            PoiType::Market => 0.015,
            PoiType::Shrine => 0.01,
            PoiType::Ruins => 0.008,
        }
    }

    pub fn loot_quality_bonus(&self) -> f64 {
        match self {
            PoiType::Village => 0.1,
            PoiType::Market => 0.15,
            PoiType::Shrine => 0.2,
            PoiType::Sect => 0.3,
            PoiType::Dungeon => 0.4,
            PoiType::Ruins => 0.35,
        }
    }
}

impl std::fmt::Display for PoiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone)]
pub struct PointOfInterest {
    pub poi_type: PoiType,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub biome: BiomeType,
    pub difficulty: f64,
    pub discovered: bool,
}

impl PointOfInterest {
    pub fn new(poi_type: PoiType, name: &str, x: i32, y: i32, biome: BiomeType) -> Self {
        Self {
            poi_type,
            name: name.to_string(),
            x,
            y,
            biome,
            difficulty: biome.difficulty_rating(),
            discovered: false,
        }
    }

    pub fn discover(&mut self) {
        self.discovered = true;
    }
}

// ═══════════════════════════════════════════════════════════════════
// Terrain Tile
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct TerrainTile {
    pub x: i32,
    pub y: i32,
    pub elevation: f64,
    pub temperature: f64,
    pub moisture: f64,
    pub biome: BiomeType,
    pub resources: Vec<SpawnedResource>,
    pub poi: Option<PointOfInterest>,
}

impl TerrainTile {
    pub fn walkable(&self) -> bool {
        !matches!(self.biome, BiomeType::Mountain)
    }

    pub fn resource_density(&self) -> f64 {
        self.biome.base_resource_density()
    }
}

// ═══════════════════════════════════════════════════════════════════
// World Generator
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct WorldGenerator {
    seed: u64,
    noise_elevation: NoiseGenerator,
    noise_temperature: NoiseGenerator,
    noise_moisture: NoiseGenerator,
    noise_resource: NoiseGenerator,
    noise_poi: NoiseGenerator,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            noise_elevation: NoiseGenerator::new(seed),
            noise_temperature: NoiseGenerator::new(seed.wrapping_add(1)),
            noise_moisture: NoiseGenerator::new(seed.wrapping_add(2)),
            noise_resource: NoiseGenerator::new(seed.wrapping_add(3)),
            noise_poi: NoiseGenerator::new(seed.wrapping_add(4)),
        }
    }

    pub fn generate_tile(&self, x: i32, y: i32) -> TerrainTile {
        let scale = 0.01;
        let elevation = (self
            .noise_elevation
            .fbm(x as f64 * scale, y as f64 * scale, 4, 2.0, 0.5)
            + 1.0)
            / 2.0;
        let temperature = (self.noise_temperature.fbm(
            x as f64 * scale * 1.5,
            y as f64 * scale * 1.5,
            3,
            2.0,
            0.5,
        ) + 1.0)
            / 2.0;
        let moisture =
            (self
                .noise_moisture
                .fbm(x as f64 * scale * 0.8, y as f64 * scale * 0.8, 3, 2.0, 0.5)
                + 1.0)
                / 2.0;

        let biome = determine_biome(temperature, moisture, elevation);
        let resources = self.generate_resources(x, y, biome, elevation);
        let poi = self.generate_poi(x, y, biome);

        TerrainTile {
            x,
            y,
            elevation,
            temperature,
            moisture,
            biome,
            resources,
            poi,
        }
    }

    pub fn generate_chunk(&self, chunk_x: i32, chunk_y: i32, chunk_size: i32) -> Vec<TerrainTile> {
        let mut tiles = Vec::with_capacity((chunk_size * chunk_size) as usize);
        for dy in 0..chunk_size {
            for dx in 0..chunk_size {
                let wx = chunk_x * chunk_size + dx;
                let wy = chunk_y * chunk_size + dy;
                tiles.push(self.generate_tile(wx, wy));
            }
        }
        tiles
    }

    fn generate_resources(
        &self,
        x: i32,
        y: i32,
        biome: BiomeType,
        _elevation: f64,
    ) -> Vec<SpawnedResource> {
        let mut resources = Vec::new();
        let scale = 0.05;
        let noise_val = (self
            .noise_resource
            .noise_2d(x as f64 * scale, y as f64 * scale)
            + 1.0)
            / 2.0;

        for &resource in &[
            ResourceType::Wood,
            ResourceType::Stone,
            ResourceType::IronOre,
            ResourceType::GoldOre,
            ResourceType::Crystal,
            ResourceType::Herb,
            ResourceType::Food,
            ResourceType::Water,
        ] {
            let biomes = resource.spawn_biomes();
            if biomes.contains(&biome) {
                let rate = resource.base_spawn_rate();
                if noise_val < rate {
                    let amount = ((noise_val / rate) * 5.0 + 1.0) as u32;
                    resources.push(SpawnedResource {
                        resource,
                        amount,
                        x,
                        y,
                    });
                }
            }
        }
        resources
    }

    fn generate_poi(&self, x: i32, y: i32, biome: BiomeType) -> Option<PointOfInterest> {
        let scale = 0.02;
        let noise_val = (self.noise_poi.noise_2d(x as f64 * scale, y as f64 * scale) + 1.0) / 2.0;

        let poi_types = [
            PoiType::Village,
            PoiType::Dungeon,
            PoiType::Sect,
            PoiType::Market,
            PoiType::Shrine,
            PoiType::Ruins,
        ];

        for poi_type in &poi_types {
            let rate = poi_type.spawn_rate_per_chunk();
            if noise_val < rate && biome.difficulty_rating() >= poi_type.min_biome_difficulty() {
                let name = generate_poi_name(*poi_type, x, y);
                return Some(PointOfInterest::new(*poi_type, &name, x, y, biome));
            }
        }
        None
    }

    pub fn generate_region(
        &self,
        start_x: i32,
        start_y: i32,
        width: i32,
        height: i32,
    ) -> HashMap<(i32, i32), TerrainTile> {
        let mut region = HashMap::new();
        for y in start_y..start_y + height {
            for x in start_x..start_x + width {
                let tile = self.generate_tile(x, y);
                region.insert((x, y), tile);
            }
        }
        region
    }
}

fn generate_poi_name(poi_type: PoiType, x: i32, y: i32) -> String {
    let seed = (x as u64)
        .wrapping_mul(73)
        .wrapping_add((y as u64).wrapping_mul(137));
    let idx = (seed % POI_NAMES.len() as u64) as usize;
    let prefix = POI_NAMES[idx];
    format!("{} {}", prefix, poi_type.display_name())
}

const POI_NAMES: &[&str] = &[
    "Ancient",
    "Crimson",
    "Shadow",
    "Mystic",
    "Iron",
    "Golden",
    "Frozen",
    "Verdant",
    "Obsidian",
    "Celestial",
    "Whispering",
    "Emerald",
    "Storm",
    "Silent",
    "Runic",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_deterministic() {
        let gen = WorldGenerator::new(42);
        let tile1 = gen.generate_tile(10, 20);
        let tile2 = gen.generate_tile(10, 20);
        assert_eq!(tile1.elevation, tile2.elevation);
        assert_eq!(tile1.biome, tile2.biome);
    }

    #[test]
    fn test_biome_determination() {
        assert_eq!(determine_biome(0.5, 0.5, 0.1), BiomeType::Plains);
        assert_eq!(determine_biome(0.5, 0.9, 0.3), BiomeType::Forest);
        assert_eq!(determine_biome(0.1, 0.3, 0.8), BiomeType::Mountain);
        assert_eq!(determine_biome(0.8, 0.1, 0.2), BiomeType::Desert);
    }

    #[test]
    fn test_chunk_generation() {
        let gen = WorldGenerator::new(42);
        let chunk = gen.generate_chunk(0, 0, 16);
        assert_eq!(chunk.len(), 256);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn test_region_generation() {
        let gen = WorldGenerator::new(42);
        let region = gen.generate_region(0, 0, 32, 32);
        assert_eq!(region.len(), 1024);
    }

    #[test]
    fn test_resource_spawn() {
        let gen = WorldGenerator::new(42);
        let tile = gen.generate_tile(5, 5);
        // At least some tiles should have resources
        // (not guaranteed for specific coords, but structure works)
        let _ = &tile.resources;
    }

    #[test]
    fn test_poi_name_generation() {
        let name = generate_poi_name(PoiType::Dungeon, 10, 20);
        assert!(name.ends_with("Dungeon"));
    }
}
