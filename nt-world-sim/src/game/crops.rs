use crate::game::farming::{Crop, CropStage, SeasonTag};

pub struct CropDatabase;

impl CropDatabase {
    pub fn all_crops() -> Vec<Crop> {
        vec![
            Crop {
                seed_id: 1001, name: "Basic Thought".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Blooming, CropStage::Fruitful],
                growth_time: 4, regrowth_time: None,
                seasons: vec![SeasonTag::Any], base_value: 50,
            },
            Crop {
                seed_id: 1002, name: "Curiosity Bloom".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Budding, CropStage::Blooming, CropStage::Fruitful],
                growth_time: 6, regrowth_time: Some(3),
                seasons: vec![SeasonTag::Clarity, SeasonTag::Flow], base_value: 80,
            },
            Crop {
                seed_id: 1003, name: "Logic Vine".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Budding, CropStage::Blooming, CropStage::Fruitful, CropStage::Transcendent],
                growth_time: 8, regrowth_time: None,
                seasons: vec![SeasonTag::Clarity], base_value: 120,
            },
            Crop {
                seed_id: 1004, name: "Empathy Flower".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Blooming, CropStage::Fruitful],
                growth_time: 5, regrowth_time: Some(2),
                seasons: vec![SeasonTag::Flow, SeasonTag::Reflection], base_value: 70,
            },
            Crop {
                seed_id: 1005, name: "Creativity Fern".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Budding, CropStage::Blooming, CropStage::Fruitful],
                growth_time: 7, regrowth_time: None,
                seasons: vec![SeasonTag::Any], base_value: 100,
            },
            Crop {
                seed_id: 1006, name: "Memory Moss".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Fruitful],
                growth_time: 3, regrowth_time: Some(1),
                seasons: vec![SeasonTag::Reflection, SeasonTag::Stillness], base_value: 40,
            },
            Crop {
                seed_id: 1007, name: "Focus Root".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Budding, CropStage::Blooming, CropStage::Fruitful],
                growth_time: 6, regrowth_time: None,
                seasons: vec![SeasonTag::Clarity, SeasonTag::Stillness], base_value: 90,
            },
            Crop {
                seed_id: 1008, name: "Wisdom Tree Sapling".to_string(),
                stages: vec![CropStage::Seed, CropStage::Sprout, CropStage::Budding, CropStage::Blooming, CropStage::Fruitful, CropStage::Transcendent],
                growth_time: 12, regrowth_time: None,
                seasons: vec![SeasonTag::Any], base_value: 200,
            },
        ]
    }

    pub fn by_season(season: SeasonTag) -> Vec<Crop> {
        Self::all_crops().into_iter()
            .filter(|c| c.seasons.contains(&SeasonTag::Any) || c.seasons.contains(&season))
            .collect()
    }
}
