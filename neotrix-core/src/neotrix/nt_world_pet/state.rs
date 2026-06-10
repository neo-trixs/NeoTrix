use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualParams {
    pub size: f64,
    pub warmth: f64,
    pub softness: f64,
    pub energy: f64,
    pub brightness: f64,
    pub creature: f64,
    pub complexity: f64,
    pub definition: f64,
}

impl Default for VisualParams {
    fn default() -> Self {
        Self {
            size: 0.5,
            warmth: 0.5,
            softness: 0.5,
            energy: 0.5,
            brightness: 0.5,
            creature: 0.5,
            complexity: 0.3,
            definition: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorParams {
    pub curiosity: f64,
    pub playfulness: f64,
    pub talkativeness: f64,
    pub reactivity: f64,
}

impl Default for BehaviorParams {
    fn default() -> Self {
        Self {
            curiosity: 0.5,
            playfulness: 0.5,
            talkativeness: 0.5,
            reactivity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PetExpression {
    Neutral,
    Excited,
    Curious,
    Content,
    Frustrated,
    Confused,
    Sleepy,
}

impl Default for PetExpression {
    fn default() -> Self {
        Self::Neutral
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetState {
    pub visual: VisualParams,
    pub target_visual: VisualParams,
    pub behavior: BehaviorParams,
    pub expression: PetExpression,
    pub level: u32,
    pub energy: f64,
    pub age_cycles: u64,
    pub conversations_had: u64,
    pub kb_nodes_at_birth: u64,
}

impl Default for PetState {
    fn default() -> Self {
        Self {
            visual: VisualParams::default(),
            target_visual: VisualParams::default(),
            behavior: BehaviorParams::default(),
            expression: PetExpression::default(),
            level: 0,
            energy: 0.5,
            age_cycles: 0,
            conversations_had: 0,
            kb_nodes_at_birth: 0,
        }
    }
}

impl PetState {
    pub fn transition_speed(&self) -> f64 {
        if self.energy > 0.7 {
            0.15
        } else if self.energy > 0.3 {
            0.08
        } else {
            0.03
        }
    }

    pub fn tick_transition(&mut self) {
        let speed = self.transition_speed();
        self.visual.size += (self.target_visual.size - self.visual.size) * speed;
        self.visual.warmth += (self.target_visual.warmth - self.visual.warmth) * speed;
        self.visual.softness += (self.target_visual.softness - self.visual.softness) * speed;
        self.visual.energy += (self.target_visual.energy - self.visual.energy) * speed;
        self.visual.brightness += (self.target_visual.brightness - self.visual.brightness) * speed;
        self.visual.creature += (self.target_visual.creature - self.visual.creature) * speed;
        self.visual.complexity += (self.target_visual.complexity - self.visual.complexity) * speed;
        self.visual.definition += (self.target_visual.definition - self.visual.definition) * speed;
    }
}
