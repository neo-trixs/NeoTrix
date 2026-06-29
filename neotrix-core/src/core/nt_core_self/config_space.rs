#[derive(Debug, Clone)]
pub struct ConfigParam {
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub is_categorical: bool,
    pub categories: Vec<String>,
}

impl ConfigParam {
    pub fn normalized(&self) -> f64 {
        if self.is_categorical {
            if self.categories.is_empty() {
                return 0.0;
            }
            let idx = self
                .categories
                .iter()
                .position(|c| c == &self.to_category_label())
                .unwrap_or(0);
            idx as f64 / (self.categories.len() - 1).max(1) as f64
        } else {
            if self.max <= self.min {
                return 0.0;
            }
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }

    fn to_category_label(&self) -> String {
        let idx = (self.value.round() as usize).min(self.categories.len().saturating_sub(1));
        self.categories.get(idx).cloned().unwrap_or_default()
    }

    pub fn random_value(&self, seed: u64) -> f64 {
        let mut rng = Lcg::new(seed);
        if self.is_categorical && !self.categories.is_empty() {
            let idx = (rng.next_f64() * self.categories.len() as f64).floor() as usize;
            let idx = idx.min(self.categories.len() - 1);
            idx as f64
        } else {
            rng.next_f64_range(self.min, self.max)
        }
    }

    pub fn clamp(&self, v: f64) -> f64 {
        if self.is_categorical {
            let len = self.categories.len().max(1);
            let idx = (v.round() as usize).min(len - 1);
            idx as f64
        } else {
            v.clamp(self.min, self.max)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigPoint {
    pub params: Vec<ConfigParam>,
}

impl ConfigPoint {
    pub fn new() -> Self {
        ConfigPoint { params: Vec::new() }
    }

    pub fn add(&mut self, param: ConfigParam) {
        self.params.push(param);
    }

    pub fn get(&self, name: &str) -> Option<&ConfigParam> {
        self.params.iter().find(|p| p.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut ConfigParam> {
        self.params.iter_mut().find(|p| p.name == name)
    }

    pub fn to_feature_vector(&self) -> Vec<f64> {
        self.params.iter().map(|p| p.normalized()).collect()
    }

    pub fn distance(&self, other: &ConfigPoint) -> f64 {
        let a = self.to_feature_vector();
        let b = other.to_feature_vector();
        let sum: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();
        sum.sqrt()
    }

    pub fn random_neighbor(&self, seed: u64) -> ConfigPoint {
        let mut rng = Lcg::new(seed);
        let mut result = self.clone();
        if result.params.is_empty() {
            return result;
        }
        let idx = (rng.next_f64() * result.params.len() as f64).floor() as usize;
        let idx = idx.min(result.params.len() - 1);
        let param = &result.params[idx];
        let new_val = if param.is_categorical {
            let mut cat_idx = (param.value.round() as isize)
                .max(0)
                .min(param.categories.len() as isize - 1) as usize;
            let offset: isize = if rng.next_f64() < 0.5 { 1 } else { -1 };
            let next = cat_idx as isize + offset;
            if next < 0 || next >= param.categories.len() as isize {
                cat_idx = (rng.next_f64() * param.categories.len() as f64).floor() as usize;
            } else {
                cat_idx = next as usize;
            }
            cat_idx.min(param.categories.len() - 1) as f64
        } else {
            let delta = param.step * if rng.next_f64() < 0.5 { 1.0 } else { -1.0 };
            (param.value + delta).clamp(param.min, param.max)
        };
        result.params[idx].value = new_val;
        result
    }

    pub fn interpolate(&self, other: &ConfigPoint, t: f64) -> ConfigPoint {
        let t = t.clamp(0.0, 1.0);
        let mut result = self.clone();
        for (i, param) in result.params.iter_mut().enumerate() {
            if i < other.params.len() {
                if param.is_categorical {
                    let self_idx = (param.value.round() as usize)
                        .min(param.categories.len().saturating_sub(1));
                    let other_idx = (other.params[i].value.round() as usize)
                        .min(other.params[i].categories.len().saturating_sub(1));
                    let mid = if t < 0.5 { self_idx } else { other_idx };
                    param.value = mid as f64;
                } else {
                    let other_val = other.params[i].value;
                    param.value = param.value + t * (other_val - param.value);
                    param.value = param.value.clamp(param.min, param.max);
                }
            }
        }
        result
    }
}

impl Default for ConfigPoint {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ExplorationResult {
    pub config: ConfigPoint,
    pub score: f64,
    pub metadata: String,
}

pub struct ConfigSpaceExplorer {
    pub explored: Vec<ExplorationResult>,
    pub n_clusters: usize,
}

impl ConfigSpaceExplorer {
    pub fn new() -> Self {
        ConfigSpaceExplorer {
            explored: Vec::new(),
            n_clusters: 5,
        }
    }

    pub fn record(&mut self, config: ConfigPoint, score: f64, metadata: &str) {
        self.explored.push(ExplorationResult {
            config,
            score,
            metadata: metadata.to_string(),
        });
    }

    pub fn cover_score(&self, resolution: usize) -> f64 {
        if self.explored.is_empty() {
            return 0.0;
        }
        let n_dims = self.explored[0].config.params.len();
        if n_dims == 0 {
            return 1.0;
        }
        let total = resolution.pow(n_dims as u32).max(1);
        let mut grid = vec![false; total];
        for r in &self.explored {
            let fv = r.config.to_feature_vector();
            let mut idx = 0usize;
            for (d, &v) in fv.iter().enumerate() {
                if d >= n_dims {
                    break;
                }
                let bin = ((v * resolution as f64).floor() as usize).min(resolution - 1);
                idx = idx * resolution + bin;
            }
            if idx < total {
                grid[idx] = true;
            }
        }
        let covered = grid.iter().filter(|&&c| c).count();
        covered as f64 / total as f64
    }

    pub fn best_configs(&self, k: usize) -> Vec<&ExplorationResult> {
        let mut sorted: Vec<&ExplorationResult> = self.explored.iter().collect();
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(k);
        sorted
    }

    pub fn unexplored_region(&self) -> Option<ConfigPoint> {
        if self.explored.is_empty() {
            return None;
        }
        let n = self.explored[0].config.params.len();
        if n == 0 {
            return None;
        }
        let mut best_dist = -1.0_f64;
        let mut best_point: Option<ConfigPoint> = None;
        let candidates = 200;
        let mut rng = Lcg::new(42);
        for _ in 0..candidates {
            let mut point = self.explored[0].config.clone();
            for param in point.params.iter_mut() {
                let raw = rng.next_f64();
                param.value = if param.is_categorical {
                    let idx = (raw * param.categories.len() as f64).floor() as usize;
                    idx.min(param.categories.len() - 1) as f64
                } else {
                    param.min + raw * (param.max - param.min)
                };
            }
            let min_dist = self
                .explored
                .iter()
                .map(|r| point.distance(&r.config))
                .fold(f64::MAX, f64::min);
            if min_dist > best_dist {
                best_dist = min_dist;
                best_point = Some(point);
            }
        }
        best_point
    }

    pub fn exploration_gap(&self, target: &ConfigPoint) -> f64 {
        if self.explored.is_empty() {
            return f64::MAX;
        }
        self.explored
            .iter()
            .map(|r| r.config.distance(target))
            .fold(f64::MAX, f64::min)
    }

    pub fn diversity_score(&self) -> f64 {
        let n = self.explored.len();
        if n < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        let mut count = 0usize;
        for i in 0..n {
            for j in (i + 1)..n {
                total += self.explored[i].config.distance(&self.explored[j].config);
                count += 1;
            }
        }
        if count == 0 {
            return 0.0;
        }
        total / count as f64
    }

    pub fn param_importance(&self, param_name: &str) -> f64 {
        let n = self.explored.len();
        if n < 3 {
            return 0.0;
        }
        let mut values = Vec::with_capacity(n);
        let mut scores = Vec::with_capacity(n);
        for r in &self.explored {
            if let Some(p) = r.config.get(param_name) {
                values.push(p.normalized());
                scores.push(r.score);
            }
        }
        if values.len() < 3 {
            return 0.0;
        }
        let m = values.len();
        let mean_v: f64 = values.iter().sum::<f64>() / m as f64;
        let mean_s: f64 = scores.iter().sum::<f64>() / m as f64;
        let mut cov = 0.0;
        let mut var_v = 0.0;
        let mut var_s = 0.0;
        for i in 0..m {
            let dv = values[i] - mean_v;
            let ds = scores[i] - mean_s;
            cov += dv * ds;
            var_v += dv * dv;
            var_s += ds * ds;
        }
        if var_v == 0.0 || var_s == 0.0 {
            return 0.0;
        }
        cov / (var_v.sqrt() * var_s.sqrt())
    }

    pub fn contour(
        &self,
        param_a: &str,
        param_b: &str,
        resolution: usize,
    ) -> Vec<Vec<Option<f64>>> {
        let mut grid = vec![vec![None; resolution]; resolution];
        let n = self.explored.len();
        if n == 0 {
            return grid;
        }
        let get_normalized = |r: &ExplorationResult, name: &str| -> Option<f64> {
            r.config.get(name).map(|p| p.normalized())
        };
        let a_is_some = self.explored[0].config.get(param_a).is_some();
        let b_is_some = self.explored[0].config.get(param_b).is_some();
        if !a_is_some || !b_is_some {
            return grid;
        }
        for i in 0..resolution {
            for j in 0..resolution {
                let target_a = (i as f64 + 0.5) / resolution as f64;
                let target_b = (j as f64 + 0.5) / resolution as f64;
                let mut weighted_sum = 0.0;
                let mut weight_total = 0.0;
                let eps = 1e-10;
                for r in &self.explored {
                    if let (Some(va), Some(vb)) =
                        (get_normalized(r, param_a), get_normalized(r, param_b))
                    {
                        let da = va - target_a;
                        let db = vb - target_b;
                        let dist = (da * da + db * db).sqrt();
                        let w = 1.0 / (dist + eps);
                        weighted_sum += w * r.score;
                        weight_total += w;
                    }
                }
                if weight_total > 0.0 {
                    grid[i][j] = Some(weighted_sum / weight_total);
                }
            }
        }
        grid
    }

    pub fn suggest_next(&self, strategy: &str) -> Option<ConfigPoint> {
        match strategy {
            "explore" => self.suggest_explore(),
            "exploit" => self.suggest_exploit(),
            "balanced" => {
                let mut rng = Lcg::new(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64,
                );
                if rng.next_f64() < 0.5 {
                    self.suggest_explore()
                } else {
                    self.suggest_exploit()
                }
            }
            _ => self.suggest_explore(),
        }
    }

    fn suggest_explore(&self) -> Option<ConfigPoint> {
        self.unexplored_region()
    }

    fn suggest_exploit(&self) -> Option<ConfigPoint> {
        if self.explored.is_empty() {
            return None;
        }
        let best = self.explored.iter().max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        let mut rng = Lcg::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        let mut point = best.config.clone();
        for param in point.params.iter_mut() {
            let noise = (rng.next_f64() - 0.5) * param.step * 2.0;
            param.value = param.clamp(param.value + noise);
        }
        Some(point)
    }
}

impl Default for ConfigSpaceExplorer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SpaceReport {
    pub num_explored: usize,
    pub cover_score: f64,
    pub diversity: f64,
    pub top_score: f64,
    pub unexplored_param: String,
}

impl ConfigSpaceExplorer {
    pub fn report(&self) -> SpaceReport {
        let top = self
            .explored
            .iter()
            .map(|r| r.score)
            .fold(f64::NEG_INFINITY, f64::max);
        let top_str = if top.is_finite() {
            format!("{:.4}", top)
        } else {
            "N/A".to_string()
        };
        let dims = self
            .explored
            .first()
            .map(|r| r.config.params.len())
            .unwrap_or(0);
        SpaceReport {
            num_explored: self.explored.len(),
            cover_score: self.cover_score(10),
            diversity: self.diversity_score(),
            top_score: top,
            unexplored_param: format!("{} dims, top={}", dims, top_str),
        }
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state >> 33
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    fn next_f64_range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_param(name: &str, value: f64, min: f64, max: f64, step: f64) -> ConfigParam {
        ConfigParam {
            name: name.to_string(),
            value,
            min,
            max,
            step,
            is_categorical: false,
            categories: Vec::new(),
        }
    }

    fn make_cat_param(name: &str, idx: usize, categories: &[&str]) -> ConfigParam {
        ConfigParam {
            name: name.to_string(),
            value: idx as f64,
            min: 0.0,
            max: (categories.len() - 1) as f64,
            step: 1.0,
            is_categorical: true,
            categories: categories.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_add_params_and_get() {
        let mut point = ConfigPoint::new();
        point.add(make_param("lr", 0.01, 0.0001, 1.0, 0.1));
        point.add(make_param("batch", 32.0, 1.0, 1024.0, 1.0));
        assert_eq!(point.params.len(), 2);
        assert!(point.get("lr").is_some());
        assert!(point.get("batch").is_some());
        assert!(point.get("nonexistent").is_none());
    }

    #[test]
    fn test_distance() {
        let mut a = ConfigPoint::new();
        a.add(make_param("x", 0.0, 0.0, 1.0, 0.1));
        a.add(make_param("y", 0.0, 0.0, 1.0, 0.1));
        let mut b = ConfigPoint::new();
        b.add(make_param("x", 1.0, 0.0, 1.0, 0.1));
        b.add(make_param("y", 0.0, 0.0, 1.0, 0.1));
        let d = a.distance(&b);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_best_configs_ordering() {
        let mut explorer = ConfigSpaceExplorer::new();
        for i in 0..5 {
            let mut point = ConfigPoint::new();
            point.add(make_param("x", i as f64, 0.0, 10.0, 1.0));
            explorer.record(point, i as f64 * 10.0, &format!("run-{}", i));
        }
        let best = explorer.best_configs(3);
        assert_eq!(best.len(), 3);
        assert_eq!(best[0].score, 40.0);
        assert_eq!(best[1].score, 30.0);
        assert_eq!(best[2].score, 20.0);
    }

    #[test]
    fn test_unexplored_region_finds_empty_space() {
        let mut explorer = ConfigSpaceExplorer::new();
        let mut point = ConfigPoint::new();
        point.add(make_param("x", 0.0, 0.0, 1.0, 0.1));
        explorer.record(point, 1.0, "corner");
        let region = explorer.unexplored_region();
        assert!(region.is_some());
        let r = region.unwrap();
        let d = r.distance(&explorer.explored[0].config);
        assert!(d > 0.5);
    }

    #[test]
    fn test_cover_score_increases_with_more_points() {
        let mut explorer = ConfigSpaceExplorer::new();
        let score_empty = explorer.cover_score(4);
        assert_eq!(score_empty, 0.0);

        let mut p1 = ConfigPoint::new();
        p1.add(make_param("x", 0.0, 0.0, 1.0, 0.1));
        explorer.record(p1, 1.0, "p1");
        let score_1 = explorer.cover_score(4);

        let mut p2 = ConfigPoint::new();
        p2.add(make_param("x", 0.9, 0.0, 1.0, 0.1));
        explorer.record(p2, 2.0, "p2");
        let score_2 = explorer.cover_score(4);

        assert!(score_2 >= score_1);
    }

    #[test]
    fn test_diversity_increases_with_spread() {
        let mut tight = ConfigSpaceExplorer::new();
        let mut spread = ConfigSpaceExplorer::new();
        for i in 0..4 {
            let mut pt = ConfigPoint::new();
            pt.add(make_param("x", 0.5, 0.0, 1.0, 0.1));
            tight.record(pt, i as f64, "tight");
        }
        for i in 0..4 {
            let mut pt = ConfigPoint::new();
            pt.add(make_param("x", i as f64 / 3.0, 0.0, 1.0, 0.1));
            spread.record(pt, i as f64, "spread");
        }
        assert!(spread.diversity_score() > tight.diversity_score());
    }

    #[test]
    fn test_param_importance() {
        let mut explorer = ConfigSpaceExplorer::new();
        for i in 0..10 {
            let mut point = ConfigPoint::new();
            point.add(make_param("lr", i as f64 * 0.1, 0.0, 1.0, 0.1));
            let score = (i as f64 * 10.0).sin();
            explorer.record(point, score, &format!("run-{}", i));
        }
        let imp = explorer.param_importance("lr");
        assert!(imp.abs() <= 1.0);
    }

    #[test]
    fn test_contour_grid() {
        let mut explorer = ConfigSpaceExplorer::new();
        for i in 0..5 {
            for j in 0..5 {
                let mut point = ConfigPoint::new();
                point.add(make_param("a", i as f64 / 4.0, 0.0, 1.0, 0.1));
                point.add(make_param("b", j as f64 / 4.0, 0.0, 1.0, 0.1));
                explorer.record(point, (i + j) as f64, &format!("{}-{}", i, j));
            }
        }
        let grid = explorer.contour("a", "b", 5);
        assert_eq!(grid.len(), 5);
        assert_eq!(grid[0].len(), 5);
        assert!(grid[2][2].is_some());
    }

    #[test]
    fn test_suggest_next_explore_vs_exploit() {
        let mut explorer = ConfigSpaceExplorer::new();
        let mut p1 = ConfigPoint::new();
        p1.add(make_param("x", 0.0, 0.0, 1.0, 0.1));
        explorer.record(p1, 100.0, "high");
        let mut p2 = ConfigPoint::new();
        p2.add(make_param("x", 1.0, 0.0, 1.0, 0.1));
        explorer.record(p2, 0.0, "low");

        let explore = explorer.suggest_next("explore");
        assert!(explore.is_some());
        let exploit = explorer.suggest_next("exploit");
        assert!(exploit.is_some());
    }

    #[test]
    fn test_empty_state_edge_case() {
        let explorer = ConfigSpaceExplorer::new();
        assert_eq!(explorer.cover_score(10), 0.0);
        assert!(explorer.best_configs(5).is_empty());
        assert!(explorer.unexplored_region().is_none());
        assert_eq!(explorer.diversity_score(), 0.0);
        assert_eq!(explorer.param_importance("x"), 0.0);
        assert!(explorer.suggest_next("explore").is_none());
        assert_eq!(explorer.exploration_gap(&ConfigPoint::new()), f64::MAX);
    }

    #[test]
    fn test_single_point_edge_case() {
        let mut explorer = ConfigSpaceExplorer::new();
        let mut point = ConfigPoint::new();
        point.add(make_param("x", 0.5, 0.0, 1.0, 0.1));
        explorer.record(point, 42.0, "only");
        assert_eq!(explorer.diversity_score(), 0.0);
        assert_eq!(explorer.best_configs(1).len(), 1);
        assert_eq!(explorer.best_configs(1)[0].score, 42.0);
    }

    #[test]
    fn test_random_neighbor() {
        let mut point = ConfigPoint::new();
        point.add(make_param("x", 0.5, 0.0, 1.0, 0.1));
        point.add(make_param("y", 0.5, 0.0, 1.0, 0.1));
        let neighbor = point.random_neighbor(12345);
        assert_eq!(neighbor.params.len(), 2);
        let d = point.distance(&neighbor);
        assert!(d > 0.0);
        assert!(d <= 2.0f64.sqrt());
    }

    #[test]
    fn test_interpolate() {
        let mut a = ConfigPoint::new();
        a.add(make_param("x", 0.0, 0.0, 1.0, 0.1));
        let mut b = ConfigPoint::new();
        b.add(make_param("x", 1.0, 0.0, 1.0, 0.1));
        let mid = a.interpolate(&b, 0.5);
        let v = mid.params[0].value;
        assert!((v - 0.5).abs() < 1e-6);
        let start = a.interpolate(&b, 0.0);
        assert!((start.params[0].value - 0.0).abs() < 1e-6);
        let end = a.interpolate(&b, 1.0);
        assert!((end.params[0].value - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_categorical_param() {
        let cats = ["relu", "tanh", "sigmoid"];
        let mut point = ConfigPoint::new();
        point.add(make_cat_param("activation", 1, &cats));
        let p = point.get("activation").unwrap();
        assert!(p.is_categorical);
        let norm = p.normalized();
        assert!((norm - 0.5).abs() < 1e-6);

        let rv = p.random_value(999);
        assert!(rv >= 0.0 && rv < 3.0);
    }

    #[test]
    fn test_clamp() {
        let p = make_param("x", 0.5, 0.0, 1.0, 0.1);
        assert_eq!(p.clamp(1.5), 1.0);
        assert_eq!(p.clamp(-0.5), 0.0);
        assert_eq!(p.clamp(0.5), 0.5);

        let cp = ConfigParam {
            name: "act".to_string(),
            value: 0.0,
            min: 0.0,
            max: 2.0,
            step: 1.0,
            is_categorical: true,
            categories: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        assert_eq!(cp.clamp(5.0), 2.0);
        assert_eq!(cp.clamp(-1.0), 0.0);
    }

    #[test]
    fn test_report() {
        let mut explorer = ConfigSpaceExplorer::new();
        let mut p = ConfigPoint::new();
        p.add(make_param("x", 0.0, 0.0, 1.0, 0.1));
        explorer.record(p, 99.0, "best");
        let r = explorer.report();
        assert_eq!(r.num_explored, 1);
        assert_eq!(r.top_score, 99.0);
    }
}
