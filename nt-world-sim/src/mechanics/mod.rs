use crate::ecs::{Entity, World, System};
use crate::engine::Vec2;

/// 意识实体组件
#[derive(Debug, Clone)]
pub struct ConsciousnessEntity {
    pub domain: String,
    pub phi: f32,
    pub coherence: f32,
    pub health: f32,
    pub energy: f32,
}

impl ConsciousnessEntity {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            phi: 0.1,
            coherence: 0.5,
            health: 100.0,
            energy: 100.0,
        }
    }
}

/// 变换组件
#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub position: Vec2,
    pub velocity: Vec2,
    pub facing: u8,
}

impl TransformComponent {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::zero(),
            facing: 0,
        }
    }
}

/// 渲染组件
#[derive(Debug, Clone)]
pub struct RenderComponent {
    pub icon: String,
    pub color: String,
    pub z_index: i32,
}

impl RenderComponent {
    pub fn new(icon: &str, color: &str) -> Self {
        Self {
            icon: icon.to_string(),
            color: color.to_string(),
            z_index: 0,
        }
    }
}

/// AI 组件
#[derive(Debug, Clone)]
pub struct AiComponent {
    pub state: String,
    pub target: Option<Vec2>,
    pub path: Option<Vec<Vec2>>,
}

impl AiComponent {
    pub fn new() -> Self {
        Self {
            state: "idle".to_string(),
            target: None,
            path: None,
        }
    }
}

/// 马斯洛需求组件
#[derive(Debug, Clone)]
pub struct MaslowNeeds {
    pub physiological: f32,
    pub safety: f32,
    pub social: f32,
    pub esteem: f32,
    pub self_actualization: f32,
}

impl MaslowNeeds {
    pub fn new() -> Self {
        Self {
            physiological: 0.2,
            safety: 0.15,
            social: 0.1,
            esteem: 0.05,
            self_actualization: 0.03,
        }
    }

    /// 获取主导需求
    pub fn dominant(&self) -> (&str, f32) {
        let levels = [
            ("physiological", self.physiological),
            ("safety", self.safety),
            ("social", self.social),
            ("esteem", self.esteem),
            ("self_actualization", self.self_actualization),
        ];
        *levels
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// 满足需求
    pub fn satisfy(&mut self, level: &str, amount: f32) {
        match level {
            "physiological" => self.physiological = (self.physiological - amount).max(0.0),
            "safety" => self.safety = (self.safety - amount).max(0.0),
            "social" => self.social = (self.social - amount).max(0.0),
            "esteem" => self.esteem = (self.esteem - amount).max(0.0),
            "self_actualization" => {
                self.self_actualization = (self.self_actualization - amount).max(0.0)
            }
            _ => {}
        }
    }
}

impl Default for MaslowNeeds {
    fn default() -> Self {
        Self::new()
    }
}

/// 社会关系组件
#[derive(Debug, Clone)]
pub struct SocialRelationship {
    pub trust: f32,
    pub affinity: f32,
    pub interactions: u32,
    pub relationship_type: String,
}

impl SocialRelationship {
    pub fn new() -> Self {
        Self {
            trust: 0.5,
            affinity: 0.5,
            interactions: 0,
            relationship_type: "neutral".to_string(),
        }
    }
}

impl Default for SocialRelationship {
    fn default() -> Self {
        Self::new()
    }
}

/// 经济组件
#[derive(Debug, Clone)]
pub struct EconomyComponent {
    pub gold: i32,
    pub inventory: Vec<(String, i32)>,
}

impl EconomyComponent {
    pub fn new(gold: i32) -> Self {
        Self {
            gold,
            inventory: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: &str, count: i32) {
        if let Some(existing) = self.inventory.iter_mut().find(|(name, _)| name == item) {
            existing.1 += count;
        } else {
            self.inventory.push((item.to_string(), count));
        }
    }

    pub fn remove_item(&mut self, item: &str, count: i32) -> bool {
        if let Some(existing) = self.inventory.iter_mut().find(|(name, _)| name == item) {
            if existing.1 >= count {
                existing.1 -= count;
                if existing.1 == 0 {
                    self.inventory.retain(|(name, _)| name != item);
                }
                return true;
            }
        }
        false
    }

    pub fn count_item(&self, item: &str) -> i32 {
        self.inventory
            .iter()
            .find(|(name, _)| name == item)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    }
}

/// 意识系统
pub struct ConsciousnessSystem;

impl System for ConsciousnessSystem {
    fn name(&self) -> &str {
        "ConsciousnessSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        // 更新所有实体的意识指标
        let entities = world.query::<ConsciousnessEntity>();
        for entity in entities {
            if let Some(consciousness) = world.get_component_mut::<ConsciousnessEntity>(entity) {
                // 简单意识指标计算
                consciousness.phi = (consciousness.phi * 0.99 + 0.01).min(1.0);
                consciousness.coherence = (consciousness.coherence * 0.995 + 0.005).min(1.0);
            }
        }
    }
}

/// 马斯洛系统
pub struct MaslowSystem;

impl System for MaslowSystem {
    fn name(&self) -> &str {
        "MaslowSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities = world.query::<MaslowNeeds>();
        for entity in entities {
            if let Some(needs) = world.get_component_mut::<MaslowNeeds>(entity) {
                // 需求随时间增加
                needs.physiological = (needs.physiological + 0.0008 * dt).min(1.0);
                needs.safety = (needs.safety + 0.0004 * dt).min(1.0);
                needs.social = (needs.social + 0.0006 * dt).min(1.0);
                needs.esteem = (needs.esteem + 0.0002 * dt).min(1.0);
                needs.self_actualization = (needs.self_actualization + 0.0001 * dt).min(1.0);
            }
        }
    }
}

/// AI 系统
pub struct AiSystem;

impl System for AiSystem {
    fn name(&self) -> &str {
        "AiSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        let entities = world.query::<AiComponent>();
        for entity in entities {
            if let Some(ai) = world.get_component_mut::<AiComponent>(entity) {
                // 简单状态机
                match ai.state.as_str() {
                    "idle" => {
                        // 随机切换状态
                        if rand::random::<f32>() < 0.01 {
                            ai.state = "exploring".to_string();
                        }
                    }
                    "exploring" => {
                        // 移动到目标
                        if let Some(target) = ai.target {
                            if let Some(transform) =
                                world.get_component_mut::<TransformComponent>(entity)
                            {
                                let direction = (target - transform.position).normalize();
                                transform.position = transform.position + direction * 0.5;
                                transform.facing = if direction.x > 0.0 { 3 } else { 1 };
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
