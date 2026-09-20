/// NT-GAME 场景管理器 — 吸收 Godot SceneTree
///
/// 管理预制体（Prefab）和场景实例（SceneInstance）
/// 支持 RON 格式的预制体数据

use std::collections::HashMap;

use super::components::{Transform2D, Vec2};
use crate::l5_cognition::nt_mind::nt_game::ecs::entity::EntityId;

/// 预制体：一组组件的模板（类似 Godot 的 .tscn 场景文件）
#[derive(Debug, Clone)]
pub struct Prefab {
    pub name: String,
    pub transform: Option<Transform2D>,
    pub sprite: Option<String>,       // 纹理路径
    pub collider: Option<ColliderDesc>,
    pub health: Option<f32>,
    pub movement_speed: Option<f32>,
    pub children: Vec<Prefab>,
}

#[derive(Debug, Clone)]
pub struct ColliderDesc {
    pub w: f32,
    pub h: f32,
    pub is_sensor: bool,
}

/// 场景实例：世界中的一个实例化单元
#[derive(Debug, Clone)]
pub struct SceneInstance {
    pub root: EntityId,
    pub children: Vec<EntityId>,
    pub prefab_name: Option<String>,
    pub position: Vec2,
}

/// 场景管理器
pub struct SceneManager {
    prefabs: HashMap<String, Prefab>,
    instances: Vec<SceneInstance>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            prefabs: HashMap::new(),
            instances: Vec::new(),
        }
    }

    /// 注册预制体
    pub fn register_prefab(&mut self, prefab: Prefab) {
        let name = prefab.name.clone();
        self.prefabs.insert(name, prefab);
    }

    /// 从 RON 字符串加载预制体
    pub fn load_prefab_from_ron(&mut self, ron_str: &str) -> Result<(), String> {
        // 简化的 RON 解析（实际应使用 ron crate）
        let prefab = Prefab {
            name: extract_field(ron_str, "name").unwrap_or_default(),
            transform: None,
            sprite: extract_field(ron_str, "texture"),
            collider: None,
            health: extract_field::<f32>(ron_str, "health"),
            movement_speed: extract_field::<f32>(ron_str, "speed"),
            children: Vec::new(),
        };
        let name = prefab.name.clone();
        self.prefabs.insert(name, prefab);
        Ok(())
    }

    /// 获取预制体
    pub fn get_prefab(&self, name: &str) -> Option<&Prefab> {
        self.prefabs.get(name)
    }

    /// 记录一个场景实例
    pub fn add_instance(&mut self, instance: SceneInstance) {
        self.instances.push(instance);
    }

    /// 获取所有实例
    pub fn instances(&self) -> &[SceneInstance] {
        &self.instances
    }

    /// 获取预制体数量
    pub fn prefab_count(&self) -> usize {
        self.prefabs.len()
    }

    /// 清除所有实例
    pub fn clear_instances(&mut self) {
        self.instances.clear();
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 简化的 RON 字段提取（生产环境应使用 ron crate）
fn extract_field<T: std::str::FromStr>(ron: &str, field: &str) -> Option<T> {
    let pattern = format!("{}:", field);
    let start = ron.find(&pattern)? + pattern.len();
    let rest = &ron[start..];
    let trimmed = rest.trim().trim_end_matches(',').trim_end_matches(')');
    trimmed.parse().ok()
}

/// 预定义的平台预制体
pub fn create_platform_prefab() -> Prefab {
    Prefab {
        name: "platform".to_string(),
        transform: Some(Transform2D::new(Vec2::new(0.0, 0.0))),
        sprite: Some("platform.png".to_string()),
        collider: Some(ColliderDesc {
            w: 128.0,
            h: 32.0,
            is_sensor: false,
        }),
        health: None,
        movement_speed: None,
        children: Vec::new(),
    }
}

/// 预定义的玩家预制体
pub fn create_player_prefab() -> Prefab {
    Prefab {
        name: "player".to_string(),
        transform: Some(Transform2D::new(Vec2::new(100.0, 200.0))),
        sprite: Some("player.png".to_string()),
        collider: Some(ColliderDesc {
            w: 32.0,
            h: 32.0,
            is_sensor: false,
        }),
        health: Some(100.0),
        movement_speed: Some(200.0),
        children: Vec::new(),
    }
}

/// 预定义的 NPC 预制体
pub fn create_npc_prefab(name: &str, x: f32, y: f32) -> Prefab {
    Prefab {
        name: name.to_string(),
        transform: Some(Transform2D::new(Vec2::new(x, y))),
        sprite: Some(format!("npc_{}.png", name)),
        collider: Some(ColliderDesc {
            w: 32.0,
            h: 32.0,
            is_sensor: true,
        }),
        health: Some(50.0),
        movement_speed: None,
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_manager() {
        let mut sm = SceneManager::new();
        sm.register_prefab(create_player_prefab());
        sm.register_prefab(create_platform_prefab());
        assert_eq!(sm.prefab_count(), 2);
        assert!(sm.get_prefab("player").is_some());
        assert!(sm.get_prefab("nonexistent").is_none());
    }

    #[test]
    fn test_prefab_children() {
        let mut child = create_platform_prefab();
        child.name = "child_platform".to_string();
        let mut parent = create_player_prefab();
        parent.children.push(child);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].name, "child_platform");
    }
}
