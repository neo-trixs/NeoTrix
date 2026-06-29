/// Per-handler profiling for consciousness pipeline hot-path analysis.
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerTier {
    Every,
    Cycle,
    Adaptive,
    Rare,
    Manual,
}

#[derive(Debug, Clone)]
pub struct HandlerStats {
    pub name: &'static str,
    pub call_count: u64,
    pub total_ns: u128,
    pub max_ns: u128,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub tier: HandlerTier,
}

#[derive(Debug, Clone)]
pub struct HandlerProfile {
    pub name: &'static str,
    pub call_count: u64,
    pub total_ns: u128,
    pub max_ns: u128,
    pub samples_ns: Vec<u128>,
}

impl HandlerProfile {
    pub fn avg_ms(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            self.total_ns as f64 / self.call_count as f64 / 1_000_000.0
        }
    }
    pub fn total_ms(&self) -> f64 {
        self.total_ns as f64 / 1_000_000.0
    }
    pub fn max_ms(&self) -> f64 {
        self.max_ns as f64 / 1_000_000.0
    }
    pub fn p50_ms(&self) -> f64 {
        percentile_ms(&self.samples_ns, 50.0)
    }
    pub fn p95_ms(&self) -> f64 {
        percentile_ms(&self.samples_ns, 95.0)
    }
    pub fn p99_ms(&self) -> f64 {
        percentile_ms(&self.samples_ns, 99.0)
    }
}

fn percentile_ms(samples: &[u128], p: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let idx = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    let idx = idx.min(sorted.len()).max(1) - 1;
    sorted[idx] as f64 / 1_000_000.0
}

/// Handler-level profiler for consciousness pipeline dispatch.
/// Tracks per-handler total/max/avg latency. Intended for single-threaded use
/// within the background loop's periodic handler dispatch.
pub struct HandlerProfiler {
    profiles: Vec<HandlerProfile>,
    pub enabled: bool,
    cycle_count: u64,
}

impl HandlerProfiler {
    pub fn new() -> Self {
        Self {
            profiles: Vec::with_capacity(64),
            enabled: true,
            cycle_count: 0,
        }
    }

    pub fn record(&mut self, name: &'static str, elapsed_ns: u128) {
        self.cycle_count += 1;
        if !self.enabled {
            return;
        }
        if let Some(p) = self.profiles.iter_mut().find(|p| p.name == name) {
            p.call_count += 1;
            p.total_ns += elapsed_ns;
            p.max_ns = p.max_ns.max(elapsed_ns);
            p.samples_ns.push(elapsed_ns);
            if p.samples_ns.len() > 100 {
                p.samples_ns.drain(0..p.samples_ns.len() - 100);
            }
        } else {
            let mut samples_ns = Vec::with_capacity(100);
            samples_ns.push(elapsed_ns);
            self.profiles.push(HandlerProfile {
                name,
                call_count: 1,
                total_ns: elapsed_ns,
                max_ns: elapsed_ns,
                samples_ns,
            });
        }
    }

    pub fn top(&self, n: usize) -> Vec<&HandlerProfile> {
        let mut sorted: Vec<_> = self.profiles.iter().collect();
        sorted.sort_by(|a, b| b.total_ns.cmp(&a.total_ns));
        sorted.truncate(n);
        sorted
    }

    pub fn report(&self) -> String {
        if !self.enabled {
            return "profiler disabled".into();
        }
        let mut s = format!(
            "=== Handler Profiler ({} handlers, {} cycles) ===\n",
            self.profiles.len(),
            self.cycle_count
        );
        for p in &self.top(5) {
            s.push_str(&format!(
                "  {:<30} calls={:>5} avg={:>8.3}ms p50={:>8.3}ms max={:>8.3}ms total={:>8.3}ms\n",
                p.name,
                p.call_count,
                p.avg_ms(),
                p.p50_ms(),
                p.max_ms(),
                p.total_ms()
            ));
        }
        s
    }

    pub fn clear(&mut self) {
        self.profiles.clear();
    }

    pub fn register_handler(&mut self, name: &'static str, _tier: HandlerTier) {
        if !self.profiles.iter().any(|p| p.name == name) {
            self.profiles.push(HandlerProfile {
                name,
                call_count: 0,
                total_ns: 0,
                max_ns: 0,
                samples_ns: Vec::with_capacity(100),
            });
        }
    }

    pub fn record_start(&mut self, _name: &str) -> Instant {
        Instant::now()
    }

    pub fn record_end(&mut self, name: &str, start: Instant) {
        let elapsed = start.elapsed().as_nanos();
        self.cycle_count += 1;
        if !self.enabled {
            return;
        }
        if let Some(p) = self.profiles.iter_mut().find(|p| p.name == name) {
            p.call_count += 1;
            p.total_ns += elapsed;
            p.max_ns = p.max_ns.max(elapsed);
            p.samples_ns.push(elapsed);
            if p.samples_ns.len() > 100 {
                p.samples_ns.drain(0..p.samples_ns.len() - 100);
            }
        }
    }

    pub fn all_stats(&self) -> Vec<HandlerStats> {
        self.profiles
            .iter()
            .map(|p| HandlerStats {
                name: p.name,
                call_count: p.call_count,
                total_ns: p.total_ns,
                max_ns: p.max_ns,
                p50_ms: p.p50_ms(),
                p95_ms: p.p95_ms(),
                p99_ms: p.p99_ms(),
                tier: HandlerTier::Cycle,
            })
            .collect()
    }

    pub fn sorted_report(&self) -> String {
        if !self.enabled {
            return "profiler disabled".into();
        }
        let mut sorted: Vec<_> = self.profiles.iter().collect();
        sorted.sort_by(|a, b| b.total_ns.cmp(&a.total_ns));
        let mut s = format!(
            "=== Handler Profiler ({} handlers, {} cycles) ===\n",
            self.profiles.len(),
            self.cycle_count
        );
        for p in &sorted {
            s.push_str(&format!(
                "  {:<30} calls={:>5} avg={:>8.3}ms p50={:>8.3}ms max={:>8.3}ms total={:>8.3}ms\n",
                p.name,
                p.call_count,
                p.avg_ms(),
                p.p50_ms(),
                p.max_ms(),
                p.total_ms()
            ));
        }
        s
    }

    pub fn total_cycles(&self) -> u64 {
        self.cycle_count
    }

    pub fn structured_report(&self) -> String {
        if !self.enabled {
            return r#"{"enabled":false}"#.into();
        }
        let mut items = Vec::with_capacity(self.profiles.len());
        for p in &self.profiles {
            items.push(format!(
                r#"{{"name":"{}","calls":{},"avg_ms":{:.3},"max_ms":{:.3},"total_ms":{:.3},"p50_ms":{:.3},"p95_ms":{:.3},"p99_ms":{:.3}}}"#,
                p.name,
                p.call_count,
                p.avg_ms(),
                p.max_ms(),
                p.total_ms(),
                p.p50_ms(),
                p.p95_ms(),
                p.p99_ms(),
            ));
        }
        format!(
            r#"{{"handlers":[{}],"total_handlers":{},"total_cycles":{}}}"#,
            items.join(","),
            self.profiles.len(),
            self.cycle_count
        )
    }

    pub fn total_samples(&self) -> usize {
        self.profiles.iter().map(|p| p.call_count as usize).sum()
    }
}
