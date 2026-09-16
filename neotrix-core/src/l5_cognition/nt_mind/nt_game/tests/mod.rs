#[cfg(test)]
mod game_engine_tests {
    // ═══════════════════════════════════════════════════════════════
    // ECS Tests
    // ═══════════════════════════════════════════════════════════════
    mod ecs_tests {
        use crate::l5_cognition::nt_mind::nt_game::ecs::*;

        #[derive(Debug, Clone)]
        struct Pos {
            x: f64,
            y: f64,
        }

        impl Component for Pos {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }

        #[test]
        fn test_entity_creation() {
            let mut em = EntityManager::new();
            let e1 = em.create();
            let e2 = em.create();
            assert_eq!(em.count(), 2);
            assert_ne!(e1, e2);
        }

        #[test]
        fn test_entity_destroy() {
            let mut em = EntityManager::new();
            let e = em.create();
            em.destroy(e);
            assert!(!em.is_alive(e));
            assert_eq!(em.count(), 0);
        }

        #[test]
        fn test_entity_tag() {
            let mut em = EntityManager::new();
            let e = em.create();
            em.tag(e, "player");
            assert!(em.has_tag(e, "player"));
            assert!(!em.has_tag(e, "enemy"));
        }

        #[test]
        fn test_alive_ids() {
            let mut em = EntityManager::new();
            let e1 = em.create();
            let e2 = em.create();
            let e3 = em.create();
            em.destroy(e2);
            let alive = em.alive_ids();
            assert_eq!(alive.len(), 2);
            assert!(alive.contains(&e1));
            assert!(alive.contains(&e3));
        }

        #[test]
        fn test_with_tag() {
            let mut em = EntityManager::new();
            let e1 = em.create();
            let e2 = em.create();
            let e3 = em.create();
            em.tag(e1, "enemy");
            em.tag(e3, "enemy");
            let enemies = em.with_tag("enemy");
            assert_eq!(enemies.len(), 2);
            assert!(enemies.contains(&e1));
            assert!(enemies.contains(&e3));
        }

        #[test]
        fn test_component_store() {
            let mut cs = ComponentStore::new();
            let e = 1;
            cs.insert(e, Pos { x: 1.0, y: 2.0 });
            let p = cs.get::<Pos>(e).unwrap();
            assert_eq!(p.x, 1.0);
            assert_eq!(p.y, 2.0);
        }

        #[test]
        fn test_component_remove() {
            let mut cs = ComponentStore::new();
            cs.insert(1, Pos { x: 0.0, y: 0.0 });
            assert!(cs.remove::<Pos>(1));
            assert!(!cs.has::<Pos>(1));
        }

        #[test]
        fn test_component_get_mut() {
            let mut cs = ComponentStore::new();
            cs.insert(1, Pos { x: 5.0, y: 5.0 });
            {
                let p = cs.get_mut::<Pos>(1).unwrap();
                p.x = 10.0;
            }
            assert_eq!(cs.get::<Pos>(1).unwrap().x, 10.0);
        }

        #[test]
        fn test_entities_with() {
            let mut cs = ComponentStore::new();
            cs.insert(1, Pos { x: 0.0, y: 0.0 });
            cs.insert(2, Pos { x: 1.0, y: 1.0 });
            let entities = cs.entities_with::<Pos>();
            assert_eq!(entities.len(), 2);
        }

        #[test]
        fn test_system_scheduler() {
            struct DummySystem;
            impl System for DummySystem {
                fn name(&self) -> &str {
                    "dummy"
                }
                fn update(&self, _: &[EntityId], _: &mut ComponentStore, _: f64) {}
            }

            let mut scheduler = SystemScheduler::new();
            scheduler.add_system(Box::new(DummySystem));
            assert_eq!(scheduler.system_count(), 1);
            assert_eq!(scheduler.system_names(), vec!["dummy"]);
        }

        #[test]
        fn test_world_creation() {
            let w = World::new();
            assert_eq!(w.entities.count(), 0);
        }

        #[test]
        fn test_world_tick() {
            let mut w = World::new();
            w.tick(0.016);
            assert!((w.time - 0.016).abs() < 0.001);
        }

        #[test]
        fn test_world_spawn_destroy() {
            let mut w = World::new();
            let e = w.spawn_entity();
            assert!(w.entities.is_alive(e));
            w.destroy_entity(e);
            assert!(!w.entities.is_alive(e));
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Tilemap Tests
    // ═══════════════════════════════════════════════════════════════
    mod tilemap_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::tilemap::*;

        #[test]
        fn test_tile_creation() {
            let t = Tile::new(1, TileType::Ground);
            assert_eq!(t.id, 1);
            assert!(t.walkable);
            assert!(t.transparent);
        }

        #[test]
        fn test_tile_builder() {
            let t = Tile::new(1, TileType::Wall)
                .with_walkable(false)
                .with_transparent(false)
                .with_metadata("name", "stone_wall");
            assert!(!t.walkable);
            assert!(!t.transparent);
            assert_eq!(t.metadata.get("name").unwrap(), "stone_wall");
        }

        #[test]
        fn test_tile_default() {
            let t = Tile::default();
            assert_eq!(t.tile_type, TileType::Empty);
        }

        #[test]
        fn test_chunk_fill() {
            let mut c = Chunk::new(0, 0);
            c.fill(Tile::new(1, TileType::Grass));
            assert_eq!(c.count_type(TileType::Grass), CHUNK_SIZE * CHUNK_SIZE);
        }

        #[test]
        fn test_chunk_get_set_tile() {
            let mut c = Chunk::new(0, 0);
            c.set_tile(5, 5, Tile::new(1, TileType::Tree));
            let t = c.get_tile(5, 5).unwrap();
            assert_eq!(t.tile_type, TileType::Tree);
        }

        #[test]
        fn test_chunk_out_of_bounds() {
            let c = Chunk::new(0, 0);
            assert!(c.get_tile(CHUNK_SIZE, 0).is_none());
            assert!(c.get_tile(0, CHUNK_SIZE).is_none());
        }

        #[test]
        fn test_chunk_dirty_flag() {
            let mut c = Chunk::new(0, 0);
            assert!(!c.dirty);
            c.set_tile(0, 0, Tile::new(1, TileType::Ground));
            assert!(c.dirty);
        }

        #[test]
        fn test_chunk_manager() {
            let mut cm = ChunkManager::new(3);
            cm.load_chunk(0, 0);
            assert_eq!(cm.loaded_chunks(), 1);
        }

        #[test]
        fn test_chunk_manager_unload() {
            let mut cm = ChunkManager::new(3);
            cm.load_chunk(0, 0);
            assert!(cm.unload_chunk(0, 0));
            assert_eq!(cm.loaded_chunks(), 0);
        }

        #[test]
        fn test_chunk_manager_get() {
            let mut cm = ChunkManager::new(3);
            cm.load_chunk(0, 0);
            assert!(cm.get_chunk(0, 0).is_some());
            assert!(cm.get_chunk(1, 0).is_none());
        }

        #[test]
        fn test_coordinate_conversion() {
            let cm = ChunkManager::new(3);
            let (cx, cy) = cm.world_to_chunk(35.0, 67.0);
            assert_eq!(cx, 1);
            assert_eq!(cy, 2);
        }

        #[test]
        fn test_world_to_local() {
            let cm = ChunkManager::new(3);
            let (lx, ly) = cm.world_to_local(35.0, 67.0);
            assert_eq!(lx, 35);
            assert_eq!(ly, 67);
        }

        #[test]
        fn test_visible_chunks() {
            let cm = ChunkManager::new(1);
            let visible = cm.visible_chunks(0, 0);
            assert_eq!(visible.len(), 9);
        }

        #[test]
        fn test_layered_map() {
            let mut m = LayeredTileMap::new(64, 64);
            m.set_tile(MapLayer::Terrain, 5, 5, Tile::new(1, TileType::Ground));
            assert!(m.is_walkable(5, 5));
        }

        #[test]
        fn test_layered_map_not_walkable() {
            let mut m = LayeredTileMap::new(64, 64);
            m.set_tile(
                MapLayer::Terrain,
                5,
                5,
                Tile::new(1, TileType::Wall).with_walkable(false),
            );
            assert!(!m.is_walkable(5, 5));
        }

        #[test]
        fn test_layered_map_object_blocks() {
            let mut m = LayeredTileMap::new(64, 64);
            m.set_tile(MapLayer::Terrain, 5, 5, Tile::new(1, TileType::Ground));
            m.set_tile(
                MapLayer::Objects,
                5,
                5,
                Tile::new(2, TileType::Tree).with_walkable(false),
            );
            assert!(!m.is_walkable(5, 5));
        }

        #[test]
        fn test_layered_map_empty_not_walkable() {
            let m = LayeredTileMap::new(64, 64);
            assert!(!m.is_walkable(5, 5));
        }

        #[test]
        fn test_layered_map_dimensions() {
            let m = LayeredTileMap::new(128, 256);
            assert_eq!(m.width(), 128);
            assert_eq!(m.height(), 256);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Event Tests
    // ═══════════════════════════════════════════════════════════════
    mod event_tests {
        use crate::l5_cognition::nt_mind::nt_game::events::*;
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        #[test]
        fn test_event_bus_subscribe_and_emit() {
            let mut bus = EventBus::new();
            let counter = Arc::new(AtomicUsize::new(0));
            let c = counter.clone();
            bus.subscribe::<DamageEvent>(move |_| {
                c.fetch_add(1, Ordering::Relaxed);
            });
            bus.emit(DamageEvent {
                source: 1,
                target: 2,
                amount: 10.0,
            });
            assert_eq!(counter.load(Ordering::Relaxed), 1);
        }

        #[test]
        fn test_event_bus_multiple_emits() {
            let mut bus = EventBus::new();
            let counter = Arc::new(AtomicUsize::new(0));
            let c = counter.clone();
            bus.subscribe::<DamageEvent>(move |_| {
                c.fetch_add(1, Ordering::Relaxed);
            });
            bus.emit(DamageEvent {
                source: 1,
                target: 2,
                amount: 10.0,
            });
            bus.emit(DamageEvent {
                source: 1,
                target: 3,
                amount: 5.0,
            });
            assert_eq!(counter.load(Ordering::Relaxed), 2);
        }

        #[test]
        fn test_event_log() {
            let mut bus = EventBus::new();
            bus.emit(TurnEndedEvent {
                turn: 1,
                player: "A".into(),
            });
            assert_eq!(bus.log_size(), 1);
        }

        #[test]
        fn test_event_log_overflow() {
            let mut bus = EventBus::new();
            for i in 0..150 {
                bus.emit(DamageEvent {
                    source: i,
                    target: i,
                    amount: 1.0,
                });
            }
            assert!(bus.log_size() <= 100);
        }

        #[test]
        fn test_event_clear_log() {
            let mut bus = EventBus::new();
            bus.emit(DamageEvent {
                source: 1,
                target: 2,
                amount: 1.0,
            });
            bus.clear_log();
            assert_eq!(bus.log_size(), 0);
        }

        #[test]
        fn test_event_no_listeners() {
            let mut bus = EventBus::new();
            bus.emit(DamageEvent {
                source: 1,
                target: 2,
                amount: 10.0,
            });
            assert_eq!(bus.listener_count::<DamageEvent>(), 0);
        }

        #[test]
        fn test_event_listener_count() {
            let mut bus = EventBus::new();
            bus.subscribe::<DamageEvent>(|_| {});
            bus.subscribe::<DamageEvent>(|_| {});
            assert_eq!(bus.listener_count::<DamageEvent>(), 2);
        }

        #[test]
        fn test_event_entity_created() {
            let mut bus = EventBus::new();
            let counter = Arc::new(AtomicUsize::new(0));
            let c = counter.clone();
            bus.subscribe::<EntityCreatedEvent>(move |e| {
                if e.entity_id == 42 {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            });
            bus.emit(EntityCreatedEvent { entity_id: 42 });
            assert_eq!(counter.load(Ordering::Relaxed), 1);
        }

        #[test]
        fn test_event_collision() {
            let mut bus = EventBus::new();
            let counter = Arc::new(AtomicUsize::new(0));
            let c = counter.clone();
            bus.subscribe::<CollisionEvent>(move |_| {
                c.fetch_add(1, Ordering::Relaxed);
            });
            bus.emit(CollisionEvent {
                entity_a: 1,
                entity_b: 2,
                x: 5.0,
                y: 5.0,
            });
            assert_eq!(counter.load(Ordering::Relaxed), 1);
        }

        #[test]
        fn test_event_type_names() {
            let e = DamageEvent {
                source: 0,
                target: 0,
                amount: 0.0,
            };
            assert_eq!(e.event_type(), "damage");

            let e = EntityCreatedEvent { entity_id: 0 };
            assert_eq!(e.event_type(), "entity_created");

            let e = GameStateChangedEvent {
                state_name: "".into(),
                old_value: "".into(),
                new_value: "".into(),
            };
            assert_eq!(e.event_type(), "game_state_changed");
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Time System Tests
    // ═══════════════════════════════════════════════════════════════
    mod time_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::time_system::*;

        #[test]
        fn test_clock_tick() {
            let mut c = GameClock::new();
            c.tick(60.0);
            assert_eq!(c.hour, 9);
        }

        #[test]
        fn test_day_wrap() {
            let mut c = GameClock::new();
            c.hour = 23;
            c.tick(120.0);
            assert_eq!(c.day, 2);
        }

        #[test]
        fn test_time_of_day() {
            let mut c = GameClock::new();
            assert_eq!(c.time_of_day(), TimeOfDay::Morning);
            c.hour = 12;
            assert_eq!(c.time_of_day(), TimeOfDay::Noon);
        }

        #[test]
        fn test_time_of_day_all_variants() {
            let mut c = GameClock::new();
            c.hour = 5;
            assert_eq!(c.time_of_day(), TimeOfDay::Dawn);
            c.hour = 7;
            assert_eq!(c.time_of_day(), TimeOfDay::Morning);
            c.hour = 11;
            assert_eq!(c.time_of_day(), TimeOfDay::Noon);
            c.hour = 14;
            assert_eq!(c.time_of_day(), TimeOfDay::Afternoon);
            c.hour = 18;
            assert_eq!(c.time_of_day(), TimeOfDay::Dusk);
            c.hour = 21;
            assert_eq!(c.time_of_day(), TimeOfDay::Night);
        }

        #[test]
        fn test_pause() {
            let mut c = GameClock::new();
            c.pause();
            c.tick(100.0);
            assert_eq!(c.hour, 8);
        }

        #[test]
        fn test_resume() {
            let mut c = GameClock::new();
            c.pause();
            c.tick(100.0);
            c.resume();
            c.tick(60.0);
            assert_eq!(c.hour, 9);
        }

        #[test]
        fn test_speed() {
            let mut c = GameClock::new();
            c.set_speed(2.0);
            c.tick(30.0);
            assert_eq!(c.hour, 9);
        }

        #[test]
        fn test_is_daytime() {
            let mut c = GameClock::new();
            assert!(c.is_daytime());
            c.hour = 3;
            assert!(!c.is_daytime());
            c.hour = 20;
            assert!(!c.is_daytime());
        }

        #[test]
        fn test_format() {
            let c = GameClock::new();
            let s = c.format();
            assert!(s.contains("Day 1"));
            assert!(s.contains("08:00"));
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Season Tests
    // ═══════════════════════════════════════════════════════════════
    mod season_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::season::*;

        #[test]
        fn test_season_advance() {
            let mut s = SeasonState::new(7);
            for _ in 0..7 {
                s.advance_day();
            }
            assert_eq!(s.current, Season::Summer);
        }

        #[test]
        fn test_season_full_cycle() {
            let mut s = SeasonState::new(1);
            assert_eq!(s.current, Season::Spring);
            s.advance_day();
            assert_eq!(s.current, Season::Summer);
            s.advance_day();
            assert_eq!(s.current, Season::Autumn);
            s.advance_day();
            assert_eq!(s.current, Season::Winter);
            s.advance_day();
            assert_eq!(s.current, Season::Spring);
        }

        #[test]
        fn test_crop_modifier() {
            assert_eq!(Season::Summer.crop_modifier(), 1.2);
            assert_eq!(Season::Winter.crop_modifier(), 0.3);
            assert_eq!(Season::Spring.crop_modifier(), 1.0);
            assert_eq!(Season::Autumn.crop_modifier(), 0.8);
        }

        #[test]
        fn test_temperature_modifier() {
            assert_eq!(Season::Summer.temperature_modifier(), 30.0);
            assert_eq!(Season::Winter.temperature_modifier(), 0.0);
        }

        #[test]
        fn test_season_names() {
            assert_eq!(Season::Spring.name(), "Spring");
            assert_eq!(Season::Summer.name(), "Summer");
            assert_eq!(Season::Autumn.name(), "Autumn");
            assert_eq!(Season::Winter.name(), "Winter");
        }

        #[test]
        fn test_season_progress() {
            let mut s = SeasonState::new(10);
            assert!((s.progress() - 0.0).abs() < 0.001);
            s.advance_day();
            s.advance_day();
            assert!((s.progress() - 0.2).abs() < 0.001);
        }

        #[test]
        fn test_season_no_advance_prematurely() {
            let mut s = SeasonState::new(5);
            assert!(!s.advance_day());
            assert_eq!(s.current, Season::Spring);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Weather Tests
    // ═══════════════════════════════════════════════════════════════
    mod weather_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::season::Season;
        use crate::l5_cognition::nt_mind::nt_game::world::weather::*;

        #[test]
        fn test_weather_visibility() {
            let w = WeatherState::new();
            assert_eq!(w.visibility_modifier(), 1.0);
        }

        #[test]
        fn test_weather_movement() {
            let mut w = WeatherState::new();
            w.current = WeatherType::Storm;
            assert_eq!(w.movement_modifier(), 0.6);
        }

        #[test]
        fn test_all_weather_visibility() {
            let mut w = WeatherState::new();
            w.current = WeatherType::Clear;
            assert_eq!(w.visibility_modifier(), 1.0);
            w.current = WeatherType::Cloudy;
            assert_eq!(w.visibility_modifier(), 0.9);
            w.current = WeatherType::Rain;
            assert_eq!(w.visibility_modifier(), 0.7);
            w.current = WeatherType::Storm;
            assert_eq!(w.visibility_modifier(), 0.5);
            w.current = WeatherType::Snow;
            assert_eq!(w.visibility_modifier(), 0.6);
            w.current = WeatherType::Fog;
            assert_eq!(w.visibility_modifier(), 0.3);
            w.current = WeatherType::Wind;
            assert_eq!(w.visibility_modifier(), 0.85);
        }

        #[test]
        fn test_all_weather_movement() {
            let mut w = WeatherState::new();
            w.current = WeatherType::Clear;
            assert_eq!(w.movement_modifier(), 1.0);
            w.current = WeatherType::Rain;
            assert_eq!(w.movement_modifier(), 0.85);
            w.current = WeatherType::Storm;
            assert_eq!(w.movement_modifier(), 0.6);
            w.current = WeatherType::Snow;
            assert_eq!(w.movement_modifier(), 0.7);
            w.current = WeatherType::Fog;
            assert_eq!(w.movement_modifier(), 0.9);
            w.current = WeatherType::Wind;
            assert_eq!(w.movement_modifier(), 0.8);
            w.current = WeatherType::Cloudy;
            assert_eq!(w.movement_modifier(), 1.0);
        }

        #[test]
        fn test_weather_tick_transition() {
            let mut w = WeatherState::new();
            w.duration_remaining = 0.0;
            w.tick(1.0, &Season::Winter);
            assert_eq!(w.current, WeatherType::Snow);
        }

        #[test]
        fn test_weather_default() {
            let w = WeatherState::default();
            assert_eq!(w.current, WeatherType::Clear);
            assert_eq!(w.intensity, 1.0);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Combat Tests
    // ═══════════════════════════════════════════════════════════════
    mod combat_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::combat::combat::*;

        #[test]
        fn test_combatant_creation() {
            let c = Combatant::new(1, "Hero").with_stats(200.0, 15.0, 8.0);
            assert_eq!(c.hp, 200.0);
            assert_eq!(c.max_hp, 200.0);
            assert_eq!(c.attack, 15.0);
            assert_eq!(c.defense, 8.0);
            assert!(c.is_alive());
        }

        #[test]
        fn test_damage() {
            let mut c = Combatant::new(1, "Hero").with_stats(100.0, 10.0, 5.0);
            let dealt = c.take_damage(20.0);
            assert!(dealt > 0.0);
            assert!(c.hp < 100.0);
        }

        #[test]
        fn test_damage_absorption() {
            let mut c = Combatant::new(1, "Tank").with_stats(100.0, 5.0, 50.0);
            let dealt = c.take_damage(30.0);
            assert_eq!(dealt, 0.0);
            assert_eq!(c.hp, 100.0);
        }

        #[test]
        fn test_heal() {
            let mut c = Combatant::new(1, "Hero").with_stats(100.0, 10.0, 5.0);
            c.take_damage(50.0);
            c.heal(30.0);
            assert_eq!(c.hp, 80.0);
        }

        #[test]
        fn test_heal_capped() {
            let mut c = Combatant::new(1, "Hero").with_stats(100.0, 10.0, 5.0);
            c.heal(50.0);
            assert_eq!(c.hp, 100.0);
        }

        #[test]
        fn test_restore_energy() {
            let mut c = Combatant::new(1, "Hero").with_stats(100.0, 10.0, 5.0);
            c.energy = 50.0;
            c.restore_energy(30.0);
            assert_eq!(c.energy, 80.0);
        }

        #[test]
        fn test_can_use_action() {
            let c = Combatant::new(1, "Hero").with_stats(100.0, 10.0, 5.0);
            let action = CombatAction::attack("Slash", 15.0).with_cost(10.0);
            assert!(c.can_use(&action));
        }

        #[test]
        fn test_cannot_use_insufficient_energy() {
            let mut c = Combatant::new(1, "Hero").with_stats(100.0, 10.0, 5.0);
            c.energy = 5.0;
            let action = CombatAction::attack("Slash", 15.0).with_cost(10.0);
            assert!(!c.can_use(&action));
        }

        #[test]
        fn test_tick_cooldowns() {
            let mut c = Combatant::new(1, "Hero").with_stats(100.0, 10.0, 5.0);
            c.cooldowns.insert("Slash".into(), 2.0);
            c.tick_cooldowns(1.0);
            assert_eq!(c.cooldowns["Slash"], 1.0);
            c.tick_cooldowns(1.0);
            assert_eq!(c.cooldowns["Slash"], 0.0);
        }

        #[test]
        fn test_combat_execute() {
            let mut cs = CombatState::new();
            cs.add_combatant(Combatant::new(1, "A").with_stats(100.0, 20.0, 5.0));
            cs.add_combatant(Combatant::new(2, "B").with_stats(100.0, 10.0, 5.0));
            cs.start();
            let attack = CombatAction::attack("Slash", 15.0);
            let result = cs.execute_action(1, 2, &attack);
            assert!(result.is_some());
            assert!(result.unwrap().damage_dealt > 0.0);
        }

        #[test]
        fn test_combat_turn() {
            let mut cs = CombatState::new();
            cs.add_combatant(Combatant::new(1, "A").with_stats(100.0, 20.0, 5.0));
            cs.add_combatant(Combatant::new(2, "B").with_stats(100.0, 10.0, 5.0));
            cs.start();
            cs.next_turn();
            assert_eq!(cs.turn, 2);
        }

        #[test]
        fn test_combat_over() {
            let mut cs = CombatState::new();
            cs.add_combatant(Combatant::new(1, "A").with_stats(100.0, 20.0, 5.0));
            cs.add_combatant(Combatant::new(2, "B").with_stats(1.0, 10.0, 0.0));
            cs.start();
            let attack = CombatAction::attack("Kill", 50.0);
            cs.execute_action(1, 2, &attack);
            assert!(cs.is_over());
            assert!(cs.winner().is_some());
        }

        #[test]
        fn test_combat_action_builder() {
            let action = CombatAction::attack("Fireball", 25.0)
                .with_type(DamageType::Fire)
                .with_cost(30.0)
                .with_range(10.0)
                .with_cooldown(5.0)
                .with_aoe(true);
            assert_eq!(action.name, "Fireball");
            assert_eq!(action.damage_type, DamageType::Fire);
            assert_eq!(action.cost, 30.0);
            assert!(action.aoe);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Inventory Tests
    // ═══════════════════════════════════════════════════════════════
    mod inventory_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::combat::inventory::*;

        fn sword() -> Item {
            Item {
                id: 1,
                name: "Sword".into(),
                description: "".into(),
                rarity: ItemRarity::Common,
                stackable: false,
                max_stack: 1,
                weight: 1.0,
                value: 10,
                item_type: ItemType::Weapon,
            }
        }

        fn potion() -> Item {
            Item {
                id: 2,
                name: "Potion".into(),
                description: "".into(),
                rarity: ItemRarity::Common,
                stackable: true,
                max_stack: 99,
                weight: 0.1,
                value: 5,
                item_type: ItemType::Consumable,
            }
        }

        fn gem() -> Item {
            Item {
                id: 3,
                name: "Gem".into(),
                description: "".into(),
                rarity: ItemRarity::Rare,
                stackable: true,
                max_stack: 50,
                weight: 0.5,
                value: 100,
                item_type: ItemType::Material,
            }
        }

        #[test]
        fn test_add_item() {
            let mut inv = Inventory::new(10);
            inv.add_item(sword(), 1);
            assert_eq!(inv.used_slots(), 1);
        }

        #[test]
        fn test_stackable() {
            let mut inv = Inventory::new(10);
            inv.add_item(potion(), 5);
            inv.add_item(potion(), 3);
            assert_eq!(inv.count_item(2), 8);
        }

        #[test]
        fn test_remove_item() {
            let mut inv = Inventory::new(10);
            inv.add_item(gem(), 10);
            inv.remove_item(3, 3);
            assert_eq!(inv.count_item(3), 7);
        }

        #[test]
        fn test_remove_all() {
            let mut inv = Inventory::new(10);
            inv.add_item(gem(), 5);
            inv.remove_item(3, 5);
            assert_eq!(inv.count_item(3), 0);
            assert_eq!(inv.used_slots(), 0);
        }

        #[test]
        fn test_inventory_full() {
            let mut inv = Inventory::new(2);
            inv.add_item(sword(), 1);
            inv.add_item(potion(), 1);
            assert!(inv.is_full());
        }

        #[test]
        fn test_inventory_not_full() {
            let mut inv = Inventory::new(10);
            inv.add_item(sword(), 1);
            assert!(!inv.is_full());
        }

        #[test]
        fn test_total_weight() {
            let mut inv = Inventory::new(10);
            inv.add_item(sword(), 1);
            inv.add_item(potion(), 5);
            let expected = 1.0 * 1.0 + 0.1 * 5.0;
            assert!((inv.total_weight() - expected).abs() < 0.001);
        }

        #[test]
        fn test_item_stack_add_capped() {
            let mut stack = ItemStack::new(potion(), 95);
            let leftover = stack.add(10);
            assert_eq!(stack.count, 99);
            assert_eq!(leftover, 6);
        }

        #[test]
        fn test_item_stack_remove() {
            let mut stack = ItemStack::new(potion(), 10);
            let removed = stack.remove(3);
            assert_eq!(removed, 3);
            assert_eq!(stack.count, 7);
        }

        #[test]
        fn test_item_stack_empty() {
            let mut stack = ItemStack::new(potion(), 1);
            stack.remove(1);
            assert!(stack.is_empty());
        }

        #[test]
        fn test_item_rarity_color() {
            assert_eq!(ItemRarity::Legendary.color(), "#ff8000");
            assert_eq!(ItemRarity::Common.color(), "#ffffff");
        }

        #[test]
        fn test_item_rarity_drop_weight() {
            assert!(ItemRarity::Common.drop_weight() > ItemRarity::Legendary.drop_weight());
        }

        #[test]
        fn test_sort_by_rarity() {
            let mut inv = Inventory::new(10);
            inv.add_item(
                Item {
                    rarity: ItemRarity::Common,
                    ..sword()
                },
                1,
            );
            inv.add_item(
                Item {
                    rarity: ItemRarity::Legendary,
                    id: 10,
                    name: "Excalibur".into(),
                    ..sword()
                },
                1,
            );
            inv.sort_by_rarity();
            if let Some(stack) = &inv.slots[0] {
                assert_eq!(stack.item.rarity, ItemRarity::Legendary);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Quest Tests
    // ═══════════════════════════════════════════════════════════════
    mod quest_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::quest::quest::*;

        #[test]
        fn test_quest_lifecycle() {
            let mut q = Quest::new(1, "Test", "Desc").with_objective(QuestObjective::kill(
                "Kill wolves",
                "wolf",
                3,
            ));
            assert_eq!(q.state, QuestState::NotStarted);
            q.start();
            assert_eq!(q.state, QuestState::Active);
            q.update_objective(0, 3);
            q.check_completion();
            assert_eq!(q.state, QuestState::Completed);
        }

        #[test]
        fn test_quest_partial_progress() {
            let mut q = Quest::new(1, "Test", "Desc").with_objective(QuestObjective::kill(
                "Kill wolves",
                "wolf",
                5,
            ));
            q.start();
            q.update_objective(0, 2);
            q.check_completion();
            assert_eq!(q.state, QuestState::Active);
        }

        #[test]
        fn test_quest_fail() {
            let mut q = Quest::new(1, "Test", "Desc").with_objective(QuestObjective::kill(
                "Kill wolves",
                "wolf",
                3,
            ));
            q.start();
            q.fail();
            assert_eq!(q.state, QuestState::Failed);
        }

        #[test]
        fn test_quest_collect_objective() {
            let mut q = Quest::new(1, "Gather", "Collect herbs")
                .with_objective(QuestObjective::collect("Pick herbs", 10, 5));
            q.start();
            q.update_objective(0, 5);
            q.check_completion();
            assert_eq!(q.state, QuestState::Completed);
        }

        #[test]
        fn test_quest_talk_to() {
            let mut q = Quest::new(1, "Talk", "Speak to NPC")
                .with_objective(QuestObjective::talk_to("Find the elder", 42));
            q.start();
            q.update_objective(0, 1);
            q.check_completion();
            assert_eq!(q.state, QuestState::Completed);
        }

        #[test]
        fn test_quest_multiple_objectives() {
            let mut q = Quest::new(1, "Multi", "Do everything")
                .with_objective(QuestObjective::kill("Kill 3 wolves", "wolf", 3))
                .with_objective(QuestObjective::collect("Gather 5 herbs", 10, 5));
            q.start();
            q.update_objective(0, 3);
            q.check_completion();
            assert_eq!(q.state, QuestState::Active);
            q.update_objective(1, 5);
            q.check_completion();
            assert_eq!(q.state, QuestState::Completed);
        }

        #[test]
        fn test_quest_with_reward() {
            let q = Quest::new(1, "Rewarded", "Get gold").with_reward(QuestReward {
                reward_type: RewardType::Gold,
                amount: 100,
            });
            assert_eq!(q.rewards.len(), 1);
        }

        #[test]
        fn test_quest_prerequisites() {
            let q = Quest::new(2, "Chained", "After quest 1").with_prerequisite(1);
            assert_eq!(q.prerequisites, vec![1]);
        }

        #[test]
        fn test_quest_level_requirement() {
            let q = Quest::new(1, "High Level", "For heroes").with_level_requirement(10);
            assert_eq!(q.level_requirement, 10);
        }

        #[test]
        fn test_quest_progress_summary() {
            let mut q = Quest::new(1, "Test", "Desc").with_objective(QuestObjective::kill(
                "Kill wolves",
                "wolf",
                3,
            ));
            q.start();
            q.update_objective(0, 1);
            let summary = q.progress_summary();
            assert!(summary.contains("1/3"));
        }

        #[test]
        fn test_quest_update_out_of_bounds() {
            let mut q = Quest::new(1, "Test", "Desc")
                .with_objective(QuestObjective::kill("Kill", "wolf", 3));
            q.start();
            q.update_objective(5, 1);
            q.check_completion();
            assert_eq!(q.state, QuestState::Active);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Loot Tests
    // ═══════════════════════════════════════════════════════════════
    mod loot_tests {
        use crate::l5_cognition::nt_mind::nt_game::world::quest::loot::*;

        #[test]
        fn test_loot_guaranteed() {
            let table = LootTable::new().with_entry(LootEntry::new(1, "Gold", 0.5).guaranteed());
            let drops = table.roll();
            assert_eq!(drops.len(), 1);
            assert_eq!(drops[0].item_name, "Gold");
        }

        #[test]
        fn test_loot_empty() {
            let table = LootTable::new();
            assert!(table.roll().is_empty());
        }

        #[test]
        fn test_loot_entry_with_count() {
            let entry = LootEntry::new(1, "Arrows", 1.0).with_count(5, 10);
            assert_eq!(entry.min_count, 5);
            assert_eq!(entry.max_count, 10);
        }

        #[test]
        fn test_loot_max_drops() {
            let mut table = LootTable::new().with_max_drops(2);
            table.entries.push(LootEntry::new(1, "A", 1.0).guaranteed());
            table.entries.push(LootEntry::new(2, "B", 1.0).guaranteed());
            table.entries.push(LootEntry::new(3, "C", 1.0).guaranteed());
            let drops = table.roll();
            assert!(drops.len() <= 2);
        }

        #[test]
        fn test_loot_expected_value() {
            let table =
                LootTable::new().with_entry(LootEntry::new(1, "Gold", 0.5).with_count(1, 3));
            let ev = table.expected_value();
            assert!((ev - 1.0).abs() < 0.001);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Behavior Tree Tests
    // ═══════════════════════════════════════════════════════════════
    mod bt_tests {
        use crate::l5_cognition::nt_mind::nt_game::ai::behavior_tree::*;

        struct Success;
        impl BehaviorNode for Success {
            fn name(&self) -> &str {
                "S"
            }
            fn tick(&self, _: &mut bt::BehaviorContext) -> NodeStatus {
                NodeStatus::Success
            }
        }
        struct Fail;
        impl BehaviorNode for Fail {
            fn name(&self) -> &str {
                "F"
            }
            fn tick(&self, _: &mut bt::BehaviorContext) -> NodeStatus {
                NodeStatus::Failure
            }
        }

        #[test]
        fn test_sequence() {
            let mut bt = BehaviorTree::new();
            bt.set_root(Box::new(Sequence::new(vec![
                Box::new(Success),
                Box::new(Success),
            ])));
            assert_eq!(bt.tick(), NodeStatus::Success);
        }

        #[test]
        fn test_sequence_fails_on_first_failure() {
            let mut bt = BehaviorTree::new();
            bt.set_root(Box::new(Sequence::new(vec![
                Box::new(Fail),
                Box::new(Success),
            ])));
            assert_eq!(bt.tick(), NodeStatus::Failure);
        }

        #[test]
        fn test_selector() {
            let mut bt = BehaviorTree::new();
            bt.set_root(Box::new(Selector::new(vec![
                Box::new(Fail),
                Box::new(Success),
            ])));
            assert_eq!(bt.tick(), NodeStatus::Success);
        }

        #[test]
        fn test_selector_fails_when_all_fail() {
            let mut bt = BehaviorTree::new();
            bt.set_root(Box::new(Selector::new(vec![
                Box::new(Fail),
                Box::new(Fail),
            ])));
            assert_eq!(bt.tick(), NodeStatus::Failure);
        }

        #[test]
        fn test_empty() {
            let mut bt = BehaviorTree::new();
            assert_eq!(bt.tick(), NodeStatus::Failure);
        }

        #[test]
        fn test_inverter() {
            let mut bt = BehaviorTree::new();
            bt.set_root(Box::new(Inverter::new(Box::new(Success))));
            assert_eq!(bt.tick(), NodeStatus::Failure);
        }

        #[test]
        fn test_inverter_failure() {
            let mut bt = BehaviorTree::new();
            bt.set_root(Box::new(Inverter::new(Box::new(Fail))));
            assert_eq!(bt.tick(), NodeStatus::Success);
        }

        #[test]
        fn test_nested_bt() {
            let mut bt = BehaviorTree::new();
            bt.set_root(Box::new(Selector::new(vec![
                Box::new(Sequence::new(vec![Box::new(Fail), Box::new(Success)])),
                Box::new(Success),
            ])));
            assert_eq!(bt.tick(), NodeStatus::Success);
        }

        #[test]
        fn test_bt_context() {
            let mut bt = BehaviorTree::new();
            bt.context().set("score", 42_i32);
            assert_eq!(*bt.context().get::<i32>("score").unwrap(), 42);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Policy Tests
    // ═══════════════════════════════════════════════════════════════
    mod policy_tests {
        use crate::l5_cognition::nt_mind::nt_game::ai::policy::*;
        use std::collections::HashMap;

        #[test]
        fn test_greedy() {
            let p = GreedyPolicy::new();
            let s = HashMap::new();
            let a = vec![Action {
                name: "atk".into(),
                parameters: HashMap::new(),
            }];
            assert!(p.choose_action(&s, &a).is_some());
        }

        #[test]
        fn test_greedy_returns_first() {
            let p = GreedyPolicy::new();
            let s = HashMap::new();
            let a = vec![
                Action {
                    name: "first".into(),
                    parameters: HashMap::new(),
                },
                Action {
                    name: "second".into(),
                    parameters: HashMap::new(),
                },
            ];
            let chosen = p.choose_action(&s, &a).unwrap();
            assert_eq!(chosen.name, "first");
        }

        #[test]
        fn test_greedy_empty_actions() {
            let p = GreedyPolicy::new();
            let s = HashMap::new();
            assert!(p.choose_action(&s, &[]).is_none());
        }

        #[test]
        fn test_random() {
            let p = RandomPolicy::new(42);
            let s = HashMap::new();
            let a = vec![
                Action {
                    name: "a".into(),
                    parameters: HashMap::new(),
                },
                Action {
                    name: "b".into(),
                    parameters: HashMap::new(),
                },
            ];
            assert!(p.choose_action(&s, &a).is_some());
        }

        #[test]
        fn test_random_empty() {
            let p = RandomPolicy::new(0);
            let s = HashMap::new();
            assert!(p.choose_action(&s, &[]).is_none());
        }

        #[test]
        fn test_epsilon_greedy() {
            let p = EpsilonGreedyPolicy::new(Box::new(GreedyPolicy::new()), 0.0);
            let s = HashMap::new();
            let a = vec![Action {
                name: "x".into(),
                parameters: HashMap::new(),
            }];
            assert!(p.choose_action(&s, &a).is_some());
        }

        #[test]
        fn test_policy_name() {
            assert_eq!(GreedyPolicy::new().name(), "greedy");
            assert_eq!(RandomPolicy::new(0).name(), "random");
            assert_eq!(
                EpsilonGreedyPolicy::new(Box::new(GreedyPolicy::new()), 0.1).name(),
                "epsilon_greedy"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Persistence Tests
    // ═══════════════════════════════════════════════════════════════
    mod persistence_tests {
        use crate::l5_cognition::nt_mind::nt_game::persistence::*;

        #[test]
        fn test_snapshot() {
            let s = GameStateSnapshot::new().with_tick(100);
            assert_eq!(s.tick, 100);
        }

        #[test]
        fn test_snapshot_default() {
            let s = GameStateSnapshot::default();
            assert_eq!(s.version, 1);
            assert_eq!(s.tick, 0);
        }

        #[test]
        fn test_snapshot_metadata() {
            let s = GameStateSnapshot::new()
                .with_metadata("key", "value")
                .with_metadata("key2", "value2");
            assert_eq!(s.metadata.get("key").unwrap(), "value");
            assert_eq!(s.metadata.get("key2").unwrap(), "value2");
        }

        #[test]
        fn test_snapshot_json_roundtrip() {
            let s = GameStateSnapshot::new()
                .with_tick(42)
                .with_metadata("mode", "test");
            let json = s.to_json().unwrap();
            let loaded = GameStateSnapshot::from_json(&json).unwrap();
            assert_eq!(loaded.tick, 42);
            assert_eq!(loaded.metadata.get("mode").unwrap(), "test");
        }

        #[test]
        fn test_replay() {
            let mut rb = ReplayBuffer::new(10);
            rb.record(ReplayFrame {
                tick: 1,
                action: "move".into(),
                state: GameStateSnapshot::new(),
                reward: 1.0,
            });
            assert_eq!(rb.len(), 1);
            assert!((rb.avg_reward() - 1.0).abs() < 0.001);
        }

        #[test]
        fn test_replay_overflow() {
            let mut rb = ReplayBuffer::new(3);
            for i in 0..5 {
                rb.record(ReplayFrame {
                    tick: i,
                    action: "act".into(),
                    state: GameStateSnapshot::new(),
                    reward: i as f64,
                });
            }
            assert_eq!(rb.len(), 3);
        }

        #[test]
        fn test_replay_total_reward() {
            let mut rb = ReplayBuffer::new(10);
            rb.record(ReplayFrame {
                tick: 0,
                action: "a".into(),
                state: GameStateSnapshot::new(),
                reward: 1.0,
            });
            rb.record(ReplayFrame {
                tick: 1,
                action: "b".into(),
                state: GameStateSnapshot::new(),
                reward: 2.0,
            });
            assert!((rb.total_reward() - 3.0).abs() < 0.001);
        }

        #[test]
        fn test_replay_avg_reward_empty() {
            let rb = ReplayBuffer::new(10);
            assert_eq!(rb.avg_reward(), 0.0);
        }

        #[test]
        fn test_replay_recent() {
            let mut rb = ReplayBuffer::new(10);
            for i in 0..5 {
                rb.record(ReplayFrame {
                    tick: i,
                    action: "a".into(),
                    state: GameStateSnapshot::new(),
                    reward: 0.0,
                });
            }
            let recent = rb.recent(3);
            assert_eq!(recent.len(), 3);
        }

        #[test]
        fn test_replay_get_frame() {
            let mut rb = ReplayBuffer::new(10);
            rb.record(ReplayFrame {
                tick: 42,
                action: "test".into(),
                state: GameStateSnapshot::new(),
                reward: 5.0,
            });
            assert!(rb.get_frame(42).is_some());
            assert!(rb.get_frame(99).is_none());
        }

        #[test]
        fn test_replay_clear() {
            let mut rb = ReplayBuffer::new(10);
            rb.record(ReplayFrame {
                tick: 0,
                action: "a".into(),
                state: GameStateSnapshot::new(),
                reward: 1.0,
            });
            rb.clear();
            assert!(rb.is_empty());
        }
    }
}
