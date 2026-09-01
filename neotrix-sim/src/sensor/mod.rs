use crate::physics::{Vec3, PhysicsState};

/// Virtual sensor array — mimics Microduck's sensor suite
pub struct SensorArray {
    /// ToF 8×8 depth matrix (VL53L5CX)
    tof_depth: [[f32; 8]; 8],
    /// IMU quaternion (body frame)
    imu_quaternion: Quaternion,
    /// IMU angular velocity
    imu_gyro: Vec3,
    /// IMU projected gravity
    imu_gravity: Vec3,
    /// Battery voltage (6.6V empty → 8.2V full)
    battery_voltage: f32,
    /// Joint positions (15 encoders)
    joint_positions: [f32; 15],
    /// Joint velocities
    joint_velocities: [f32; 15],
    /// Joint temperatures (15)
    joint_temperatures: [f32; 15],
    /// Motor supply voltage (read from bus)
    motor_voltage: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quaternion {
    pub fn from_euler(roll: f32, pitch: f32, yaw: f32) -> Self {
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();

        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }
}

impl SensorArray {
    pub fn new(physics: &PhysicsState) -> Self {
        let mut s = Self {
            tof_depth: [[0.3; 8]; 8],
            imu_quaternion: Quaternion::default(),
            imu_gyro: Vec3::default(),
            imu_gravity: Vec3::new(0.0, 0.0, -9.81),
            battery_voltage: 8.0,
            joint_positions: [0.0; 15],
            joint_velocities: [0.0; 15],
            joint_temperatures: [25.0; 15],
            motor_voltage: 8.0,
        };
        s.update(physics);
        s
    }

    pub fn update(&mut self, physics: &PhysicsState) {
        // IMU from physics state
        self.imu_quaternion = Quaternion::from_euler(
            physics.roll, physics.pitch, physics.yaw,
        );
        self.imu_gyro = physics.angular_velocity;
        self.imu_gravity = physics.projected_gravity;

        // ToF: simulate floor at distance based on height
        let height = physics.position.z.max(0.0);
        for row in 0..8 {
            for col in 0..8 {
                // Add noise
                let noise = (row as f32 * 0.01 + col as f32 * 0.01).sin() * 0.01;
                self.tof_depth[row][col] = height + noise;
            }
        }

        // Battery drains slowly (1 hour = 18000 ticks at 50Hz)
        self.battery_voltage = (self.battery_voltage - 0.00001).max(6.6);
        self.motor_voltage = self.battery_voltage;

        // Joint temperatures rise with activity
        for i in 0..15 {
            let activity = physics.joint_velocities[i].abs();
            self.joint_temperatures[i] += activity * 0.001;
            self.joint_temperatures[i] -= 0.0005; // cooling
            self.joint_temperatures[i] = self.joint_temperatures[i].clamp(20.0, 80.0);
        }
    }

    pub fn tof_depth(&self) -> &[[f32; 8]; 8] {
        &self.tof_depth
    }

    pub fn imu_quaternion(&self) -> Quaternion {
        self.imu_quaternion
    }

    pub fn imu_gyro(&self) -> Vec3 {
        self.imu_gyro
    }

    pub fn imu_gravity(&self) -> Vec3 {
        self.imu_gravity
    }

    pub fn battery_level(&self) -> f32 {
        (self.battery_voltage - 6.6) / (8.2 - 6.6)
    }

    pub fn battery_voltage(&self) -> f32 {
        self.battery_voltage
    }

    pub fn joint_positions(&self) -> &[f32; 15] {
        &self.joint_positions
    }

    pub fn joint_velocities(&self) -> &[f32; 15] {
        &self.joint_velocities
    }

    pub fn max_joint_temperature(&self) -> f32 {
        self.joint_temperatures.iter().cloned().fold(f32::MIN, f32::max)
    }

    pub fn motor_voltage(&self) -> f32 {
        self.motor_voltage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_level_range() {
        let physics = PhysicsState::new();
        let sensors = SensorArray::new(&physics);
        let level = sensors.battery_level();
        assert!(level >= 0.0 && level <= 1.0);
    }

    #[test]
    fn tof_all_positive() {
        let physics = PhysicsState::new();
        let sensors = SensorArray::new(&physics);
        for row in sensors.tof_depth() {
            for &val in row {
                assert!(val >= 0.0);
            }
        }
    }

    #[test]
    fn quaternion_from_zero_euler() {
        let q = Quaternion::from_euler(0.0, 0.0, 0.0);
        assert!((q.w - 1.0).abs() < 1e-6);
        assert!(q.x.abs() < 1e-6);
        assert!(q.y.abs() < 1e-6);
        assert!(q.z.abs() < 1e-6);
    }
}
