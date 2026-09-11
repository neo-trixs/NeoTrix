use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Pheromone types — each encodes a different stigmergic signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PheromoneType {
    /// Food found at this location
    Food,
    /// Danger or threat detected
    Danger,
    /// Good resting spot
    Rest,
    /// Social interaction occurred
    Social,
    /// Trail left by exploration
    Explore,
    /// Territorial marker
    Territory,
}

impl PheromoneType {
    /// Default decay rate per tick — higher = faster fade.
    pub fn base_decay_rate(&self) -> f32 {
        match self {
            PheromoneType::Food => 0.003,
            PheromoneType::Danger => 0.005,
            PheromoneType::Rest => 0.002,
            PheromoneType::Social => 0.008,
            PheromoneType::Explore => 0.01,
            PheromoneType::Territory => 0.001,
        }
    }

    /// Base deposit strength for this pheromone type.
    pub fn base_strength(&self) -> f32 {
        match self {
            PheromoneType::Food => 1.0,
            PheromoneType::Danger => 1.5,
            PheromoneType::Rest => 0.6,
            PheromoneType::Social => 0.4,
            PheromoneType::Explore => 0.3,
            PheromoneType::Territory => 0.8,
        }
    }

    /// How strongly this pheromone type influences movement direction.
    /// 1.0 = full attraction, -1.0 = full repulsion.
    pub fn influence(&self) -> f32 {
        match self {
            PheromoneType::Food => 0.8,     // attract
            PheromoneType::Danger => -1.0,   // repel
            PheromoneType::Rest => 0.5,      // attract (when tired)
            PheromoneType::Social => 0.6,    // attract
            PheromoneType::Explore => 0.2,   // mild attract
            PheromoneType::Territory => 0.3, // mild attract
        }
    }
}

/// A single pheromone marker deposited in the shared environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pheromone {
    pub ptype: PheromoneType,
    pub position: [f32; 2],
    pub strength: f32,
    pub decay_rate: f32,
    pub deposited_by: String,
    pub deposited_tick: u64,
}

impl Pheromone {
    pub fn new(
        ptype: PheromoneType,
        position: [f32; 2],
        deposited_by: &str,
        tick: u64,
    ) -> Self {
        Self {
            ptype,
            position,
            strength: ptype.base_strength(),
            decay_rate: ptype.base_decay_rate(),
            deposited_by: deposited_by.to_string(),
            deposited_tick: tick,
        }
    }

    /// Current effective strength after decay.
    pub fn effective_strength(&self, current_tick: u64) -> f32 {
        let age = current_tick.saturating_sub(self.deposited_tick) as f32;
        (self.strength - self.decay_rate * age).max(0.0)
    }

    pub fn is_alive(&self, current_tick: u64) -> bool {
        self.effective_strength(current_tick) > 0.01
    }
}

/// Aggregated pheromone signal at a point — used for decision-making.
#[derive(Debug, Clone, Default)]
pub struct PheromoneSignal {
    pub food_attract: f32,
    pub danger_repel: f32,
    pub rest_attract: f32,
    pub social_attract: f32,
    pub explore_attract: f32,
    pub territory_attract: f32,
    pub total_strength: f32,
}

impl PheromoneSignal {
    /// Net directional influence: positive = approach, negative = avoid.
    pub fn net_valence(&self) -> f32 {
        self.food_attract * 0.3
            + self.danger_repel * -0.4
            + self.rest_attract * 0.15
            + self.social_attract * 0.1
            + self.explore_attract * 0.05
    }

    /// Dominant signal type for behavioral override.
    pub fn dominant_type(&self) -> Option<PheromoneType> {
        let signals = [
            (PheromoneType::Food, self.food_attract),
            (PheromoneType::Danger, self.danger_repel),
            (PheromoneType::Rest, self.rest_attract),
            (PheromoneType::Social, self.social_attract),
            (PheromoneType::Explore, self.explore_attract),
            (PheromoneType::Territory, self.territory_attract),
        ];
        signals.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .filter(|(_, s)| *s > 0.1)
            .map(|(t, _)| *t)
    }
}

/// The shared pheromone field — the stigmergic coordination substrate.
/// All agents deposit into and sense from this single field.
pub struct PheromoneField {
    /// All active pheromones.
    pub pheromones: Vec<Pheromone>,
    /// Spatial index: grid cell → pheromone indices.
    grid_index: HashMap<(i32, i32), Vec<usize>>,
    /// Cell size for spatial hashing.
    cell_size: f32,
    /// Maximum pheromones before pruning.
    max_pheromones: usize,
    /// Deposition log for EventBus emission.
    pub pending_deposits: Vec<Pheromone>,
}

impl PheromoneField {
    pub fn new(cell_size: f32, max_pheromones: usize) -> Self {
        Self {
            pheromones: Vec::new(),
            grid_index: HashMap::new(),
            cell_size,
            max_pheromones,
            pending_deposits: Vec::new(),
        }
    }

    /// Cell key for spatial hashing.
    fn cell_key(&self, pos: [f32; 2]) -> (i32, i32) {
        (
            (pos[0] / self.cell_size).floor() as i32,
            (pos[1] / self.cell_size).floor() as i32,
        )
    }

    /// Deposit a pheromone at a position.
    pub fn deposit(
        &mut self,
        ptype: PheromoneType,
        position: [f32; 2],
        agent_id: &str,
        tick: u64,
    ) {
        // Check if same type already nearby — stack strength instead of duplicating
        let merge_radius = self.cell_size * 0.5;
        let key = self.cell_key(position);

        // Search neighboring cells for same-type pheromone to merge
        let mut merged = false;
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor_key = (key.0 + dx, key.1 + dy);
                if let Some(indices) = self.grid_index.get(&neighbor_key) {
                    for &idx in indices {
                        if idx < self.pheromones.len() {
                            let p = &mut self.pheromones[idx];
                            if p.ptype == ptype {
                                let dist = ((p.position[0] - position[0]).powi(2)
                                    + (p.position[1] - position[1]).powi(2))
                                .sqrt();
                                if dist < merge_radius {
                                    // Stack: add strength, average position
                                    p.strength = (p.strength + ptype.base_strength()).min(3.0);
                                    p.position[0] = (p.position[0] + position[0]) * 0.5;
                                    p.position[1] = (p.position[1] + position[1]) * 0.5;
                                    p.deposited_tick = tick;
                                    merged = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if merged { break; }
            }
            if merged { break; }
        }

        if !merged {
            let pheromone = Pheromone::new(ptype, position, agent_id, tick);
            let idx = self.pheromones.len();
            self.pheromones.push(pheromone);
            self.grid_index.entry(key).or_default().push(idx);
            self.pending_deposits.push(self.pheromones[idx].clone());
        }
    }

    /// Sense pheromones within a radius of a position.
    pub fn sense(&self, position: [f32; 2], radius: f32, current_tick: u64) -> PheromoneSignal {
        let mut signal = PheromoneSignal::default();
        let key = self.cell_key(position);
        let cells_to_check = (radius / self.cell_size).ceil() as i32 + 1;

        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                let neighbor_key = (key.0 + dx, key.1 + dy);
                if let Some(indices) = self.grid_index.get(&neighbor_key) {
                    for &idx in indices {
                        if idx < self.pheromones.len() {
                            let p = &self.pheromones[idx];
                            let eff = p.effective_strength(current_tick);
                            if eff <= 0.0 { continue; }

                            let dist = ((p.position[0] - position[0]).powi(2)
                                + (p.position[1] - position[1]).powi(2))
                            .sqrt();
                            if dist > radius { continue; }

                            // Inverse-distance weighting
                            let weight = eff * (1.0 - dist / radius);
                            match p.ptype {
                                PheromoneType::Food => signal.food_attract += weight,
                                PheromoneType::Danger => signal.danger_repel += weight,
                                PheromoneType::Rest => signal.rest_attract += weight,
                                PheromoneType::Social => signal.social_attract += weight,
                                PheromoneType::Explore => signal.explore_attract += weight,
                                PheromoneType::Territory => signal.territory_attract += weight,
                            }
                            signal.total_strength += weight;
                        }
                    }
                }
            }
        }

        signal
    }

    /// Find the direction toward the strongest pheromone of a given type within radius.
    pub fn strongest_direction(
        &self,
        position: [f32; 2],
        ptype: PheromoneType,
        radius: f32,
        current_tick: u64,
    ) -> Option<[f32; 2]> {
        let mut best_strength = 0.0f32;
        let mut best_pos = [0.0f32; 2];
        let key = self.cell_key(position);
        let cells_to_check = (radius / self.cell_size).ceil() as i32 + 1;

        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                let neighbor_key = (key.0 + dx, key.1 + dy);
                if let Some(indices) = self.grid_index.get(&neighbor_key) {
                    for &idx in indices {
                        if idx < self.pheromones.len() {
                            let p = &self.pheromones[idx];
                            if p.ptype != ptype { continue; }
                            let eff = p.effective_strength(current_tick);
                            if eff <= 0.0 { continue; }
                            let dist = ((p.position[0] - position[0]).powi(2)
                                + (p.position[1] - position[1]).powi(2))
                            .sqrt();
                            if dist > radius { continue; }
                            if eff > best_strength {
                                best_strength = eff;
                                best_pos = p.position;
                            }
                        }
                    }
                }
            }
        }

        if best_strength > 0.01 {
            Some(best_pos)
        } else {
            None
        }
    }

    /// Decay all pheromones and remove dead ones. Call once per tick.
    pub fn decay(&mut self, current_tick: u64) {
        self.pheromones.retain(|p| p.is_alive(current_tick));
        // Rebuild grid index after pruning
        self.grid_index.clear();
        for (idx, p) in self.pheromones.iter().enumerate() {
            let key = self.cell_key(p.position);
            self.grid_index.entry(key).or_default().push(idx);
        }
    }

    /// Prune if over capacity — remove weakest pheromones.
    pub fn prune(&mut self, current_tick: u64) {
        if self.pheromones.len() <= self.max_pheromones { return; }
        self.pheromones.sort_by(|a, b| {
            b.effective_strength(current_tick)
                .partial_cmp(&a.effective_strength(current_tick))
                .unwrap()
        });
        self.pheromones.truncate(self.max_pheromones);
        // Rebuild grid index
        self.grid_index.clear();
        for (idx, p) in self.pheromones.iter().enumerate() {
            let key = self.cell_key(p.position);
            self.grid_index.entry(key).or_default().push(idx);
        }
    }

    pub fn total_pheromones(&self) -> usize {
        self.pheromones.len()
    }

    pub fn pending_deposits_drain(&mut self) -> Vec<Pheromone> {
        self.pending_deposits.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pheromone_creation() {
        let p = Pheromone::new(PheromoneType::Food, [10.0, 20.0], "agent_0", 0);
        assert_eq!(p.ptype, PheromoneType::Food);
        assert_eq!(p.position, [10.0, 20.0]);
        assert_eq!(p.deposited_by, "agent_0");
        assert!(p.strength > 0.0);
    }

    #[test]
    fn pheromone_decay() {
        let p = Pheromone::new(PheromoneType::Food, [10.0, 20.0], "agent_0", 0);
        assert!(p.is_alive(0));
        // After many ticks, should decay
        assert!(!p.is_alive(1000));
    }

    #[test]
    fn field_deposit_and_sense() {
        let mut field = PheromoneField::new(50.0, 1000);
        field.deposit(PheromoneType::Food, [100.0, 100.0], "agent_0", 0);
        assert_eq!(field.total_pheromones(), 1);

        let signal = field.sense([100.0, 100.0], 50.0, 0);
        assert!(signal.food_attract > 0.0);
    }

    #[test]
    fn field_merge_stacks_strength() {
        let mut field = PheromoneField::new(50.0, 1000);
        field.deposit(PheromoneType::Food, [100.0, 100.0], "agent_0", 0);
        field.deposit(PheromoneType::Food, [101.0, 100.0], "agent_1", 0);
        // Should merge since same type and close together
        assert_eq!(field.total_pheromones(), 1);
        let signal = field.sense([100.0, 100.0], 50.0, 0);
        assert!(signal.food_attract > 1.0); // stacked
    }

    #[test]
    fn field_different_types_dont_merge() {
        let mut field = PheromoneField::new(50.0, 1000);
        field.deposit(PheromoneType::Food, [100.0, 100.0], "agent_0", 0);
        field.deposit(PheromoneType::Danger, [100.0, 100.0], "agent_1", 0);
        assert_eq!(field.total_pheromones(), 2);
    }

    #[test]
    fn strongest_direction() {
        let mut field = PheromoneField::new(50.0, 1000);
        field.deposit(PheromoneType::Food, [200.0, 100.0], "agent_0", 0);
        let dir = field.strongest_direction([100.0, 100.0], PheromoneType::Food, 150.0, 0);
        assert!(dir.is_some());
        let dir = dir.unwrap();
        assert!(dir[0] > 100.0); // should point toward food (positive x)
    }

    #[test]
    fn decay_removes_old_pheromones() {
        let mut field = PheromoneField::new(50.0, 1000);
        field.deposit(PheromoneType::Explore, [100.0, 100.0], "agent_0", 0);
        assert_eq!(field.total_pheromones(), 1);
        field.decay(500); // Explore decays fast (0.01/tick)
        assert_eq!(field.total_pheromones(), 0);
    }

    #[test]
    fn signal_net_valence() {
        let mut signal = PheromoneSignal::default();
        signal.food_attract = 2.0;
        signal.danger_repel = 1.0;
        let valence = signal.net_valence();
        assert!(valence > 0.0); // food wins over danger
    }

    #[test]
    fn signal_dominant_type() {
        let mut signal = PheromoneSignal::default();
        signal.danger_repel = 3.0;
        signal.food_attract = 1.0;
        assert_eq!(signal.dominant_type(), Some(PheromoneType::Danger));
    }

    #[test]
    fn field_prune_respects_limit() {
        let mut field = PheromoneField::new(10.0, 5);
        for i in 0..10 {
            field.deposit(
                PheromoneType::Explore,
                [i as f32 * 100.0, 0.0],
                "agent_0",
                0,
            );
        }
        assert_eq!(field.total_pheromones(), 10);
        field.prune(0);
        assert!(field.total_pheromones() <= 5);
    }
}
