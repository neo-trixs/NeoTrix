use super::world::World;

/// System: 逻辑处理器
pub trait System: Send + Sync {
    /// 更新系统
    fn update(&mut self, world: &mut World, dt: f32);

    /// 系统优先级 (越小越先执行)
    fn priority(&self) -> i32 {
        0
    }

    /// 系统名称 (用于调试)
    fn name(&self) -> &str {
        "UnnamedSystem"
    }

    /// 系统是否启用
    fn enabled(&self) -> bool {
        true
    }
}

/// System scheduler: 系统调度器
pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
    sorted: bool,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            sorted: false,
        }
    }

    /// 添加系统
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
        self.sorted = false;
    }

    /// 运行所有系统
    pub fn run(&mut self, world: &mut World, dt: f32) {
        if !self.sorted {
            self.systems.sort_by_key(|s| s.priority());
            self.sorted = true;
        }

        for system in &mut self.systems {
            if system.enabled() {
                system.update(world, dt);
            }
        }
    }

    /// 获取系统数量
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// 清空系统
    pub fn clear(&mut self) {
        self.systems.clear();
        self.sorted = false;
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// 移动系统示例
pub struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, _world: &mut World, _dt: f32) {
        // TODO: 实现移动逻辑
    }

    fn priority(&self) -> i32 {
        100 // 低优先级
    }

    fn name(&self) -> &str {
        "MovementSystem"
    }
}

/// 渲染系统示例
pub struct RenderingSystem;

impl System for RenderingSystem {
    fn update(&mut self, _world: &mut World, _dt: f32) {
        // TODO: 实现渲染逻辑
    }

    fn priority(&self) -> i32 {
        1000 // 最高优先级 (最后执行)
    }

    fn name(&self) -> &str {
        "RenderingSystem"
    }
}

/// 物理系统示例
pub struct PhysicsSystem;

impl System for PhysicsSystem {
    fn update(&mut self, _world: &mut World, _dt: f32) {
        // TODO: 实现物理逻辑
    }

    fn priority(&self) -> i32 {
        50 // 中优先级
    }

    fn name(&self) -> &str {
        "PhysicsSystem"
    }
}

/// AI系统示例
pub struct AISystem;

impl System for AISystem {
    fn update(&mut self, world: &mut World, dt: f32) {
        // TODO: 实现AI逻辑
    }

    fn priority(&self) -> i32 {
        200 // 低优先级
    }

    fn name(&self) -> &str {
        "AISystem"
    }
}
