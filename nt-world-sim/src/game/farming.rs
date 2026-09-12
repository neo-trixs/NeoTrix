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
