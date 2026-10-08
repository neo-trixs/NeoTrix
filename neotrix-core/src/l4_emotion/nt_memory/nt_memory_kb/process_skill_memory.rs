#[derive(Debug, Clone)]
pub struct ProcessSkill {
    pub name: String,
    pub steps: Vec<String>,
    pub success_count: u32,
    pub fail_count: u32,
    pub avg_duration_ms: u64,
}

pub struct ProcessSkillMemory {
    skills: Vec<ProcessSkill>,
}

impl Default for ProcessSkillMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessSkillMemory {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn register(&mut self, name: &str, steps: Vec<String>) {
        self.skills.push(ProcessSkill {
            name: name.to_string(),
            steps,
            success_count: 0,
            fail_count: 0,
            avg_duration_ms: 0,
        });
    }

    pub fn record_success(&mut self, name: &str, duration_ms: u64) {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.success_count += 1;
            let n = s.success_count as f64;
            s.avg_duration_ms =
                (s.avg_duration_ms as f64 * ((n - 1.0) / n) + duration_ms as f64 / n) as u64;
        }
    }

    pub fn record_failure(&mut self, name: &str) {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.fail_count += 1;
        }
    }

    pub fn success_rate(&self, name: &str) -> f64 {
        self.skills
            .iter()
            .find(|s| s.name == name)
            .map_or(0.0, |s| {
                let total = s.success_count + s.fail_count;
                if total == 0 {
                    0.0
                } else {
                    s.success_count as f64 / total as f64
                }
            })
    }

    /// 成功率最高的那条流程记忆。
    ///
    /// ⚠️ 此处曾编译不过（E0599 `no method named success_rate`）：
    ///   原实现写成 `a.success_rate()`，把 `&ProcessSkill` 当成
    ///   `ProcessSkillMemory` 调用 —— 但 `success_rate(&self, name: &str)`
    ///   是**按名字查**的方法，签名里要 `name`。
    ///   ⇒ `a` 是单个 skill，没有「按名字查自己」这种语义。
    ///
    /// ✅ 正确做法：直接用该 skill **自己的**计数字段算成功率。
    ///   把「按名查询」与「就地计算」分开，两个语义都不再混淆。
    pub fn best_skill(&self) -> Option<&ProcessSkill> {
        fn rate(s: &ProcessSkill) -> f64 {
            let total = s.success_count + s.fail_count;
            if total == 0 {
                0.0
            } else {
                s.success_count as f64 / total as f64
            }
        }
        self.skills.iter().max_by(|a, b| {
            rate(a)
                .partial_cmp(&rate(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn skills(&self) -> &[ProcessSkill] {
        &self.skills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `best_skill` 的回归防护。
    ///
    /// 它曾编译不过（把 `&ProcessSkill` 当 `ProcessSkillMemory` 调
    /// `success_rate()`）。修法是引入局部 `rate(&ProcessSkill)`。
    /// ⇒ 本测试锁定「排序依据 = 该 skill 自己的成功率」，
    ///   并覆盖**全零成功率**的边界（`max_by` 在全相等时的行为）。
    #[test]
    fn best_skill_picks_highest_success_rate() {
        let mut m = ProcessSkillMemory::new();
        m.register("bad", vec![]);
        m.register("good", vec![]);
        m.record_failure("bad");
        m.record_success("good", 100);
        m.record_success("good", 100);
        m.record_failure("good");
        let best = m.best_skill().expect("至少有一条");
        assert_eq!(best.name, "good", "成功率 2/3 应胜出 0/1");
    }

    /// 空表 ⇒ `None`（不能 panic，调用方多半直接 unwrap_or_default）。
    #[test]
    fn best_skill_on_empty_is_none() {
        assert!(ProcessSkillMemory::new().best_skill().is_none());
    }

    /// 全零成功率时不得 panic，且必须仍返回某条（不能因为「无法比较」而空）。
    #[test]
    fn best_skill_with_all_zero_rates_does_not_panic() {
        let mut m = ProcessSkillMemory::new();
        m.register("a", vec![]);
        m.register("b", vec![]);
        // 二者成功率都是 0.0 ⇒ partial_cmp 返回 Some(Equal)
        let best = m.best_skill().expect("全零也应返回一条");
        assert!(best.name == "a" || best.name == "b");
    }

    /// 未注册的技能名 ⇒ 成功率 0.0（不是 panic）。
    #[test]
    fn success_rate_of_unknown_skill_is_zero() {
        let m = ProcessSkillMemory::new();
        assert_eq!(m.success_rate("nope"), 0.0);
    }

    #[test]
    fn test_register_and_record() {
        let mut m = ProcessSkillMemory::new();
        m.register("deploy", vec!["build".into(), "test".into()]);
        m.record_success("deploy", 5000);
        m.record_success("deploy", 3000);
        assert_eq!(m.success_rate("deploy"), 1.0);
    }

    #[test]
    fn test_failure() {
        let mut m = ProcessSkillMemory::new();
        m.register("s", vec![]);
        m.record_success("s", 100);
        m.record_failure("s");
        assert_eq!(m.success_rate("s"), 0.5);
    }
}
