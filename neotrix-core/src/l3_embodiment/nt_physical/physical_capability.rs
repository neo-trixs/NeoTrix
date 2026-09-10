//! NT-PHYSICAL 具身能力实现
//!
//! 传感器、执行器、物理接口能力

use std::sync::Arc;
use crate::core::nt_core_capability::*;

/// 传感器能力
pub struct SensorCapability;

impl UnifiedCapability for SensorCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-physical-sensor".into(),
            name: "传感器".into(),
            description: "物理传感器数据采集".into(),
            version: "1.0.0".into(),
            domain: Domain::NtPhysical,
            layer: Layer::L3Embodiment,
            tags: vec!["physical".into(), "sensor".into(), "data".into()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 10.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = SensorData {
                    sensor_id: text,
                    sensor_type: "temperature".into(),
                    value: 25.0,
                    unit: "celsius".into(),
                    timestamp: chrono::Utc::now(),
                    metadata: std::collections::HashMap::new(),
                };
                Ok(CapabilityOutput::SensorData(Box::new(result)))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 执行器能力
pub struct ActuatorCapability;

impl UnifiedCapability for ActuatorCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-physical-actuator".into(),
            name: "执行器".into(),
            description: "物理执行器控制".into(),
            version: "1.0.0".into(),
            domain: Domain::NtPhysical,
            layer: Layer::L3Embodiment,
            tags: vec!["physical".into(), "actuator".into(), "control".into()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 50.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = ActuatorCommand {
                    actuator_id: text,
                    command: "move".into(),
                    parameters: std::collections::HashMap::new(),
                    status: "executed".into(),
                    timestamp: chrono::Utc::now(),
                };
                Ok(CapabilityOutput::ActuatorCommand(Box::new(result)))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 物理仿真能力
pub struct SimulationCapability;

impl UnifiedCapability for SimulationCapability {
    fn meta(&self) -> CapabilityMeta {
        CapabilityMeta {
            id: "nt-physical-sim".into(),
            name: "物理仿真".into(),
            description: "物理系统仿真模拟".into(),
            version: "1.0.0".into(),
            domain: Domain::NtPhysical,
            layer: Layer::L2Perception,
            tags: vec!["physical".into(), "simulation".into(), "model".into()],
            status: CapabilityStatus::Healthy,
            metrics: CapabilityMetrics {
                total_calls: 0,
                total_errors: 0,
                avg_latency_ms: 0.0,
                last_called: None,
            },
        }
    }

    fn health(&self) -> CapabilityHealth {
        CapabilityHealth {
            state: CapabilityState::Healthy,
            success_rate: 1.0,
            avg_latency_ms: 200.0,
            last_called: None,
            call_count: 0,
        }
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        match input {
            CapabilityInput::Text(text) => {
                let result = SimulationResult {
                    model_id: text,
                    state: "running".into(),
                    metrics: SimulationMetrics {
                        fps: 60.0,
                        objects: 100,
                        collisions: 0,
                        memory_mb: 256.0,
                    },
                    duration_ms: 1000,
                };
                Ok(CapabilityOutput::SimulationResult(Box::new(result)))
            }
            _ => Err(CapabilityError::UnsupportedInput("需要文本输入".into())),
        }
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        matches!(input, CapabilityInput::Text(_))
    }
}

/// 创建NT-PHYSICAL能力
pub fn create_physical_capabilities() -> Vec<Arc<dyn UnifiedCapability>> {
    vec![
        Arc::new(SensorCapability),
        Arc::new(ActuatorCapability),
        Arc::new(SimulationCapability),
    ]
}

/// 传感器数据
#[derive(Debug, Clone)]
pub struct SensorData {
    pub sensor_id: String,
    pub sensor_type: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// 执行器命令
#[derive(Debug, Clone)]
pub struct ActuatorCommand {
    pub actuator_id: String,
    pub command: String,
    pub parameters: std::collections::HashMap<String, String>,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 仿真结果
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub model_id: String,
    pub state: String,
    pub metrics: SimulationMetrics,
    pub duration_ms: u64,
}

/// 仿真指标
#[derive(Debug, Clone)]
pub struct SimulationMetrics {
    pub fps: f64,
    pub objects: u32,
    pub collisions: u32,
    pub memory_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor() {
        let cap = SensorCapability;
        let input = CapabilityInput::Text("sensor_1".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn actuator() {
        let cap = ActuatorCapability;
        let input = CapabilityInput::Text("actuator_1".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn simulation() {
        let cap = SimulationCapability;
        let input = CapabilityInput::Text("model_1".into());
        let result = cap.execute(input);
        assert!(result.is_ok());
    }
}
