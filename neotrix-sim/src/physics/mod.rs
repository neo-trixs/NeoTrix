use crate::actuator::ServoBus;
use crate::world::World;

/// Simplified 3D physics state for the robot body
pub struct PhysicsState {
    /// Position in world frame
    pub position: Vec3,
    /// Linear velocity
    pub velocity: Vec3,
    /// Angular velocity (gyro reading)
    pub angular_velocity: Vec3,
    /// Euler angles
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    /// Projected gravity in body frame
    pub projected_gravity: Vec3,
    /// Joint positions (mirrored from actuators)
    pub joint_positions: [f32; 15],
    /// Joint velocities
    pub joint_velocities: [f32; 15],
    /// Is the robot falling?
    falling: bool,
    /// Fall detection timer (debounce)
    fall_timer: f32,
    /// Ground contact (is foot on ground?)
    ground_contact: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len < 1e-8 {
            return *self;
        }
        Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
}

const GRAVITY: f32 = 9.81;
const GROUND_Z: f32 = 0.0;
const ROBOT_HEIGHT: f32 = 0.25; // 25cm
const MASS: f32 = 0.8; // 800g

impl PhysicsState {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, ROBOT_HEIGHT),
            velocity: Vec3::default(),
            angular_velocity: Vec3::default(),
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            projected_gravity: Vec3::new(0.0, 0.0, -1.0),
            joint_positions: [0.0; 15],
            joint_velocities: [0.0; 15],
            falling: false,
            fall_timer: 0.0,
            ground_contact: true,
        }
    }

    pub fn step(&mut self, _world: &World, actuators: &ServoBus) {
        let dt = 0.02; // 50Hz

        // Read actuator state
        let readings = actuators.sync_read();
        self.joint_positions = readings.positions;
        self.joint_velocities = readings.velocities;

        // Gravity in body frame (depends on orientation)
        let gravity_world = Vec3::new(0.0, 0.0, -GRAVITY);
        self.projected_gravity = self.rotate_vector_inv(&gravity_world);

        // Simple pendulum dynamics for balance
        let left_ankle_torque = self.joint_positions[4] * 2.0;
        let right_ankle_torque = self.joint_positions[9] * 2.0;
        let hip_torque = (self.joint_positions[2] + self.joint_positions[7]) * 1.0;

        let moment_of_inertia = MASS * ROBOT_HEIGHT * ROBOT_HEIGHT / 3.0;
        let net_torque = left_ankle_torque + right_ankle_torque + hip_torque;
        let angular_acc = net_torque / moment_of_inertia;

        // Update orientation
        self.pitch += self.angular_velocity.y * dt;
        self.roll += self.angular_velocity.x * dt;
        self.yaw += self.angular_velocity.z * dt;

        // Angular velocity from torque
        self.angular_velocity.z += angular_acc * dt;
        self.angular_velocity.x *= 0.95;
        self.angular_velocity.y *= 0.95;
        self.angular_velocity.z *= 0.95;

        // Gravity effect on pitch/roll
        self.pitch += self.projected_gravity.x * dt * 0.1;
        self.roll += self.projected_gravity.y * dt * 0.1;

        // Ground contact check
        self.ground_contact = self.position.z <= GROUND_Z + ROBOT_HEIGHT + 0.01;

        if self.ground_contact {
            self.velocity.z = self.velocity.z.max(0.0);
            self.position.z = (GROUND_Z + ROBOT_HEIGHT).max(self.position.z);

            // Walking locomotion from leg actions
            let left_phase = self.joint_positions[2]; // left hip pitch
            let right_phase = self.joint_positions[7]; // right hip pitch
            let forward_force = (left_phase - right_phase) * 2.0;
            self.velocity.x += forward_force * dt;

            // Lateral from hip roll
            let lateral_force = (self.joint_positions[1] - self.joint_positions[6]) * 1.0;
            self.velocity.y += lateral_force * dt;

            // Bob up/down from knee action
            let left_knee = self.joint_positions[3].abs();
            let right_knee = self.joint_positions[8].abs();
            let bob = (left_knee + right_knee) * 0.05;
            self.position.z = ROBOT_HEIGHT + bob;
        } else {
            // Free fall
            self.velocity.z -= GRAVITY * dt;
        }

        // Update position
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
        self.position.z += self.velocity.z * dt;

        // Ground collision
        if self.position.z < GROUND_Z + ROBOT_HEIGHT * 0.3 {
            self.position.z = GROUND_Z + ROBOT_HEIGHT * 0.3;
            self.velocity.z = 0.0;
            self.ground_contact = true;
        }

        // Friction
        self.velocity.x *= 0.98;
        self.velocity.y *= 0.98;

        // Fall detection (debounced)
        let tilt = self.roll.abs() + self.pitch.abs();
        if tilt > 0.5 {
            self.fall_timer += dt;
            if self.fall_timer > 0.2 {
                self.falling = true;
            }
        } else {
            self.fall_timer = (self.fall_timer - dt).max(0.0);
            if self.fall_timer <= 0.0 {
                self.falling = false;
            }
        }
    }

    /// Rotate vector from world to body frame (simplified)
    fn rotate_vector_inv(&self, v: &Vec3) -> Vec3 {
        let cr = self.roll.cos();
        let sr = self.roll.sin();
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();

        Vec3 {
            x: cp * v.x + sp * sr * v.y + sp * cr * v.z,
            y: cr * v.y - sr * v.z,
            z: -sp * v.x + cp * sr * v.y + cp * cr * v.z,
        }
    }

    pub fn is_falling(&self) -> bool {
        self.falling
    }

    pub fn speed(&self) -> f32 {
        self.velocity.length()
    }

    pub fn height(&self) -> f32 {
        self.position.z
    }

    pub fn tilt_angle(&self) -> f32 {
        (self.roll * self.roll + self.pitch * self.pitch).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_height() {
        let p = PhysicsState::new();
        assert!((p.position.z - ROBOT_HEIGHT).abs() < 1e-6);
    }

    #[test]
    fn gravity_pulls_down() {
        let mut p = PhysicsState::new();
        let world = World::new();
        let actuators = ServoBus::new();
        // Step a few times with no joint actions
        for _ in 0..50 {
            p.step(&world, &actuators);
        }
        // Should have fallen somewhat
        assert!(p.position.z < ROBOT_HEIGHT + 0.1);
    }

    #[test]
    fn fall_detection() {
        let mut p = PhysicsState::new();
        p.roll = 1.0; // severe tilt
        p.fall_timer = 0.3; // debounce exceeded
        p.falling = true;
        assert!(p.is_falling());
    }

    #[test]
    fn vec3_operations() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let c = a.cross(&b);
        assert!((c.z - 1.0).abs() < 1e-6);
    }
}
