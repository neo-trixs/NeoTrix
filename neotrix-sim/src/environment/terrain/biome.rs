// Biome - Terrain classification system
// Maps heightmap + climate data to biome types
// Each biome affects: resource types, agent movement cost, visibility, danger

use serde::{Serialize, Deserialize};
use super::heightmap::Heightmap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    DeepWater,
    ShallowWater,
    Beach,
    Plain,
    Forest,
    DenseForest,
    Swamp,
    Desert,
    Savanna,
    Tundra,
    Snow,
    Mountain,
    Volcanic,
}

impl BiomeType {
    /// Movement cost multiplier (1.0 = normal, 2.0 = twice as slow)
    pub fn movement_cost(&self) -> f32 {
        match self {
            BiomeType::DeepWater => 3.0,
            BiomeType::ShallowWater => 2.0,
            BiomeType::Beach => 1.2,
            BiomeType::Plain => 1.0,
            BiomeType::Forest => 1.3,
            BiomeType::DenseForest => 1.8,
            BiomeType::Swamp => 2.5,
            BiomeType::Desert => 1.5,
            BiomeType::Savanna => 1.1,
            BiomeType::Tundra => 1.6,
            BiomeType::Snow => 2.0,
            BiomeType::Mountain => 2.5,
            BiomeType::Volcanic => 3.0,
        }
    }

    /// Danger level (0.0 safe, 1.0 lethal)
    pub fn danger_level(&self) -> f32 {
        match self {
            BiomeType::DeepWater => 0.8,
            BiomeType::Volcanic => 0.9,
            BiomeType::Mountain => 0.5,
            BiomeType::Swamp => 0.3,
            BiomeType::Desert => 0.4,
            BiomeType::DenseForest => 0.2,
            _ => 0.1,
        }
    }

    /// Visibility range multiplier
    pub fn visibility_modifier(&self) -> f32 {
        match self {
            BiomeType::DenseForest => 0.5,
            BiomeType::Forest => 0.7,
            BiomeType::Swamp => 0.6,
            BiomeType::Mountain => 1.5,
            BiomeType::Plain | BiomeType::Savanna => 1.2,
            BiomeType::DeepWater => 0.3,
            _ => 1.0,
        }
    }

    /// Resource types available
    pub fn resource_types(&self) -> Vec<&'static str> {
        match self {
            BiomeType::Forest | BiomeType::DenseForest => vec!["wood", "berries", "mushrooms", "herbs"],
            BiomeType::Plain | BiomeType::Savanna => vec!["grass", "seeds", "clay"],
            BiomeType::Desert => vec!["sand", "cactus", "minerals"],
            BiomeType::Mountain => vec!["stone", "ore", "gems"],
            BiomeType::Swamp => vec!["reed", "mud", "fungus"],
            BiomeType::DeepWater | BiomeType::ShallowWater => vec!["fish", "seaweed", "shells"],
            BiomeType::Snow | BiomeType::Tundra => vec!["ice", "moss"],
            BiomeType::Beach => vec!["sand", "driftwood", "shells"],
            BiomeType::Volcanic => vec!["obsidian", "sulfur", "rare_minerals"],
        }
    }
}

/// Biome map - classification of all terrain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMap {
    width: usize,
    height: usize,
    data: Vec<Vec<BiomeType>>,
}

impl BiomeMap {
    /// Generate biome map from heightmap + temperature + moisture
    pub fn generate(heightmap: &Heightmap, temperature: &[Vec<f32>], moisture: &[Vec<f32>]) -> Self {
        let w = heightmap.config().width;
        let h = heightmap.config().height;
        let mut data = vec![vec![BiomeType::Plain; w]; h];

        for y in 0..h {
            for x in 0..w {
                let height = heightmap.data()[y][x];
                let temp = temperature.get(y).and_then(|row| row.get(x)).copied().unwrap_or(0.5);
                let moist = moisture.get(y).and_then(|row| row.get(x)).copied().unwrap_or(0.5);

                data[y][x] = Self::classify(height, temp, moist);
            }
        }

        Self { width: w, height: h, data }
    }

    fn classify(height: f32, temperature: f32, moisture: f32) -> BiomeType {
        // Height-based classification first
        if height < -0.3 { return BiomeType::DeepWater; }
        if height < -0.1 { return BiomeType::ShallowWater; }
        if height < 0.0 { return BiomeType::Beach; }
        if height > 0.8 { return BiomeType::Volcanic; }
        if height > 0.7 { return BiomeType::Mountain; }

        // Temperature + moisture based
        if temperature < 0.2 {
            if moisture > 0.5 { return BiomeType::Snow; }
            return BiomeType::Tundra;
        }

        if temperature > 0.7 {
            if moisture < 0.3 { return BiomeType::Desert; }
            if moisture < 0.5 { return BiomeType::Savanna; }
        }

        if moisture > 0.7 { return BiomeType::Swamp; }
        if moisture > 0.5 { return BiomeType::DenseForest; }
        if moisture > 0.3 { return BiomeType::Forest; }

        BiomeType::Plain
    }

    /// Get biome at world position
    pub fn biome_at(&self, wx: f32, wy: f32, world_width: f32, world_height: f32) -> BiomeType {
        let x = ((wx / world_width) * self.width as f32) as usize;
        let y = ((wy / world_height) * self.height as f32) as usize;
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        self.data[y][x]
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn data(&self) -> &[Vec<BiomeType>] { &self.data }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_water() {
        assert_eq!(BiomeMap::classify(-0.5, 0.5, 0.5), BiomeType::DeepWater);
        assert_eq!(BiomeMap::classify(-0.2, 0.5, 0.5), BiomeType::ShallowWater);
        assert_eq!(BiomeMap::classify(-0.05, 0.5, 0.5), BiomeType::Beach);
    }

    #[test]
    fn classify_mountains() {
        assert_eq!(BiomeMap::classify(0.75, 0.5, 0.5), BiomeType::Mountain);
        assert_eq!(BiomeMap::classify(0.85, 0.5, 0.5), BiomeType::Volcanic);
    }

    #[test]
    fn classify_temperature_extremes() {
        assert_eq!(BiomeMap::classify(0.5, 0.1, 0.8), BiomeType::Snow);
        assert_eq!(BiomeMap::classify(0.5, 0.1, 0.3), BiomeType::Tundra);
        assert_eq!(BiomeMap::classify(0.5, 0.8, 0.2), BiomeType::Desert);
        assert_eq!(BiomeMap::classify(0.5, 0.8, 0.4), BiomeType::Savanna);
    }

    #[test]
    fn classify_moisture_gradient() {
        assert_eq!(BiomeMap::classify(0.5, 0.5, 0.8), BiomeType::Swamp);
        assert_eq!(BiomeMap::classify(0.5, 0.5, 0.6), BiomeType::DenseForest);
        assert_eq!(BiomeMap::classify(0.5, 0.5, 0.4), BiomeType::Forest);
        assert_eq!(BiomeMap::classify(0.5, 0.5, 0.2), BiomeType::Plain);
    }

    #[test]
    fn biome_properties() {
        assert!(BiomeType::DeepWater.movement_cost() > 1.0);
        assert!(BiomeType::Volcanic.danger_level() > 0.5);
        assert!(BiomeType::Plain.visibility_modifier() >= 1.0);
        assert!(!BiomeType::Forest.resource_types().is_empty());
    }

    #[test]
    fn biome_map_lookup() {
        let hm = super::heightmap::Heightmap::generate(
            42,
            super::heightmap::HeightmapConfig::default(),
        );
        let temp = vec![vec![0.5f32; 128]; 128];
        let moist = vec![vec![0.5f32; 128]; 128];
        let bm = BiomeMap::generate(&hm, &temp, &moist);
        let biome = bm.biome_at(50.0, 50.0, 128.0 * 0.02, 128.0 * 0.02);
        // Should be one of the valid types
        let _ = biome;
    }
}
