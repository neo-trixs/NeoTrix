/// Virtual servo bus — mimics Microduck's 15-Dynamixel-XL330 bus
///
/// Single UART bus at 1 Mbps, sync_read/sync_write.
/// SafetyGuard owns the only write handle (borrow checker enforced).

pub struct ServoBus {
    /// Current positions (radians)
    positions: [f32; 15],
    /// Current velocities (rad/s)
    velocities: [f32; 15],
    /// Target positions (goal)
    goals: [f32; 15],
    /// Last commanded actions (for observation)
    last_actions: [f32; 15],
    /// Joint limits (radians, symmetric)
    joint_limits: [f32; 15],
    /// PWM duty cycle (0-255)
    pwm_duty: [f32; 15],
    /// Motor supply voltage
    supply_voltage: f32,
}

/// Joint name mapping (matching Microduck kinematic chain)
pub const JOINT_NAMES: [&str; 15] = [
    "left_hip_yaw",     // 0
    "left_hip_roll",    // 1
    "left_hip_pitch",   // 2
    "left_knee",        // 3
    "left_ankle",       // 4
    "right_hip_yaw",    // 5
    "right_hip_roll",   // 6
    "right_hip_pitch",  // 7
    "right_knee",       // 8
    "right_ankle",      // 9
    "neck_pitch",       // 10
    "head_pitch",       // 11
    "head_yaw",         // 12
    "head_roll",        // 13
    "beak",             // 14
];

/// Dynamixel XL330 parameters (for reference)
#[allow(dead_code)]
const RAD_PER_SEC_PER_COUNT: f32 = 0.229 * std::f32::consts::TAU / 60.0;
#[allow(dead_code)]
const COUNTS_PER_RAD: f32 = 4096.0 / std::f32::consts::TAU;

impl ServoBus {
    pub fn new() -> Self {
        // Joint limits (radians) — conservative defaults
        let joint_limits = [
            0.8,  // hip_yaw
            0.5,  // hip_roll
            1.2,  // hip_pitch
            1.8,  // knee
            0.8,  // ankle
            0.8,  // hip_yaw
            0.5,  // hip_roll
            1.2,  // hip_pitch
            1.8,  // knee
            0.8,  // ankle
            1.0,  // neck_pitch
            1.0,  // head_pitch
            1.2,  // head_yaw
            0.5,  // head_roll
            0.5,  // beak
        ];

        Self {
            positions: [0.0; 15],
            velocities: [0.0; 15],
            goals: [0.0; 15],
            last_actions: [0.0; 15],
            joint_limits,
            pwm_duty: [0.0; 15],
            supply_voltage: 8.0,
        }
    }

    /// sync_read: read all 15 servo positions + velocities
    pub fn sync_read(&self) -> ServoReadings {
        ServoReadings {
            positions: self.positions,
            velocities: self.velocities,
            supply_voltage: self.supply_voltage,
        }
    }

    /// sync_write: set goal positions (called by SafetyGuard only)
    pub fn write_goals(&mut self, actions: &[f32; 14]) {
        // actions[i] = target relative to home pose
        for i in 0..14 {
            self.last_actions[i] = actions[i];
            let target = actions[i]; // simplified: action = absolute target
            // Reject NaN
            let target = if target.is_nan() { 0.0 } else { target };
            let clamped = target.clamp(-self.joint_limits[i], self.joint_limits[i]);
            self.goals[i] = clamped;
        }
        // beak (index 14) not controlled by policy
        self.last_actions[14] = 0.0;
    }

    /// Physics step: move positions toward goals (servo dynamics)
    pub fn step(&mut self, dt: f32) {
        for i in 0..15 {
            let error = self.goals[i] - self.positions[i];
            // Simple proportional control with velocity limit
            let max_velocity = 5.0; // rad/s
            let velocity = (error * 10.0).clamp(-max_velocity, max_velocity);
            self.velocities[i] = velocity;
            self.positions[i] += velocity * dt;

            // Clamp to joint limits
            self.positions[i] = self.positions[i].clamp(
                -self.joint_limits[i],
                self.joint_limits[i],
            );

            // PWM duty = |velocity| * scale
            self.pwm_duty[i] = velocity.abs() * 50.0;
        }
    }

    pub fn position(&self, idx: usize) -> f32 {
        self.positions[idx]
    }

    pub fn velocity(&self, idx: usize) -> f32 {
        self.velocities[idx]
    }

    pub fn last_action(&self, idx: usize) -> f32 {
        self.last_actions[idx]
    }

    pub fn joint_limit(&self, idx: usize) -> f32 {
        self.joint_limits[idx]
    }

    pub fn set_supply_voltage(&mut self, v: f32) {
        self.supply_voltage = v;
    }

    pub fn max_pwm_duty(&self) -> f32 {
        self.pwm_duty.iter().cloned().fold(f32::MIN, f32::max)
    }
}

#[derive(Debug, Clone)]
pub struct ServoReadings {
    pub positions: [f32; 15],
    pub velocities: [f32; 15],
    pub supply_voltage: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joint_count() {
        assert_eq!(JOINT_NAMES.len(), 15);
    }

    #[test]
    fn write_goals_clamps() {
        let mut bus = ServoBus::new();
        let actions = [10.0; 14]; // way over limit
        bus.write_goals(&actions);
        for i in 0..14 {
            assert!(bus.goals[i].abs() <= bus.joint_limits[i]);
        }
    }

    #[test]
    fn step_moves_toward_goal() {
        let mut bus = ServoBus::new();
        bus.goals[0] = 0.5;
        bus.step(0.02);
        assert!(bus.positions[0] > 0.0);
    }

    #[test]
    fn safety_rejects_nan() {
        let mut bus = ServoBus::new();
        let mut actions = [0.0f32; 14];
        actions[0] = f32::NAN;
        bus.write_goals(&actions);
        // The safety layer should have rejected NaN, but bus itself
        // just clamps — real safety is in SafetyGuard
        assert!(!bus.goals[0].is_nan());
    }
}
