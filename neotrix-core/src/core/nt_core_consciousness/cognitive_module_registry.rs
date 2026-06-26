use std::collections::HashMap;

/// 认知模块的运行时阶段标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModulePhase {
    Initial,
    PreRefinery,
    PostRefinery,
    PreDualPath,
    PostDualPath,
    PreBeliefVerify,
    PostBeliefVerify,
    Final,
}

impl ModulePhase {
    pub fn all() -> &'static [ModulePhase] {
        &[
            ModulePhase::Initial,
            ModulePhase::PreRefinery,
            ModulePhase::PostRefinery,
            ModulePhase::PreDualPath,
            ModulePhase::PostDualPath,
            ModulePhase::PreBeliefVerify,
            ModulePhase::PostBeliefVerify,
            ModulePhase::Final,
        ]
    }
}

/// 认知模块特质 — 任何想接入 ConsciousnessPipeline 的模块只需实现此 trait
pub trait CognitiveModule: std::fmt::Debug + Send + Sync {
    /// 模块名称
    fn name(&self) -> &'static str;
    /// 期望在哪个阶段之后运行
    fn phase(&self) -> ModulePhase;
    /// 执行一次 tick，返回是否产生了有意义的变化
    fn tick(&mut self) -> bool;
    /// 优雅降级: 模块失效时返回 true
    fn is_crash_safe(&self) -> bool {
        true
    }
}

/// 模块注册表 — 替代硬编码的 8 阶段管道
///
/// 任何实现了 CognitiveModule 的模块都可以注册到这里。
/// Pipeline 在 run_full_cycle 中按阶段遍历所有注册模块。
#[derive(Debug)]
pub struct ModuleRegistry {
    modules: HashMap<ModulePhase, Vec<Box<dyn CognitiveModule>>>,
    /// 模块名 → 健康状态
    health: HashMap<String, bool>,
}

impl Clone for ModuleRegistry {
    fn clone(&self) -> Self {
        Self {
            modules: HashMap::new(),
            health: self.health.clone(),
        }
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            health: HashMap::new(),
        }
    }

    /// 注册一个认知模块
    pub fn register(&mut self, module: Box<dyn CognitiveModule>) {
        let phase = module.phase();
        let name = module.name().to_string();
        self.modules.entry(phase).or_default().push(module);
        self.health.insert(name, true);
    }

    /// 运行指定阶段的所有模块
    pub fn run_phase(&mut self, phase: ModulePhase) -> usize {
        let count = 0;
        if let Some(modules) = self.modules.get_mut(&phase) {
            for module in modules.iter_mut() {
                let name = module.name().to_string();
                let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| module.tick()));
                match ok {
                    Ok(_) => {}
                    Err(_) => {
                        if module.is_crash_safe() {
                            self.health.insert(name, false);
                        }
                    }
                }
            }
            modules.len()
        } else {
            count
        }
    }

    /// 运行从某个阶段开始的所有后续阶段
    pub fn run_from(&mut self, from: ModulePhase) -> usize {
        let mut total = 0;
        let phases = ModulePhase::all();
        let start = phases.iter().position(|p| *p == from).unwrap_or(0);
        for phase in &phases[start..] {
            total += self.run_phase(*phase);
        }
        total
    }

    /// 运行所有阶段
    pub fn run_all(&mut self) -> usize {
        self.run_from(ModulePhase::Initial)
    }

    /// 当前注册的模块数量
    pub fn count(&self) -> usize {
        self.modules.values().map(|v| v.len()).sum()
    }

    /// 所有健康模块数量
    pub fn healthy_count(&self) -> usize {
        self.health.values().filter(|h| **h).count()
    }

    /// 健康状态报告
    pub fn health_report(&self) -> Vec<(&str, bool)> {
        self.health.iter().map(|(k, v)| (k.as_str(), *v)).collect()
    }

    /// 所有注册模块的名称列表
    pub fn all_names(&self) -> Vec<String> {
        self.health.keys().cloned().collect()
    }
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyModule {
        name: &'static str,
        phase: ModulePhase,
        ticked: bool,
    }

    impl CognitiveModule for DummyModule {
        fn name(&self) -> &'static str {
            self.name
        }
        fn phase(&self) -> ModulePhase {
            self.phase
        }
        fn tick(&mut self) -> bool {
            self.ticked = true;
            true
        }
    }

    #[test]
    fn test_register_and_run() {
        let mut reg = ModuleRegistry::new();
        reg.register(Box::new(DummyModule {
            name: "test1",
            phase: ModulePhase::PostRefinery,
            ticked: false,
        }));
        reg.register(Box::new(DummyModule {
            name: "test2",
            phase: ModulePhase::PostRefinery,
            ticked: false,
        }));
        assert_eq!(reg.count(), 2);
        let n = reg.run_phase(ModulePhase::PostRefinery);
        assert_eq!(n, 2);
    }

    #[test]
    fn test_run_all_phases() {
        let mut reg = ModuleRegistry::new();
        reg.register(Box::new(DummyModule {
            name: "m1",
            phase: ModulePhase::Initial,
            ticked: false,
        }));
        reg.register(Box::new(DummyModule {
            name: "m2",
            phase: ModulePhase::Final,
            ticked: false,
        }));
        let total = reg.run_all();
        assert_eq!(total, 2);
    }

    #[test]
    fn test_run_from() {
        let mut reg = ModuleRegistry::new();
        reg.register(Box::new(DummyModule {
            name: "m1",
            phase: ModulePhase::Initial,
            ticked: false,
        }));
        let total = reg.run_from(ModulePhase::PostRefinery);
        assert_eq!(total, 0);
    }

    #[test]
    fn test_empty_registry() {
        let mut reg = ModuleRegistry::new();
        assert_eq!(reg.run_all(), 0);
        assert_eq!(reg.count(), 0);
        assert_eq!(reg.healthy_count(), 0);
    }

    #[test]
    fn test_health_tracking() {
        let mut reg = ModuleRegistry::new();
        reg.register(Box::new(DummyModule {
            name: "m1",
            phase: ModulePhase::Initial,
            ticked: false,
        }));
        reg.run_all();
        assert_eq!(reg.healthy_count(), 1);
    }
}
