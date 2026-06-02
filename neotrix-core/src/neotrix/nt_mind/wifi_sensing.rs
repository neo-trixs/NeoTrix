use std::collections::{HashMap, VecDeque};
use chrono::Utc;

/// 空间特征（从 CSI 信号中提取）
#[derive(Debug, Clone)]
pub struct SpatialFeatures {
    pub timestamp: i64,
    pub presence: Vec<ZonePresence>,
    pub vital_signs: Option<VitalSigns>,
    pub motion: Vec<MotionEvent>,
}

/// 区域存在检测
#[derive(Debug, Clone)]
pub struct ZonePresence {
    pub zone_id: String,
    pub probability: f64,
    pub person_count: usize,
    pub confidence: f64,
}

/// 生命体征
#[derive(Debug, Clone)]
pub struct VitalSigns {
    pub breathing_rate: f32,  // BPM
    pub heart_rate: f32,      // BPM
    pub confidence: f32,
}

/// 运动事件
#[derive(Debug, Clone)]
pub struct MotionEvent {
    pub zone_id: String,
    pub motion_type: MotionType,
    pub intensity: f32,      // 0.0-1.0
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MotionType {
    Enter, Exit, Walk, Gesture, Fall, Idle, Unknown,
}

/// 房间区域
#[derive(Debug, Clone)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub label: Option<String>,
}

/// 空间模型：区域划分 + 人追踪
#[derive(Debug, Clone)]
pub struct SpatialModel {
    zones: Vec<Zone>,
    occupants: HashMap<String, OccupantTrace>,
    history: VecDeque<SpatialFeatures>,
    max_history: usize,
    heatmap: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
struct OccupantTrace {
    #[allow(dead_code)]
    id: String,
    current_zone: Option<String>,
    #[allow(dead_code)]
    first_seen: i64,
    last_seen: i64,
    total_visits: usize,
}

impl SpatialModel {
    pub fn new(max_history: usize) -> Self {
        Self { zones: Vec::new(), occupants: HashMap::new(),
            history: VecDeque::with_capacity(max_history), max_history, heatmap: HashMap::new() }
    }

    pub fn add_zone(&mut self, id: &str, name: &str) {
        self.zones.push(Zone { id: id.to_string(), name: name.to_string(), label: None });
    }

    pub fn update(&mut self, features: SpatialFeatures) {
        let ts = features.timestamp;
        for zone in &features.presence {
            *self.heatmap.entry(zone.zone_id.clone()).or_insert(0.0) += zone.probability * 0.1;
            if zone.probability > 0.6 {
                let occupant = self.occupants.entry(format!("occ_{}", zone.zone_id)).or_insert(
                    OccupantTrace { id: format!("occ_{}", zone.zone_id), current_zone: None,
                        first_seen: ts, last_seen: ts, total_visits: 0 });
                if occupant.current_zone.is_none() {
                    occupant.current_zone = Some(zone.zone_id.clone());
                    occupant.total_visits += 1;
                }
                occupant.last_seen = ts;
            }
        }
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(features);
    }

    pub fn occupant_count(&self) -> usize {
        let cutoff = Utc::now().timestamp() - 300;
        self.occupants.values().filter(|o| o.last_seen > cutoff).count()
    }

    pub fn zone_heatmap(&self) -> &HashMap<String, f64> { &self.heatmap }
    pub fn zones(&self) -> &[Zone] { &self.zones }
}

/// WiFi 感知引擎
pub struct WifiSensingEngine {
    model: SpatialModel,
    source: SensingSource,
    enabled: bool,
}

pub enum SensingSource {
    /// 模拟模式（开发和测试）
    Simulation { interval_secs: u64, simulate_people: usize },
    /// RuView HTTP bridge
    RuViewBridge { url: String, api_key: Option<String> },
    /// MCP bridge to external sensing service
    McpBridge,
}

impl WifiSensingEngine {
    pub fn new(source: SensingSource, max_history: usize) -> Self {
        let mut model = SpatialModel::new(max_history);
        model.add_zone("zone_1", "Room Center");
        model.add_zone("zone_2", "Desk Area");
        model.add_zone("zone_3", "Entrance");
        Self { model, source, enabled: true }
    }

    /// 生成模拟数据（无硬件时使用）
    pub fn simulate_tick(&mut self) -> SpatialFeatures {
        let now = Utc::now().timestamp();
        let people_count = match &self.source {
            SensingSource::Simulation { simulate_people, .. } => *simulate_people,
            _ => 1,
        };
        let mut zones = Vec::new();
        for i in 0..self.model.zones().len() {
            let prob = if i == 0 { 0.85 } else if i == 1 { 0.30 } else { 0.05 };
            zones.push(ZonePresence {
                zone_id: format!("zone_{}", i + 1),
                probability: prob + (rand_prob() * 0.1 - 0.05),
                person_count: if prob > 0.5 { people_count } else { 0 },
                confidence: 0.75 + rand_prob() * 0.2,
            });
        }
        let vs = VitalSigns {
            breathing_rate: (14.0 + rand_prob() * 6.0) as f32,
            heart_rate: (68.0 + rand_prob() * 20.0) as f32,
            confidence: (0.7 + rand_prob() * 0.2) as f32,
        };
        let motion = vec![MotionEvent {
            zone_id: "zone_1".into(), motion_type: MotionType::Idle,
            intensity: 0.1, timestamp: now,
        }];
        SpatialFeatures { timestamp: now, presence: zones, vital_signs: Some(vs), motion }
    }

    /// 更新世界模型（将空间数据注入）
    pub fn update_world(&mut self) -> Option<&SpatialModel> {
        if !self.enabled { return None; }
        let features = match &self.source {
            SensingSource::Simulation { .. } => self.simulate_tick(),
            _ => return None,
        };
        self.model.update(features);
        Some(&self.model)
    }

    pub fn current_status(&self) -> WifiStatus {
        WifiStatus {
            enabled: self.enabled,
            occupant_count: self.model.occupant_count(),
            zone_count: self.model.zones().len(),
            heatmap: self.model.zone_heatmap().clone(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    pub fn model(&self) -> &SpatialModel { &self.model }
}

#[derive(Debug, Clone)]
pub struct WifiStatus {
    pub enabled: bool,
    pub occupant_count: usize,
    pub zone_count: usize,
    pub heatmap: HashMap<String, f64>,
}

fn rand_prob() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_sensing_engine_creation() {
        let engine = WifiSensingEngine::new(SensingSource::Simulation { interval_secs: 1, simulate_people: 1 }, 100);
        assert!(engine.enabled);
        assert_eq!(engine.model().zones().len(), 3);
    }

    #[test]
    fn test_simulate_tick() {
        let mut engine = WifiSensingEngine::new(SensingSource::Simulation { interval_secs: 1, simulate_people: 1 }, 100);
        let features = engine.simulate_tick();
        assert!(!features.presence.is_empty());
        assert!(features.vital_signs.is_some());
        let vs = features.vital_signs.expect("vital_signs should be ok in test");
        assert!(vs.breathing_rate > 0.0);
        assert!(vs.heart_rate > 0.0);
    }

    #[test]
    fn test_update_world() {
        let mut engine = WifiSensingEngine::new(SensingSource::Simulation { interval_secs: 1, simulate_people: 2 }, 100);
        for _ in 0..5 {
            engine.update_world();
        }
        let status = engine.current_status();
        assert!(status.occupant_count > 0 || status.zone_count > 0);
        assert!(!status.heatmap.is_empty());
    }

    #[test]
    fn test_spatial_model_occupant_tracking() {
        let mut model = SpatialModel::new(50);
        model.add_zone("test_zone", "Test Area");
        let features = SpatialFeatures {
            timestamp: Utc::now().timestamp(),
            presence: vec![ZonePresence { zone_id: "test_zone".into(),
                probability: 0.9, person_count: 1, confidence: 0.85 }],
            vital_signs: None,
            motion: vec![],
        };
        model.update(features);
        assert_eq!(model.occupant_count(), 1);
    }

    #[test]
    fn test_simulate_tick_produces_valid_vitals() {
        let mut engine = WifiSensingEngine::new(SensingSource::Simulation { interval_secs: 1, simulate_people: 1 }, 100);
        for _ in 0..10 {
            let features = engine.simulate_tick();
            let vs = features.vital_signs.expect("vital_signs should be ok in test");
            // Normal human range
            assert!(vs.breathing_rate >= 6.0 && vs.breathing_rate <= 30.0);
            assert!(vs.heart_rate >= 40.0 && vs.heart_rate <= 120.0);
        }
    }

    #[test]
    fn test_status_report() {
        let mut engine = WifiSensingEngine::new(SensingSource::Simulation { interval_secs: 1, simulate_people: 1 }, 100);
        engine.update_world();
        let status = engine.current_status();
        assert!(status.enabled);
        // Heatmap should have data after update
        let total_heat: f64 = status.heatmap.values().sum();
        assert!(total_heat > 0.0);
    }
}
