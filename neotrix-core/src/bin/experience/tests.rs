    use super::*;

    #[test]
    fn test_value_roundtrip() {
        let long = "x".repeat(2000);
        let encoded = value_encode(&long);
        assert!(encoded.len() >= 4 && &encoded[..4] == VALUE_MAGIC);
        assert_eq!(value_decode(&encoded).unwrap(), long);
        let short = "abc";
        assert_eq!(value_encode(short), short.as_bytes());
    }

    #[test]
    fn test_concept_hash_stable() {
        assert_eq!(concept_hash("neotrix"), concept_hash("neotrix"));
        assert_eq!(concept_hash("hebb").len(), 16);
        assert_ne!(concept_hash("a"), concept_hash("b"));
    }

    #[test]
    fn test_extract_concepts_en() {
        let s = "DelegateEngine wired to E8 with area and note and this and check";
        let c = extract_concepts(s);
        assert!(c.contains("DelegateEngine"));
        assert!(!c.contains("area"));
        assert!(!c.contains("this"));
        assert!(!c.contains("check"));
    }

    #[test]
    fn test_extract_concepts_cn() {
        let c = extract_concepts("神经网络化 是 关键一步");
        assert!(c.contains("神经网络化"));
    }

    #[test]
    fn test_extract_concepts_long_cn_window() {
        let c = extract_concepts("这是一个非常长的中文概念短语用于测试滑窗切词行为");
        // 至少应切出一些 3-4 字窗口且不含纯停用字
        assert!(c.iter().any(|t| t.chars().count() == 3 || t.chars().count() == 4));
    }

    #[test]
    fn test_cycle_sort_key() {
        assert_eq!(cycle_sort_key("160d"), (160, "d".to_string()));
        assert_eq!(cycle_sort_key("201"), (201, "201".to_string()));
        assert!(cycle_sort_key("201b") < cycle_sort_key("201c"));
    }

    #[test]
    fn test_is_stale_and_norm() {
        let ts = now_ts();
        let v = json!({"verify_by": ts - 100});
        assert!(is_stale(v.get("verify_by"), ts));
        let v2 = json!({"verify_by": ts + 100});
        assert!(!is_stale(v2.get("verify_by"), ts));
        let v3 = json!({"verify_by": "not-a-date"});
        assert!(!is_stale(v3.get("verify_by"), ts));
        let e = json!({"verify_by": Value::Null});
        assert_eq!(
            norm_verify_by(&e, ts),
            Some(ts + VERIFY_DEFAULT_DAYS * DAY)
        );
    }

    #[test]
    fn test_scan_values_handles_text_and_blob() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE kv_store (namespace TEXT, key TEXT, value BLOB, updated_at INTEGER)",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO kv_store VALUES ('experience','concept_a','plain text',1)",
            [],
        )
        .unwrap();
        let encoded = value_encode("compressed content here");
        conn.execute(
            "INSERT INTO kv_store (namespace,key,value,updated_at) VALUES ('experience','concept_b',?1,1)",
            params![encoded],
        )
        .unwrap();
        let rows = scan_values(&conn, "concept_");
        assert_eq!(rows.len(), 2, "scan must read both TEXT and BLOB rows");
        assert!(rows.iter().any(|(k, v)| k == "concept_a" && v == "plain text"));
        assert!(
            rows.iter()
                .any(|(k, v)| k == "concept_b" && v == "compressed content here")
        );
        assert_eq!(kv_get(&conn, NS, "concept_a").as_deref(), Some("plain text"));
        assert_eq!(
            kv_get(&conn, NS, "concept_b").as_deref(),
            Some("compressed content here")
        );
    }

    // ─── R-P97: absorb-node 测试 ──────────────────────────────
    /// [根因 a] 82d06141 起 cmd_absorb_node 经 KnowledgeBase::open(None) 写
    /// $HOME/.neotrix/knowledge.db (不再直写传入 conn) — 旧 node_test_db() 内存库
    /// 恒空且写路径污染生产 KB (失败日志: fresh 库上报 "duplicate ... hub=true")。
    /// 正确语义: HOME 重定向到每测试独立临时目录, 断言 conn 打开同一 KB 文件。
    /// 本 bin 仅这 4 个测试触碰 HOME: 本地锁串行化防 set_var 进程级竞态
    /// (lib 的 TEST_ENV_LOCK 为 #[cfg(test)], bin 测试不可见; 跨测试进程 env 不共享)。
    fn with_isolated_kb_home(name: &str, f: impl FnOnce(&Connection)) {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = std::env::temp_dir().join(format!(
            "nt-experience-node-test-{}-{}",
            name,
            std::process::id()
        ));
        std::fs::create_dir_all(home.join(".neotrix")).expect("create temp home");
        std::env::set_var("HOME", &home);
        let conn = Connection::open(home.join(".neotrix").join("knowledge.db"))
            .expect("open isolated kb file");
        crate::nt_memory_schema::initialize(&conn).expect("init isolated schema");
        f(&conn);
        drop(conn);
        std::fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn test_absorb_node_insert_and_fts() {
        with_isolated_kb_home("insert_and_fts", |conn| {
            let node = json!({
                "url": "https://example.github.io/demo/",
                "title": "Demo Page",
                "summary": "A test article",
                "content": "This is a test article body with enough length to be meaningful for the FTS index.",
                "node_type": "article",
                "language": "en",
                "domain": "example.github.io",
                "importance": 0.7,
            });
            // 写临时文件 (cmd_absorb_node 读文件)
            let path = std::env::temp_dir().join(format!("nt_test_node_{}.json", std::process::id()));
            std::fs::write(&path, node.to_string()).unwrap();
            let dry_run = false;
            let apply_cap = false;
            cmd_absorb_node(conn, path.to_str().unwrap(), dry_run, apply_cap);
            // 节点已写入 (按 title 定域: ensure_hub 的 KB-<DOMAIN> 枢纽同表落盘)
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM nodes WHERE title='Demo Page'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "node inserted");
            // FTS 已同步 (防 PA011 desync; title 定域隔离枢纽行)
            let fts_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM nodes_fts WHERE title='Demo Page'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(fts_count, 1, "FTS row inserted");
            // [根因 b] id 由管道 uuid::new_v4 派生 (batch_ 前缀为旧裸 SQL 方案, 82d06141 移除)
            let id: String = conn
                .query_row("SELECT id FROM nodes WHERE title='Demo Page'", [], |r| r.get(0))
                .unwrap();
            assert!(!id.is_empty(), "pipeline-derived node id must be non-empty");
            std::fs::remove_file(&path).ok();
        });
    }

    #[test]
    fn test_absorb_node_duplicate_dedup() {
        with_isolated_kb_home("duplicate_dedup", |conn| {
            let node = json!({
                "url": "https://example.github.io/demo/",
                "title": "Demo Page",
                "content": "Same URL must be deduplicated.",
                "node_type": "article",
            });
            let path = std::env::temp_dir().join(format!("nt_test_node2_{}.json", std::process::id()));
            std::fs::write(&path, node.to_string()).unwrap();
            cmd_absorb_node(conn, path.to_str().unwrap(), false, false);
            cmd_absorb_node(conn, path.to_str().unwrap(), false, false);
            // title 定域: 隔离 ensure_hub 枢纽行, 只计内容节点
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM nodes WHERE title='Demo Page'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "duplicate URL must not double-insert");
            let fts_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM nodes_fts WHERE title='Demo Page'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(fts_count, 1, "FTS also deduplicated");
            std::fs::remove_file(&path).ok();
        });
    }

    #[test]
    fn test_absorb_node_dry_run_and_capability() {
        with_isolated_kb_home("dry_run_capability", |conn| {
            let node = json!({
                "url": "https://example.github.io/cap/",
                "title": "Cap Page",
                "content": "Capability mapping test node with sufficient content length.",
                "node_type": "article",
                "capability": {"branch": "NT-MIND", "capability": "generate", "evidence": "test"},
            });
            let path = std::env::temp_dir().join(format!("nt_test_node3_{}.json", std::process::id()));
            std::fs::write(&path, node.to_string()).unwrap();
            // dry-run: 不写入
            cmd_absorb_node(conn, path.to_str().unwrap(), true, false);
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM nodes WHERE title='Cap Page'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 0, "dry-run must not insert");
            // 实际写入 + capability (url 为 normalize_url 归一形: 去尾斜杠)
            cmd_absorb_node(conn, path.to_str().unwrap(), false, true);
            let meta: String = conn
                .query_row("SELECT metadata FROM nodes WHERE url='https://example.github.io/cap'",
                           [], |r| r.get(0))
                .unwrap();
            let m: Value = serde_json::from_str(&meta).unwrap();
            assert_eq!(m["absorbed_capability"]["branch"], "NT-MIND");
            assert_eq!(m["absorbed_capability"]["capability"], "generate");
            assert_eq!(m["absorbed_capability"]["evidence"], "test");
            std::fs::remove_file(&path).ok();
        });
    }

    #[test]
    fn test_update_node_metadata_merge_and_dry_run() {
        with_isolated_kb_home("update_metadata", |conn| {
            // 先插入一个带初始 metadata 的节点
            let node = json!({
                "url": "https://example.github.io/meta/",
                "title": "Meta Page",
                "content": "Metadata update test node with sufficient content length.",
                "node_type": "article",
                "meta": {"existing": "keep-me"},
            });
            let path = std::env::temp_dir().join(format!("nt_test_meta_node_{}.json", std::process::id()));
            std::fs::write(&path, node.to_string()).unwrap();
            cmd_absorb_node(conn, path.to_str().unwrap(), false, false);
            // [根因 b] title 定域取内容节点 id — LIMIT 1 可能命中后插入的枢纽行
            let nid: String = conn
                .query_row("SELECT id FROM nodes WHERE title='Meta Page'", [], |r| r.get(0))
                .unwrap();
            std::fs::remove_file(&path).ok();

            // dry-run: 不写入
            let updates = json!([{
                "node_id": nid,
                "patch": {"absorbed_capability": {"branch": "NT-ACT", "capability": "execute"}}
            }]);
            let up = std::env::temp_dir().join(format!("nt_test_update_{}.json", std::process::id()));
            std::fs::write(&up, updates.to_string()).unwrap();
            cmd_update_node_metadata(conn, up.to_str().unwrap(), true);
            let meta: String = conn
                .query_row("SELECT metadata FROM nodes WHERE id=?1", params![nid], |r| r.get(0))
                .unwrap();
            let m: Value = serde_json::from_str(&meta).unwrap();
            assert_eq!(m["existing"], "keep-me", "dry-run must not modify metadata");
            assert!(m.get("absorbed_capability").is_none(), "dry-run must not add capability");

            // 实际写入: 合并 patch, 保留既有字段
            cmd_update_node_metadata(conn, up.to_str().unwrap(), false);
            let meta: String = conn
                .query_row("SELECT metadata FROM nodes WHERE id=?1", params![nid], |r| r.get(0))
                .unwrap();
            let m: Value = serde_json::from_str(&meta).unwrap();
            assert_eq!(m["existing"], "keep-me", "existing metadata preserved");
            assert_eq!(m["absorbed_capability"]["branch"], "NT-ACT");
            assert_eq!(m["absorbed_capability"]["capability"], "execute");
            std::fs::remove_file(&up).ok();
        });
    }

    #[test]
    fn test_high_signal_words() {
        let ws = high_signal_words("the neural network training on GPU failed");
        // 停用词 the/on 剔除, 数字剔除, 短词剔除
        assert!(!ws.contains(&"the".to_string()));
        assert!(!ws.contains(&"on".to_string()));
        assert!(ws.contains(&"neural".to_string()));
        assert!(ws.contains(&"training".to_string()));
        // 去重
        let ws2 = high_signal_words("error error error retry");
        assert_eq!(ws2.iter().filter(|w| *w == "error").count(), 1);
        assert!(ws2.contains(&"retry".to_string()));
    }

    #[test]
    fn test_distill_pattern_aggregates() {
        let contents = vec![
            "neural network training failed on GPU memory".to_string(),
            "neural network training needs more GPU memory".to_string(),
            "neural network training error GPU memory overflow".to_string(),
        ];
        let p = distill_pattern("NT-CORE", &contents);
        assert!(p.starts_with("[蒸馏-NT-CORE]"));
        assert!(p.contains("聚合 3 条经验"));
        // 高频词 neural/network/training 应出现在关键词区
        assert!(p.contains("neural"));
        assert!(p.contains("training"));
    }

    #[test]
    fn test_cmd_distill_dry_run_marks_nothing() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        // 插入 3 条同主题经验 (NT-CORE)
        for i in 0..3 {
            let key = format!("branch_test_{}", i);
            let v = json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": format!("neural network training error GPU memory case {}", i),
                "evidence": "file.rs:1", "source": "test",
                "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            });
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        // dry-run: 不落盘蒸馏, 不标记
        cmd_distill(&mut conn, Some("NT-CORE"), 3, true);
        let cnt: i64 = conn
            .query_row("SELECT COUNT(*) FROM kv_store WHERE namespace=?1", params![NS], |r| r.get(0))
            .unwrap();
        assert_eq!(cnt, 3, "dry-run must not add distilled pattern");
        let marked: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM kv_store WHERE namespace=?1 AND value LIKE '%distilled%'",
                params![NS],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(marked, 0, "dry-run must not mark distilled");
    }

    #[test]
    fn test_cmd_distill_creates_pattern_and_marks() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        for i in 0..3 {
            let key = format!("branch_test_{}", i);
            let v = json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": format!("neural network training error GPU memory case {}", i),
                "evidence": "file.rs:1", "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            });
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        cmd_distill(&mut conn, Some("NT-CORE"), 3, false);
        // 新增 1 条蒸馏 pattern (key 前缀 branch_distill_)
        let patterns: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM kv_store WHERE namespace=?1 AND key LIKE 'branch_distill_%'",
                params![NS],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(patterns, 1, "应生成 1 条蒸馏模式");
        // 3 条原始经验标记 distilled (解码后检查)
        let mut marked = 0;
        for (_, value) in scan_values(&conn, "branch_test_") {
            let j: Value = serde_json::from_str(&value).unwrap();
            if j.get("distilled").and_then(|x| x.as_bool()).unwrap_or(false) {
                marked += 1;
            }
        }
        assert_eq!(marked, 3, "3 条原始经验应标记 distilled");
    }

    #[test]
    fn test_cmd_distill_generates_consciousness_entry() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        for i in 0..3 {
            let key = format!("branch_test_{}", i);
            let v = json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": format!("neural network training error GPU memory case {}", i),
                "evidence": "file.rs:1", "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            });
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        cmd_distill(&mut conn, Some("NT-CORE"), 3, false);
        // 意识体维度条目生成
        let ckey: String = conn
            .query_row(
                "SELECT key FROM kv_store WHERE namespace=?1 AND key LIKE 'branch_consciousness_%'",
                params![NS],
                |r| r.get(0),
            )
            .unwrap();
        let (_, cvalue) = scan_values(&conn, "branch_consciousness_")
            .into_iter()
            .next()
            .unwrap();
        let c: Value = serde_json::from_str(&cvalue).unwrap();
        assert_eq!(c["dimension"], "consciousness");
        assert_eq!(c["type"], "insight");
        assert!(c["content"].as_str().unwrap().contains("意识体蒸馏"));
        assert!(c["distilled_from"].as_array().unwrap().len() >= 3);
        let _ = ckey;
    }

    #[test]
    fn test_query_filters_distilled_by_default() {
        let conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        let now = now_ts();
        // 2 条普通 + 1 条 distilled
        let mut entries = vec![
            ("branch_a_1", json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": "neural network training tip one", "evidence": "f:1",
                "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            })),
            ("branch_a_2", json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": "neural network training tip two", "evidence": "f:2",
                "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            })),
            ("branch_a_3", json!({
                "schema_version": 1, "type": "insight", "session_id": "t",
                "cycle": "1", "ts": now, "domain": "NT-CORE",
                "content": "neural network training distilled old", "evidence": "f:3",
                "distilled": true, "verify_by": now + VERIFY_DEFAULT_DAYS * DAY,
            })),
        ];
        for (k, v) in entries.drain(..) {
            conn.execute(
                "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, k, value_encode(&v.to_string()), now],
            )
            .unwrap();
        }
        // 默认过滤 distilled → 2 条
        let res = cmd_query(&conn, "neural", None, None, 10, false, false, false, false);
        assert_eq!(res, 2, "默认应过滤 distilled 条目, 得到 {}", res);
        // include_distilled → 3 条
        let res2 = cmd_query(&conn, "neural", None, None, 10, false, false, false, true);
        assert_eq!(res2, 3, "include_distilled 应含原始条目");
    }

    #[test]
    fn test_field_stage_tick_kv_get_equivalence() {
        // W4 场账本写路径: 追加型经验写入 stage→tick 后, 下游 kv_get 读回必须与
        // 直写 kv_set 完全等价 (ns/key/value 三元组不变), 且未 tick 前不可见。
        let conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();

        let branch = json!({
            "schema_version": 1, "type": "insight", "session_id": "sess_w4",
            "cycle": "1200", "ts": 1, "domain": "NT-MEMORY",
            "content": "field ledger roundtrip", "evidence": "tests.rs"
        });
        let audit = json!({"idx": 0, "decision": "written", "session_id": "sess_w4"});

        // 暂存阶段: 不入正式状态 — kv_get 不可见
        kv_stage(&conn, NS, "branch_1200_0_w4aabb", &branch.to_string());
        kv_stage(&conn, "audit", "audit_1200_sess_w4_0", &audit.to_string());
        assert_eq!(kv_get(&conn, NS, "branch_1200_0_w4aabb"), None);
        assert_eq!(kv_get(&conn, "audit", "audit_1200_sess_w4_0"), None);

        // 一批一解: 统一求解下一版本
        let r = crate::nt_field_ledger::field_tick(&conn).unwrap().expect("receipt");
        assert_eq!(r.drained, 2, "两条暂存都参与合并");
        assert_eq!(r.applied, 2, "新键无冲突, 全部落账");
        assert_eq!(r.version, 1);

        // 读回等价: 与直写 kv_set 的可见结果一致
        assert_eq!(
            kv_get(&conn, NS, "branch_1200_0_w4aabb").as_deref(),
            Some(branch.to_string().as_str())
        );
        assert_eq!(
            kv_get(&conn, "audit", "audit_1200_sess_w4_0").as_deref(),
            Some(audit.to_string().as_str())
        );

        // 空集 tick 幂等 + 哈希链完整
        assert!(crate::nt_field_ledger::field_tick(&conn).unwrap().is_none());
        assert!(crate::nt_field_ledger::field_verify_chain(&conn).unwrap());
    }

    #[test]
    fn test_cmd_rune_set_and_get() {
        let conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        // set crimson
        cmd_rune(&conn, "set", Some("crimson"), Some("NT-MEMORY"));
        let val = kv_get(&conn, NS, "rune:NT-MEMORY:crimson");
        assert!(val.is_some(), "rune should be stored");
        let v: Value = serde_json::from_str(&val.unwrap()).unwrap();
        assert_eq!(v["color"], "crimson");
        assert_eq!(v["module"], "NT-MEMORY");
        // get should not panic (count query)
        cmd_rune(&conn, "get", None, Some("NT-MEMORY"));
        // invalid color should not insert
        cmd_rune(&conn, "set", Some("invalid"), Some("NT-MEMORY"));
        let invalid = kv_get(&conn, NS, "rune:NT-MEMORY:invalid");
        assert!(invalid.is_none(), "invalid color must not be stored");
    }

    #[test]
    fn test_cmd_constellation_audit_and_mature() {
        let conn = Connection::open_in_memory().unwrap();
        crate::nt_memory_schema::initialize(&conn).unwrap();
        // insert a maturity record
        conn.execute(
            "INSERT INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![NS, "maturity:NT-CORE", "4", now_ts()],
        )
        .unwrap();
        // audit should not panic (covers both target=all and specific)
        cmd_constellation(&conn, "audit", None);
        cmd_constellation(&conn, "audit", Some("NT-CORE"));
        cmd_constellation(&conn, "mature", None);
        cmd_constellation(&conn, "mature", Some("NT-CORE"));
        // empty domain case already covered by audit with C0
    }

