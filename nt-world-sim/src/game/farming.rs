use crate::core::Component;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropStage {
    Seed,
    Sprout,
    Budding,
    Blooming,
    Fruitful,
    Transcendent,
}

impl CropStage {
    pub fn name(&self) -> &str {
        match self {
            CropStage::Seed => "Seed",
            CropStage::Sprout => "Sprout",
            CropStage::Budding => "Budding",
            CropStage::Blooming => "Blooming",
            CropStage::Fruitful => "Fruitful",
            CropStage::Transcendent => "Transcendent",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            CropStage::Seed => 0,
            CropStage::Sprout => 1,
            CropStage::Budding => 2,
            CropStage::Blooming => 3,
            CropStage::Fruitful => 4,
            CropStage::Transcendent => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonTag {
    Clarity,
    Flow,
    Reflection,
    Stillness,
    Any,
}

#[derive(Debug, Clone)]
pub struct Crop {
    pub seed_id: u32,
    pub name: String,
    pub stages: Vec<CropStage>,
    pub growth_time: u32,
    pub regrowth_time: Option<u32>,
    pub seasons: Vec<SeasonTag>,
    pub base_value: u32,
}

#[derive(Debug, Clone)]
pub struct GrowingCrop {
    pub crop: Crop,
    pub stage: CropStage,
    pub growth_points: f32,
    pub day_planted: u32,
    pub day_last_watered: u32,
}

impl GrowingCrop {
    pub fn new(crop: Crop, current_day: u32) -> Self {
        Self {
            crop,
            stage: CropStage::Seed,
            growth_points: 0.0,
            day_planted: current_day,
            day_last_watered: current_day,
        }
    }

    pub fn advance_day(&mut self, awareness_level: u32, season_mod: f32) {
        if self.crop.stages.is_empty() {
            return;
        }
        let stage_idx = self.crop.stages.iter().position(|s| *s == self.stage).unwrap_or(0);
        let growth_per_day = 1.0 * (1.0 + awareness_level as f32 * 0.05) * season_mod;
        self.growth_points += growth_per_day;

        let points_per_stage = self.crop.growth_time as f32 / self.crop.stages.len() as f32;
        if self.growth_points >= points_per_stage && stage_idx < self.crop.stages.len() - 1 {
            self.stage = self.crop.stages[stage_idx + 1];
            self.growth_points = 0.0;
        }
    }

    pub fn is_ready(&self) -> bool {
        self.stage == CropStage::Fruitful || self.stage == CropStage::Transcendent
    }

    pub fn stage_progress(&self) -> f32 {
        if self.crop.stages.is_empty() {
            return 0.0;
        }
        let points_per_stage = self.crop.growth_time as f32 / self.crop.stages.len() as f32;
        self.growth_points / points_per_stage
    }
}

#[derive(Debug, Clone)]
pub struct FarmPlot {
    pub x: u32,
    pub y: u32,
    pub tilled: bool,
    pub watered: bool,
    pub crop: Option<GrowingCrop>,
    pub soil_quality: f32,
}

impl FarmPlot {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y, tilled: false, watered: false, crop: None, soil_quality: 1.0 }
    }

    pub fn till(&mut self) -> bool {
        if !self.tilled && self.crop.is_none() {
            self.tilled = true;
            true
        } else {
            false
        }
    }

    pub fn plant(&mut self, crop: Crop, current_day: u32) -> bool {
        if self.tilled && self.crop.is_none() {
            self.crop = Some(GrowingCrop::new(crop, current_day));
            true
        } else {
            false
        }
    }

    pub fn water(&mut self) {
        if self.tilled && self.crop.is_some() {
            self.watered = true;
        }
    }

    pub fn harvest(&mut self) -> Option<Crop> {
        if let Some(ref growing) = self.crop {
            if growing.is_ready() {
                let crop = growing.crop.clone();
                self.crop = None;
                self.tilled = true;
                return Some(crop);
            }
        }
        None
    }
}

pub struct CropDatabase {
    pub crops: HashMap<u32, Crop>,
}

impl CropDatabase {
    pub fn new() -> Self {
        Self { crops: HashMap::new() }
    }

    pub fn register(&mut self, crop: Crop) {
        self.crops.insert(crop.seed_id, crop);
    }

    pub fn get(&self, seed_id: u32) -> Option<&Crop> {
        self.crops.get(&seed_id)
    }

    pub fn seed_count(&self) -> usize {
        self.crops.len()
    }
}

impl Default for CropDatabase {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CropState {
    Empty,
    Tilled,
    Seeded { seed_id: u32, day_planted: u32 },
    Growing { seed_id: u32, day_planted: u32, growth: f32 },
    Ready { seed_id: u32 },
    Withered,
}

#[derive(Debug, Clone)]
pub struct CropTile {
    pub state: CropState,
    pub watered: bool,
    pub fertilized: bool,
}

impl CropTile {
    pub fn new() -> Self {
        Self { state: CropState::Empty, watered: false, fertilized: false }
    }

    pub fn till(&mut self) {
        if self.state == CropState::Empty {
            self.state = CropState::Tilled;
        }
    }

    pub fn plant(&mut self, seed_id: u32, current_day: u32) {
        if self.state == CropState::Tilled {
            self.state = CropState::Seeded { seed_id, day_planted: current_day };
        }
    }

    pub fn water(&mut self) {
        self.watered = true;
    }

    pub fn grow(&mut self, current_day: u32, growth_days: u32, season_modifier: f32) {
        if let CropState::Seeded { seed_id, day_planted } = self.state {
            let days_elapsed = current_day - day_planted;
            let growth = (days_elapsed as f32 * season_modifier) / growth_days as f32;
            if growth >= 1.0 {
                self.state = CropState::Ready { seed_id };
            } else {
                self.state = CropState::Growing { seed_id, day_planted, growth };
            }
        } else if let CropState::Growing { seed_id, day_planted, .. } = self.state {
            let days_elapsed = current_day - day_planted;
            let growth = (days_elapsed as f32 * season_modifier) / growth_days as f32;
            if growth >= 1.0 {
                self.state = CropState::Ready { seed_id };
            } else {
                self.state = CropState::Growing { seed_id, day_planted, growth };
            }
        }
        self.watered = false;
    }

    pub fn harvest(&mut self) -> Option<u32> {
        if let CropState::Ready { seed_id } = self.state {
            self.state = CropState::Tilled;
            Some(seed_id)
        } else {
            None
        }
    }
}

impl Default for CropTile {
    fn default() -> Self { Self::new() }
}

impl Component for FarmPlot {}

pub struct FarmingSystem;

impl FarmingSystem {
    pub fn update_day(tiles: &mut [CropTile], current_day: u32, growth_days: u32, season_mod: f32) {
        for tile in tiles.iter_mut() {
            if tile.watered {
                tile.grow(current_day, growth_days, season_mod);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crop_lifecycle() {
        let mut tile = CropTile::new();
        tile.till();
        assert_eq!(tile.state, CropState::Tilled);

        tile.plant(1, 1);
        assert!(matches!(tile.state, CropState::Seeded { .. }));

        tile.water();
        tile.grow(5, 4, 1.0);
        assert!(matches!(tile.state, CropState::Ready { .. }));

        let harvested = tile.harvest();
        assert_eq!(harvested, Some(1));
    }
}
