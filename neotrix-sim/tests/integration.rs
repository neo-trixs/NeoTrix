/// Integration tests — complete perception→emotion→reasoning→action→feedback loop

#[cfg(test)]
mod perception_emotion_loop {
    use neotrix_sim::SimState;
    use neotrix_sim::feel::{EmotionType, SystemEvent};
    use neotrix_sim::physics::Vec3;
    use neotrix_sim::world::{Obstacle, ObstacleType};

    /// Helper: process events and update bridge
    fn process_and_update(state: &mut SimState, events: &[SystemEvent]) {
        state.feel.process_events(events);
        state.core.update_from_feel(&state.feel);
    }

    #[test]
    fn sensor_triggers_emotion_modulates_attention() {
        let mut state = SimState::new();

        state.tick();
        let baseline = state.feel.get_emotion(EmotionType::Curiosity);
        assert!(baseline < 0.3);

        state.world.add_obstacle(Obstacle {
            position: Vec3::new(0.5, 0.0, 0.035),
            size: Vec3::new(0.07, 0.07, 0.07),
            obstacle_type: ObstacleType::Ball,
        });

        state.tick();

        assert!(state.feel.get_emotion(EmotionType::Curiosity) > 0.5);
        assert!(state.core.novelty_weight() > 0.0);
        assert!(state.core.domain_weight("NT-WORLD") > 1.0);
    }

    #[test]
    fn error_triggers_anxiety_safety_bias() {
        let mut state = SimState::new();

        process_and_update(&mut state, &[SystemEvent::ErrorRate { rate: 0.9 }]);

        assert!(state.feel.get_emotion(EmotionType::Anxiety) > 0.5);
        assert!(state.core.domain_weight("NT-SHIELD") > 1.0);
        assert!(state.core.transition_bias.risk_bias < 0.0);
    }

    #[test]
    fn goal_achievement_positive_emotion() {
        let mut state = SimState::new();

        process_and_update(&mut state, &[SystemEvent::GoalAchieved { difficulty: 0.8 }]);

        assert!(state.feel.get_emotion(EmotionType::Satisfaction) > 0.4);
        assert!(state.feel.get_emotion(EmotionType::Joy) > 0.2);
        assert!(state.feel.pad.valence > 0.0);
    }

    #[test]
    fn battery_fatigue_conservative() {
        let mut state = SimState::new();

        process_and_update(&mut state, &[SystemEvent::BatteryLow { level: 0.1 }]);

        assert!(state.feel.get_emotion(EmotionType::Fatigue) > 0.5);
        assert!(state.core.transition_bias.exploration_bias < 0.0);
    }

    #[test]
    fn walking_control_loop() {
        let mut state = SimState::new();
        for _ in 0..100 {
            state.tick();
        }
        assert!(state.physics.height() > 0.2);
        assert!(!state.physics.is_falling());
        assert_eq!(state.observe().len(), 61);
    }

    #[test]
    fn fall_recovery_loop() {
        let mut state = SimState::new();
        state.physics.angular_velocity.x = 3.0;

        for _ in 0..50 { state.tick(); }
        assert!(state.physics.is_falling() || state.physics.tilt_angle() > 0.5);

        for _ in 0..200 { state.tick(); }
        assert!(state.physics.height() > 0.15);
    }

    #[test]
    fn emotion_homeostasis() {
        let mut state = SimState::new();

        process_and_update(&mut state, &[SystemEvent::NoveltyDetected { score: 1.0 }]);
        let peak = state.feel.get_emotion(EmotionType::Curiosity);
        assert!(peak > 0.7);

        // Run many ticks — emotion should decay but not vanish
        for _ in 0..200 { state.tick(); }

        let decayed = state.feel.get_emotion(EmotionType::Curiosity);
        assert!(decayed < peak, "Should decay");
        assert!(decayed > 0.01, "Should not vanish completely");
    }

    #[test]
    fn flow_state_emergence() {
        let mut state = SimState::new();
        process_and_update(&mut state, &[SystemEvent::TaskInProgress { duration: 5.0, difficulty: 0.5 }]);
        assert!(state.feel.get_emotion(EmotionType::Flow) > 0.0);
    }

    #[test]
    fn multi_emotion_pad_coherence() {
        let mut state = SimState::new();
        process_and_update(&mut state, &[
            SystemEvent::NoveltyDetected { score: 0.8 },
            SystemEvent::GoalAchieved { difficulty: 0.6 },
            SystemEvent::BatteryLow { level: 0.4 },
        ]);
        assert!(state.feel.emotions.len() >= 2);
        assert!(state.feel.pad.valence.abs() <= 1.0);
        assert!(state.feel.pad.arousal.abs() <= 1.0);
    }
}

#[cfg(test)]
mod multi_agent_loop {
    use neotrix_sim::world::fleet::SimFleet;
    use neotrix_sim::feel::EmotionType;

    #[test]
    fn fleet_tick() {
        let mut fleet = SimFleet::new(&["Alpha", "Beta", "Gamma"]);
        fleet.tick();
        assert_eq!(fleet.tick, 1);
        assert_eq!(fleet.agents.len(), 3);
    }

    #[test]
    fn group_emotion_emergence() {
        let mut fleet = SimFleet::new(&["A", "B", "C"]);
        fleet.update_group_field();
        assert!(fleet.group_field.consensus > 0.5);
    }

    #[test]
    fn ble_delivery() {
        let mut fleet = SimFleet::new(&["Near", "Far"]);
        fleet.tick();
        let total: usize = fleet.agents.iter().map(|a| a.received_beacons.len()).sum();
        assert!(total > 0);
    }

    #[test]
    fn joy_propagation() {
        let mut fleet = SimFleet::new(&["Leader", "Follower"]);
        fleet.agents[0].state.feel.process_events(&[
            neotrix_sim::feel::SystemEvent::GoalAchieved { difficulty: 0.9 },
        ]);
        fleet.tick();
        let trust = fleet.agents[1].state.feel.get_emotion(EmotionType::Trust);
        assert!(trust >= 0.0);
    }
}

#[cfg(test)]
mod evolution_loop {
    use neotrix_sim::SimState;

    struct EvolutionReward {
        total_distance: f32,
        fall_count: u32,
        recovery_count: u32,
        energy_efficiency: f32,
    }

    impl EvolutionReward {
        fn compute(&self) -> f32 {
            self.total_distance * 10.0
                - self.fall_count as f32 * 5.0
                + self.recovery_count as f32 * 3.0
                - (1.0 - self.energy_efficiency) * 20.0
        }
    }

    #[test]
    fn evolution_reward_computation() {
        let r = EvolutionReward { total_distance: 2.0, fall_count: 1, recovery_count: 1, energy_efficiency: 0.8 };
        assert!(r.compute() > 0.0);
    }

    #[test]
    fn policy_improvement_signal() {
        let ep1 = EvolutionReward { total_distance: 0.5, fall_count: 3, recovery_count: 0, energy_efficiency: 0.3 };
        let ep2 = EvolutionReward { total_distance: 2.0, fall_count: 1, recovery_count: 2, energy_efficiency: 0.8 };
        assert!(ep2.compute() > ep1.compute());
    }

    #[test]
    fn emotion_explores_exploits() {
        let mut s1 = SimState::new();
        s1.feel.process_events(&[neotrix_sim::feel::SystemEvent::NoveltyDetected { score: 1.0 }]);
        s1.core.update_from_feel(&s1.feel);
        let curiosity_exp = s1.core.transition_bias.exploration_bias;

        let mut s2 = SimState::new();
        s2.feel.process_events(&[neotrix_sim::feel::SystemEvent::BatteryLow { level: 0.1 }]);
        s2.core.update_from_feel(&s2.feel);
        let fatigue_exp = s2.core.transition_bias.exploration_bias;

        assert!(curiosity_exp > fatigue_exp);
    }
}
