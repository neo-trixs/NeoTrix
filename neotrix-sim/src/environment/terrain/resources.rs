// Resources - Resource nodes distributed across the terrain
// Resources are affected by biome, season, and agent harvesting

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    // Organic
    Wood,
    Berries,
    Mushrooms,
    Herbs,
    Grass,
    Seeds,

    // Mineral
    Stone,
    Ore,
    Gems,
    Sand,
    Clay,

    // Food
    Fish,
    Meat,

    // Special
    Water,
    Energy,
    Knowledge,
}

impl ResourceType {
    pub fn nutrition_value(&self) -> f32 {
        match self {
            ResourceType::Berries | ResourceType::Fish | ResourceType::Meat => 30.0,
            ResourceType::Mushrooms | ResourceType::Herbs => 15.0,
            ResourceType::Grass | ResourceType::Seeds => 10.0,
            ResourceType::Water => 20.0,
            _ => 0.0,
        }
    }

    pub fn energy_value(&self) -> f32 {
        match self {
            ResourceType::Energy => 50.0,
            ResourceType::Wood => 20.0,
            ResourceType::Ore => 15.0,
            _ => 5.0,
        }
    }
}

/// A resource node in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: String,
    pub resource_type: ResourceType,
    pub position: (f32, f32),
    pub amount: f32,
    pub max_amount: f32,
    pub regeneration_rate: f32,
    pub depleted: bool,
}

impl ResourceNode {
    pub fn harvest(&mut self, amount: f32) -> f32 {
        let harvested = amount.min(self.amount);
        self.amount -= harvested;
        if self.amount <= 0.0 {
            self.amount = 0.0;
            self.depleted = true;
        }
        harvested
    }

    pub fn regenerate(&mut self, season_modifier: f32) {
        if self.depleted {
            // Slower regeneration when depleted
            self.amount += self.regeneration_rate * season_modifier * 0.1;
        } else {
            self.amount = (self.amount + self.regeneration_rate * season_modifier).min(self.max_amount);
        }
        if self.amount > 0.0 {
            self.depleted = false;
        }
    }
}

/// Resource distribution across the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDistribution {
    pub nodes: Vec<ResourceNode>,
    pub density_by_biome: std::collections::HashMap<String, f32>,
}

impl ResourceDistribution {
    pub fn generate(seed: u64, width: f32, height: f32, biome_map: &super::biome::BiomeMap) -> Self {
        let mut nodes = Vec::new();
        let mut state = seed;

        for y in 0..biome_map.height() {
            for x in 0..biome_map.width() {
                let biome = biome_map.data()[y][x];
                let resources = biome.resource_types();

                // Pseudo-random: skip most cells
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                if (state % 10) > 2 { continue; } // 30% chance of resource

                if let Some(resource_name) = resources.get((state as usize) % resources.len()) {
                    let resource_type = match *resource_name {
                        "wood" => ResourceType::Wood,
                        "berries" => ResourceType::Berries,
                        "mushrooms" => ResourceType::Mushrooms,
                        "herbs" => ResourceType::Herbs,
                        "grass" => ResourceType::Grass,
                        "seeds" => ResourceType::Seeds,
                        "stone" => ResourceType::Stone,
                        "ore" => ResourceType::Ore,
                        "gems" => ResourceType::Gems,
                        "sand" => ResourceType::Sand,
                        "clay" => ResourceType::Clay,
                        "fish" => ResourceType::Fish,
                        "reed" => ResourceType::Grass,
                        "mud" => ResourceType::Clay,
                        "fungus" => ResourceType::Mushrooms,
                        "ice" => ResourceType::Water,
                        "moss" => ResourceType::Herbs,
                        "driftwood" => ResourceType::Wood,
                        "shells" => ResourceType::Stone,
                        "obsidian" => ResourceType::Stone,
                        "sulfur" => ResourceType::Energy,
                        "rare_minerals" => ResourceType::Gems,
                        "seaweed" => ResourceType::Herbs,
                        _ => ResourceType::Stone,
                    };

                    let wx = (x as f32 / biome_map.width() as f32) * width;
                    let wy = (y as f32 / biome_map.height() as f32) * height;
                    let amount = 50.0 + (state % 100) as f32;

                    nodes.push(ResourceNode {
                        id: format!("res_{x}_{y}"),
                        resource_type,
                        position: (wx, wy),
                        amount,
                        max_amount: amount,
                        regeneration_rate: 0.5,
                        depleted: false,
                    });
                }
            }
        }

        Self {
            nodes,
            density_by_biome: std::collections::HashMap::new(),
        }
    }

    /// Find nearest resource of given type within radius
    pub fn nearest(&self, pos: (f32, f32), resource_type: ResourceType, radius: f32) -> Option<&ResourceNode> {
        self.nodes.iter()
            .filter(|n| n.resource_type == resource_type && !n.depleted)
            .filter(|n| {
                let dx = n.position.0 - pos.0;
                let dy = n.position.1 - pos.1;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .min_by(|a, b| {
                let da = ((a.position.0 - pos.0).powi(2) + (a.position.1 - pos.1).powi(2)).sqrt();
                let db = ((b.position.0 - pos.0).powi(2) + (b.position.1 - pos.1).powi(2)).sqrt();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Regenerate all resources (called each tick)
    pub fn regenerate_all(&mut self, season_modifier: f32) {
        for node in &mut self.nodes {
            node.regenerate(season_modifier);
        }
    }

    pub fn total_nodes(&self) -> usize { self.nodes.len() }
    pub fn depleted_nodes(&self) -> usize { self.nodes.iter().filter(|n| n.depleted).count() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_type_values() {
        assert!(ResourceType::Berries.nutrition_value() > 0.0);
        assert!(ResourceType::Energy.energy_value() > 0.0);
        assert_eq!(ResourceType::Stone.nutrition_value(), 0.0);
    }

    #[test]
    fn harvest_and_deplete() {
        let mut node = ResourceNode {
            id: "t".into(),
            resource_type: ResourceType::Wood,
            position: (0.0, 0.0),
            amount: 10.0,
            max_amount: 10.0,
            regeneration_rate: 1.0,
            depleted: false,
        };
        let h = node.harvest(15.0);
        assert_eq!(h, 10.0);
        assert!(node.depleted);
    }

    #[test]
    fn regeneration_restores() {
        let mut node = ResourceNode {
            id: "t".into(),
            resource_type: ResourceType::Ore,
            position: (0.0, 0.0),
            amount: 0.0,
            max_amount: 100.0,
            regeneration_rate: 5.0,
            depleted: true,
        };
        node.regenerate(1.0);
        assert!(!node.depleted);
        assert!(node.amount > 0.0);
    }

    #[test]
    fn resource_distribution_generate() {
        let hm = super::super::heightmap::Heightmap::generate(
            42,
            super::super::heightmap::HeightmapConfig::default(),
        );
        let temp = vec![vec![0.5f32; 128]; 128];
        let moist = vec![vec![0.5f32; 128]; 128];
        let bm = super::super::biome::BiomeMap::generate(&hm, &temp, &moist);
        let dist = ResourceDistribution::generate(99, 25.6, 25.6, &bm);
        assert!(dist.total_nodes() > 0);
    }
}
