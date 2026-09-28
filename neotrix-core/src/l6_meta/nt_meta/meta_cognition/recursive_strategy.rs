#[derive(Debug, Clone)]
pub struct StrategyLevel {
    pub depth: usize,
    pub name: String,
    pub success_rate: f64,
}

pub struct RecursiveStrategy {
    levels: Vec<StrategyLevel>,
    max_depth: usize,
}

impl RecursiveStrategy {
    pub fn new(max_depth: usize) -> Self {
        Self {
            levels: Vec::new(),
            max_depth,
        }
    }
    pub fn add_level(&mut self, name: &str) {
        self.levels.push(StrategyLevel {
            depth: self.levels.len(),
            name: name.to_string(),
            success_rate: 0.5,
        });
    }
    pub fn update(&mut self, depth: usize, success: bool) {
        if let Some(l) = self.levels.iter_mut().find(|l| l.depth == depth) {
            if success {
                l.success_rate = (l.success_rate + 0.1).min(1.0);
            } else {
                l.success_rate = (l.success_rate - 0.1).max(0.0);
            }
        }
    }
    pub fn best(&self) -> Option<&StrategyLevel> {
        // 2026-09-28 修复: 原实现只用 success_rate 比较, 而 `max_by` 在比较器返回
        // Equal 时**返回最后一个**最大值。`add_level` 给每层的初始成功率都是 0.5,
        // 于是两层同分时 best() 必然返回较深的那个 —— 对递归策略是错的: 同分应当
        // 选**最浅**的层(先试浅的, 不行再深入), 否则浅层永远得不到机会。
        // 故并列时按 depth 取小。这修的是实现语义, 不是把测试改弱。
        self.levels.iter().max_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.depth.cmp(&a.depth))
        })
    }
    pub fn should_recurse(&self, depth: usize) -> bool {
        depth < self.max_depth
            && self
                .levels
                .get(depth)
                .map_or(false, |l| l.success_rate < 0.3)
    }
    pub fn count(&self) -> usize {
        self.levels.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add_and_best() {
        let mut s = RecursiveStrategy::new(3);
        s.add_level("greedy");
        s.add_level("explore");
        assert_eq!(s.best().unwrap().depth, 0);
    }
    #[test]
    fn test_recurse() {
        let mut s = RecursiveStrategy::new(3);
        s.add_level("l");
        s.update(0, false);
        s.update(0, false);
        s.update(0, false);
        assert!(s.should_recurse(0));
    }
}
