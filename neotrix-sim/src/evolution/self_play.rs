use crate::agents::SimAgent;
use crate::evolution::FitnessLandscape;

#[derive(Debug, Clone)]
pub struct HistoricalAgent {
    pub agent: SimAgent,
    pub tick: u64,
    pub wins: u32,
    pub losses: u32,
}

pub struct SelfPlaySystem {
    pub history: Vec<HistoricalAgent>,
    pub max_history: usize,
    pub matchups: Vec<(usize, usize, bool)>,
}

impl SelfPlaySystem {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            max_history: 100,
            matchups: Vec::new(),
        }
    }

    pub fn record_agent(&mut self, agent: &SimAgent, tick: u64) {
        let hist = HistoricalAgent {
            agent: agent.clone(),
            tick,
            wins: 0,
            losses: 0,
        };
        self.history.push(hist);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn select_opponent(&self, current: &SimAgent) -> Option<&HistoricalAgent> {
        if self.history.is_empty() {
            return None;
        }
        let current_fitness = current.fitness() as f32;
        self.history.iter()
            .min_by_key(|h| {
                let diff = (h.agent.fitness() as f32 - current_fitness).abs();
                (diff * 1000.0) as u64
            })
    }

    pub fn record_matchup(&mut self, idx_a: usize, idx_b: usize, a_won: bool) {
        self.matchups.push((idx_a, idx_b, a_won));
        if a_won {
            if let Some(a) = self.history.get_mut(idx_a) {
                a.wins += 1;
            }
            if let Some(b) = self.history.get_mut(idx_b) {
                b.losses += 1;
            }
        } else {
            if let Some(a) = self.history.get_mut(idx_a) {
                a.losses += 1;
            }
            if let Some(b) = self.history.get_mut(idx_b) {
                b.wins += 1;
            }
        }
    }

    pub fn get_elite(&self) -> Option<&HistoricalAgent> {
        self.history.iter()
            .filter(|h| h.wins + h.losses > 0)
            .max_by(|a, b| {
                let ra = a.wins as f32 / (a.wins + a.losses) as f32;
                let rb = b.wins as f32 / (b.wins + b.losses) as f32;
                ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn test_self_play_record() {
        let mut system = SelfPlaySystem::new();
        let agent = SimAgent::new(0, Vec2::zero());
        system.record_agent(&agent, 0);
        assert_eq!(system.history.len(), 1);
    }

    #[test]
    fn test_self_play_opponent() {
        let mut system = SelfPlaySystem::new();
        let a1 = SimAgent::new(0, Vec2::zero());
        let a2 = SimAgent::new(1, Vec2::new(10.0, 10.0));
        system.record_agent(&a1, 0);
        system.record_agent(&a2, 1);
        
        let current = SimAgent::new(2, Vec2::new(5.0, 5.0));
        let opp = system.select_opponent(&current);
        assert!(opp.is_some());
    }

    #[test]
    fn test_self_play_matchup() {
        let mut system = SelfPlaySystem::new();
        let a1 = SimAgent::new(0, Vec2::zero());
        let a2 = SimAgent::new(1, Vec2::new(10.0, 10.0));
        system.record_agent(&a1, 0);
        system.record_agent(&a2, 1);
        
        system.record_matchup(0, 1, true);
        assert_eq!(system.history[0].wins, 1);
        assert_eq!(system.history[1].losses, 1);
    }

    #[test]
    fn test_elite_selection() {
        let mut system = SelfPlaySystem::new();
        let a1 = SimAgent::new(0, Vec2::zero());
        let a2 = SimAgent::new(1, Vec2::new(10.0, 10.0));
        system.record_agent(&a1, 0);
        system.record_agent(&a2, 1);

        system.record_matchup(0, 1, true);
        system.record_matchup(0, 1, true);

        let elite = system.get_elite();
        assert!(elite.is_some());
        assert_eq!(elite.unwrap().wins, 2);
    }

    #[test]
    fn test_history_max_size() {
        let mut system = SelfPlaySystem::new();
        system.max_history = 3;
        for i in 0..5 {
            let agent = SimAgent::new(i, Vec2::zero());
            system.record_agent(&agent, i);
        }
        assert_eq!(system.history.len(), 3);
    }
}
