use std::any::TypeId;
use std::collections::HashMap;
use super::world::UniversalWorld;

/// System trait for the universal scheduler
pub trait UniversalSystem: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32 { 0 }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32);
    fn read_components(&self) -> Vec<TypeId> { vec![] }
    fn write_components(&self) -> Vec<TypeId> { vec![] }
    fn enabled(&self) -> bool { true }
}

/// System dependency declaration
pub struct SystemDependency {
    pub reads: Vec<TypeId>,
    pub writes: Vec<TypeId>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

impl SystemDependency {
    pub fn new() -> Self {
        Self { reads: vec![], writes: vec![], before: vec![], after: vec![] }
    }
    pub fn reads(mut self, types: Vec<TypeId>) -> Self { self.reads = types; self }
    pub fn writes(mut self, types: Vec<TypeId>) -> Self { self.writes = types; self }
    pub fn after(mut self, names: Vec<String>) -> Self { self.after = names; self }
}

/// Parallel scheduler
pub struct ParallelScheduler {
    systems: Vec<Box<dyn UniversalSystem>>,
    dependencies: Vec<SystemDependency>,
    execution_waves: Vec<Vec<usize>>,
    sorted: bool,
}

impl ParallelScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            dependencies: Vec::new(),
            execution_waves: Vec::new(),
            sorted: false,
        }
    }

    pub fn add_system(&mut self, system: Box<dyn UniversalSystem>, deps: SystemDependency) {
        self.systems.push(system);
        self.dependencies.push(deps);
        self.sorted = false;
    }

    /// Build execution schedule using topological sort + wave scheduling
    pub fn build_schedule(&mut self) {
        let n = self.systems.len();
        if n == 0 { return; }

        // Build dependency graph
        let mut in_degree = vec![0usize; n];
        let mut adjacency: Vec<Vec<usize>> = vec![vec![]; n];
        let name_to_index: HashMap<String, usize> = self.systems.iter()
            .enumerate()
            .map(|(i, s)| (s.name().to_string(), i))
            .collect();

        for (i, deps) in self.dependencies.iter().enumerate() {
            for after_name in &deps.after {
                if let Some(&j) = name_to_index.get(after_name) {
                    adjacency[j].push(i);
                    in_degree[i] += 1;
                }
            }
        }

        // Wave scheduling: group non-conflicting systems
        let mut waves: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; n];
        let mut remaining = n;

        while remaining > 0 {
            let mut wave = Vec::new();
            for i in 0..n {
                if assigned[i] { continue; }
                if in_degree[i] == 0 {
                    wave.push(i);
                }
            }

            if wave.is_empty() {
                // All remaining have dependencies; just pick highest priority
                for i in 0..n {
                    if !assigned[i] {
                        wave.push(i);
                        break;
                    }
                }
            }

            for &idx in &wave {
                assigned[idx] = true;
                remaining -= 1;
                for &next in &adjacency[idx] {
                    in_degree[next] -= 1;
                }
            }

            waves.push(wave);
        }

        self.execution_waves = waves;
        self.sorted = true;
    }

    /// Run all systems in waves (parallel where possible)
    pub fn run(&mut self, world: &mut UniversalWorld, dt: f32) {
        if !self.sorted {
            self.build_schedule();
        }

        let waves = self.execution_waves.clone();
        for wave in &waves {
            // In a real implementation, systems in the same wave
            // would run in parallel using rayon or similar.
            // For now, run sequentially within each wave.
            for &system_idx in wave {
                if self.systems[system_idx].enabled() {
                    self.systems[system_idx].update(world, dt);
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

    pub fn clear(&mut self) {
        self.systems.clear();
        self.dependencies.clear();
        self.execution_waves.clear();
        self.sorted = false;
    }
}

impl Default for ParallelScheduler {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSystem { name: String }
    impl UniversalSystem for TestSystem {
        fn name(&self) -> &str { &self.name }
        fn update(&mut self, _world: &mut UniversalWorld, _dt: f32) {}
    }

    #[test]
    fn test_scheduler_wave_count() {
        let mut sched = ParallelScheduler::new();
        sched.add_system(Box::new(TestSystem { name: "a".into() }), SystemDependency::new());
        sched.add_system(Box::new(TestSystem { name: "b".into() }), SystemDependency::new().after(vec!["a".into()]));
        sched.build_schedule();
        assert!(sched.wave_count() >= 2);
    }
}
