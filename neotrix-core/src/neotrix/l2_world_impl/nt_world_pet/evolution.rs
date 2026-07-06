use crate::neotrix::nt_world_pet::state::PetState;

pub struct GrowthTable;

impl GrowthTable {
    pub fn nodes_for_level(level: u32) -> u64 {
        match level {
            0 => 500,
            1 => 2000,
            2 => 10000,
            3 => 50000,
            _ => 100000 + (level as u64 - 3) * 100000,
        }
    }

    pub fn conversations_for_level(level: u32) -> u64 {
        match level {
            0 => 10,
            1 => 50,
            2 => 200,
            3 => 1000,
            _ => 1000 + (level as u64 - 3) * 2000,
        }
    }

    pub fn check_level_up(state: &mut PetState, total_kb_nodes: u64) -> Vec<LevelUpReward> {
        let mut rewards = Vec::new();
        let current_nodes = total_kb_nodes.saturating_sub(state.kb_nodes_at_birth);

        loop {
            let nodes_needed = Self::nodes_for_level(state.level);
            let convs_needed = Self::conversations_for_level(state.level);

            if current_nodes >= nodes_needed && state.conversations_had >= convs_needed {
                state.level += 1;
                let reward = LevelUpReward {
                    new_level: state.level,
                    size_bonus: 0.05,
                    complexity_bonus: 0.1,
                };
                Self::apply_reward(state, &reward);
                rewards.push(reward);
            } else {
                break;
            }
        }

        rewards
    }

    fn apply_reward(state: &mut PetState, reward: &LevelUpReward) {
        state.target_visual.size = (state.target_visual.size + reward.size_bonus).min(1.0);
        state.target_visual.complexity = (state.target_visual.complexity + reward.complexity_bonus).min(1.0);
    }
}

#[derive(Debug, Clone)]
pub struct LevelUpReward {
    pub new_level: u32,
    pub size_bonus: f64,
    pub complexity_bonus: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_0_to_1() {
        let mut state = PetState {
            conversations_had: 10,
            kb_nodes_at_birth: 0,
            ..Default::default()
        };
        let rewards = GrowthTable::check_level_up(&mut state, 500);
        assert_eq!(state.level, 1);
        assert_eq!(rewards.len(), 1);
    }

    #[test]
    fn test_not_enough_convs() {
        let mut state = PetState {
            conversations_had: 5,
            kb_nodes_at_birth: 0,
            ..Default::default()
        };
        let rewards = GrowthTable::check_level_up(&mut state, 500);
        assert_eq!(state.level, 0);
        assert!(rewards.is_empty());
    }

    #[test]
    fn test_double_level_up() {
        let mut state = PetState {
            conversations_had: 60,
            kb_nodes_at_birth: 0,
            ..Default::default()
        };
        let rewards = GrowthTable::check_level_up(&mut state, 5000);
        assert_eq!(state.level, 2);
        assert_eq!(rewards.len(), 2);
    }
}
