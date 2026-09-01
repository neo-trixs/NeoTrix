use neotrix_sim::SimState;
use neotrix_sim::feel::EmotionType;
use neotrix_sim::physics::Vec3;

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("  NeoTrix Consciousness Embodiment Simulator v0.1");
    println!("  NT-CORE × NT-FEEL × NT-PHYSICAL co-evolution");
    println!("═══════════════════════════════════════════════════════");
    println!();

    let mut state = SimState::new();

    // ═══ Phase 0: Walking ═══
    println!("▸ Phase 0: Walking (500 ticks @ 50Hz = 10s)");
    println!("{:<8} {:<8} {:<8} {:<6} {:<14} {:<8}",
        "Tick", "Height", "Speed", "Fall", "Dominant", "Battery");
    println!("{}", "─".repeat(58));

    for tick in 0..500 {
        state.tick();
        if tick % 100 == 0 {
            let dominant = state.feel.dominant_emotion()
                .map(|e| format!("{:?}", e))
                .unwrap_or_else(|| "None".into());
            println!("{:<8} {:<8.3} {:<8.3} {:<6} {:<14} {:<8.2}",
                tick, state.physics.height(), state.physics.speed(),
                state.physics.is_falling(), dominant, state.sensors.battery_level());
        }
    }

    // ═══ Phase 1: Push → Fall → Recover ═══
    println!();
    println!("▸ Phase 1: Push event (angular velocity spike)");

    // Strong push to the side
    state.physics.angular_velocity.x = 3.0;
    state.physics.angular_velocity.z = 1.5;

    for tick in 0..200 {
        state.tick();
        if tick % 20 == 0 {
            let dominant = state.feel.dominant_emotion()
                .map(|e| format!("{:?}", e))
                .unwrap_or_else(|| "None".into());
            println!("  t={:<4} tilt={:.3} falling={} vel=({:.2},{:.2},{:.2}) emotion={}",
                tick, state.physics.tilt_angle(), state.physics.is_falling(),
                state.physics.velocity.x, state.physics.velocity.y, state.physics.velocity.z,
                dominant);
        }
    }

    // ═══ Phase 2: Novelty discovery ═══
    println!();
    println!("▸ Phase 2: Novelty event (new obstacle)");

    state.world.add_obstacle(neotrix_sim::world::Obstacle {
        position: Vec3::new(0.5, 0.3, 0.035),
        size: Vec3::new(0.07, 0.07, 0.07),
        obstacle_type: neotrix_sim::world::ObstacleType::Ball,
    });

    for tick in 0..200 {
        state.tick();
        if tick % 20 == 0 {
            let curiosity = state.feel.get_emotion(EmotionType::Curiosity);
            let satisfaction = state.feel.get_emotion(EmotionType::Satisfaction);
            println!("  t={:<4} curiosity={:.3} satisfaction={:.3} pad=({:.2}, {:.2}, {:.2})",
                tick, curiosity, satisfaction,
                state.feel.pad.valence, state.feel.pad.arousal, state.feel.pad.dominance);
        }
    }

    // ═══ Phase 3: Long run → Fatigue ═══
    println!();
    println!("▸ Phase 3: Extended operation (fatigue accumulation)");

    for tick in 0..1000 {
        state.tick();
        if tick % 200 == 0 {
            let fatigue = state.feel.get_emotion(EmotionType::Fatigue);
            let battery = state.sensors.battery_level();
            println!("  t={:<4} battery={:.3} fatigue={:.3} pad_v={:.3}",
                tick, battery, fatigue, state.feel.pad.valence);
        }
    }

    // ═══ Phase 4: Multi-emotion interaction ═══
    println!();
    println!("▸ Phase 4: Combined events (curiosity + goal + error)");

    // Simulate: discover something → achieve goal → hit error
    state.world.add_obstacle(neotrix_sim::world::Obstacle {
        position: Vec3::new(1.0, 0.0, 0.05),
        size: Vec3::new(0.1, 0.1, 0.1),
        obstacle_type: neotrix_sim::world::ObstacleType::Ramp,
    });

    for tick in 0..300 {
        state.tick();

        // At tick 50: achieve a goal
        if tick == 50 {
            state.feel.process_events(&[
                neotrix_sim::feel::SystemEvent::GoalAchieved { difficulty: 0.8 },
            ]);
        }
        // At tick 150: encounter error
        if tick == 150 {
            state.feel.process_events(&[
                neotrix_sim::feel::SystemEvent::ErrorRate { rate: 0.9 },
            ]);
        }
        // At tick 250: recover
        if tick == 250 {
            state.feel.process_events(&[
                neotrix_sim::feel::SystemEvent::RecoverySuccess,
            ]);
        }

        if tick % 50 == 0 {
            let emotions: Vec<String> = state.feel.emotions.iter()
                .map(|e| format!("{}:{:.2}", format!("{:?}", e.type_)[..3].to_lowercase(), e.intensity))
                .collect();
            println!("  t={:<4} emotions=[{}] pad=({:.2},{:.2},{:.2})",
                tick, emotions.join(", "),
                state.feel.pad.valence, state.feel.pad.arousal, state.feel.pad.dominance);
        }
    }

    // ═══ Final Report ═══
    println!();
    println!("═══ Final State ═══");
    println!("Total ticks: {}", state.tick);
    println!("Active emotions: {}", state.feel.emotions.len());
    println!("PAD: ({:.3}, {:.3}, {:.3})",
        state.feel.pad.valence, state.feel.pad.arousal, state.feel.pad.dominance);

    if let Some(dominant) = state.feel.dominant_emotion() {
        println!("Dominant: {:?}", dominant);
    }

    println!();
    println!("CT branch priorities:");
    for (name, weight) in &state.core.ct_priorities.weights {
        if !name.is_empty() {
            println!("  {}: {:.2}", name, weight);
        }
    }

    println!();
    println!("Attention focus width: {:.3}", state.core.focus_width());
    println!("E8 exploration bias: {:.3}", state.core.transition_bias.exploration_bias);
    println!("E8 risk bias: {:.3}", state.core.transition_bias.risk_bias);

    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("  Simulation complete.");
    println!("  Consciousness embodiment: NT-CORE ✓ NT-FEEL ✓ NT-PHYSICAL ✓");
    println!("═══════════════════════════════════════════════════════");
}
