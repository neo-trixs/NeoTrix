#![forbid(unsafe_code)]

// Dead test: references removed game modules (CrystalWorld, CrystalCard, etc.)
// Gated behind a feature that doesn't exist to prevent compilation.
#[cfg(all(test, feature = "crystal_game_disabled"))]
mod crystal_integration_tests {
    use neotrix::l5_cognition::nt_mind::nt_game::*;
    use neotrix::l5_cognition::nt_mind::nt_game::crystal_signal::CrystalValue;
    use neotrix::l5_cognition::nt_mind::nt_game::crystal_behavior::{BehaviorStatus, BehaviorNodeType};
    use neotrix::l5_cognition::nt_mind::nt_game::crystal_state::{CrystalState, CrystalTransition};
    use neotrix::l5_cognition::nt_mind::nt_game::crystal_card::{CrystalEffectType, CrystalRarity, CrystalTargetType};
    use std::collections::HashMap;

    // ── helpers ──────────────────────────────────────────────────────
    #[derive(Debug)]
    struct Position { x: f32, y: f32 }
    impl CrystalComponent for Position {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    }

    #[derive(Debug)]
    struct Health { current: i32, max: i32 }
    impl CrystalComponent for Health {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    }

    fn make_state(id: u32, name: &str) -> CrystalState {
        CrystalState { id, name: name.to_string(), on_enter: None, on_exit: None }
    }

    // ── Test 1: ECS + Scene Tree bridge ─────────────────────────────
    #[test]
    fn test_ecs_and_scene_tree_bridge() {
        // arrange
        let mut world = CrystalWorld::new();
        let mut tree = CrystalSceneTree::new();

        // act — spawn entity
        let entity = world.spawn();
        world.add_component(entity, Position { x: 1.0, y: 2.0 });
        world.add_component(entity, Health { current: 100, max: 100 });

        // act — create scene node
        let root = tree.root();
        let node = tree.create_node("PlayerNode", root);

        // assert — ECS works independently
        let pos = world.get_component::<Position>(entity).expect("entity should have Position");
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        let hp = world.get_component::<Health>(entity).expect("entity should have Health");
        assert_eq!(hp.current, 100);

        // assert — scene tree works independently
        let node_data = tree.get_node(node).expect("node should exist");
        assert_eq!(node_data.name, "PlayerNode");
        assert_eq!(node_data.parent, Some(root));
        assert_eq!(tree.node_count(), 2);
        assert!(tree.children(root).contains(&node));

        // assert — query works
        let entities_with_pos = world.query::<Position>();
        assert_eq!(entities_with_pos.len(), 1);
        assert_eq!(entities_with_pos[0], entity);
    }

    // ── Test 2: Signal + Event Bus coordination ─────────────────────
    #[test]
    fn test_signal_and_event_bus_coordination() {
        // arrange
        let mut signals = CrystalSignalSystem::new();
        let mut bus = CrystalEventBus::new();

        // act — connect signals
        let conn1 = signals.connect("damaged", 42, "on_damaged");
        let conn2 = signals.connect("healed", 42, "on_healed");
        assert!(signals.has_connections("damaged"));
        assert!(signals.has_connections("healed"));

        // act — create event channels
        let ch_combat = bus.create_channel("combat_events");
        let ch_ui = bus.create_channel("ui_updates");
        bus.subscribe(ch_combat, 100);
        bus.subscribe(ch_combat, 200);
        bus.subscribe(ch_ui, 300);

        // assert — signal system handles events independently
        let results = signals.emit("damaged", &[CrystalValue::Int(50)]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 42);
        assert_eq!(results[0].1, "on_damaged");
        assert!(matches!(&results[0].2[0], CrystalValue::Int(50)));

        // assert — event bus handles subscribers independently
        assert_eq!(bus.subscriber_count(ch_combat), 2);
        assert_eq!(bus.subscriber_count(ch_ui), 1);

        // act — disconnect signal
        assert!(signals.disconnect(conn1));
        assert!(!signals.has_connections("damaged"));
        assert!(signals.has_connections("healed"));

        // act — unsubscribe event bus
        bus.unsubscribe(ch_combat, 100);
        assert_eq!(bus.subscriber_count(ch_combat), 1);

        // act — cleanup
        signals.cleanup(42);
        assert_eq!(signals.connection_count(), 0);
        bus.cleanup_subscriber(300);
        assert_eq!(bus.subscriber_count(ch_ui), 0);
    }

    // ── Test 3: Card System lifecycle ───────────────────────────────
    #[test]
    fn test_card_system_lifecycle() {
        // arrange — build cards
        let slash = CrystalCard::new(1, "Slash", CrystalCardType::Attack, 1)
            .with_effect(CrystalCardEffect {
                effect_type: CrystalEffectType::Damage,
                value: 6,
                duration: None,
            })
            .with_target(CrystalTargetType::SingleEnemy)
            .with_rarity(CrystalRarity::Starter);

        let shield = CrystalCard::new(2, "Shield", CrystalCardType::Skill, 1)
            .with_effect(CrystalCardEffect {
                effect_type: CrystalEffectType::Block,
                value: 5,
                duration: None,
            });

        let ultimate = CrystalCard::new(3, "Ultimate", CrystalCardType::Power, 3)
            .with_effect(CrystalCardEffect {
                effect_type: CrystalEffectType::Damage,
                value: 20,
                duration: None,
            })
            .with_effect(CrystalCardEffect {
                effect_type: CrystalEffectType::ApplyVulnerable,
                value: 2,
                duration: Some(2),
            });

        // assert — card properties
        assert_eq!(slash.cost, 1);
        assert_eq!(slash.effects.len(), 1);
        assert_eq!(ultimate.effects.len(), 2);
        assert_eq!(ultimate.effects[1].duration, Some(2));

        // act — build deck
        let mut deck = CrystalDeck::new(vec![slash.clone(), shield.clone(), ultimate.clone()]);
        assert_eq!(deck.total_cards(), 3);
        assert_eq!(deck.draw_pile.len(), 3);

        // act — draw cards
        let drawn = deck.draw(2);
        assert_eq!(drawn.len(), 2);
        assert_eq!(deck.hand.len(), 2);
        assert_eq!(deck.draw_pile.len(), 1);

        // act — play card
        let played = deck.play_card(1);
        assert!(played.is_some());
        assert_eq!(played.unwrap().name, "Slash");
        assert_eq!(deck.hand.len(), 1);
        assert_eq!(deck.discard_pile.len(), 1);

        // act — exhaust card
        let exhausted = deck.exhaust_card(2);
        assert!(exhausted.is_some());
        assert_eq!(deck.hand.len(), 0);
        assert_eq!(deck.exhaust_pile.len(), 1);
        assert_eq!(deck.total_cards(), 3);

        // act — discard remaining
        deck.discard_hand();
        assert_eq!(deck.discard_pile.len(), 1);

        // act — upgrade card
        let mut upgraded_slash = CrystalCard::new(10, "Slash", CrystalCardType::Attack, 1)
            .with_effect(CrystalCardEffect {
                effect_type: CrystalEffectType::Damage,
                value: 6,
                duration: None,
            });
        upgraded_slash.upgrade();
        assert!(upgraded_slash.upgraded);
        assert_eq!(upgraded_slash.name, "Slash+");
        assert_eq!(upgraded_slash.effects[0].value, 9); // 6 * 1.5 = 9
    }

    // ── Test 4: State Machine + Behavior Tree coordination ──────────
    #[test]
    fn test_state_machine_and_behavior_tree_coordination() {
        // arrange — state machine
        let mut sm = CrystalStateMachine::new();
        sm.add_state(make_state(1, "Idle"));
        sm.add_state(make_state(2, "Chasing"));
        sm.add_state(make_state(3, "Attacking"));
        sm.add_transition(CrystalTransition {
            from: 1, to: 2, event: 100, guard: None, action: None,
        });
        sm.add_transition(CrystalTransition {
            from: 2, to: 3, event: 200, guard: None, action: None,
        });
        sm.add_transition(CrystalTransition {
            from: 3, to: 1, event: 300, guard: None, action: None,
        });
        sm.current_state = Some(1);

        // act — drive state machine through transitions
        assert!(sm.handle_event(100));
        assert_eq!(sm.current_state(), Some(2));
        assert!(sm.handle_event(200));
        assert_eq!(sm.current_state(), Some(3));
        assert!(sm.handle_event(300));
        assert_eq!(sm.current_state(), Some(1));

        // assert — guard blocks transition
        sm.add_transition(CrystalTransition {
            from: 1, to: 2, event: 999,
            guard: Some(Box::new(|| false)),
            action: None,
        });
        sm.current_state = Some(1);
        assert!(!sm.handle_event(999));
        assert_eq!(sm.current_state(), Some(1));

        // arrange — behavior tree
        let mut bt = CrystalBehaviorTree::new();
        bt.add_node(CrystalBehaviorNode {
            id: 0, name: "Root".into(),
            node_type: BehaviorNodeType::Sequence,
            children: vec![1, 2],
        });
        bt.add_node(CrystalBehaviorNode {
            id: 1, name: "CheckEnemy".into(),
            node_type: BehaviorNodeType::Condition("has_enemy".into()),
            children: vec![],
        });
        bt.add_node(CrystalBehaviorNode {
            id: 2, name: "Attack".into(),
            node_type: BehaviorNodeType::Action("attack".into()),
            children: vec![],
        });
        bt.set_root(0);

        // act & assert — tick behavior tree
        let status = bt.tick();
        assert_eq!(status, BehaviorStatus::Success);

        // assert — selector tree
        let mut bt_sel = CrystalBehaviorTree::new();
        bt_sel.add_node(CrystalBehaviorNode {
            id: 10, name: "SelRoot".into(),
            node_type: BehaviorNodeType::Selector,
            children: vec![11, 12],
        });
        bt_sel.add_node(CrystalBehaviorNode {
            id: 11, name: "Action1".into(),
            node_type: BehaviorNodeType::Action("act1".into()),
            children: vec![],
        });
        bt_sel.add_node(CrystalBehaviorNode {
            id: 12, name: "Action2".into(),
            node_type: BehaviorNodeType::Action("act2".into()),
            children: vec![],
        });
        bt_sel.set_root(10);
        assert_eq!(bt_sel.tick(), BehaviorStatus::Success);

        // assert — no root returns Failure
        assert_eq!(CrystalBehaviorTree::new().tick(), BehaviorStatus::Failure);
    }

    // ── Test 5: Memory Hierarchy + Router coordination ──────────────
    #[test]
    fn test_memory_hierarchy_and_router_coordination() {
        // arrange
        let hierarchy = CrystalMemoryHierarchy::new(
            8 * 1024 * 1024 * 1024,  // 8GB VRAM
            16 * 1024 * 1024 * 1024, // 16GB RAM
            None,
        );
        assert!(hierarchy.is_ok());

        let mut router = CrystalRouter::new();

        // act — route a context
        let context = vec![1, 2, 3, 4, 5];
        let decision = router.route(&context, 0);
        assert!(decision.is_ok());

        let decision = decision.unwrap();
        // assert — routing decision has 6 experts
        assert_eq!(decision.experts.len(), 6);
        assert_eq!(decision.weights.len(), 6);
        assert!(decision.confidence > 0.0);

        // assert — experts are correctly indexed (layer 0: experts 0-5)
        for i in 0..6 {
            assert_eq!(decision.experts[i], i as u64);
        }

        // act — route on different layer
        let decision2 = router.route(&context, 1).unwrap();
        for i in 0..6 {
            assert_eq!(decision2.experts[i], (6 + i) as u64);
        }

        // act — predict next layer
        let predicted = router.predict_next_layer(&context, 2);
        assert_eq!(predicted.len(), 6);
        for i in 0..6 {
            assert_eq!(predicted[i], (12 + i) as u64);
        }

        // act — update pins
        router.update_pins(0, 0.5);
        router.update_pins(0, 0.3);

        // act — prefetch on hierarchy
        let mut hierarchy = hierarchy.unwrap();
        hierarchy.prefetch(&[1, 2, 3]);
        hierarchy.update_heat(1, 0.9);

        // assert — LRU cache tracks prefetched experts
        assert!(hierarchy.is_cached(1));
    }

    // ── Test 6: Melting Engine end-to-end ───────────────────────────
    #[test]
    fn test_melting_engine_end_to_end() {
        // arrange
        let mut engine = MeltingEngine::new();

        let input = MeltInput {
            input_type: InputType::Code {
                language: "rust".to_string(),
                content: "struct Player {\n    x: f32,\n    y: f32,\n    name: String,\n    health: u32,\n}".to_string(),
            },
            source: "test_source".to_string(),
            metadata: HashMap::new(),
        };

        // act
        let output = engine.melt(input);

        // assert
        assert!(output.is_ok());
        let output = output.unwrap();

        assert_eq!(output.component_name, "CrystalPlayer");
        assert!(output.code.contains("pub x: f32"));
        assert!(output.code.contains("pub y: f32"));
        assert!(output.code.contains("pub name: String"));
        assert!(output.code.contains("pub health: u32"));
        assert!(output.code.contains("pub fn new()"));
        assert!(!output.documentation.is_empty());
        assert!(output.documentation.contains("CrystalPlayer"));

        // assert — component is registered
        let registered = engine.get_component("CrystalPlayer");
        assert!(registered.is_some());
        assert_eq!(engine.get_all_components().len(), 1);

        // act — melt a second input (scene tree pattern)
        let input2 = MeltInput {
            input_type: InputType::Code {
                language: "gdscript".to_string(),
                content: "class Enemy extends Node\n    parent: Node\n    health: i32".to_string(),
            },
            source: "test_source_2".to_string(),
            metadata: HashMap::new(),
        };
        let output2 = engine.melt(input2).unwrap();
        assert_eq!(output2.component_name, "CrystalEnemy");
        assert!(output2.code.contains("pub parent: Node"));
        assert!(output2.code.contains("pub health: i32"));
        assert_eq!(engine.get_all_components().len(), 2);
    }

    // ── Test 7: Full pipeline simulation ────────────────────────────
    #[test]
    fn test_full_pipeline_simulation() {
        // arrange — all components
        let mut world = CrystalWorld::new();
        let mut tree = CrystalSceneTree::new();
        let mut signals = CrystalSignalSystem::new();
        let mut bus = CrystalEventBus::new();
        let mut resource_mgr = CrystalResourceManager::new();
        let mut router = CrystalRouter::new();

        // simulate a game turn

        // 1. spawn entity in ECS
        let entity = world.spawn();
        world.add_component(entity, Position { x: 0.0, y: 0.0 });
        world.add_component(entity, Health { current: 80, max: 100 });
        assert_eq!(world.query::<Health>().len(), 1);

        // 2. create scene node for entity
        let root = tree.root();
        let node = tree.create_node("GameEntity", root);
        assert_eq!(tree.node_count(), 2);

        // 3. add card component (resource)
        let card_data = b"fireball_card_data";
        let handle = resource_mgr.insert("FireballCard", card_data.to_vec());
        assert_eq!(resource_mgr.count(), 1);
        let res = resource_mgr.get(handle).expect("resource should exist");
        assert_eq!(res.name, "FireballCard");
        assert_eq!(res.data, card_data);

        // 4. draw card from deck
        let cards = create_starter_deck();
        let mut deck = CrystalDeck::new(cards);
        let drawn = deck.draw(3);
        assert_eq!(drawn.len(), 3);
        assert_eq!(deck.hand.len(), 3);

        // 5. connect signal for damage event
        signals.connect("entity_damaged", entity.0, "on_damage");
        assert!(signals.has_connections("entity_damaged"));

        // 6. create event channel and subscribe
        let ch = bus.create_channel("turn_events");
        bus.subscribe(ch, entity.0);
        assert_eq!(bus.subscriber_count(ch), 1);

        // 7. route through inference
        let context: Vec<u32> = drawn.iter().map(|c| c.id as u32).collect();
        let decision = router.route(&context, 0).unwrap();
        assert_eq!(decision.experts.len(), 6);

        // 8. verify no panics, all results are Some/Ok
        assert!(world.get_component::<Position>(entity).is_some());
        assert!(tree.get_node(node).is_some());
        assert!(resource_mgr.get(handle).is_some());
        assert!(deck.play_card(drawn[0].id).is_some());

        // cleanup
        signals.cleanup(entity.0);
        bus.cleanup_subscriber(entity.0);
        resource_mgr.remove(handle);
        assert_eq!(resource_mgr.count(), 0);
    }

    // ── Test 8: Resource Manager lifecycle ───────────────────────────
    #[test]
    fn test_resource_manager_lifecycle() {
        // arrange
        let mut mgr = CrystalResourceManager::new();
        assert_eq!(mgr.count(), 0);

        // act — insert resources
        let h1 = mgr.insert("texture_png", vec![0x89, 0x50, 0x4E, 0x47]);
        let h2 = mgr.insert("model_gltf", vec![0x67, 0x6C, 0x54, 0x46]);
        let h3 = mgr.insert("shader_glsl", b"void main() {}".to_vec());
        assert_eq!(mgr.count(), 3);

        // assert — get by handle
        let r1 = mgr.get(h1).expect("texture resource should exist");
        assert_eq!(r1.name, "texture_png");
        assert_eq!(r1.data, vec![0x89, 0x50, 0x4E, 0x47]);
        assert_eq!(r1.ref_count, 1);

        let r2 = mgr.get(h2).expect("model resource should exist");
        assert_eq!(r2.name, "model_gltf");

        let r3 = mgr.get(h3).expect("shader resource should exist");
        assert_eq!(r3.name, "shader_glsl");
        assert_eq!(r3.data, b"void main() {}");

        // assert — get_mut works
        {
            let r_mut = mgr.get_mut(h1).expect("should get mutable");
            r_mut.data.push(0x0D);
            r_mut.ref_count += 1;
        }
        let r1_updated = mgr.get(h1).unwrap();
        assert_eq!(r1_updated.data.len(), 5);
        assert_eq!(r1_updated.ref_count, 2);

        // act — remove resources
        let removed = mgr.remove(h2);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "model_gltf");
        assert_eq!(mgr.count(), 2);

        // assert — removed resource is gone
        assert!(mgr.get(h2).is_none());
        assert!(mgr.get(h1).is_some());
        assert!(mgr.get(h3).is_some());

        // act — remove remaining
        mgr.remove(h1);
        mgr.remove(h3);
        assert_eq!(mgr.count(), 0);

        // assert — handles are invalidated after removal
        assert!(mgr.get(h1).is_none());
        assert!(mgr.get(h3).is_none());
    }

    // ── Test 9: Speculator + Router integration ─────────────────────
    #[test]
    fn test_speculator_and_router_integration() {
        let mut router = CrystalRouter::new();
        let speculator = CrystalSpeculator::new(SpeculationConfig {
            enabled: true,
            draft_length: 5,
        });

        // route context to get expert selection
        let context = vec![10, 20, 30];
        let decision = router.route(&context, 0).unwrap();
        assert_eq!(decision.experts.len(), 6);

        // use expert weights as hidden state for speculative decode
        let hidden: Vec<f32> = decision.weights.iter().map(|w| *w as f32).collect();
        let tokens = speculator.speculative_decode(&context, &hidden).unwrap();

        // speculative decode produces some tokens (accepted ones)
        // acceptance depends on token values mod 3
        assert!(tokens.len() <= 5);
    }

    // ── Test 10: EcsSceneBridge ────────────────────────────────────
    #[test]
    fn test_ecs_scene_bridge() {
        use neotrix::l5_cognition::nt_mind::nt_game::bridge_ecs_scene::EcsSceneBridge;

        let mut world = CrystalWorld::new();
        let mut tree = CrystalSceneTree::new();
        let mut bridge = EcsSceneBridge::new();

        let entity = world.spawn();
        world.add_component(entity, Position { x: 5.0, y: 10.0 });
        let node = tree.create_node("EntityNode", tree.root());

        bridge.bind(entity, node);
        assert_eq!(bridge.entity_node(entity), Some(node));
        assert_eq!(bridge.node_entity(node), Some(entity));
        assert_eq!(bridge.count(), 1);

        // unbind
        assert_eq!(bridge.unbind_entity(entity), Some(node));
        assert!(bridge.entity_node(entity).is_none());
        assert_eq!(bridge.count(), 0);
    }

    // ── Test 11: SignalEventBridge ─────────────────────────────────
    #[test]
    fn test_signal_event_bridge() {
        use neotrix::l5_cognition::nt_mind::nt_game::bridge_signal_event::SignalEventBridge;

        let mut signals = CrystalSignalSystem::new();
        let mut bus = CrystalEventBus::new();
        let mut bridge = SignalEventBridge::new();

        let ch = bus.create_channel("combat_events");
        bridge.register("on_hit", ch);

        signals.connect("on_hit", 1, "take_damage");
        let forwarded = bridge.forward("on_hit", &[CrystalValue::Int(30)], &signals, &mut bus);
        assert_eq!(forwarded, 1);
        assert_eq!(bus.subscriber_count(ch), 1);
    }

    // ── Test 12: CardEffectExecutor ───────────────────────────────
    #[test]
    fn test_card_effect_executor() {
        use neotrix::l5_cognition::nt_mind::nt_game::bridge_card_ecs::{CardEffectExecutor, HealthComponent, CardHolder};

        let mut world = CrystalWorld::new();
        let source = world.spawn();
        world.add_component(source, CardHolder::new(3));

        let target = world.spawn();
        world.add_component(target, HealthComponent::new(100));

        // damage
        let dmg = CrystalCardEffect {
            effect_type: CrystalEffectType::Damage,
            value: 40,
            duration: None,
        };
        assert!(CardEffectExecutor::execute(&dmg, source, target, &mut world));
        let hp = world.get_component::<HealthComponent>(target).unwrap();
        assert_eq!(hp.current, 60);

        // heal
        let heal = CrystalCardEffect {
            effect_type: CrystalEffectType::Heal,
            value: 15,
            duration: None,
        };
        assert!(CardEffectExecutor::execute(&heal, source, target, &mut world));
        let hp = world.get_component::<HealthComponent>(target).unwrap();
        assert_eq!(hp.current, 75);

        // block
        let block = CrystalCardEffect {
            effect_type: CrystalEffectType::Block,
            value: 8,
            duration: None,
        };
        assert!(CardEffectExecutor::execute(&block, source, target, &mut world));
        let holder = world.get_component::<CardHolder>(target);
        assert!(holder.is_none()); // target has no CardHolder

        // gain energy
        let energy = CrystalCardEffect {
            effect_type: CrystalEffectType::GainEnergy,
            value: 2,
            duration: None,
        };
        assert!(CardEffectExecutor::execute(&energy, source, source, &mut world));
        let holder = world.get_component::<CardHolder>(source).unwrap();
        assert_eq!(holder.energy, 5);
    }

    // ── Test 13: StateBehaviorBridge ──────────────────────────────
    #[test]
    fn test_state_behavior_bridge() {
        use neotrix::l5_cognition::nt_mind::nt_game::bridge_state_behavior::StateBehaviorBridge;

        let mut sm = CrystalStateMachine::new();
        sm.add_state(make_state(1, "idle"));
        sm.add_state(make_state(2, "combat"));
        sm.add_transition(CrystalTransition {
            from: 1, to: 2, event: 100, guard: None, action: None,
        });
        sm.current_state = Some(1);

        let mut bt = CrystalBehaviorTree::new();
        bt.add_node(CrystalBehaviorNode {
            id: 0, name: "Root".into(),
            node_type: BehaviorNodeType::Sequence,
            children: vec![1],
        });
        bt.add_node(CrystalBehaviorNode {
            id: 1, name: "DoSomething".into(),
            node_type: BehaviorNodeType::Action("act".into()),
            children: vec![],
        });
        bt.set_root(0);

        let mut bridge = StateBehaviorBridge::new();
        bridge.register(1, 0);
        bridge.register(2, 0);

        let status = bridge.tick(&sm, &bt);
        assert_eq!(status, BehaviorStatus::Success);
        assert_eq!(bridge.tick_count(), 1);

        // transition and tick again
        sm.handle_event(100);
        let status = bridge.tick(&sm, &bt);
        assert_eq!(status, BehaviorStatus::Success);
    }

    // ── Test 14: MemoryResourceBridge ─────────────────────────────
    #[test]
    fn test_memory_resource_bridge() {
        use neotrix::l5_cognition::nt_mind::nt_game::bridge_memory_resource::{MemoryResourceBridge, CachePolicy};

        let mut mgr = CrystalResourceManager::new();
        let mut mem = CrystalMemoryHierarchy::new(8 * 1024 * 1024, 16 * 1024 * 1024, None).unwrap();
        let mut bridge = MemoryResourceBridge::new(CachePolicy::Adaptive);

        let h1 = mgr.insert("weights_a", vec![1, 2, 3]);
        let h2 = mgr.insert("weights_b", vec![4, 5, 6]);

        assert!(bridge.cache_resource(h1, &mgr, &mut mem));
        assert!(bridge.cache_resource(h2, &mgr, &mut mem));
        assert!(bridge.is_cached(h1));
        assert_eq!(bridge.cached_count(), 2);

        // access updates count
        bridge.access(h1, &mut mem);
        assert_eq!(bridge.access_count(h1), Some(2)); // 1 cache + 1 access

        // uncache
        assert!(bridge.uncache(h1));
        assert!(!bridge.is_cached(h1));
        assert_eq!(bridge.cached_count(), 1);
    }

    // ── Test 15: Full game turn simulation ────────────────────────
    #[test]
    fn test_full_game_turn_simulation() {
        use neotrix::l5_cognition::nt_mind::nt_game::bridge_card_ecs::{CardHolder, HealthComponent, CardEffectExecutor};
        use neotrix::l5_cognition::nt_mind::nt_game::bridge_ecs_scene::EcsSceneBridge;

        let mut world = CrystalWorld::new();
        let mut tree = CrystalSceneTree::new();
        let mut ecs_scene = EcsSceneBridge::new();

        // setup player
        let player = world.spawn();
        world.add_component(player, HealthComponent::new(100));
        world.add_component(player, CardHolder::new(3));
        let player_node = tree.create_node("Player", tree.root());
        ecs_scene.bind(player, player_node);

        // setup enemy
        let enemy = world.spawn();
        world.add_component(enemy, HealthComponent::new(60));
        let enemy_node = tree.create_node("Enemy", tree.root());
        ecs_scene.bind(enemy, enemy_node);

        // verify bindings
        assert!(ecs_scene.entity_node(player).is_some());
        assert!(ecs_scene.entity_node(enemy).is_some());

        // player draws cards
        {
            let holder = world.get_component_mut::<CardHolder>(player).unwrap();
            holder.deck.end_turn();
            holder.start_turn();
        }

        // player attacks enemy
        let dmg = CrystalCardEffect {
            effect_type: CrystalEffectType::Damage,
            value: 25,
            duration: None,
        };
        assert!(CardEffectExecutor::execute(&dmg, player, enemy, &mut world));

        // verify enemy health reduced
        let enemy_hp = world.get_component::<HealthComponent>(enemy).unwrap();
        assert_eq!(enemy_hp.current, 35);
        assert!(enemy_hp.is_alive());

        // enemy attacks player
        let counter = CrystalCardEffect {
            effect_type: CrystalEffectType::Damage,
            value: 15,
            duration: None,
        };
        assert!(CardEffectExecutor::execute(&counter, enemy, player, &mut world));

        let player_hp = world.get_component::<HealthComponent>(player).unwrap();
        assert_eq!(player_hp.current, 85);

        // cleanup: despawn enemy
        assert!(world.despawn(enemy));
        assert!(ecs_scene.unbind_entity(enemy).is_some());
        assert!(tree.remove_node(enemy_node));
    }

    // ── Test 16: Cross-module signal → event → resource pipeline ───
    #[test]
    fn test_cross_module_signal_event_resource_pipeline() {
        let mut signals = CrystalSignalSystem::new();
        let mut bus = CrystalEventBus::new();
        let mut mgr = CrystalResourceManager::new();

        // create channels
        let damage_ch = bus.create_channel("damage");
        let heal_ch = bus.create_channel("heal");

        // connect signals
        signals.connect("on_hit", 1, "take_damage");
        signals.connect("on_hit", 2, "take_damage");
        signals.connect("on_heal", 1, "restore_health");

        // subscribe to events
        bus.subscribe(damage_ch, 1);
        bus.subscribe(damage_ch, 2);
        bus.subscribe(heal_ch, 1);

        // emit damage signal
        let damage_results = signals.emit("on_hit", &[
            CrystalValue::Int(25),
            CrystalValue::String("fire".to_string()),
        ]);
        assert_eq!(damage_results.len(), 2);

        // emit heal signal
        let heal_results = signals.emit("on_heal", &[CrystalValue::Int(10)]);
        assert_eq!(heal_results.len(), 1);

        // store damage log as resource
        let log_data = format!("damage: {} hits", damage_results.len());
        let log_handle = mgr.insert("damage_log", log_data.into_bytes());
        assert_eq!(mgr.count(), 1);

        // verify full state
        assert_eq!(bus.subscriber_count(damage_ch), 2);
        assert_eq!(bus.subscriber_count(heal_ch), 1);
        assert_eq!(signals.connection_count(), 3);
        let log = mgr.get(log_handle).unwrap();
        assert_eq!(String::from_utf8_lossy(&log.data), "damage: 2 hits");
    }
}
