use super::types::{Skill, SkillMeta, SkillOutput};
use super::registry::{SkillDiscovery, SkillInjector};
use crate::CapabilityVector;

// ==============================
//  3. Executor — 执行引擎
// ==============================

pub struct SkillExecutor {
    /// 缓存最近执行结果
    history: Vec<SkillOutput>,
}

impl Default for SkillExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillExecutor {
    pub fn new() -> Self {
        Self { history: Vec::new() }
    }

    /// 执行 Skill
    pub fn execute(&mut self, skill: &Skill, args: &[(&str, &str)]) -> SkillOutput {
        let start = std::time::Instant::now();

        let output = format!(
            "Executing skill '{}' with {} args",
            skill.meta.name,
            args.len()
        );

        let elapsed = start.elapsed().as_millis() as u64;
        let confidence_delta = 0.05; // 每次成功执行小幅提升

        let result = SkillOutput {
            skill_name: skill.meta.name.clone(),
            success: true,
            output,
            execution_time_ms: elapsed,
            confidence_delta,
        };

        self.history.push(result.clone());
        result
    }

    /// 执行 Skill 并更新置信度（ECC 持续学习模式）
    pub fn execute_with_stats(&mut self, skill: &mut Skill, args: &[(&str, &str)]) -> SkillOutput {
        let start = std::time::Instant::now();

        let output = format!(
            "Executing skill '{}' with {} args",
            skill.meta.name,
            args.len()
        );

        let elapsed = start.elapsed().as_millis() as u64;
        skill.update_confidence(true, elapsed);

        let result = SkillOutput {
            skill_name: skill.meta.name.clone(),
            success: true,
            output,
            execution_time_ms: elapsed,
            confidence_delta: skill.stats.confidence,
        };

        self.history.push(result.clone());
        result
    }

    pub fn history(&self) -> &[SkillOutput] {
        &self.history
    }
}

// ==============================
//  4. Activation — 条件激活
// ==============================

pub struct SkillActivator {
    /// 激活记录
    active: Vec<String>,
}

impl Default for SkillActivator {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillActivator {
    pub fn new() -> Self {
        Self { active: Vec::new() }
    }

    /// 根据任务描述自动选择激活的技能
    pub fn activate_for_task<'a>(&mut self, task: &str, all_skills: &'a [&'a Skill]) -> Vec<&'a Skill> {
        let task_lower = task.to_lowercase();
        let mut matched: Vec<&Skill> = Vec::new();

        for skill in all_skills {
            let keyword_match = skill.meta.triggers.iter().any(|t| task_lower.contains(&t.to_lowercase()));
            let condition_match = match &skill.meta.condition {
                Some(cond) => self.evaluate_condition(cond, &task_lower),
                None => true,
            };
            let confidence_ok = skill.stats.confidence >= 0.1;
            // ATT&CK 技术过滤: 任务匹配 attack ID 则自动激活 (Decepticon 风格)
            let mitre_match = skill.meta.mitre_attack_ids.iter()
                .any(|id| task_lower.contains(&id.to_lowercase()));

            if (keyword_match || mitre_match) && condition_match && confidence_ok {
                matched.push(skill);
                if !self.active.contains(&skill.meta.name) {
                    self.active.push(skill.meta.name.clone());
                }
            }
        }

        matched.sort_by(|a, b| b.stats.confidence.partial_cmp(&a.stats.confidence).unwrap_or(std::cmp::Ordering::Equal));
        matched
    }

    /// 按 ATT&CK 技术 ID 过滤技能 (Decepticon 风格)
    pub fn filter_by_mitre<'a>(technique_ids: &[&str], all_skills: &'a [&'a Skill]) -> Vec<&'a Skill> {
        all_skills.iter()
            .filter(|s| s.meta.mitre_attack_ids.iter().any(|id| technique_ids.contains(&id.as_str())))
            .copied()
            .collect()
    }

    /// 评估条件表达式（ECC 兼容格式）
    ///
    /// 格式: `key:value` 或 `key=value`
    /// 示例: `filetype:rust`, `task=design`, `lang:python`
    fn evaluate_condition(&self, condition: &str, task: &str) -> bool {
        let cond = condition.trim();
        if cond.is_empty() { return true; }

        // 支持 `key:value` 和 `key=value` 格式
        let parts: Vec<&str> = if cond.contains(':') {
            cond.splitn(2, ':').collect()
        } else if cond.contains('=') {
            cond.splitn(2, '=').collect()
        } else {
            // 纯关键词条件：直接匹配
            return task.contains(&cond.to_lowercase());
        };

        if parts.len() == 2 {
            let key = parts[0].trim().to_lowercase();
            let val = parts[1].trim().to_lowercase();
            match key.as_str() {
                "filetype" | "lang" => task.contains(&val),
                "task" | "type" => task.contains(&val),
                "framework" => task.contains(&val),
                _ => task.contains(&val), // fallback: 值作为关键词
            }
        } else {
            true
        }
    }

    pub fn active_skills(&self) -> &[String] {
        &self.active
    }

    pub fn deactivate(&mut self, name: &str) {
        self.active.retain(|n| n != name);
    }
}

// ==============================
//  5. Skills 引擎（统一入口）
// ==============================

pub struct SkillsEngine {
    pub discovery: SkillDiscovery,
    pub executor: SkillExecutor,
    pub activator: SkillActivator,
}

impl SkillsEngine {
    pub fn new() -> Self {
        Self {
            discovery: SkillDiscovery::new(),
            executor: SkillExecutor::new(),
            activator: SkillActivator::new(),
        }
    }

    /// 初始化：发现本地 Skills
    pub fn init(&mut self) -> Vec<String> {
        self.discovery.discover_local()
    }

    /// 为任务激活 Skills + 注入 Prompt
    pub fn prepare_prompt(&mut self, task: &str, base_prompt: &str) -> String {
        let all = self.discovery.list();
        let activated = self.activator.activate_for_task(task, &all);
        SkillInjector::inject(&activated, base_prompt)
    }

    pub fn find_relevant_skills(&self, task_vector: &CapabilityVector, top_k: usize) -> Vec<(SkillMeta, f64)> {
        let mut scored: Vec<(SkillMeta, f64)> = Vec::new();
        for skill in self.discovery.list() {
            let keyword_score = self.calculate_keyword_match(&skill.meta, task_vector);
            scored.push((skill.meta.clone(), keyword_score));
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).collect()
    }

    fn calculate_keyword_match(&self, meta: &SkillMeta, task_vector: &CapabilityVector) -> f64 {
        if task_vector.arr().is_empty() {
            return 0.0;
        }
        let mut score: f64 = 0.0;
        let keywords = ["code", "design", "security", "test", "deploy", "data", "web", "api", "ui", "agent"];
        let high_dims: Vec<&str> = task_vector.arr().iter()
            .enumerate()
            .filter(|(_, &v)| v > 0.7)
            .map(|(i, _)| crate::core::nt_core_cap::FIELD_NAMES.get(i).copied().unwrap_or("unknown"))
            .collect();
        for dim in &high_dims {
            if meta.name.to_lowercase().contains(dim) || meta.description.to_lowercase().contains(dim) {
                score += 0.3;
            }
        }
        for kw in &keywords {
            if meta.name.contains(kw) || meta.description.contains(*kw) {
                score += 0.1;
            }
        }
        score.min(1.0)
    }

    pub fn prioritized_skills(&self, task_vector: &CapabilityVector) -> Vec<(SkillMeta, f64)> {
        self.find_relevant_skills(task_vector, self.discovery.len())
    }

    pub fn activate_best_skill(&self, task_vector: &CapabilityVector) -> Option<SkillMeta> {
        let results = self.find_relevant_skills(task_vector, 1);
        results.into_iter().next().map(|(meta, _)| meta)
    }
}

impl Default for SkillsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::skills::types::*;

    fn make_skill(name: &str) -> Skill {
        Skill::new(
            SkillMeta {
                name: name.into(), description: format!("{} description", name),
                version: "1.0".into(), author: None, origin: None,
                triggers: vec!["test".into()], condition: None,
                requires_tools: vec![], requires_capabilities: vec![],
                mitre_attack_ids: vec![],
            },
            SkillSource::LocalDir("./skills".into()),
            "content".into(),
            "prompt".into(),
        )
    }

    #[test]
    fn test_skill_executor_new() {
        let ex = SkillExecutor::new();
        assert!(ex.history().is_empty());
    }

    #[test]
    fn test_skill_executor_execute() {
        let mut ex = SkillExecutor::new();
        let skill = make_skill("test-skill");
        let output = ex.execute(&skill, &[("arg1", "val1")]);
        assert!(output.success);
        assert_eq!(output.skill_name, "test-skill");
    }

    #[test]
    fn test_skill_executor_history() {
        let mut ex = SkillExecutor::new();
        let skill = make_skill("s1");
        ex.execute(&skill, &[]);
        assert_eq!(ex.history().len(), 1);
    }

    #[test]
    fn test_skill_executor_execute_with_stats() {
        let mut ex = SkillExecutor::new();
        let mut skill = make_skill("s2");
        let output = ex.execute_with_stats(&mut skill, &[("a", "b")]);
        assert!(output.success);
        assert!(skill.stats.use_count > 0);
    }

    #[test]
    fn test_skill_activator_new() {
        let act = SkillActivator::new();
        assert!(act.active_skills().is_empty());
    }

    #[test]
    fn test_skill_activator_activate_for_task() {
        let mut act = SkillActivator::new();
        let skill = make_skill("testing");
        let all = vec![&skill];
        let matched = act.activate_for_task("run test", &all);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_skill_activator_no_match() {
        let mut act = SkillActivator::new();
        let skill = Skill::new(
            SkillMeta {
                name: "nosuchskill".into(), description: "Niche skill".into(),
                version: "1.0".into(), author: None, origin: None,
                triggers: vec!["xyznonexistent".into()], condition: None,
                requires_tools: vec![], requires_capabilities: vec![],
                mitre_attack_ids: vec![],
            },
            SkillSource::LocalDir("./skills".into()),
            "content".into(),
            "prompt".into(),
        );
        let skill_ref = &skill;
        let all = vec![skill_ref];
        let matched = act.activate_for_task("testing code without any matching trigger", &all);
        assert!(matched.is_empty());
    }

    #[test]
    fn test_skill_activator_deactivate() {
        let mut act = SkillActivator::new();
        let skill = Skill::new(
            SkillMeta {
                name: "x-skill".into(), description: "x skill".into(),
                version: "1.0".into(), author: None, origin: None,
                triggers: vec!["test".into(), "x".into()], condition: None,
                requires_tools: vec![], requires_capabilities: vec![],
                mitre_attack_ids: vec![],
            },
            SkillSource::LocalDir("./skills".into()),
            "content".into(),
            "prompt".into(),
        );
        let skill_ref = &skill;
        let all = vec![skill_ref];
        act.activate_for_task("test x", &all);
        assert_eq!(act.active_skills().len(), 1);
        act.deactivate("x-skill");
        assert!(act.active_skills().is_empty());
    }

    #[test]
    fn test_evaluate_condition_keyword() {
        let act = SkillActivator::new();
        assert!(act.evaluate_condition("rust", "write rust code"));
        assert!(!act.evaluate_condition("python", "write rust code"));
    }

    #[test]
    fn test_evaluate_condition_key_value() {
        let act = SkillActivator::new();
        assert!(act.evaluate_condition("filetype:rust", "rust code"));
        assert!(!act.evaluate_condition("filetype:python", "rust code"));
    }

    #[test]
    fn test_evaluate_condition_empty() {
        let act = SkillActivator::new();
        assert!(act.evaluate_condition("", "anything"));
    }

    #[test]
    fn test_filter_by_mitre() {
        let mut s = make_skill("secure");
        s.meta.mitre_attack_ids = vec!["T1595".into()];
        let all = vec![&s];
        let filtered = SkillActivator::filter_by_mitre(&["T1595"], &all);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_skills_engine_new() {
        let engine = SkillsEngine::new();
        assert!(engine.discovery.list().is_empty());
    }
}
