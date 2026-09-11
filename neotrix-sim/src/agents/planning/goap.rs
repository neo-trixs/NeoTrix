use std::collections::{HashMap, VecDeque};

use crate::agents::sim_agent::SimAgent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GOAPState {
    pub properties: HashMap<String, bool>,
}

impl GOAPState {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: bool) {
        self.properties.insert(key.to_string(), value);
    }

    pub fn satisfies(&self, preconditions: &GOAPState) -> bool {
        for (key, value) in &preconditions.properties {
            if self.properties.get(key) != Some(value) {
                return false;
            }
        }
        true
    }

    pub fn apply(&mut self, effects: &GOAPState) {
        for (key, value) in &effects.properties {
            self.properties.insert(key.to_string(), *value);
        }
    }
}

#[derive(Debug, Clone)]
pub struct GOAPAction {
    pub name: String,
    pub cost: f32,
    pub preconditions: GOAPState,
    pub effects: GOAPState,
}

impl GOAPAction {
    pub fn new(name: &str, cost: f32) -> Self {
        Self {
            name: name.to_string(),
            cost,
            preconditions: GOAPState::new(),
            effects: GOAPState::new(),
        }
    }

    pub fn with_precondition(mut self, key: &str, value: bool) -> Self {
        self.preconditions.set(key, value);
        self
    }

    pub fn with_effect(mut self, key: &str, value: bool) -> Self {
        self.effects.set(key, value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct GOAPPlan {
    pub actions: Vec<GOAPAction>,
    pub total_cost: f32,
}

pub struct GOAPPlanner {
    pub actions: Vec<GOAPAction>,
    pub max_iterations: usize,
}

impl GOAPPlanner {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            max_iterations: 100,
        }
    }

    pub fn add_action(&mut self, action: GOAPAction) {
        self.actions.push(action);
    }

    pub fn plan(&self, world_state: &GOAPState, goal: &GOAPState) -> Option<GOAPPlan> {
        let mut queue: VecDeque<(GOAPState, Vec<GOAPAction>, f32)> = VecDeque::new();
        let mut visited: Vec<GOAPState> = Vec::new();

        queue.push_back((world_state.clone(), Vec::new(), 0.0));

        let mut iterations = 0;

        while let Some((current_state, plan_so_far, current_cost)) = queue.pop_front() {
            iterations += 1;
            if iterations > self.max_iterations {
                return None;
            }

            if current_state.satisfies(goal) {
                return Some(GOAPPlan {
                    actions: plan_so_far,
                    total_cost: current_cost,
                });
            }

            if visited.contains(&current_state) {
                continue;
            }
            visited.push(current_state.clone());

            for action in &self.actions {
                if current_state.satisfies(&action.preconditions) {
                    let mut new_state = current_state.clone();
                    new_state.apply(&action.effects);

                    if !visited.contains(&new_state) {
                        let mut new_plan = plan_so_far.clone();
                        new_plan.push(action.clone());
                        queue.push_back((new_state, new_plan, current_cost + action.cost));
                    }
                }
            }
        }

        None
    }
}

pub fn agent_to_world_state(agent: &SimAgent) -> GOAPState {
    let mut state = GOAPState::new();
    state.set("has_energy", agent.core.energy > 20.0);
    state.set("is_hungry", agent.core.hunger > 50.0);
    state.set("is_healthy", agent.core.health > 30.0);
    state.set("is_nearby_agent", false);
    state.set("can_trade", agent.core.energy > 10.0);
    state
}

pub fn create_survival_plan(agent: &SimAgent) -> Option<GOAPPlan> {
    let mut planner = GOAPPlanner::new();

    planner.add_action(
        GOAPAction::new("rest", 0.3).with_effect("has_energy", true),
    );

    planner.add_action(
        GOAPAction::new("gather_food", 1.0)
            .with_precondition("has_energy", true)
            .with_effect("has_food", true),
    );

    planner.add_action(
        GOAPAction::new("eat", 0.5)
            .with_precondition("has_food", true)
            .with_effect("is_hungry", false),
    );

    planner.add_action(
        GOAPAction::new("heal", 0.8)
            .with_precondition("has_energy", true)
            .with_effect("is_healthy", true),
    );

    planner.add_action(
        GOAPAction::new("explore", 1.5)
            .with_precondition("has_energy", true)
            .with_effect("is_nearby_agent", true),
    );

    planner.add_action(
        GOAPAction::new("trade", 1.2)
            .with_precondition("has_energy", true)
            .with_precondition("is_nearby_agent", true)
            .with_effect("has_food", true),
    );

    let world_state = agent_to_world_state(agent);
    let mut goal = GOAPState::new();
    goal.set("is_hungry", false);

    planner.plan(&world_state, &goal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn test_goap_state_satisfies() {
        let mut state = GOAPState::new();
        state.set("has_energy", true);
        state.set("is_hungry", false);

        let mut preconditions = GOAPState::new();
        preconditions.set("has_energy", true);

        assert!(state.satisfies(&preconditions));
    }

    #[test]
    fn test_goap_state_apply() {
        let mut state = GOAPState::new();
        state.set("has_energy", true);

        let mut effects = GOAPState::new();
        effects.set("has_food", true);

        state.apply(&effects);
        assert!(state.satisfies(&effects));
    }

    #[test]
    fn test_goap_planner_simple() {
        let mut planner = GOAPPlanner::new();
        planner.add_action(
            GOAPAction::new("gather_food", 1.0)
                .with_precondition("has_energy", true)
                .with_effect("has_food", true),
        );

        let mut world = GOAPState::new();
        world.set("has_energy", true);

        let mut goal = GOAPState::new();
        goal.set("has_food", true);

        let plan = planner.plan(&world, &goal);
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].name, "gather_food");
    }

    #[test]
    fn test_goap_planner_chain() {
        let mut planner = GOAPPlanner::new();
        planner.add_action(
            GOAPAction::new("rest", 0.3).with_effect("has_energy", true),
        );
        planner.add_action(
            GOAPAction::new("gather_food", 1.0)
                .with_precondition("has_energy", true)
                .with_effect("has_food", true),
        );

        let world = GOAPState::new();
        let mut goal = GOAPState::new();
        goal.set("has_food", true);

        let plan = planner.plan(&world, &goal);
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert_eq!(plan.actions.len(), 2);
    }

    #[test]
    fn test_goap_planner_no_plan() {
        let mut planner = GOAPPlanner::new();
        planner.add_action(
            GOAPAction::new("gather_food", 1.0)
                .with_precondition("has_energy", true)
                .with_effect("has_food", true),
        );

        let world = GOAPState::new();
        let mut goal = GOAPState::new();
        goal.set("has_food", true);

        let plan = planner.plan(&world, &goal);
        assert!(plan.is_none());
    }

    #[test]
    fn test_agent_to_world_state() {
        let mut agent = SimAgent::new(1, Vec2::new(0.0, 0.0));
        agent.core.energy = 50.0;
        agent.core.hunger = 60.0;
        agent.core.health = 80.0;

        let state = agent_to_world_state(&agent);
        assert!(state.properties.get("has_energy") == Some(&true));
        assert!(state.properties.get("is_hungry") == Some(&true));
        assert!(state.properties.get("is_healthy") == Some(&true));
    }

    #[test]
    fn test_create_survival_plan() {
        let mut agent = SimAgent::new(1, Vec2::new(0.0, 0.0));
        agent.core.energy = 50.0;
        agent.core.hunger = 70.0;

        let plan = create_survival_plan(&agent);
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert!(!plan.actions.is_empty());
        assert!(plan.total_cost > 0.0);
    }
}
