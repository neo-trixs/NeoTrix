#![allow(dead_code)]

use std::collections::HashMap;

/// A registered skill
#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub version: u32,
    pub description: String,
    pub category: String,
    pub invocation_count: u64,
    pub avg_success_rate: f64,
}

/// Skill registry — manage reusable capabilities
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillEntry>,
    pub max_skills: usize,
}

impl SkillRegistry {
    pub fn new() -> Self {
        SkillRegistry {
            skills: HashMap::new(),
            max_skills: 500,
        }
    }

    pub fn register(
        &mut self,
        name: &str,
        version: u32,
        description: &str,
        category: &str,
    ) -> bool {
        if self.skills.len() >= self.max_skills {
            return false;
        }
        self.skills.insert(
            name.into(),
            SkillEntry {
                name: name.into(),
                version,
                description: description.into(),
                category: category.into(),
                invocation_count: 0,
                avg_success_rate: 1.0,
            },
        );
        true
    }

    pub fn get(&self, name: &str) -> Option<&SkillEntry> {
        self.skills.get(name)
    }

    pub fn invoke(&mut self, name: &str, success: bool) -> Option<f64> {
        let entry = self.skills.get_mut(name)?;
        entry.invocation_count += 1;
        let total = entry.invocation_count as f64;
        entry.avg_success_rate = entry.avg_success_rate * ((total - 1.0) / total)
            + if success { 1.0 / total } else { 0.0 };
        Some(entry.avg_success_rate)
    }

    pub fn by_category(&self, category: &str) -> Vec<&SkillEntry> {
        self.skills
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    pub fn most_used(&self, n: usize) -> Vec<&SkillEntry> {
        let mut entries: Vec<&SkillEntry> = self.skills.values().collect();
        entries.sort_unstable_by(|a, b| b.invocation_count.cmp(&a.invocation_count));
        entries.into_iter().take(n).collect()
    }

    pub fn best_performing(&self, n: usize) -> Vec<&SkillEntry> {
        let mut entries: Vec<&SkillEntry> = self.skills.values().collect();
        entries
            .sort_unstable_by(|a, b| b.avg_success_rate.partial_cmp(&a.avg_success_rate).unwrap_or(std::cmp::Ordering::Equal));
        entries.into_iter().take(n).collect()
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        self.skills.remove(name).is_some()
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }

    pub fn report(&self) -> String {
        let total = self.skills.len();
        let categories: std::collections::HashSet<String> =
            self.skills.values().map(|s| s.category.clone()).collect();
        format!(
            "SkillRegistry: {} skills in {} categories",
            total,
            categories.len()
        )
    }
}

/// Dependency edge between skills
#[derive(Debug, Clone)]
pub struct SkillDependency {
    pub skill_name: String,
    pub depends_on: Vec<String>,
    pub optional: bool,
}

/// Versioned invocation statistics for a skill
#[derive(Debug, Clone)]
pub struct SkillVersion {
    pub skill_name: String,
    pub version: u32,
    pub invocation_count: u64,
    pub success_count: u64,
    pub avg_duration_ms: f64,
}

impl SkillVersion {
    pub fn success_rate(&self) -> f64 {
        if self.invocation_count == 0 {
            return 1.0;
        }
        self.success_count as f64 / self.invocation_count as f64
    }
}

/// Orchestrates multi-skill execution with dependency resolution
#[derive(Debug, Clone)]
pub struct SkillOrchestrator {
    pub skills: HashMap<String, SkillEntry>,
    pub versions: HashMap<String, SkillVersion>,
    pub dependencies: HashMap<String, SkillDependency>,
    execution_plan_cache: HashMap<String, Vec<String>>,
}

impl SkillOrchestrator {
    pub fn new() -> Self {
        SkillOrchestrator {
            skills: HashMap::new(),
            versions: HashMap::new(),
            dependencies: HashMap::new(),
            execution_plan_cache: HashMap::new(),
        }
    }

    pub fn register_skill(&mut self, name: &str, description: &str, tags: Vec<&str>, version: u32) {
        self.skills.entry(name.into()).or_insert(SkillEntry {
            name: name.into(),
            version,
            description: description.into(),
            category: tags.into_iter().collect::<Vec<_>>().join(","),
            invocation_count: 0,
            avg_success_rate: 1.0,
        });
        self.versions.entry(name.into()).or_insert(SkillVersion {
            skill_name: name.into(),
            version,
            invocation_count: 0,
            success_count: 0,
            avg_duration_ms: 0.0,
        });
    }

    pub fn register_dependency(&mut self, skill: &str, depends_on: &[&str], optional: bool) {
        self.dependencies.insert(
            skill.into(),
            SkillDependency {
                skill_name: skill.into(),
                depends_on: depends_on.iter().map(|s| s.to_string()).collect(),
                optional,
            },
        );
    }

    pub fn resolve_execution_plan(&mut self, skill: &str) -> Result<Vec<String>, String> {
        if let Some(cached) = self.execution_plan_cache.get(skill) {
            return Ok(cached.clone());
        }

        let mut visited: HashMap<String, usize> = HashMap::new();
        let mut plan: Vec<String> = Vec::new();

        self.visit(skill, &mut visited, &mut plan, &mut Vec::new())?;

        self.execution_plan_cache.insert(skill.into(), plan.clone());
        Ok(plan)
    }

    fn visit(
        &self,
        current: &str,
        visited: &mut HashMap<String, usize>,
        plan: &mut Vec<String>,
        stack: &mut Vec<String>,
    ) -> Result<(), String> {
        match visited.get(current) {
            Some(&0) => {
                return Err(format!(
                    "Circular dependency: {} -> {}",
                    stack.join(" -> "),
                    current
                ))
            }
            Some(&1) => return Ok(()),
            _ => {}
        }

        if !self.skills.contains_key(current) {
            return Err(format!("Skill '{}' not found in registry", current));
        }

        visited.insert(current.into(), 0);
        stack.push(current.into());

        if let Some(dep) = self.dependencies.get(current) {
            for dep_skill in &dep.depends_on {
                if !dep.optional && !self.skills.contains_key(dep_skill) {
                    stack.pop();
                    return Err(format!(
                        "Required dependency '{}' for '{}' not found",
                        dep_skill, current
                    ));
                }
                if self.skills.contains_key(dep_skill) {
                    self.visit(dep_skill, visited, plan, stack)?;
                }
            }
        }

        stack.pop();
        visited.insert(current.into(), 1);
        plan.push(current.into());
        Ok(())
    }

    pub fn record_invocation(&mut self, skill: &str, success: bool, duration_ms: f64) {
        if let Some(ver) = self.versions.get_mut(skill) {
            ver.invocation_count += 1;
            if success {
                ver.success_count += 1;
            }
            let total = ver.invocation_count as f64;
            ver.avg_duration_ms =
                ver.avg_duration_ms * ((total - 1.0) / total) + duration_ms / total;
        }
        if let Some(entry) = self.skills.get_mut(skill) {
            entry.invocation_count += 1;
            let total = entry.invocation_count as f64;
            entry.avg_success_rate = entry.avg_success_rate * ((total - 1.0) / total)
                + if success { 1.0 / total } else { 0.0 };
        }
    }

    pub fn most_reliable_skills(&self, n: usize) -> Vec<(&str, f64)> {
        let mut list: Vec<(&str, f64)> = self
            .versions
            .values()
            .map(|v| (v.skill_name.as_str(), v.success_rate()))
            .collect();
        list.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        list.into_iter().take(n).collect()
    }

    pub fn least_reliable_skills(&self, n: usize) -> Vec<(&str, f64)> {
        let mut list: Vec<(&str, f64)> = self
            .versions
            .values()
            .filter(|v| v.invocation_count > 0)
            .map(|v| (v.skill_name.as_str(), v.success_rate()))
            .collect();
        list.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        list.into_iter().take(n).collect()
    }

    pub fn dependency_depth(&self, skill: &str) -> usize {
        fn depth_of(
            name: &str,
            deps: &HashMap<String, SkillDependency>,
            visited: &mut Vec<String>,
        ) -> usize {
            if visited.contains(&name.into()) {
                return 0;
            }
            visited.push(name.into());
            let mut max_d = 0;
            if let Some(dep) = deps.get(name) {
                for d in &dep.depends_on {
                    let d = depth_of(d, deps, visited);
                    if d > max_d {
                        max_d = d;
                    }
                }
            }
            max_d + 1
        }
        depth_of(skill, &self.dependencies, &mut Vec::new())
    }

    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    pub fn total_invocations(&self) -> u64 {
        self.versions.values().map(|v| v.invocation_count).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut r = SkillRegistry::new();
        assert!(r.register("code_review", 1, "Review code", "engineering"));
        assert!(r.get("code_review").is_some());
    }

    #[test]
    fn test_invoke_tracks_count() {
        let mut r = SkillRegistry::new();
        r.register("test", 1, "test", "t");
        r.invoke("test", true);
        r.invoke("test", true);
        assert_eq!(r.get("test").unwrap().invocation_count, 2);
    }

    #[test]
    fn test_by_category() {
        let mut r = SkillRegistry::new();
        r.register("a", 1, "a", "cat1");
        r.register("b", 1, "b", "cat2");
        r.register("c", 1, "c", "cat1");
        assert_eq!(r.by_category("cat1").len(), 2);
    }

    #[test]
    fn test_most_used_ordering() {
        let mut r = SkillRegistry::new();
        r.register("a", 1, "a", "t");
        r.register("b", 1, "b", "t");
        r.invoke("b", true);
        r.invoke("b", true);
        r.invoke("a", true);
        let top = r.most_used(1);
        assert_eq!(top[0].name, "b");
    }

    #[test]
    fn test_unregister() {
        let mut r = SkillRegistry::new();
        r.register("a", 1, "a", "t");
        assert!(r.unregister("a"));
        assert!(r.get("a").is_none());
    }

    #[test]
    fn test_count() {
        let mut r = SkillRegistry::new();
        r.register("a", 1, "a", "t");
        r.register("b", 1, "b", "t");
        assert_eq!(r.count(), 2);
    }

    #[test]
    fn test_report() {
        let mut r = SkillRegistry::new();
        r.register("a", 1, "a", "t");
        let rep = r.report();
        assert!(rep.contains("SkillRegistry"));
    }

    // --- SkillOrchestrator tests ---

    #[test]
    fn test_register_skill() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("scan", "Network scan", vec!["security", "recon"], 1);
        assert_eq!(o.skill_count(), 1);
        assert!(o.skills.contains_key("scan"));
        assert!(o.versions.contains_key("scan"));
    }

    #[test]
    fn test_register_dependency() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("recon", "Recon", vec!["intel"], 1);
        o.register_skill("scan", "Scan", vec!["security"], 1);
        o.register_dependency("scan", &["recon"], false);
        assert!(o.dependencies.contains_key("scan"));
        assert_eq!(o.dependencies["scan"].depends_on, vec!["recon"]);
    }

    #[test]
    fn test_resolve_linear_plan() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("osint", "OSINT gather", vec!["intel"], 1);
        o.register_skill("analyze", "Analyze", vec!["core"], 2);
        o.register_skill("report", "Generate report", vec!["output"], 3);
        o.register_dependency("analyze", &["osint"], false);
        o.register_dependency("report", &["analyze"], false);
        let plan = o.resolve_execution_plan("report").unwrap();
        assert_eq!(plan, vec!["osint", "analyze", "report"]);
    }

    #[test]
    fn test_resolve_circular_dep_error() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("a", "A", vec!["x"], 1);
        o.register_skill("b", "B", vec!["x"], 1);
        o.register_skill("c", "C", vec!["x"], 1);
        o.register_dependency("a", &["b"], false);
        o.register_dependency("b", &["c"], false);
        o.register_dependency("c", &["a"], false);
        let result = o.resolve_execution_plan("a");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular"));
    }

    #[test]
    fn test_record_invocation() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("test", "Test", vec!["qa"], 1);
        o.record_invocation("test", true, 12.5);
        o.record_invocation("test", false, 8.0);
        let v = &o.versions["test"];
        assert_eq!(v.invocation_count, 2);
        assert_eq!(v.success_count, 1);
        assert!(v.avg_duration_ms > 0.0);
    }

    #[test]
    fn test_most_reliable_skills() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("good", "Good", vec![], 1);
        o.register_skill("bad", "Bad", vec![], 1);
        o.record_invocation("good", true, 1.0);
        o.record_invocation("good", true, 1.0);
        o.record_invocation("bad", false, 1.0);
        let top = o.most_reliable_skills(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0, "good");
        assert!((top[0].1 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_least_reliable_skills() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("good", "Good", vec![], 1);
        o.register_skill("bad", "Bad", vec![], 1);
        o.record_invocation("good", true, 1.0);
        o.record_invocation("bad", false, 1.0);
        o.record_invocation("bad", false, 1.0);
        let bottom = o.least_reliable_skills(1);
        assert_eq!(bottom.len(), 1);
        assert_eq!(bottom[0].0, "bad");
        assert!((bottom[0].1 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_dependency_depth() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("a", "A", vec![], 1);
        o.register_skill("b", "B", vec![], 1);
        o.register_skill("c", "C", vec![], 1);
        o.register_skill("d", "D", vec![], 1);
        o.register_dependency("a", &["b"], false);
        o.register_dependency("b", &["c"], false);
        o.register_dependency("c", &["d"], false);
        assert_eq!(o.dependency_depth("a"), 4);
        assert_eq!(o.dependency_depth("b"), 3);
        assert_eq!(o.dependency_depth("d"), 1);
    }

    #[test]
    fn test_skill_count() {
        let mut o = SkillOrchestrator::new();
        assert_eq!(o.skill_count(), 0);
        o.register_skill("a", "A", vec![], 1);
        o.register_skill("b", "B", vec![], 1);
        assert_eq!(o.skill_count(), 2);
    }

    #[test]
    fn test_total_invocations() {
        let mut o = SkillOrchestrator::new();
        o.register_skill("x", "X", vec![], 1);
        o.register_skill("y", "Y", vec![], 1);
        assert_eq!(o.total_invocations(), 0);
        o.record_invocation("x", true, 1.0);
        o.record_invocation("x", true, 1.0);
        o.record_invocation("y", true, 1.0);
        assert_eq!(o.total_invocations(), 3);
    }
}
