use std::collections::{HashMap, VecDeque};

pub struct AgentSnapshot {
    pub id: u32,
    pub fitness: f32,
    pub strategy_hash: u64,
}

pub struct SelfPlay {
    pub history: Vec<AgentSnapshot>,
    pub win_rates: HashMap<(u64, u64), f32>,
}

impl SelfPlay {
    pub fn new() -> Self {
        SelfPlay {
            history: Vec::new(),
            win_rates: HashMap::new(),
        }
    }

    pub fn record(&mut self, agent: AgentSnapshot) {
        self.history.push(agent);
    }

    pub fn get_opponent(&self, strategy_hash: u64) -> Option<&AgentSnapshot> {
        self.history
            .iter()
            .find(|a| a.strategy_hash != strategy_hash)
    }

    pub fn update_win_rate(&mut self, winner_hash: u64, loser_hash: u64) {
        *self.win_rates
            .entry((winner_hash, loser_hash))
            .or_insert(0.5) += 0.05;
    }

    pub fn get_matchup(&self, a: u64, b: u64) -> f32 {
        self.win_rates.get(&(a, b)).copied().unwrap_or(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty() {
        let sp = SelfPlay::new();
        assert!(sp.history.is_empty());
        assert!(sp.win_rates.is_empty());
    }

    #[test]
    fn record_adds_snapshot() {
        let mut sp = SelfPlay::new();
        sp.record(AgentSnapshot { id: 1, fitness: 0.8, strategy_hash: 100 });
        assert_eq!(sp.history.len(), 1);
    }

    #[test]
    fn get_opponent_finds_different_strategy() {
        let mut sp = SelfPlay::new();
        sp.record(AgentSnapshot { id: 1, fitness: 0.8, strategy_hash: 100 });
        sp.record(AgentSnapshot { id: 2, fitness: 0.6, strategy_hash: 200 });
        let opp = sp.get_opponent(100).unwrap();
        assert_eq!(opp.strategy_hash, 200);
    }

    #[test]
    fn update_win_rate_increments() {
        let mut sp = SelfPlay::new();
        sp.update_win_rate(100, 200);
        assert!((sp.get_matchup(100, 200) - 0.55).abs() < 0.001);
    }

    #[test]
    fn get_matchup_defaults_to_05() {
        let sp = SelfPlay::new();
        assert_eq!(sp.get_matchup(1, 2), 0.5);
    }
}
