pub mod sensor;
pub mod actuator;
pub mod physics;
pub mod feel;
pub mod core_bridge;
pub mod world;

// NT-WORLD-SIM: consciousness evolution simulation
pub mod agents;
pub mod environment;
pub mod consciousness;
pub mod evolution;
pub mod society;
pub mod foundation;
pub mod world_sim;
pub mod bridge;
pub mod safety;

use std::time::{Duration, Instant};

pub const TICK_HZ: f64 = 50.0;
pub const TICK_DURATION: Duration = Duration::from_millis(20);

pub struct SimState {
    pub world: world::World,
    pub physics: physics::PhysicsState,
    pub sensors: sensor::SensorArray,
    pub actuators: actuator::ServoBus,
    pub feel: feel::EmotionEngine,
    pub core: core_bridge::CoreBridge,
    pub tick: u64,
    pub wall_time: Instant,
}

impl SimState {
    pub fn new() -> Self {
        let physics = physics::PhysicsState::new();
        let sensors = sensor::SensorArray::new(&physics);
        Self {
            world: world::World::new(),
            sensors,
            actuators: actuator::ServoBus::new(),
            physics,
            feel: feel::EmotionEngine::new(),
            core: core_bridge::CoreBridge::new(),
            tick: 0,
            wall_time: Instant::now(),
        }
    }

    /// 50Hz tick: physics → sensors → observe → policy → safety → write → feel
    pub fn tick(&mut self) {
        self.tick += 1;

        // 1. Physics step
        self.physics.step(&self.world, &self.actuators);

        // 2. Update sensors from physics state
        self.sensors.update(&self.physics);

        // 3. Collect observation vector (61-dim)
        let obs = self.observe();

        // 4. Policy inference (simplified: reactive controller)
        let actions = self.policy(&obs);

        // 5. Safety clamp
        let safe_actions = self.safety(&actions);

        // 6. Write to actuators
        self.actuators.write_goals(&safe_actions);

        // 7. Emotion update from system events
        let events = self.collect_events();
        self.feel.process_events(&events);

        // 8. Core bridge: emotion → attention weights
        self.core.update_from_feel(&self.feel);
    }

    /// 61-dimensional observation vector
    pub fn observe(&self) -> [f32; 61] {
        let mut obs = [0.0f32; 61];
        let physics = &self.physics;

        // gyro(3) - angular velocity
        obs[0] = physics.angular_velocity.x;
        obs[1] = physics.angular_velocity.y;
        obs[2] = physics.angular_velocity.z;

        // projected_gravity(3)
        obs[3] = physics.projected_gravity.x;
        obs[4] = physics.projected_gravity.y;
        obs[5] = physics.projected_gravity.z;

        // joint_pos(14) - positions in radians
        for i in 0..14 {
            obs[6 + i] = self.actuators.position(i);
        }

        // joint_vel(14) - velocities
        for i in 0..14 {
            obs[20 + i] = self.actuators.velocity(i);
        }

        // last_action(14)
        for i in 0..14 {
            obs[34 + i] = self.actuators.last_action(i);
        }

        // command(13) - vel(3) + head(4) + body(6)
        obs[48] = self.world.command_velocity.x;
        obs[49] = self.world.command_velocity.y;
        obs[50] = self.world.command_velocity.z;
        // head(4) and body(6) left as 0 for now

        obs
    }

    /// Simplified reactive policy
    fn policy(&self, obs: &[f32; 61]) -> [f32; 14] {
        let mut actions = [0.0f32; 14];

        let roll = obs[0];
        let pitch = obs[1];
        let is_falling = pitch.abs() > 0.3 || roll.abs() > 0.3;

        if is_falling {
            // Recovery: try to stand
            for i in 0..14 {
                actions[i] = -obs[6 + i] * 0.5; // move toward home
            }
        } else {
            // Simple walk: periodic leg alternation
            let phase = (self.tick as f32 * 0.125).sin();
            // Left leg
            actions[0] = phase * 0.2;   // hip_yaw
            actions[1] = phase * 0.1;   // hip_roll
            actions[2] = phase * 0.3;   // hip_pitch
            actions[3] = -phase.abs() * 0.4; // knee
            actions[4] = phase * 0.1;   // ankle
            // Right leg (opposite phase)
            actions[5] = -phase * 0.2;
            actions[6] = -phase * 0.1;
            actions[7] = -phase * 0.3;
            actions[8] = phase.abs() * 0.4;
            actions[9] = -phase * 0.1;
            // Head tracks command
            actions[10] = obs[48] * 0.5; // neck_pitch
            actions[11] = obs[49] * 0.5; // head_pitch
            actions[12] = obs[50] * 0.5; // head_yaw
            actions[13] = 0.0;           // head_roll
        }

        actions
    }

    /// Safety: clamp all actions to joint limits
    fn safety(&self, actions: &[f32; 14]) -> [f32; 14] {
        let mut safe = [0.0f32; 14];
        for i in 0..14 {
            let limit = self.actuators.joint_limit(i);
            safe[i] = actions[i].clamp(-limit, limit);
            // Reject NaN
            if safe[i].is_nan() {
                safe[i] = 0.0;
            }
        }
        safe
    }

    fn collect_events(&self) -> Vec<feel::SystemEvent> {
        let mut events = Vec::new();

        // Battery level
        events.push(feel::SystemEvent::BatteryLow {
            level: self.sensors.battery_level(),
        });

        // Falling detection
        if self.physics.is_falling() {
            events.push(feel::SystemEvent::ErrorRate { rate: 0.8 });
        } else {
            events.push(feel::SystemEvent::ErrorRate { rate: 0.1 });
        }

        // Novelty: check if environment has changed recently
        if self.world.novelty_score() > 0.5 {
            events.push(feel::SystemEvent::NoveltyDetected {
                score: self.world.novelty_score(),
            });
        }

        // Goal progress
        if self.physics.speed() > 0.1 {
            events.push(feel::SystemEvent::GoalAchieved { difficulty: 0.5 });
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_state_creation() {
        let state = SimState::new();
        assert_eq!(state.tick, 0);
    }

    #[test]
    fn sim_tick_runs() {
        let mut state = SimState::new();
        state.tick();
        assert_eq!(state.tick, 1);
    }

    #[test]
    fn observe_61dim() {
        let state = SimState::new();
        let obs = state.observe();
        assert_eq!(obs.len(), 61);
    }

    #[test]
    fn safety_clamps_nan() {
        let state = SimState::new();
        let actions = [f32::NAN; 14];
        let safe = state.safety(&actions);
        for a in &safe {
            assert!(!a.is_nan());
        }
    }
}
