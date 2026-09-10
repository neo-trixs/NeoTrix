use std::collections::HashMap;

use crate::agents::sim_agent::AgentAction;

/// Cost of performing an action (energy + time)
#[derive(Debug, Clone)]
pub struct ActionCost {
    pub energy: f32,
    pub time: u64,
    pub risk: f32,
}

/// Default costs for each action type
pub struct ActionCostTable {
    costs: HashMap<String, ActionCost>,
}

impl ActionCostTable {
    pub fn new() -> Self {
        let mut costs = HashMap::new();
        costs.insert("move".to_string(), ActionCost { energy: 1.0, time: 1, risk: 0.0 });
        costs.insert("eat".to_string(), ActionCost { energy: 0.5, time: 2, risk: 0.0 });
        costs.insert("rest".to_string(), ActionCost { energy: -5.0, time: 5, risk: 0.0 });
        costs.insert("harvest".to_string(), ActionCost { energy: 3.0, time: 3, risk: 0.1 });
        costs.insert("trade".to_string(), ActionCost { energy: 2.0, time: 2, risk: 0.05 });
        costs.insert("talk".to_string(), ActionCost { energy: 1.0, time: 1, risk: 0.0 });
        costs.insert("attack".to_string(), ActionCost { energy: 8.0, time: 2, risk: 0.3 });
        costs.insert("build".to_string(), ActionCost { energy: 10.0, time: 5, risk: 0.15 });
        costs.insert("explore".to_string(), ActionCost { energy: 2.0, time: 2, risk: 0.1 });
        costs.insert("think".to_string(), ActionCost { energy: 0.5, time: 1, risk: 0.0 });
        Self { costs }
    }

    pub fn cost_for(&self, action: &AgentAction) -> ActionCost {
        let key = match action {
            AgentAction::Move { .. } => "move",
            AgentAction::Eat { .. } => "eat",
            AgentAction::Rest => "rest",
            AgentAction::Harvest { .. } => "harvest",
            AgentAction::Trade { .. } => "trade",
            AgentAction::Talk { .. } => "talk",
            AgentAction::Attack { .. } => "attack",
            AgentAction::Build { .. } => "build",
            AgentAction::Explore { .. } => "explore",
            AgentAction::Think => "think",
        };
        self.costs.get(key).cloned().unwrap_or(ActionCost { energy: 1.0, time: 1, risk: 0.0 })
    }

    /// Can the agent afford this action?
    pub fn can_afford(&self, action: &AgentAction, energy: f32, health: f32) -> bool {
        let cost = self.cost_for(action);
        energy >= cost.energy && health > 20.0
    }

    /// Get all affordable actions from a list
    pub fn affordable_actions<'a>(&self, actions: &'a [AgentAction], energy: f32, health: f32) -> Vec<&'a AgentAction> {
        actions.iter().filter(|a| self.can_afford(a, energy, health)).collect()
    }

    /// Risk-adjusted cost (energy * (1.0 + risk))
    pub fn risk_adjusted_cost(&self, action: &AgentAction) -> f32 {
        let cost = self.cost_for(action);
        cost.energy * (1.0 + cost.risk)
    }
}

/// Budget tracker for an agent
pub struct ActionBudget {
    pub total_spent: f32,
    pub total_gained: f32,
    pub actions_taken: u64,
    pub risk_exposure: f64,
}

impl ActionBudget {
    pub fn new() -> Self {
        Self {
            total_spent: 0.0,
            total_gained: 0.0,
            actions_taken: 0,
            risk_exposure: 0.0,
        }
    }

    pub fn record_action(&mut self, action: &AgentAction, table: &ActionCostTable) {
        let cost = table.cost_for(action);
        if cost.energy > 0.0 {
            self.total_spent += cost.energy;
        } else {
            self.total_gained += cost.energy.abs();
        }
        self.actions_taken += 1;
        self.risk_exposure += cost.risk as f64;
    }

    pub fn efficiency(&self) -> f64 {
        if self.total_spent == 0.0 {
            return 1.0;
        }
        self.total_gained as f64 / self.total_spent as f64
    }

    pub fn risk_per_action(&self) -> f64 {
        if self.actions_taken == 0 {
            return 0.0;
        }
        self.risk_exposure / self.actions_taken as f64
    }

    pub fn reset(&mut self) {
        self.total_spent = 0.0;
        self.total_gained = 0.0;
        self.actions_taken = 0;
        self.risk_exposure = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn cost_for_returns_correct_costs() {
        let table = ActionCostTable::new();

        let move_cost = table.cost_for(&AgentAction::Move { target: Vec2::new(1.0, 1.0) });
        assert_eq!(move_cost.energy, 1.0);
        assert_eq!(move_cost.time, 1);
        assert_eq!(move_cost.risk, 0.0);

        let eat_cost = table.cost_for(&AgentAction::Eat { resource_id: "food".to_string() });
        assert_eq!(eat_cost.energy, 0.5);
        assert_eq!(eat_cost.time, 2);

        let rest_cost = table.cost_for(&AgentAction::Rest);
        assert_eq!(rest_cost.energy, -5.0);

        let harvest_cost = table.cost_for(&AgentAction::Harvest { resource_id: "tree".to_string() });
        assert_eq!(harvest_cost.energy, 3.0);
        assert_eq!(harvest_cost.risk, 0.1);

        let trade_cost = table.cost_for(&AgentAction::Trade { target_id: "a".to_string(), item: "wood".to_string(), amount: 1 });
        assert_eq!(trade_cost.energy, 2.0);

        let talk_cost = table.cost_for(&AgentAction::Talk { target_id: "a".to_string(), message: "hi".to_string() });
        assert_eq!(talk_cost.energy, 1.0);

        let attack_cost = table.cost_for(&AgentAction::Attack { target_id: "a".to_string() });
        assert_eq!(attack_cost.energy, 8.0);
        assert_eq!(attack_cost.risk, 0.3);

        let build_cost = table.cost_for(&AgentAction::Build { position: Vec2::new(0.0, 0.0), structure_type: "house".to_string() });
        assert_eq!(build_cost.energy, 10.0);
        assert_eq!(build_cost.time, 5);

        let explore_cost = table.cost_for(&AgentAction::Explore { direction: Vec2::new(1.0, 0.0) });
        assert_eq!(explore_cost.energy, 2.0);

        let think_cost = table.cost_for(&AgentAction::Think);
        assert_eq!(think_cost.energy, 0.5);
    }

    #[test]
    fn can_afford_with_sufficient_energy() {
        let table = ActionCostTable::new();
        let action = AgentAction::Move { target: Vec2::new(1.0, 0.0) };
        assert!(table.can_afford(&action, 10.0, 50.0));
    }

    #[test]
    fn can_afford_with_insufficient_energy() {
        let table = ActionCostTable::new();
        let action = AgentAction::Build { position: Vec2::new(0.0, 0.0), structure_type: "house".to_string() };
        assert!(!table.can_afford(&action, 5.0, 50.0));
    }

    #[test]
    fn can_afford_low_health_blocks() {
        let table = ActionCostTable::new();
        let action = AgentAction::Move { target: Vec2::new(1.0, 0.0) };
        assert!(!table.can_afford(&action, 100.0, 15.0));
    }

    #[test]
    fn affordable_actions_filters_correctly() {
        let table = ActionCostTable::new();
        let actions = vec![
            AgentAction::Move { target: Vec2::new(1.0, 0.0) },
            AgentAction::Build { position: Vec2::new(0.0, 0.0), structure_type: "house".to_string() },
            AgentAction::Think,
        ];
        let affordable = table.affordable_actions(&actions, 5.0, 50.0);
        // move (1.0) and think (0.5) affordable, build (10.0) not
        assert_eq!(affordable.len(), 2);
    }

    #[test]
    fn risk_adjusted_cost_includes_risk() {
        let table = ActionCostTable::new();

        let move_rac = table.risk_adjusted_cost(&AgentAction::Move { target: Vec2::new(1.0, 0.0) });
        assert_eq!(move_rac, 1.0); // risk=0, so 1.0 * 1.0

        let attack_rac = table.risk_adjusted_cost(&AgentAction::Attack { target_id: "a".to_string() });
        assert!((attack_rac - 10.4).abs() < 0.01); // 8.0 * 1.3
    }

    #[test]
    fn budget_tracking_accumulates() {
        let table = ActionCostTable::new();
        let mut budget = ActionBudget::new();

        budget.record_action(&AgentAction::Move { target: Vec2::new(1.0, 0.0) }, &table);
        assert_eq!(budget.total_spent, 1.0);
        assert_eq!(budget.actions_taken, 1);

        budget.record_action(&AgentAction::Attack { target_id: "a".to_string() }, &table);
        assert_eq!(budget.total_spent, 9.0);
        assert_eq!(budget.actions_taken, 2);
    }

    #[test]
    fn budget_rest_gains_energy() {
        let table = ActionCostTable::new();
        let mut budget = ActionBudget::new();

        budget.record_action(&AgentAction::Rest, &table);
        assert_eq!(budget.total_spent, 0.0);
        assert_eq!(budget.total_gained, 5.0);
    }

    #[test]
    fn efficiency_calculation() {
        let table = ActionCostTable::new();
        let mut budget = ActionBudget::new();

        // No actions: efficiency is 1.0
        assert_eq!(budget.efficiency(), 1.0);

        // Rest gains 5, move costs 1 => gain/spend = 5/1 = 5.0
        budget.record_action(&AgentAction::Rest, &table);
        budget.record_action(&AgentAction::Move { target: Vec2::new(0.0, 0.0) }, &table);
        assert!((budget.efficiency() - 5.0).abs() < 0.01);
    }

    #[test]
    fn risk_per_action_calculation() {
        let table = ActionCostTable::new();
        let mut budget = ActionBudget::new();

        assert_eq!(budget.risk_per_action(), 0.0);

        // move (risk=0) + attack (risk=0.3) => avg 0.15
        budget.record_action(&AgentAction::Move { target: Vec2::new(0.0, 0.0) }, &table);
        budget.record_action(&AgentAction::Attack { target_id: "a".to_string() }, &table);
        assert!((budget.risk_per_action() - 0.15).abs() < 0.001);
    }

    #[test]
    fn budget_reset() {
        let table = ActionCostTable::new();
        let mut budget = ActionBudget::new();

        budget.record_action(&AgentAction::Attack { target_id: "a".to_string() }, &table);
        budget.record_action(&AgentAction::Rest, &table);

        budget.reset();
        assert_eq!(budget.total_spent, 0.0);
        assert_eq!(budget.total_gained, 0.0);
        assert_eq!(budget.actions_taken, 0);
        assert_eq!(budget.risk_exposure, 0.0);
    }
}
