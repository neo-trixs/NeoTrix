#[derive(Debug, Clone)]
pub struct PhiConfig {
    pub num_nodes: usize,
    pub cache_ticks: u64,
    pub complex_overlap: usize,
}

impl Default for PhiConfig {
    fn default() -> Self {
        Self {
            num_nodes: 16,
            cache_ticks: 15,
            complex_overlap: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhiResult {
    pub phi_value: f64,
    pub main_complex: Vec<usize>,
    pub phi_star: f64,
    pub cached_at: u64,
}

impl PhiResult {
    pub fn new(phi: f64, complex: Vec<usize>, phi_star: f64, tick: u64) -> Self {
        Self {
            phi_value: phi,
            main_complex: complex,
            phi_star,
            cached_at: tick,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhiCache {
    pub last_result: Option<PhiResult>,
    pub tick_counter: u64,
    pub config: PhiConfig,
}

impl PhiCache {
    pub fn new(config: PhiConfig) -> Self {
        Self {
            last_result: None,
            tick_counter: 0,
            config,
        }
    }

    pub fn get_or_compute(&mut self, tick: u64, activity: &[f64]) -> PhiResult {
        if self.should_refresh(tick) {
            let r = self.compute_phi(activity, tick);
            self.last_result = Some(r.clone());
            r
        } else {
            self.last_result
                .clone()
                .unwrap_or_else(|| self.compute_phi(activity, tick))
        }
    }

    fn compute_phi(&self, activity: &[f64], tick: u64) -> PhiResult {
        let n = activity.len().min(self.config.num_nodes);
        if n == 0 {
            return PhiResult::new(0.0, vec![], 0.0, tick);
        }
        let mean: f64 = activity.iter().sum::<f64>() / n as f64;
        let var: f64 = activity.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        PhiResult::new(
            (var * 2.0).min(1.0),
            (0..n).collect(),
            (var * 2.0).min(1.0),
            tick,
        )
    }

    fn should_refresh(&self, tick: u64) -> bool {
        self.last_result
            .as_ref()
            .map_or(true, |r| tick - r.cached_at >= self.config.cache_ticks)
    }
}
