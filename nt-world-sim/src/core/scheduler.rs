use std::any::TypeId;
use std::collections::VecDeque;

use super::world::UniversalWorld;

pub trait UniversalSystem: Send + Sync {
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    fn name(&self) -> &str {
        "UnnamedSystem"
    }
}

pub struct SystemDependency {
    pub reads: Vec<TypeId>,
    pub writes: Vec<TypeId>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

impl SystemDependency {
    pub fn new() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
        }
    }

    pub fn reads<T: 'static>(mut self) -> Self {
        self.reads.push(TypeId::of::<T>());
        self
    }

    pub fn writes<T: 'static>(mut self) -> Self {
        self.writes.push(TypeId::of::<T>());
        self
    }

    pub fn before(mut self, name: impl Into<String>) -> Self {
        self.before.push(name.into());
        self
    }

    pub fn after(mut self, name: impl Into<String>) -> Self {
        self.after.push(name.into());
        self
    }

    pub fn conflicts_with(&self, other: &SystemDependency) -> bool {
        for w in &self.writes {
            if other.reads.contains(w) || other.writes.contains(w) {
                return true;
            }
        }
        for r in &self.reads {
            if other.writes.contains(r) {
                return true;
            }
        }
        false
    }
}

impl Default for SystemDependency {
    fn default() -> Self {
        Self::new()
    }
}

struct SystemEntry {
    name: String,
    system: Box<dyn UniversalSystem>,
    dependency: SystemDependency,
}

pub struct ParallelScheduler {
    systems: Vec<SystemEntry>,
    execution_waves: Vec<Vec<usize>>,
    scheduled: bool,
}

impl ParallelScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            execution_waves: Vec::new(),
            scheduled: false,
        }
    }

    pub fn add_system(
        &mut self,
        system: Box<dyn UniversalSystem>,
        deps: SystemDependency,
    ) {
        let name = system.name().to_string();
        self.systems.push(SystemEntry {
            name,
            system,
            dependency: deps,
        });
        self.scheduled = false;
    }

    pub fn build_schedule(&mut self) {
        let n = self.systems.len();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut in_degree: Vec<usize> = vec![0; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }

                let must_before = self.systems[i].dependency.before.iter()
                    .any(|name| name == &self.systems[j].name);
                let must_after = self.systems[i].dependency.after.iter()
                    .any(|name| name == &self.systems[j].name);

                if must_before {
                    adjacency[i].push(j);
                    in_degree[j] += 1;
                } else if must_after {
                    adjacency[j].push(i);
                    in_degree[i] += 1;
                } else if self.systems[i].dependency.conflicts_with(&self.systems[j].dependency) {
                    let i_writes_any = self.systems[i].dependency.writes.iter()
                        .any(|w| self.systems[j].dependency.reads.contains(w)
                            || self.systems[j].dependency.writes.contains(w));
                    if i_writes_any {
                        adjacency[i].push(j);
                        in_degree[j] += 1;
                    } else {
                        adjacency[j].push(i);
                        in_degree[i] += 1;
                    }
                }
            }
        }

        let mut waves: Vec<Vec<usize>> = Vec::new();
        let mut remaining: VecDeque<usize> = (0..n).collect();

        while !remaining.is_empty() {
            let mut wave: Vec<usize> = Vec::new();
            let mut next_remaining: VecDeque<usize> = VecDeque::new();

            for &idx in &remaining {
                if in_degree[idx] == 0 {
                    wave.push(idx);
                } else {
                    next_remaining.push_back(idx);
                }
            }

            if wave.is_empty() {
                for &idx in &remaining {
                    wave.push(idx);
                }
                break;
            }

            for &idx in &wave {
                for &neighbor in &adjacency[idx] {
                    in_degree[neighbor] -= 1;
                }
            }

            waves.push(wave);
            remaining = next_remaining;
        }

        self.execution_waves = waves;
        self.scheduled = true;
    }

    pub fn run_parallel(&mut self, world: &mut UniversalWorld, dt: f32) {
        if !self.scheduled {
            self.build_schedule();
        }

        let waves = self.execution_waves.clone();
        for wave in &waves {
            for &system_idx in wave {
                if let Some(entry) = self.systems.get_mut(system_idx) {
                    entry.system.update(world, dt);
                }
            }
        }
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    pub fn wave_count(&self) -> usize {
        self.execution_waves.len()
    }

    pub fn get_wave(&self, index: usize) -> Option<&Vec<usize>> {
        self.execution_waves.get(index)
    }

    pub fn clear(&mut self) {
        self.systems.clear();
        self.execution_waves.clear();
        self.scheduled = false;
    }

    pub fn is_scheduled(&self) -> bool {
        self.scheduled
    }

    pub fn system_name(&self, index: usize) -> Option<&str> {
        self.systems.get(index).map(|e| e.name.as_str())
    }
}

impl Default for ParallelScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct PosVelSystem;

    impl UniversalSystem for PosVelSystem {
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {
            COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        fn name(&self) -> &str {
            "PosVelSystem"
        }
    }

    struct RenderSystem;

    impl UniversalSystem for RenderSystem {
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {
            COUNTER.fetch_add(10, Ordering::SeqCst);
        }

        fn name(&self) -> &str {
            "RenderSystem"
        }
    }

    struct AISystem;

    impl UniversalSystem for AISystem {
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {
            COUNTER.fetch_add(100, Ordering::SeqCst);
        }

        fn name(&self) -> &str {
            "AISystem"
        }
    }

    fn reset_counter() {
        COUNTER.store(0, Ordering::SeqCst);
    }

    #[test]
    fn test_system_dependency_no_conflict() {
        #[derive(Clone, Debug, PartialEq)]
        struct Position { x: f32 }
        impl Component for Position {}

        let dep_a = SystemDependency::new().reads::<Position>();
        let dep_b = SystemDependency::new().reads::<Position>();
        assert!(!dep_a.conflicts_with(&dep_b));
    }

    #[test]
    fn test_system_dependency_write_read_conflict() {
        #[derive(Clone, Debug, PartialEq)]
        struct Position { x: f32 }
        impl Component for Position {}

        let dep_writer = SystemDependency::new().writes::<Position>();
        let dep_reader = SystemDependency::new().reads::<Position>();
        assert!(dep_writer.conflicts_with(&dep_reader));
    }

    #[test]
    fn test_system_dependency_write_write_conflict() {
        #[derive(Clone, Debug, PartialEq)]
        struct Position { x: f32 }
        impl Component for Position {}

        let dep_a = SystemDependency::new().writes::<Position>();
        let dep_b = SystemDependency::new().writes::<Position>();
        assert!(dep_a.conflicts_with(&dep_b));
    }

    #[test]
    fn test_system_dependency_explicit_ordering() {
        let dep_a = SystemDependency::new().before("B");
        let dep_b = SystemDependency::new().after("A");
        assert!(dep_a.before.contains(&"B".to_string()));
        assert!(dep_b.after.contains(&"A".to_string()));
    }

    #[test]
    fn test_scheduler_single_system() {
        reset_counter();
        let mut scheduler = ParallelScheduler::new();
        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new(),
        );
        scheduler.build_schedule();

        assert_eq!(scheduler.wave_count(), 1);
        assert_eq!(scheduler.system_count(), 1);

        let mut world = UniversalWorld::new();
        scheduler.run_parallel(&mut world, 0.016);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_scheduler_parallel_independent() {
        reset_counter();
        let mut scheduler = ParallelScheduler::new();

        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new(),
        );
        scheduler.add_system(
            Box::new(RenderSystem),
            SystemDependency::new(),
        );
        scheduler.add_system(
            Box::new(AISystem),
            SystemDependency::new(),
        );

        scheduler.build_schedule();
        assert_eq!(scheduler.wave_count(), 1);

        let mut world = UniversalWorld::new();
        scheduler.run_parallel(&mut world, 0.016);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 111);
    }

    #[test]
    fn test_scheduler_sequential_dependency() {
        reset_counter();
        let mut scheduler = ParallelScheduler::new();

        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new().before("RenderSystem"),
        );
        scheduler.add_system(
            Box::new(RenderSystem),
            SystemDependency::new(),
        );

        scheduler.build_schedule();
        assert_eq!(scheduler.wave_count(), 2);

        let mut world = UniversalWorld::new();
        scheduler.run_parallel(&mut world, 0.016);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_scheduler_chain() {
        reset_counter();
        let mut scheduler = ParallelScheduler::new();

        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new().before("AISystem"),
        );
        scheduler.add_system(
            Box::new(AISystem),
            SystemDependency::new().before("RenderSystem"),
        );
        scheduler.add_system(
            Box::new(RenderSystem),
            SystemDependency::new(),
        );

        scheduler.build_schedule();
        assert_eq!(scheduler.wave_count(), 3);

        let mut world = UniversalWorld::new();
        scheduler.run_parallel(&mut world, 0.016);

        assert_eq!(COUNTER.load(Ordering::SeqCst), 111);
    }

    #[test]
    fn test_scheduler_wave_detection() {
        #[derive(Clone, Debug, PartialEq)]
        struct Health { hp: i32 }
        impl Component for Health {}

        let mut scheduler = ParallelScheduler::new();

        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new().reads::<Health>(),
        );
        scheduler.add_system(
            Box::new(RenderSystem),
            SystemDependency::new().writes::<Health>(),
        );
        scheduler.add_system(
            Box::new(AISystem),
            SystemDependency::new().reads::<Health>(),
        );

        scheduler.build_schedule();

        assert!(scheduler.wave_count() >= 2);
        assert!(scheduler.is_scheduled());
    }

    #[test]
    fn test_scheduler_clear() {
        let mut scheduler = ParallelScheduler::new();
        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new(),
        );
        assert_eq!(scheduler.system_count(), 1);

        scheduler.clear();
        assert_eq!(scheduler.system_count(), 0);
        assert!(!scheduler.is_scheduled());
    }

    #[test]
    fn test_scheduler_system_names() {
        let mut scheduler = ParallelScheduler::new();
        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new(),
        );
        scheduler.add_system(
            Box::new(RenderSystem),
            SystemDependency::new(),
        );

        assert_eq!(scheduler.system_name(0), Some("PosVelSystem"));
        assert_eq!(scheduler.system_name(1), Some("RenderSystem"));
        assert_eq!(scheduler.system_name(5), None);
    }

    #[test]
    fn test_scheduler_auto_build() {
        reset_counter();
        let mut scheduler = ParallelScheduler::new();
        scheduler.add_system(
            Box::new(PosVelSystem),
            SystemDependency::new(),
        );

        assert!(!scheduler.is_scheduled());

        let mut world = UniversalWorld::new();
        scheduler.run_parallel(&mut world, 0.016);

        assert!(scheduler.is_scheduled());
        assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
    }
}
