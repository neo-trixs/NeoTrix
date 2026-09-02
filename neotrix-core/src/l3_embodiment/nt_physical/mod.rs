//! L3 Embodiment Layer - Physical Modules
//!
//! 具身骨架: 传感器 + 执行器 + 安全内核 + 电源管理 + 身体图式
//! 吸收来源: Blender-MCP/Unity-MCP (3D 工具集成)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 物理具身核心 — 传感器/执行器/安全内核
pub struct PhysicalEmbodiment {
    sensors: Vec<Sensor>,
    motors: Vec<Motor>,
    safety_kernel: SafetyKernel,
    power_manager: PowerManager,
    body_schema: BodySchema,
}

/// 传感器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    pub id: String,
    pub sensor_type: SensorType,
    pub status: SensorStatus,
    pub data: Option<serde_json::Value>,
    pub last_reading: Option<chrono::DateTime<chrono::Utc>>,
}

/// 传感器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    Camera,
    Microphone,
    IMU,
    GPS,
    Temperature,
    Proximity,
    Force,
    Custom(String),
}

/// 传感器状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensorStatus {
    Active,
    Inactive,
    Error,
    Calibration,
}

/// 执行器 (电机)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Motor {
    pub id: String,
    pub motor_type: MotorType,
    pub status: MotorStatus,
    pub position: f64,
    pub velocity: f64,
    pub torque: f64,
}

/// 执行器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MotorType {
    Servo,
    Stepper,
    DC,
    Brushless,
    Pneumatic,
    Hydraulic,
    Custom(String),
}

/// 执行器状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MotorStatus {
    Running,
    Stopped,
    Error,
    Overheated,
}

/// 安全内核
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyKernel {
    pub enabled: bool,
    pub rules: Vec<SafetyRule>,
    pub violations: Vec<SafetyViolation>,
    pub emergency_stop: bool,
}

/// 安全规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRule {
    pub id: String,
    pub rule_type: String,
    pub condition: String,
    pub action: String,
    pub priority: u8,
}

/// 安全违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyViolation {
    pub rule_id: String,
    pub description: String,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

/// 电源管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerManager {
    pub battery_level: f64, // 0.0-1.0
    pub charging: bool,
    pub power_mode: PowerMode,
    pub consumption: f64, // watts
}

/// 电源模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerMode {
    Normal,
    PowerSave,
    Performance,
    Idle,
    Sleep,
}

/// 身体图式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodySchema {
    pub parts: Vec<BodyPart>,
    pub joints: Vec<Joint>,
    pub dimensions: HashMap<String, f64>,
}

/// 身体部件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPart {
    pub id: String,
    pub name: String,
    pub part_type: String,
    pub sensors: Vec<String>,
    pub motors: Vec<String>,
}

/// 关节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub id: String,
    pub name: String,
    pub joint_type: String,
    pub limits: JointLimits,
    pub current_angle: f64,
}

/// 关节限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointLimits {
    pub min_angle: f64,
    pub max_angle: f64,
    pub max_velocity: f64,
    pub max_torque: f64,
}

/// 3D 工具集成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIntegrationResult {
    pub tool: String,
    pub command: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

impl PhysicalEmbodiment {
    /// 创建新的物理具身
    pub fn new() -> Self {
        Self {
            sensors: vec![],
            motors: vec![],
            safety_kernel: SafetyKernel {
                enabled: true,
                rules: vec![],
                violations: vec![],
                emergency_stop: false,
            },
            power_manager: PowerManager {
                battery_level: 1.0,
                charging: false,
                power_mode: PowerMode::Normal,
                consumption: 0.0,
            },
            body_schema: BodySchema {
                parts: vec![],
                joints: vec![],
                dimensions: HashMap::new(),
            },
        }
    }

    /// 添加传感器
    pub fn add_sensor(&mut self, sensor: Sensor) {
        self.sensors.push(sensor);
    }

    /// 添加执行器
    pub fn add_motor(&mut self, motor: Motor) {
        self.motors.push(motor);
    }

    /// 读取传感器数据
    pub fn read_sensor(&self, sensor_id: &str) -> Option<&Sensor> {
        self.sensors.iter().find(|s| s.id == sensor_id)
    }

    /// 控制执行器
    pub fn control_motor(
        &mut self,
        motor_id: &str,
        position: Option<f64>,
        velocity: Option<f64>,
    ) -> Result<(), String> {
        // 检查安全内核
        if self.safety_kernel.emergency_stop {
            return Err("Emergency stop active".into());
        }

        if let Some(motor) = self.motors.iter_mut().find(|m| m.id == motor_id) {
            if let Some(pos) = position {
                motor.position = pos;
            }
            if let Some(vel) = velocity {
                motor.velocity = vel;
            }
            Ok(())
        } else {
            Err(format!("Motor {} not found", motor_id))
        }
    }

    /// 紧急停止
    pub fn emergency_stop(&mut self) {
        self.safety_kernel.emergency_stop = true;
        for motor in &mut self.motors {
            motor.velocity = 0.0;
            motor.status = MotorStatus::Stopped;
        }
    }

    /// 重置紧急停止
    pub fn reset_emergency_stop(&mut self) {
        self.safety_kernel.emergency_stop = false;
    }

    /// 检查安全规则
    pub fn check_safety(&mut self) -> Vec<SafetyViolation> {
        let mut violations = vec![];

        // 检查电池电量
        if self.power_manager.battery_level < 0.1 {
            violations.push(SafetyViolation {
                rule_id: "low_battery".into(),
                description: "Battery level below 10%".into(),
                severity: "warning".into(),
                timestamp: chrono::Utc::now(),
                resolved: false,
            });
        }

        // 检查电机过热
        for motor in &self.motors {
            if motor.status == MotorStatus::Overheated {
                violations.push(SafetyViolation {
                    rule_id: "motor_overheat".into(),
                    description: format!("Motor {} is overheated", motor.id),
                    severity: "critical".into(),
                    timestamp: chrono::Utc::now(),
                    resolved: false,
                });
            }
        }

        self.safety_kernel.violations.extend(violations.clone());
        violations
    }

    /// 集成 3D 工具 (Blender-MCP/Unity-MCP)
    pub fn integrate_3d_tool(
        &self,
        tool: &str,
        command: &str,
        params: serde_json::Value,
    ) -> ToolIntegrationResult {
        // TODO: 实际集成 Blender/Unity MCP 协议
        ToolIntegrationResult {
            tool: tool.to_string(),
            command: command.to_string(),
            success: true,
            output: serde_json::json!({
                "status": "simulated",
                "tool": tool,
                "command": command,
                "params": params,
            }),
            error: None,
        }
    }

    /// 获取具身状态快照
    pub fn snapshot(&self) -> PhysicalSnapshot {
        PhysicalSnapshot {
            sensor_count: self.sensors.len(),
            motor_count: self.motors.len(),
            active_sensors: self.sensors.iter().filter(|s| s.status == SensorStatus::Active).count(),
            running_motors: self.motors.iter().filter(|m| m.status == MotorStatus::Running).count(),
            battery_level: self.power_manager.battery_level,
            emergency_stop: self.safety_kernel.emergency_stop,
            violations: self.safety_kernel.violations.iter().filter(|v| !v.resolved).count(),
        }
    }
}

/// 物理具身状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalSnapshot {
    pub sensor_count: usize,
    pub motor_count: usize,
    pub active_sensors: usize,
    pub running_motors: usize,
    pub battery_level: f64,
    pub emergency_stop: bool,
    pub violations: usize,
}

// 动态-音效同步模式库
pub mod audio_sync_library;

// 视频时序稳定性
pub mod video_temporal_stabilizer;
