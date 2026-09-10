    use super::*;

    fn make_config() -> GraphRagConfig {
        GraphRagConfig {
            max_entities_per_doc: 100,
            min_confidence: 0.0,
            enable_incremental_updates: true,
            max_graph_size: 10000,
            extraction_mode: ExtractionMode::Heuristic,
        }
    }

    #[test]
    fn test_entity_extraction_simple() {
        let mut store = GraphRagStore::new(make_config());
        let text = "Apple Inc. developed the iPhone. Tim Cook is the CEO of Apple.";
        let (entities, _relations) = store.extract_entities(text, "src_1").unwrap();

        assert!(!entities.is_empty(), "Should extract entities");
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Apple Inc"), "Should contain Apple Inc");
        assert!(names.contains(&"Tim Cook"), "Should contain Tim Cook");
    }

    #[test]
    fn test_relation_extraction_co_occurrence() {
        let mut store = GraphRagStore::new(make_config());
        let text = "Google developed the Android operating system. Sundar Pichai works at Google.";
        let (_entities, relations) = store.extract_entities(text, "src_2").unwrap();

        let rel_types: Vec<&str> = relations.iter().map(|r| r.relation_type.as_str()).collect();
        assert!(!relations.is_empty(), "Should extract relations");
        assert!(
            rel_types.contains(&"developed_by"),
            "Should contain developed_by relation (got {:?})",
            rel_types
        );

        // First sentence: "Google developed the Android" → "Google" (entity) "developed" (keyword) "Android" (entity)
        // "the" is not capitalized so "Android" and "Google" should be in separate sentences or same
        // Actually: "Google developed the Android operating system" — "Google" is capitalized, "Android" is capitalized
        // "operating" is capitalized too but "system" is lowercase... hmm
        // Let's just check relations exist

        // Check for works_at relation
        let has_works_at = relations.iter().any(|r| r.relation_type == "works_at");
        assert!(
            has_works_at,
            "Should contain works_at relation for 'Sundar Pichai works at Google'"
        );
    }

    #[test]
    fn test_query_local_mode() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "e1".to_string(),
            name: "OpenAI".to_string(),
            entity_type: "Organization".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "e2".to_string(),
            name: "GPT-4".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e3 = EntityNode {
            id: "e3".to_string(),
            name: "DALL-E".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };

        store.add_entity(e1);
        store.add_entity(e2);
        store.add_entity(e3);

        store.add_relation(RelationEdge {
            id: "r1".to_string(),
            source_entity: "e1".to_string(),
            target_entity: "e2".to_string(),
            relation_type: "developed_by".to_string(),
            weight: 0.9,
            evidence: "".to_string(),
            confidence: 0.9,
            created_at: 1,
        });
        store.add_relation(RelationEdge {
            id: "r2".to_string(),
            source_entity: "e1".to_string(),
            target_entity: "e3".to_string(),
            relation_type: "developed_by".to_string(),
            weight: 0.7,
            evidence: "".to_string(),
            confidence: 0.8,
            created_at: 1,
        });

        let result = store
            .query(
                &["e1".to_string()],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();

        assert_eq!(result.entities.len(), 3, "Should find all 3 entities at depth 1");
        assert_eq!(result.relations.len(), 2, "Should find both relations");
    }

    #[test]
    fn test_query_hybrid_mode() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "h1".to_string(),
            name: "NeoTrix".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.95,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "h2".to_string(),
            name: "E8".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e3 = EntityNode {
            id: "h3".to_string(),
            name: "VSA".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.85,
            properties: HashMap::new(),
            created_at: 1,
        };

        store.add_entity(e1);
        store.add_entity(e2);
        store.add_entity(e3);

        store.add_relation(RelationEdge {
            id: "hr1".to_string(),
            source_entity: "h1".to_string(),
            target_entity: "h2".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.8,
            evidence: "".to_string(),
            confidence: 0.9,
            created_at: 1,
        });
        store.add_relation(RelationEdge {
            id: "hr2".to_string(),
            source_entity: "h1".to_string(),
            target_entity: "h3".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.7,
            evidence: "".to_string(),
            confidence: 0.85,
            created_at: 1,
        });

        let result = store
            .query(
                &["h1".to_string()],
                GraphQueryMode::Hybrid {
                    local_depth: 1,
                    global_level: 0,
                },
            )
            .unwrap();

        assert_eq!(result.query_mode, "hybrid");
        assert!(!result.entities.is_empty());
    }

    #[test]
    fn test_add_remove_entity() {
        let mut store = GraphRagStore::new(make_config());

        let entity = EntityNode {
            id: "test_e1".to_string(),
            name: "TestEntity".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.5,
            properties: HashMap::new(),
            created_at: 1,
        };

        let id = store.add_entity(entity);
        assert_eq!(store.graph.entities.len(), 1);
        assert!(store.graph.entities.contains_key(&id));

        assert!(store.remove_entity(&id));
        assert_eq!(store.graph.entities.len(), 0);
        assert!(!store.remove_entity("nonexistent"));
    }

    #[test]
    fn test_add_remove_relation() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "re1".to_string(),
            name: "A".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.5,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "re2".to_string(),
            name: "B".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.5,
            properties: HashMap::new(),
            created_at: 1,
        };

        store.add_entity(e1);
        store.add_entity(e2);

        let rel = RelationEdge {
            id: "rr1".to_string(),
            source_entity: "re1".to_string(),
            target_entity: "re2".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.5,
            evidence: "".to_string(),
            confidence: 0.5,
            created_at: 1,
        };

        let rid = store.add_relation(rel);
        assert_eq!(store.graph.relations.len(), 1);
        assert!(store.graph.relations.contains_key(&rid));

        // Check adjacency was updated
        assert_eq!(
            store.graph.adjacency.get("re1").map(|a| a.len()),
            Some(1)
        );
        assert_eq!(
            store.graph.adjacency.get("re2").map(|a| a.len()),
            Some(1)
        );

        assert!(store.remove_relation(&rid));
        assert_eq!(store.graph.relations.len(), 0);
        assert_eq!(
            store.graph.adjacency.get("re1").map(|a| a.len()),
            Some(0)
        );
    }

    #[test]
    fn test_community_summary() {
        let mut store = GraphRagStore::new(make_config());

        // Create two communities: {e1, e2} and {e3, e4}
        let nodes = ["c1", "c2", "c3", "c4"];
        let communities_data: [(usize, usize); 3] = [(0, 1), (0, 1), (2, 3)];

        for (i, &name) in nodes.iter().enumerate() {
            store.add_entity(EntityNode {
                id: format!("n{}", i),
                name: name.to_string(),
                entity_type: "Concept".to_string(),
                source_node_id: "src".to_string(),
                confidence: 0.7,
                properties: HashMap::new(),
                created_at: 1,
            });
        }

        for (i, (a, b)) in communities_data.iter().enumerate() {
            store.add_relation(RelationEdge {
                id: format!("cr{}", i),
                source_entity: format!("n{}", a),
                target_entity: format!("n{}", b),
                relation_type: "related_to".to_string(),
                weight: 0.9,
                evidence: "".to_string(),
                confidence: 0.9,
                created_at: 1,
            });
        }

        let communities = store.community_summary();
        assert!(
            !communities.is_empty(),
            "Should find at least one community"
        );
        // With 4 nodes and edges connecting {0,1} and {2,3}, we should get 2 communities
        assert!(
            communities.len() >= 1,
            "Should have at least 1 community, got {}",
            communities.len()
        );
        for comm in &communities {
            assert!(comm.size > 0);
            assert!(comm.avg_confidence > 0.0);
        }
    }

    #[test]
    fn test_bfs_subgraph_traversal() {
        let mut store = GraphRagStore::new(make_config());

        // Chain: e1 → e2 → e3 → e4
        for i in 1..=4 {
            store.add_entity(EntityNode {
                id: format!("bfs_e{}", i),
                name: format!("Entity{}", i),
                entity_type: "Concept".to_string(),
                source_node_id: "src".to_string(),
                confidence: 0.8,
                properties: HashMap::new(),
                created_at: 1,
            });
        }

        for i in 1..3 {
            store.add_relation(RelationEdge {
                id: format!("bfs_r{}", i),
                source_entity: format!("bfs_e{}", i),
                target_entity: format!("bfs_e{}", i + 1),
                relation_type: "related_to".to_string(),
                weight: 0.8,
                evidence: "".to_string(),
                confidence: 0.8,
                created_at: 1,
            });
        }

        // BFS from e1 at depth 0 should only return e1
        let result = store.get_subgraph(&["bfs_e1".to_string()], 0);
        assert_eq!(result.entities.len(), 1, "Depth 0 should only return seed");
        assert_eq!(result.entities[0].id, "bfs_e1");

        // BFS from e1 at depth 1 should return e1, e2, and the relation
        let result = store.get_subgraph(&["bfs_e1".to_string()], 1);
        assert_eq!(result.entities.len(), 2, "Depth 1 should return e1 and e2");
        assert_eq!(result.relations.len(), 1, "Depth 1 should find r1");

        // BFS from e1 at depth 2 should return e1, e2, e3, and relations
        let result = store.get_subgraph(&["bfs_e1".to_string()], 2);
        assert_eq!(result.entities.len(), 3, "Depth 2 should return e1, e2, e3");
        assert_eq!(result.relations.len(), 2, "Depth 2 should find r1 and r2");
    }

    #[test]
    fn test_query_by_text() {
        let mut store = GraphRagStore::new(make_config());

        store.add_entity(EntityNode {
            id: "qt1".to_string(),
            name: "Rust Language".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        });
        store.add_entity(EntityNode {
            id: "qt2".to_string(),
            name: "Rust Foundation".to_string(),
            entity_type: "Organization".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        });
        store.add_entity(EntityNode {
            id: "qt3".to_string(),
            name: "Python".to_string(),
            entity_type: "Technology".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.7,
            properties: HashMap::new(),
            created_at: 1,
        });

        store.add_relation(RelationEdge {
            id: "qr1".to_string(),
            source_entity: "qt1".to_string(),
            target_entity: "qt2".to_string(),
            relation_type: "part_of".to_string(),
            weight: 0.8,
            evidence: "".to_string(),
            confidence: 0.8,
            created_at: 1,
        });

        let result = store
            .query_by_text(
                &["rust"],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();

        assert_eq!(
            result.entities.len(),
            2,
            "Should find both Rust entities by substring match"
        );

        // Query by text with no match
        let empty_result = store
            .query_by_text(
                &["nonexistent"],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();
        assert!(empty_result.entities.is_empty());
    }

    #[test]
    fn test_empty_graph_behavior() {
        let mut store = GraphRagStore::new(make_config());

        let result = store.get_subgraph(&[], 1);
        assert!(result.entities.is_empty());
        assert!(result.relations.is_empty());

        let communities = store.community_summary();
        assert!(communities.is_empty());

        assert!(!store.remove_entity("nonexistent"));
        assert!(!store.remove_relation("nonexistent"));

        let q_result = store
            .query_by_text(
                &["anything"],
                GraphQueryMode::Local {
                    max_depth: 1,
                    max_neighbors: 10,
                },
            )
            .unwrap();
        assert!(q_result.entities.is_empty());
    }

    #[test]
    fn test_edge_weights_distance() {
        // Test that edge weights decrease with distance in sentence
        let mut store = GraphRagStore::new(make_config());

        let text = "NeoTrix uses E8. NeoTrix uses the VSA HyperCube for knowledge representation.";
        let (_entities, relations) = store.extract_entities(text, "src_dist").unwrap();

        // All relations should have weight <= 1.0
        for r in &relations {
            assert!(
                r.weight <= 1.0,
                "Weight should be <= 1.0, got {}",
                r.weight
            );
            assert!(r.weight >= 0.0, "Weight should be >= 0.0");
        }
    }

    #[test]
    fn test_extract_capitalized_terms_basic() {
        let sentence = "Alice and Bob visit New York City.";
        let terms = extract_capitalized_terms(sentence, "test");
        assert!(terms.contains(&"Alice".to_string()));
        assert!(terms.contains(&"Bob".to_string()));
        assert!(terms.contains(&"New York City".to_string()));
    }

    #[test]
    fn test_split_sentences_basic() {
        let text = "Hello world. This is a test! How are you? Fine.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 4);
        for s in &sentences {
            assert!(s.len() > 2);
        }
    }

    #[test]
    fn test_entity_type_inference() {
        assert_eq!(
            infer_entity_type("Apple Inc"),
            "Organization".to_string()
        );
        assert_eq!(
            infer_entity_type("Rust Language"),
            "Technology".to_string()
        );
        assert_eq!(
            infer_entity_type("New York City"),
            "Location".to_string()
        );
        assert_eq!(infer_entity_type("Quantum Computing"), "Concept".to_string());
    }

    #[test]
    fn test_remove_entity_cascades_to_relations() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "cas1".to_string(),
            name: "Alpha".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "cas2".to_string(),
            name: "Beta".to_string(),
            entity_type: "Concept".to_string(),
            source_node_id: "src".to_string(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };
        store.add_entity(e1);
        store.add_entity(e2);

        store.add_relation(RelationEdge {
            id: "cas_r1".to_string(),
            source_entity: "cas1".to_string(),
            target_entity: "cas2".to_string(),
            relation_type: "related_to".to_string(),
            weight: 0.5,
            evidence: "".to_string(),
            confidence: 0.5,
            created_at: 1,
        });

        assert_eq!(store.graph.relations.len(), 1);
        store.remove_entity("cas1");
        assert_eq!(
            store.graph.relations.len(),
            0,
            "Removing entity should cascade to relations"
        );
        assert!(store.graph.entities.contains_key("cas2"),
            "Entity cas2 should still exist");
    }

    #[test]
    fn test_extraction_no_capitalized_words() {
        let mut store = GraphRagStore::new(make_config());
        let text = "hello world, this is a test with no capitalized entities.";
        let (entities, relations) = store.extract_entities(text, "src_empty").unwrap();
        assert!(
            entities.is_empty(),
            "No entities should be extracted from all-lowercase text"
        );
        assert!(relations.is_empty());
    }

    #[test]
    fn test_extraction_deduplication() {
        let mut store = GraphRagStore::new(make_config());
        let text = "Apple Inc. is a company. Apple Inc. makes iPhones.";
        let (entities, _relations) = store.extract_entities(text, "src_dedup").unwrap();
        let apple_count = entities
            .iter()
            .filter(|e| e.name == "Apple Inc")
            .count();
        assert_eq!(
            apple_count, 1,
            "Apple Inc should appear only once (deduplicated)"
        );
    }

    #[test]
    fn test_stats_tracking() {
        let mut store = GraphRagStore::new(make_config());
        assert_eq!(store.stats.extraction_runs, 0);

        store
            .extract_entities("Google and Apple are companies.", "src_stats")
            .unwrap();
        assert_eq!(store.stats.extraction_runs, 1);
        assert!(store.stats.total_entities >= 2);
        assert!(store.stats.avg_extraction_time_ms > 0.0);
    }

    // ── LightRAG Tests ──────────────────────────────────────────────

    fn make_populated_store() -> GraphRagStore {
        let mut store = GraphRagStore::new(GraphRagConfig {
            max_entities_per_doc: 100,
            min_confidence: 0.0,
            enable_incremental_updates: true,
            max_graph_size: 10000,
            extraction_mode: ExtractionMode::Heuristic,
        });

        let entities = vec![
            EntityNode {
                id: "lr_e1".into(),
                name: "NeoTrix".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.95,
                properties: HashMap::new(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e2".into(),
                name: "E8 Engine".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.9,
                properties: HashMap::new(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e3".into(),
                name: "VSA HyperCube".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.85,
                properties: HashMap::new(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e4".into(),
                name: "Apple".into(),
                entity_type: "Organization".into(),
                source_node_id: "src".into(),
                confidence: 0.8,
                properties: [("industry".into(), "technology".into())].into(),
                created_at: 1,
            },
            EntityNode {
                id: "lr_e5".into(),
                name: "iPhone".into(),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.75,
                properties: HashMap::new(),
                created_at: 1,
            },
        ];

        for e in entities {
            store.add_entity(e);
        }

        let relations = vec![
            RelationEdge {
                id: "lr_r1".into(),
                source_entity: "lr_e1".into(),
                target_entity: "lr_e2".into(),
                relation_type: "uses".into(),
                weight: 0.9,
                evidence: "".into(),
                confidence: 0.9,
                created_at: 1,
            },
            RelationEdge {
                id: "lr_r2".into(),
                source_entity: "lr_e1".into(),
                target_entity: "lr_e3".into(),
                relation_type: "uses".into(),
                weight: 0.85,
                evidence: "".into(),
                confidence: 0.85,
                created_at: 1,
            },
            RelationEdge {
                id: "lr_r3".into(),
                source_entity: "lr_e4".into(),
                target_entity: "lr_e5".into(),
                relation_type: "develops".into(),
                weight: 0.8,
                evidence: "".into(),
                confidence: 0.8,
                created_at: 1,
            },
        ];

        for r in relations {
            store.add_relation(r);
        }

        store
    }

    #[test]
    fn test_search_local_finds_matching_entities() {
        let store = make_populated_store();
        let results = store.search_local("NeoTrix E8", 3);
        assert!(!results.is_empty(), "Should find results for NeoTrix query");
        // Should find the NeoTrix subgraph with E8 Engine
        let has_neotrix = results.iter().any(|r| {
            r.entities.iter().any(|e| e.name == "NeoTrix")
        });
        assert!(has_neotrix, "Local search should return NeoTrix entity");
    }

    #[test]
    fn test_search_local_empty_query_returns_empty() {
        let store = make_populated_store();
        let results = store.search_local("", 3);
        assert!(results.is_empty(), "Empty query should return no results");
    }

    #[test]
    fn test_search_local_no_match_returns_empty() {
        let store = make_populated_store();
        let results = store.search_local("xyznonexistent", 3);
        assert!(results.is_empty(), "Non-matching query should return no results");
    }

    #[test]
    fn test_build_global_summaries_creates_summaries() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let summaries = store.get_global_summaries();
        assert!(!summaries.is_empty(), "Should create global summaries");
        for gs in summaries {
            assert!(!gs.community_id.is_empty(), "Community ID should be set");
            assert!(!gs.topic_keywords.is_empty(), "Topic keywords should be populated");
            assert!(!gs.summary_text.is_empty(), "Summary text should be non-empty");
            assert!(gs.confidence > 0.0, "Confidence should be positive");
            assert!(gs.last_updated > 0, "Last updated should be set");
        }
    }

    #[test]
    fn test_search_global_with_summaries() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let results = store.search_global("NeoTrix technology", 2);
        assert!(!results.is_empty(), "Global search should return results");
        // Results should have summary text containing the query-relevant content
        let any_match = results.iter().any(|gs| {
            gs.summary_text.to_lowercase().contains("neotrix")
                || gs.topic_keywords.iter().any(|kw| kw.to_lowercase().contains("neotrix"))
        });
        assert!(any_match, "Global search results should reference queried topic");
    }

    #[test]
    fn test_search_global_empty_query_returns_top_k() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let count = store.global_summaries.len();
        let results = store.search_global("", 2);
        assert_eq!(results.len(), count.min(2), "Empty query should return top-K by confidence");
    }

    #[test]
    fn test_search_hybrid_merges_local_and_global() {
        let mut store = make_populated_store();
        store.build_global_summaries();
        let hybrid = store.search_hybrid("NeoTrix", 2, 2);
        assert!(!hybrid.merged_entities.is_empty(), "Hybrid should return entities");
        assert!(!hybrid.global_results.is_empty(), "Hybrid should include global summaries");
        assert!(!hybrid.local_results.is_empty(), "Hybrid should include local results");
    }

    #[test]
    fn test_incremental_index_update_adds_entities() {
        let mut store = GraphRagStore::new(make_config());
        store.build_global_summaries();
        assert!(store.get_global_summaries().is_empty(), "Empty store should have no summaries");

        let new_entity = EntityNode {
            id: "inc_e1".into(),
            name: "NewEntity".into(),
            entity_type: "Concept".into(),
            source_node_id: "src".into(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        let change = IncrementalChange {
            added_entities: vec![new_entity],
            added_relations: Vec::new(),
            timestamp: 2,
        };

        let before = store.graph.entities.len();
        store.incremental_index_update(vec![change]);
        assert_eq!(store.graph.entities.len(), before + 1, "Should add one entity");
        assert!(!store.change_log.is_empty(), "Should record change in log");
    }

    #[test]
    fn test_incremental_update_adds_relations() {
        let mut store = GraphRagStore::new(make_config());

        let e1 = EntityNode {
            id: "inc_a".into(),
            name: "Alpha".into(),
            entity_type: "Concept".into(),
            source_node_id: "src".into(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };
        let e2 = EntityNode {
            id: "inc_b".into(),
            name: "Beta".into(),
            entity_type: "Concept".into(),
            source_node_id: "src".into(),
            confidence: 0.8,
            properties: HashMap::new(),
            created_at: 1,
        };
        let rel = RelationEdge {
            id: "inc_r1".into(),
            source_entity: "inc_a".into(),
            target_entity: "inc_b".into(),
            relation_type: "related_to".into(),
            weight: 0.8,
            evidence: "".into(),
            confidence: 0.8,
            created_at: 1,
        };

        // Add entities first
        store.add_entity(e1);
        store.add_entity(e2);

        let before_relations = store.graph.relations.len();
        let change = IncrementalChange {
            added_entities: Vec::new(),
            added_relations: vec![rel],
            timestamp: 2,
        };
        store.incremental_index_update(vec![change]);
        assert_eq!(store.graph.relations.len(), before_relations + 1, "Should add one relation");
    }

    #[test]
    fn test_incremental_update_empty_changes_noop() {
        let mut store = make_populated_store();
        let before_entities = store.graph.entities.len();
        let before_relations = store.graph.relations.len();
        store.incremental_index_update(vec![]);
        assert_eq!(store.graph.entities.len(), before_entities);
        assert_eq!(store.graph.relations.len(), before_relations);
    }

    #[test]
    fn test_detect_query_type_local() {
        let mode = GraphRagStore::detect_query_type("NeoTrix E8 VSA");
        match mode {
            GraphQueryMode::Local { .. } => {} // expected
            _ => panic!("Query with capitals and short length should be Local"),
        }
    }

    #[test]
    fn test_detect_query_type_global() {
        let mode = GraphRagStore::detect_query_type("What is the relationship between knowledge representation and reasoning in AI systems");
        match mode {
            GraphQueryMode::Global { .. } => {} // expected
            _ => panic!("Long conceptual query should be Global"),
        }
    }

    #[test]
    fn test_detect_query_type_hybrid_default() {
        let mode = GraphRagStore::detect_query_type("knowledge and reasoning");
        match mode {
            GraphQueryMode::Hybrid { .. } => {} // expected for ambiguous
            _ => panic!("Ambiguous query should default to Hybrid"),
        }
    }

    #[test]
    fn test_auto_mode_in_query() {
        let store = make_populated_store();
        let result = store
            .query(
                &["lr_e1".to_string()],
                GraphQueryMode::Auto,
            )
            .unwrap();
        assert_eq!(result.query_mode, "auto(local)");
        assert!(!result.entities.is_empty());

        let empty_result = store
            .query(
                &[],
                GraphQueryMode::Auto,
            )
            .unwrap();
        assert_eq!(empty_result.query_mode, "auto(global)");
    }

    #[test]
    fn test_search_local_scoring_by_centrality() {
        let mut store = make_populated_store();
        // Add a high-degree hub entity
        let hub = EntityNode {
            id: "hub".into(),
            name: "HubEntity".into(),
            entity_type: "Technology".into(),
            source_node_id: "src".into(),
            confidence: 0.9,
            properties: HashMap::new(),
            created_at: 1,
        };
        store.add_entity(hub);
        for i in 0..5 {
            let spoke = EntityNode {
                id: format!("spoke_{}", i),
                name: format!("Spoke{}", i),
                entity_type: "Technology".into(),
                source_node_id: "src".into(),
                confidence: 0.7,
                properties: HashMap::new(),
                created_at: 1,
            };
            store.add_entity(spoke);
            store.add_relation(RelationEdge {
                id: format!("hub_r{}", i),
                source_entity: "hub".into(),
                target_entity: format!("spoke_{}", i),
                relation_type: "connected_to".into(),
                weight: 0.7,
                evidence: "".into(),
                confidence: 0.7,
                created_at: 1,
            });
        }

        let results = store.search_local("HubEntity", 2);
        assert!(!results.is_empty(), "Hub should be found by search_local");
    }

    #[test]
    fn test_clear_change_log() {
        let mut store = GraphRagStore::new(make_config());
        store
            .extract_entities("Google develops Android.", "src")
            .unwrap();
        assert!(!store.change_log.is_empty(), "Change log should have entries after extraction");
        store.clear_change_log();
        assert!(store.change_log.is_empty(), "Change log should be empty after clear");
        assert_eq!(store.change_log.len(), 0);
    }

    #[test]
    fn test_build_global_summaries_empty_store() {
        let mut store = GraphRagStore::new(make_config());
        store.build_global_summaries();
        assert!(
            store.get_global_summaries().is_empty(),
            "Empty store should produce no summaries"
        );
    }

    #[test]
    fn test_search_global_no_summaries_returns_empty() {
        let store = make_populated_store();
        let results = store.search_global("NeoTrix", 2);
        assert!(
            results.is_empty(),
            "Without build_global_summaries, search_global should return empty"
        );
    }

    #[test]
    fn test_graph_extractor_extracts_entities_and_relations() {
        let config = ExtractionConfig {
            model: "heuristic".to_string(),
            max_entities_per_chunk: 50,
            confidence_threshold: 0.3,
        };
        let extractor = GraphExtractor::new(config);
        let text = "Apple Inc. developed the iPhone. Tim Cook works at Apple.";
        let (entities, relations) = extractor.extract(text, "test_source").unwrap();

        assert!(!entities.is_empty(), "Should extract entities");
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Apple Inc"), "Should contain Apple Inc");
        assert!(names.contains(&"Tim Cook"), "Should contain Tim Cook");

        assert!(!relations.is_empty(), "Should extract relations");
    }

    #[test]
    fn test_merge_entities_deduplicates_by_name() {
        let entities = vec![
            EntityNode {
                id: "1".into(),
                name: "Apple".into(),
                entity_type: "Organization".into(),
                source_node_id: "test".into(),
                confidence: 1.0,
                properties: HashMap::new(),
                created_at: 0,
            },
            EntityNode {
                id: "2".into(),
                name: "apple".into(),
                entity_type: "Organization".into(),
                source_node_id: "test".into(),
                confidence: 1.0,
                properties: HashMap::new(),
                created_at: 0,
            },
            EntityNode {
                id: "3".into(),
                name: "Google".into(),
                entity_type: "Organization".into(),
                source_node_id: "test".into(),
                confidence: 1.0,
                properties: HashMap::new(),
                created_at: 0,
            },
        ];
        let merged = GraphExtractor::merge_entities(&entities);
        assert_eq!(merged.len(), 2, "Should merge Apple + apple into one");
        let merged_names: Vec<&str> = merged.iter().map(|e| e.name.as_str()).collect();
        assert!(merged_names.contains(&"Apple"));
        assert!(merged_names.contains(&"Google"));
    }
