use std::time::Instant;

pub(crate) const PROXY_STALE_SECS: u64 = 120;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpeedTier {
    Fast,
    Medium,
    Slow,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum NodeSelectionStrategy {
    #[default]
    Auto,
    Fastest,
    LeastLatency,
    LeastFailure,
    WeightedRandom,
    GeoPreferred(String),
    RoundRobin,
    Adaptive,
}

impl NodeSelectionStrategy {
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "fastest" => Self::Fastest,
            "leastlatency" | "least_latency" => Self::LeastLatency,
            "leastfailure" | "least_failure" => Self::LeastFailure,
            "weightedrandom" | "weighted_random" => Self::WeightedRandom,
            "roundrobin" | "round_robin" => Self::RoundRobin,
            "adaptive" => Self::Adaptive,
            "auto" => Self::Auto,
            _ => {
                if let Some(region) = name.strip_prefix("geo:") {
                    Self::GeoPreferred(region.to_string())
                } else {
                    Self::Auto
                }
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Fastest => "fastest",
            Self::LeastLatency => "least_latency",
            Self::LeastFailure => "least_failure",
            Self::WeightedRandom => "weighted_random",
            Self::GeoPreferred(_) => "geo_preferred",
            Self::RoundRobin => "round_robin",
            Self::Adaptive => "adaptive",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyNode {
    pub url: String,
    pub tag: String,
    pub latency_ms: Option<f64>,
    pub last_success: Option<Instant>,
    pub fail_count: u64,
    pub success_count: u64,
    pub from_subscription: bool,
    pub geo_tag: Option<String>,
    pub ip_addr: Option<String>,
    pub timezone: Option<String>,
}

impl ProxyNode {
    pub(crate) fn is_stale(&self) -> bool {
        self.last_success.is_none_or(|t| t.elapsed().as_secs() > PROXY_STALE_SECS)
    }

    pub fn speed_tier(&self) -> SpeedTier {
        match self.latency_ms {
            Some(ms) if ms < 500.0 => SpeedTier::Fast,
            Some(ms) if ms < 2000.0 => SpeedTier::Medium,
            Some(_) => SpeedTier::Slow,
            None => SpeedTier::Unknown,
        }
    }

    pub(crate) fn score(&self) -> f64 {
        let latency = self.latency_ms.unwrap_or(9999.0);
        let success_rate = if self.success_count + self.fail_count > 0 {
            self.success_count as f64 / (self.success_count + self.fail_count) as f64
        } else {
            0.5
        };
        let sub_bonus = if self.from_subscription { 2.0 } else { 1.0 };
        (1.0 / (latency.max(1.0))) * success_rate * sub_bonus
    }
}
