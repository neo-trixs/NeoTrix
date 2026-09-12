use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};

/// 事件 trait
pub trait Event: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

/// 事件处理器 trait
pub trait EventHandler: Send + Sync {
    fn handle(&mut self, event: &dyn Event, world: &mut crate::ecs::World);
}

/// 事件总线
pub struct EventBus {
    queues: HashMap<TypeId, VecDeque<Box<dyn Event>>>,
    handlers: HashMap<TypeId, Vec<Box<dyn EventHandler>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// 发送事件
    pub fn send<E: Event + 'static>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        let queue = self.queues.entry(type_id).or_insert_with(VecDeque::new);
        queue.push_back(Box::new(event));
    }

    /// 注册事件处理器
    pub fn add_handler<E: Event + 'static>(&mut self, handler: Box<dyn EventHandler>) {
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.entry(type_id).or_insert_with(Vec::new);
        handlers.push(handler);
    }

    /// 处理所有事件
    pub fn process(&mut self, world: &mut crate::ecs::World) {
        // 复制事件队列，避免借用冲突
        let type_ids: Vec<TypeId> = self.queues.keys().copied().collect();
        
        for type_id in type_ids {
            if let Some(queue) = self.queues.get_mut(&type_id) {
                while let Some(event) = queue.pop_front() {
                    if let Some(handlers) = self.handlers.get_mut(&type_id) {
                        for handler in handlers.iter_mut() {
                            handler.handle(event.as_ref(), world);
                        }
                    }
                }
            }
        }
    }

    /// 清空事件队列
    pub fn clear(&mut self) {
        self.queues.clear();
    }

    /// 获取事件数量
    pub fn event_count(&self) -> usize {
        self.queues.values().map(|q| q.len()).sum()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// 移动事件
#[derive(Debug)]
pub struct MoveEvent {
    pub entity: crate::ecs::Entity,
    pub from: crate::engine::Vec2,
    pub to: crate::engine::Vec2,
}

impl Event for MoveEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 碰撞事件
#[derive(Debug)]
pub struct CollisionEvent {
    pub entity_a: crate::ecs::Entity,
    pub entity_b: crate::ecs::Entity,
    pub normal: crate::engine::Vec2,
    pub penetration: f32,
}

impl Event for CollisionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 输入事件
#[derive(Debug)]
pub struct InputEvent {
    pub key: Option<crate::engine::KeyCode>,
    pub mouse_button: Option<crate::engine::MouseButton>,
    pub mouse_position: Option<crate::engine::Vec2>,
    pub pressed: bool,
}

impl Event for InputEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 游戏事件
#[derive(Debug)]
pub struct GameEvent {
    pub event_type: String,
    pub data: Box<dyn Any + Send + Sync>,
}

impl Event for GameEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
