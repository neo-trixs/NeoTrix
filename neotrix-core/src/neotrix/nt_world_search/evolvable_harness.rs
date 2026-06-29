use std::collections::{HashMap, VecDeque};
use std::time::Instant;

const MAX_TRACES: usize = 10_000;
const SUCCESS_WEIGHT: f64 = 0.6;
const DURATION_WEIGHT: f64 = 0.4;
const RECENT_WINDOW: usize = 20;
const STRUGGLE_THRESHOLD: f64 = 0.5;

#[derive(Clone, Debug)]
pub struct TraceRecord {
    pub tool_name: String,
    pub duration_ms: u64,
    pub success: bool,
    pub token_estimate: u64,
    pub context_before: String,
    pub context_after: String,
    pub error: Option<String>,
    pub timestamp: Instant,
}

#[derive(Clone, Debug)]
pub struct ToolProfile {
    pub tool_name: String,
    pub call_count: u64,
    pub success_count: u64,
    pub avg_duration_ms: f64,
    pub total_tokens: u64,
    pub last_called: Instant,
    pub recent_success_rate: Vec<bool>,
    pub priority_score: f64,
}

impl ToolProfile {
    fn compute_priority(&self) -> f64 {
        if self.call_count == 0 {
            return 0.0;
        }
        let success_rate = self.success_count as f64 / self.call_count as f64;
        let inv_duration = 1.0 / (1.0 + self.avg_duration_ms / 1000.0);
        success_rate * SUCCESS_WEIGHT + inv_duration * DURATION_WEIGHT
    }
}

#[derive(Clone, Debug)]
pub struct HarnessSummary {
    pub total_calls: u64,
    pub total_adaptations: u64,
    pub avg_success_rate: f64,
    pub avg_duration_ms: f64,
    pub tool_count: usize,
}

#[derive(Clone, Debug)]
pub struct EvolvableHarness {
    traces: VecDeque<TraceRecord>,
    profiles: HashMap<String, ToolProfile>,
    adaptation_count: u64,
}

impl Default for EvolvableHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolvableHarness {
    pub fn new() -> Self {
        Self {
            traces: VecDeque::with_capacity(MAX_TRACES),
            profiles: HashMap::new(),
            adaptation_count: 0,
        }
    }

    pub fn record_trace(&mut self, trace: TraceRecord) {
        let tool = trace.tool_name.clone();
        let duration = trace.duration_ms;
        let success = trace.success;
        let tokens = trace.token_estimate;

        let profile = self.profiles.entry(tool).or_insert(ToolProfile {
            tool_name: trace.tool_name.clone(),
            call_count: 0,
            success_count: 0,
            avg_duration_ms: 0.0,
            total_tokens: 0,
            last_called: trace.timestamp,
            recent_success_rate: Vec::with_capacity(RECENT_WINDOW),
            priority_score: 0.0,
        });

        let prev_success_rate = if profile.call_count > 0 {
            profile.success_count as f64 / profile.call_count as f64
        } else {
            0.0
        };

        profile.call_count += 1;
        if success {
            profile.success_count += 1;
        }
        profile.total_tokens += tokens;
        profile.last_called = trace.timestamp;

        if profile.call_count == 1 {
            profile.avg_duration_ms = duration as f64;
        } else {
            let n = profile.call_count as f64;
            profile.avg_duration_ms =
                profile.avg_duration_ms * ((n - 1.0) / n) + duration as f64 / n;
        }

        profile.recent_success_rate.push(success);
        if profile.recent_success_rate.len() > RECENT_WINDOW {
            profile.recent_success_rate.remove(0);
        }

        profile.priority_score = profile.compute_priority();

        let new_success_rate = profile.success_count as f64 / profile.call_count as f64;
        if (new_success_rate - prev_success_rate).abs() > f64::EPSILON {
            self.adaptation_count += 1;
        }

        self.traces.push_back(trace);
        if self.traces.len() > MAX_TRACES {
            self.traces.pop_front();
        }
    }

    pub fn get_profile(&self, tool: &str) -> Option<&ToolProfile> {
        self.profiles.get(tool)
    }

    pub fn suggest_priority<'a>(&self, tools: &'a [&str]) -> Vec<(&'a str, f64)> {
        let mut scored: Vec<(&str, f64)> = tools
            .iter()
            .map(|t| {
                let score = self
                    .profiles
                    .get(*t)
                    .map(|p| p.priority_score)
                    .unwrap_or(0.0);
                (*t, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    pub fn adaptation_rate(&self) -> f64 {
        let total = self.traces.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        self.adaptation_count as f64 / total
    }

    pub fn summary(&self) -> HarnessSummary {
        let tool_count = self.profiles.len();
        let total_calls: u64 = self.profiles.values().map(|p| p.call_count).sum();

        if tool_count == 0 {
            return HarnessSummary {
                total_calls: 0,
                total_adaptations: self.adaptation_count,
                avg_success_rate: 0.0,
                avg_duration_ms: 0.0,
                tool_count: 0,
            };
        }

        let avg_success_rate = self
            .profiles
            .values()
            .map(|p| {
                if p.call_count > 0 {
                    p.success_count as f64 / p.call_count as f64
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / tool_count as f64;

        let avg_duration_ms = self
            .profiles
            .values()
            .map(|p| p.avg_duration_ms)
            .sum::<f64>()
            / tool_count as f64;

        HarnessSummary {
            total_calls,
            total_adaptations: self.adaptation_count,
            avg_success_rate,
            avg_duration_ms,
            tool_count,
        }
    }

    pub fn top_performers(&self, n: usize) -> Vec<(String, f64)> {
        let mut profiles: Vec<(&ToolProfile, f64)> = self
            .profiles
            .values()
            .map(|p| (p, p.priority_score))
            .collect();
        profiles.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        profiles
            .into_iter()
            .take(n)
            .map(|(p, s)| (p.tool_name.clone(), s))
            .collect()
    }

    pub fn struggling_tools(&self) -> Vec<(String, f64)> {
        self.profiles
            .values()
            .filter(|p| p.call_count > 0)
            .map(|p| {
                let rate = p.success_count as f64 / p.call_count as f64;
                (p.tool_name.clone(), rate)
            })
            .filter(|(_, rate)| *rate < STRUGGLE_THRESHOLD)
            .collect()
    }
}

static EVOLVABLE_HARNESS: std::sync::OnceLock<std::sync::Mutex<EvolvableHarness>> =
    std::sync::OnceLock::new();

pub fn global_harness() -> &'static std::sync::Mutex<EvolvableHarness> {
    EVOLVABLE_HARNESS.get_or_init(|| std::sync::Mutex::new(EvolvableHarness::new()))
}

pub fn record_tool_call(tool_name: &str, duration_ms: u64, success: bool) {
    if let Ok(mut harness) = global_harness().lock() {
        harness.record_trace(TraceRecord {
            tool_name: tool_name.to_string(),
            duration_ms,
            success,
            token_estimate: 0,
            context_before: String::new(),
            context_after: String::new(),
            error: None,
            timestamp: Instant::now(),
        });
    }
}

pub fn suggest_tool_order(tools: &[&str]) -> Vec<String> {
    if let Ok(harness) = global_harness().lock() {
        let scored = harness.suggest_priority(tools);
        scored.into_iter().map(|(t, _)| t.to_string()).collect()
    } else {
        tools.iter().map(|t| t.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn make_trace(tool: &str, dur: u64, ok: bool) -> TraceRecord {
        TraceRecord {
            tool_name: tool.to_string(),
            duration_ms: dur,
            success: ok,
            token_estimate: 100,
            context_before: "before".into(),
            context_after: "after".into(),
            error: if ok { None } else { Some("err".into()) },
            timestamp: Instant::now(),
        }
    }

    #[serial]
    #[test]
    fn test_record_trace_creates_profile() {
        let mut h = EvolvableHarness::new();
        h.record_trace(make_trace("search", 50, true));
        let p = h.get_profile("search").unwrap();
        assert_eq!(p.call_count, 1);
        assert_eq!(p.success_count, 1);
        assert!(p.priority_score > 0.0);
    }

    #[test]
    fn test_profile_aggregation() {
        let mut h = EvolvableHarness::new();
        h.record_trace(make_trace("read", 100, true));
        h.record_trace(make_trace("read", 200, false));
        h.record_trace(make_trace("read", 150, true));
        let p = h.get_profile("read").unwrap();
        assert_eq!(p.call_count, 3);
        assert_eq!(p.success_count, 2);
        assert_eq!(p.recent_success_rate.len(), 3);
    }

    #[test]
    fn test_suggest_priority_ordering() {
        let mut h = EvolvableHarness::new();
        h.record_trace(make_trace("fast", 10, true));
        h.record_trace(make_trace("fast", 10, true));
        h.record_trace(make_trace("slow", 5000, false));
        h.record_trace(make_trace("slow", 5000, false));

        let result = h.suggest_priority(&["fast", "slow"]);
        assert_eq!(result[0].0, "fast");
        assert_eq!(result[1].0, "slow");
        assert!(result[0].1 > result[1].1);
    }

    #[test]
    fn test_struggling_tools() {
        let mut h = EvolvableHarness::new();
        h.record_trace(make_trace("bad", 100, false));
        h.record_trace(make_trace("bad", 100, false));
        h.record_trace(make_trace("good", 100, true));
        h.record_trace(make_trace("good", 100, true));

        let struggling = h.struggling_tools();
        assert_eq!(struggling.len(), 1);
        assert_eq!(struggling[0].0, "bad");
    }

    #[test]
    fn test_adaptation_rate() {
        let mut h = EvolvableHarness::new();
        assert_eq!(h.adaptation_rate(), 0.0);
        h.record_trace(make_trace("a", 10, true));
        assert!(h.adaptation_rate() > 0.0);
    }

    #[test]
    fn test_summary() {
        let mut h = EvolvableHarness::new();
        h.record_trace(make_trace("x", 50, true));
        h.record_trace(make_trace("y", 150, false));
        let s = h.summary();
        assert_eq!(s.total_calls, 2);
        assert_eq!(s.tool_count, 2);
    }

    #[test]
    fn test_top_performers() {
        let mut h = EvolvableHarness::new();
        h.record_trace(make_trace("alpha", 10, true));
        h.record_trace(make_trace("alpha", 10, true));
        h.record_trace(make_trace("beta", 200, false));
        let top = h.top_performers(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0, "alpha");
    }

    #[test]
    fn test_deque_max_traces() {
        let mut h = EvolvableHarness::new();
        for i in 0..MAX_TRACES + 50 {
            h.record_trace(make_trace(&format!("t_{}", i), 10, true));
        }
        assert_eq!(h.traces.len(), MAX_TRACES);
    }

    #[test]
    fn test_global_harness_singleton() {
        let h1 = global_harness();
        let h2 = global_harness();
        assert!(std::ptr::eq(h1, h2));
    }

    #[test]
    fn test_global_record_tool_call() {
        record_tool_call("global_test", 42, true);
        let h = global_harness().lock().unwrap_or_else(|e| e.into_inner());
        let p = h.get_profile("global_test").unwrap();
        assert_eq!(p.call_count, 1);
    }

    #[test]
    fn test_suggest_tool_order_global() {
        record_tool_call("pref_a", 5, true);
        record_tool_call("pref_a", 5, true);
        record_tool_call("pref_b", 500, false);
        let ordered = suggest_tool_order(&["pref_a", "pref_b"]);
        assert_eq!(ordered[0], "pref_a");
    }
}
