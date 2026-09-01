/// Simulation world — ground plane, obstacles, command signals
///
/// Represents the physical environment the robot operates in.

pub mod fleet;

use crate::physics::Vec3;

pub struct World {
    /// Ground level (z coordinate)
    pub ground_level: f32,
    /// Gravity vector
    pub gravity: Vec3,
    /// Command velocity (from gamepad/controller)
    pub command_velocity: Vec3,
    /// Head pose command (neck_pitch, head_pitch, head_yaw, head_roll)
    pub head_command: [f32; 4],
    /// Body pose command (6-DOF)
    pub body_command: [f32; 6],
    /// Novelty score (0=stable, 1=new environment)
    novelty: f32,
    /// Time since last novelty event
    novelty_timer: f32,
    /// Obstacles in the world
    obstacles: Vec<Obstacle>,
    /// Number of steps
    step_count: u64,
}

pub struct Obstacle {
    pub position: Vec3,
    pub size: Vec3,
    pub obstacle_type: ObstacleType,
}

pub enum ObstacleType {
    Ball,
    Wall,
    Ramp,
}

impl World {
    pub fn new() -> Self {
        Self {
            ground_level: 0.0,
            gravity: Vec3::new(0.0, 0.0, -9.81),
            command_velocity: Vec3::new(0.5, 0.0, 0.0), // walk forward
            head_command: [0.0; 4],
            body_command: [0.0; 6],
            novelty: 0.0,
            novelty_timer: 0.0,
            obstacles: vec![
                Obstacle {
                    position: Vec3::new(1.0, 0.0, 0.035),
                    size: Vec3::new(0.07, 0.07, 0.07),
                    obstacle_type: ObstacleType::Ball,
                },
            ],
            step_count: 0,
        }
    }

    pub fn step(&mut self) {
        self.step_count += 1;

        // Novelty decays over time
        self.novelty_timer += 0.02;
        if self.novelty_timer > 5.0 {
            self.novelty = (self.novelty - 0.01).max(0.0);
        }

        // Random novelty events (simulate environment changes)
        if self.step_count % 500 == 0 {
            self.novelty = 0.8;
            self.novelty_timer = 0.0;
        }
    }

    pub fn novelty_score(&self) -> f32 {
        self.novelty
    }

    pub fn set_command_velocity(&mut self, v: Vec3) {
        self.command_velocity = v;
    }

    pub fn set_head_command(&mut self, cmd: [f32; 4]) {
        self.head_command = cmd;
    }

    pub fn add_obstacle(&mut self, obs: Obstacle) {
        self.obstacles.push(obs);
        self.novelty = 1.0;
        self.novelty_timer = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_creation() {
        let world = World::new();
        assert_eq!(world.ground_level, 0.0);
    }

    #[test]
    fn novelty_decays() {
        let mut world = World::new();
        world.novelty = 1.0;
        world.novelty_timer = 0.0;
        // Step past decay threshold
        for _ in 0..300 {
            world.step();
        }
        assert!(world.novelty < 1.0);
    }

    #[test]
    fn add_obstacle_increases_novelty() {
        let mut world = World::new();
        world.novelty = 0.0;
        world.add_obstacle(Obstacle {
            position: Vec3::new(2.0, 0.0, 0.05),
            size: Vec3::new(0.1, 0.1, 0.1),
            obstacle_type: ObstacleType::Ball,
        });
        assert!(world.novelty > 0.5);
    }
}
