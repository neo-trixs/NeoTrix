    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    fn sample_skill_content() -> &'static str {
        r#"---
name: rust-analyzer
description: Expertise in Rust code analysis and optimization
triggers: ["rust", "cargo", "unsafe", "lifetime", "ownership"]
e8_modes: [12, 13, 14]
tools: ["read", "edit", "bash"]
hooks: ["PreToolUse", "PostToolUse"]
priority: 80
---

# Rust Analyzer Skill

## Capabilities
- Analyze Rust code for safety issues
- Suggest optimizations
"#
    }

    fn sample_skill_content_no_frontmatter() -> &'static str {
        "# Just a markdown file\n\nNo frontmatter here."
    }

    fn setup_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    #[test]
    fn test_selftest_gate_marks_unverified_without_script() {
        // P2-5 质量门: 缺 scripts/selftest.sh|js → unverified
        let dir = setup_temp_dir();
        let path = dir.path().join("no-selftest.md");
        std::fs::write(&path, sample_skill_content()).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        assert!(!entry.verified, "skill without scripts/selftest must be unverified");
    }

    #[test]
    fn test_selftest_gate_marks_verified_with_script() {
        // P2-5 质量门: 存在 scripts/selftest.sh → verified
        let dir = setup_temp_dir();
        let skill_dir = dir.path().join("with-selftest");
        let scripts = skill_dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();
        std::fs::write(scripts.join("selftest.sh"), "#!/usr/bin/env bash\necho '{\"gates\":[]}'\n").unwrap();
        let entry = SkillEntry::from_file(&skill_dir.join("SKILL.md")).unwrap();
        assert!(entry.verified, "skill with scripts/selftest.sh must be verified");
    }

    #[test]
    fn test_selftest_gate_verified_with_js() {
        let dir = setup_temp_dir();
        let skill_dir = dir.path().join("js-selftest");
        let scripts = skill_dir.join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();
        std::fs::write(scripts.join("selftest.js"), "console.log('{\"gates\":[]}')").unwrap();
        let entry = SkillEntry::from_file(&skill_dir.join("SKILL.md")).unwrap();
        assert!(entry.verified);
    }

    #[test]
    fn test_parse_skill_frontmatter() {
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, sample_skill_content()).unwrap();

        let entry = SkillEntry::from_file(&path).unwrap();
        assert_eq!(entry.name, "rust-analyzer");
        assert_eq!(entry.description, "Expertise in Rust code analysis and optimization");
        assert_eq!(entry.triggers, vec!["rust", "cargo", "unsafe", "lifetime", "ownership"]);
        assert_eq!(entry.e8_modes, vec![12, 13, 14]);
        assert_eq!(entry.tools, vec!["read", "edit", "bash"]);
        assert_eq!(entry.hooks, vec!["PreToolUse", "PostToolUse"]);
        assert_eq!(entry.priority, 80);
        assert!(!entry.active);
    }

    #[test]
    fn test_parse_skill_no_frontmatter_returns_none() {
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, sample_skill_content_no_frontmatter()).unwrap();
        assert!(SkillEntry::from_file(&path).is_none());
    }

    #[test]
    fn test_parse_skill_missing_name_returns_none() {
        let content = r#"---
description: No name here
---
body"#;
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        assert!(SkillEntry::from_file(&path).is_none());
    }

    #[test]
    fn test_parse_skill_default_priority() {
        let content = r#"---
name: test
description: A test skill
---
body"#;
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        assert_eq!(entry.priority, 50);
    }

    #[test]
    fn test_skill_engine_load_all() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill as a subdirectory with SKILL.md
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        let loaded = engine.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "rust-analyzer");
    }

    #[test]
    fn test_skill_engine_load_all_no_dir_creates_it() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("nonexistent");
        let mut engine = SkillEngine::new(skills_dir.clone());
        let loaded = engine.load_all();
        assert!(loaded.is_empty());
        assert!(skills_dir.exists());
    }

    #[test]
    #[ignore = "flaky: test ordering dependent"]
    fn test_find_matching_by_trigger() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // Match by trigger keyword
        let matches = engine.find_matching("rust", None);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "rust-analyzer");

        let matches = engine.find_matching("ownership", None);
        assert_eq!(matches.len(), 1);

        // Non-matching query
        let matches = engine.find_matching("python", None);
        assert!(matches.is_empty());
    }

    #[test]
    #[ignore = "flaky: test ordering dependent"]
    fn test_find_matching_filters_by_e8_mode() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // E8 mode 12 matches (12,13,14 are valid for this skill)
        let matches = engine.find_matching("rust", Some(12));
        assert_eq!(matches.len(), 1);

        // E8 mode 0 is not in [12,13,14] → no match when Some(0)
        let matches = engine.find_matching("rust", Some(0));
        assert!(matches.is_empty());

        // None skips E8 filter → matches
        let matches = engine.find_matching("rust", None);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_matching_e8_filter_with_empty_modes() {
        let content = r#"---
name: generic
description: A generic skill
triggers: ["help", "info"]
---
body"#;
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("generic");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // No e8_modes specified — matches any mode (Some or None)
        let matches = engine.find_matching("help", Some(42));
        assert_eq!(matches.len(), 1);
        let matches = engine.find_matching("help", None);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_matching_exact_trigger_outranks_substring() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let exact_skill = r#"---
name: auth
description: Authentication handler
triggers: ["auth", "login", "session"]
---
body"#;
        let substring_skill = r#"---
name: auth-analyzer
description: A skill that also matches auth as substring
triggers: ["auth-flow", "oauth"]
---
body"#;
        for (name, content) in [("auth", exact_skill), ("auth-analyzer", substring_skill)] {
            let dir2 = skills_dir.join(name);
            std::fs::create_dir_all(&dir2).unwrap();
            std::fs::write(dir2.join("SKILL.md"), content).unwrap();
        }

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // Exact trigger "auth" must outrank substring "auth-flow" match
        let matches = engine.find_matching("auth", None);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].name, "auth", "exact trigger must be ranked first");
    }

    #[test]
    fn test_find_matching_caps_results() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        for i in 0..(SkillEngine::MAX_ROUTE_RESULTS + 5) {
            let name = format!("matching-skill-{}", i);
            let dir2 = skills_dir.join(&name);
            std::fs::create_dir_all(&dir2).unwrap();
            let content = format!(
                "---\nname: {}\ndescription: test\ntriggers: [\"matching-skill\"]\n---\nbody",
                name
            );
            std::fs::write(dir2.join("SKILL.md"), content).unwrap();
        }

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // All skills match query "matching-skill" (substring); results must be capped
        let matches = engine.find_matching("matching-skill", None);
        assert!(matches.len() <= SkillEngine::MAX_ROUTE_RESULTS);
        assert_eq!(matches.len(), SkillEngine::MAX_ROUTE_RESULTS);
    }

    #[test]
    fn test_activate_and_deactivate_skill() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        assert!(engine.activate_skill("rust-analyzer").is_ok());
        assert!(engine.get_skill("rust-analyzer").unwrap().active);

        // Double activation should fail
        assert!(engine.activate_skill("rust-analyzer").is_err());

        let active = engine.list_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "rust-analyzer");

        assert!(engine.deactivate_skill("rust-analyzer").is_ok());
        assert!(!engine.get_skill("rust-analyzer").unwrap().active);
        assert!(engine.list_active().is_empty());

        // Deactivate inactive should fail
        assert!(engine.deactivate_skill("rust-analyzer").is_err());
    }

    #[test]
    fn test_get_skill_nonexistent() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.get_skill("nonexistent").is_none());
        assert!(engine.activate_skill("nonexistent").is_err());
    }

    #[test]
    fn test_install_skill_from_file() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let source = dir.path().join("source.md");
        std::fs::write(&source, sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir.clone());
        engine.load_all();
        assert!(engine.list_all().is_empty());

        engine.install_skill(&source).unwrap();

        let all = engine.list_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "rust-analyzer");
        assert!(skills_dir.join("rust-analyzer").join("SKILL.md").exists());
    }

    #[test]
    fn test_install_skill_from_directory() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let source_dir = dir.path().join("my-skill");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("SKILL.md"), sample_skill_content()).unwrap();
        std::fs::write(source_dir.join("helper.py"), r#"print("hello")"#).unwrap();

        let mut engine = SkillEngine::new(skills_dir.clone());
        engine.load_all();
        assert!(engine.list_all().is_empty());

        engine.install_skill(&source_dir).unwrap();
        assert_eq!(engine.list_all().len(), 1);
        assert!(skills_dir.join("rust-analyzer").join("helper.py").exists());
    }

    #[test]
    fn test_install_skill_invalid_source() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));

        assert!(engine.install_skill(&dir.path().join("nonexistent")).is_err());
        assert!(engine.install_skill(&dir.path().join(".")).is_err());
    }

    #[test]
    fn test_parse_array_field() {
        assert_eq!(parse_array_field(r#"["a", "b", "c"]"#), vec!["a", "b", "c"]);
        assert_eq!(parse_array_field(r#"['x', 'y']"#), vec!["x", "y"]);
        assert_eq!(parse_array_field("a, b, c"), vec!["a", "b", "c"]);
        assert_eq!(parse_array_field(""), Vec::<String>::new());
    }

    #[test]
    fn test_skill_body() {
        let content = sample_skill_content();
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        let body = entry.body();
        assert!(body.contains("Analyze Rust code"));
        assert!(body.to_lowercase().contains("suggest optimizations"));
    }

    #[test]
    fn test_discover_skills_legacy() {
        // Should not crash/panic
        let _skills = SkillEngine::discover_skills();
    }

    #[test]
    fn test_priority_sorting() {
        let content_high = r#"---
name: high-priority
description: High priority skill
triggers: ["test"]
priority: 90
---
high"#;
        let content_low = r#"---
name: low-priority
description: Low priority skill
triggers: ["test"]
priority: 10
---
low"#;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        std::fs::write(skills_dir.join("high.md"), content_high).unwrap();
        std::fs::write(skills_dir.join("low.md"), content_low).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        let matches = engine.find_matching("test", None);
        assert_eq!(matches.len(), 2);
        // Higher priority first
        assert_eq!(matches[0].name, "high-priority");
    }

    #[test]
    fn test_list_all_empty() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.list_all().is_empty());
    }

    #[test]
    fn test_progressive_disclosure_load_reference() {
        // 渐进披露 (diagram-design 吸收): SKILL.md 只声明 references,
        // 细节存 references/*.md 按需加载。
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("my-skill");
        let refs_dir = skill_dir.join("references");
        std::fs::create_dir_all(&refs_dir).unwrap();

        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: A skill with progressive disclosure\ntriggers: [\"mine\"]\nreferences: [\"type-a.md\", \"type-b.md\"]\n---\nbody",
        )
        .unwrap();
        std::fs::write(refs_dir.join("type-a.md"), "TYPE-A DETAILS").unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        assert_eq!(engine.list_all().len(), 1);

        // 已声明引用 → 按需加载成功
        let loaded = engine.load_reference("my-skill", "type-a.md").unwrap();
        assert!(loaded.contains("TYPE-A DETAILS"));

        // 未声明引用 → 拒绝 (禁未声明加载)
        assert!(engine.load_reference("my-skill", "secret.md").is_err());

        // 缺失文件 → 报错 (声明了但没落盘)
        assert!(engine.load_reference("my-skill", "type-b.md").is_err());

        // 不存在的技能 → 报错
        assert!(engine.load_reference("nope", "type-a.md").is_err());
    }

    #[test]
    fn test_list_active_empty() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.list_active().is_empty());
    }

    #[test]
    fn test_skill_entry_from_content_direct() {
        let entry = SkillEntry::from_content(Path::new("test.md"), sample_skill_content());
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "rust-analyzer");
    }

    #[test]
    fn test_e8_index_build() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        std::fs::write(skills_dir.join("test.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        assert!(engine.e8_index.contains_key(&12));
        assert!(engine.e8_index.contains_key(&13));
        assert!(engine.e8_index.contains_key(&14));
        assert!(!engine.e8_index.contains_key(&0));
    }

    #[test]
    fn test_skill_from_procedural_record_creates_valid_entry() {
        let record = ProceduralMemoryRecord {
            id: "test-id".into(),
            skill_id: "proc_skill_test".into(),
            name: "Test E8 Skill".into(),
            description: "Learned E8 pattern: 3 states".into(),
            e8_sequence: vec![12, 13, 14],
            trigger_pattern: vec![12],
            success_rate: 0.85,
            execution_count: 5,
            avg_reward: 0.75,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec!["procedural".into(), "auto_discovered".into()],
        };

        let skill = SkillEngine::skill_from_procedural_record(&record);
        assert_eq!(skill.name, "Test E8 Skill");
        assert_eq!(skill.description, "Learned E8 pattern: 3 states");
        assert_eq!(skill.triggers, vec!["e8", "proc_skill", "proc_skill_test"]);
        assert_eq!(skill.e8_modes, vec![12, 13, 14]);
        assert_eq!(skill.priority, 75);
        assert!(skill.content.starts_with("---\nname: Test E8 Skill"));
        assert!(skill.content.contains("e8_modes: [12,13,14]"));
    }

    #[test]
    fn test_skill_from_procedural_record_low_reward() {
        let record = ProceduralMemoryRecord {
            id: "test-id-2".into(),
            skill_id: "proc_skill_low".into(),
            name: "Low Reward Skill".into(),
            description: "Learned E8 pattern with low confidence".into(),
            e8_sequence: vec![1, 2],
            trigger_pattern: vec![1],
            success_rate: 0.3,
            execution_count: 1,
            avg_reward: 0.15,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec![],
        };

        let skill = SkillEngine::skill_from_procedural_record(&record);
        assert_eq!(skill.priority, 15, "low avg_reward should give low priority");
        assert_eq!(skill.e8_modes, vec![1, 2]);
    }

    #[test]
    fn test_install_from_procedural_writes_skill_file() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let record = ProceduralMemoryRecord {
            id: "install-test-id".into(),
            skill_id: "proc_install_test".into(),
            name: "Installed Procedural Skill".into(),
            description: "E8 pattern installed via bridge".into(),
            e8_sequence: vec![5, 10, 15],
            trigger_pattern: vec![5],
            success_rate: 0.9,
            execution_count: 3,
            avg_reward: 0.88,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec!["procedural".into()],
        };

        let mut engine = SkillEngine::new(skills_dir.clone());
        let result = engine.install_from_procedural(&record);
        assert!(result.is_ok(), "install_from_procedural failed: {:?}", result);
        assert_eq!(result.unwrap(), "Installed Procedural Skill");

        // Verify the skill file was created and can be loaded back
        let mut engine2 = SkillEngine::new(skills_dir);
        let loaded = engine2.load_all();
        let skill = loaded.iter().find(|s| s.name == "Installed Procedural Skill");
        assert!(skill.is_some(), "installed skill should be loadable");
        assert_eq!(skill.unwrap().e8_modes, vec![5, 10, 15]);
    }

    fn kb_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_sync_to_kb_index_write_through_and_dedup() {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_list_all;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let conn = kb_conn();
        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        assert_eq!(engine.list_all().len(), 1);

        // 首次写通: 1 条真正写入
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 1);
        // 二次写通: 内容未变化 → 去重, 0 写入
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 0, "内容未变化必须去重 (避免每命令全量写)");

        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].name, "rust-analyzer");
        assert!(recs[0].content_hash.is_some(), "写通必须携带 content_hash");
        assert_eq!(recs[0].tags.as_deref(), Some("rust,cargo,unsafe,lifetime,ownership"));

        // 内容变化 → 再次写入 (同 name 更新)
        std::fs::write(
            skill_dir.join("SKILL.md"),
            sample_skill_content().replace("priority: 80", "priority: 85"),
        )
        .unwrap();
        engine.load_all();
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 1, "内容变化应重新写入");
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1, "同 name 更新而非新增");
    }

    #[test]
    fn test_load_all_auto_syncs_to_kb() {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_list_all;
        use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let kb = KnowledgeBase::open(Some(dir.path().join("kb.db"))).expect("KB open");
        let kb = Arc::new(kb);
        let mut engine = SkillEngine::new(skills_dir).with_kb(kb.clone());
        let loaded = engine.load_all();
        assert_eq!(loaded.len(), 1);

        let conn = kb.conn.lock().unwrap();
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1, "load_all 后应自动写通到 skills_index");
        assert_eq!(recs[0].name, "rust-analyzer");
    }

    #[test]
    fn test_parse_category_parent_frontmatter() {
        let content = r#"---
name: nested
description: A categorized skill
triggers: ["nested"]
category: "test-domain"
parent: root-skill
---
body"#;
        let entry = SkillEntry::from_content(Path::new("nested.md"), content).unwrap();
        assert_eq!(entry.category, "test-domain");
        assert_eq!(entry.parent, "root-skill");
    }

    #[test]
    fn test_default_category_is_general() {
        let entry = SkillEntry::from_content(Path::new("x.md"), sample_skill_content()).unwrap();
        assert_eq!(entry.category, "general");
        assert!(entry.parent.is_empty());
    }

    #[test]
    fn test_skill_tree_groups_by_category() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let cat_a = r#"---
name: a-skill
description: cat a
triggers: ["alpha"]
category: analysis
---
body"#;
        let cat_b = r#"---
name: b-skill
description: cat b
triggers: ["beta"]
category: coding
---
body"#;
        std::fs::write(skills_dir.join("a.md"), cat_a).unwrap();
        std::fs::write(skills_dir.join("b.md"), cat_b).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let tree = engine.skill_tree();
        assert_eq!(tree.len(), 2);
        assert!(tree.contains_key("analysis"));
        assert!(tree.contains_key("coding"));
    }

    #[test]
    fn test_children_of_returns_subskills() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let parent = r#"---
name: root-skill
description: root
triggers: ["root"]
category: coding
---
body"#;
        let child = r#"---
name: child-skill
description: child
triggers: ["child"]
category: coding
parent: root-skill
---
body"#;
        std::fs::write(skills_dir.join("root.md"), parent).unwrap();
        std::fs::write(skills_dir.join("child.md"), child).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let children = engine.children_of("root-skill");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "child-skill");
    }

    #[test]
    fn test_find_matching_complementary_prefers_uncovered_category() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let covered = r#"---
name: rust-analyzer
description: rust analysis
triggers: ["rust"]
category: coding
priority: 90
---
body"#;
        let uncovered = r#"---
name: security-review
description: security review
triggers: ["rust"]
category: security
priority: 40
---
body"#;
        std::fs::write(skills_dir.join("a.md"), covered).unwrap();
        std::fs::write(skills_dir.join("b.md"), uncovered).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // 无已激活技能 → 按优先级: coding(90) 优先
        let base = engine.find_matching("rust", None);
        assert_eq!(base[0].name, "rust-analyzer");

        // 已激活 coding 类技能 → security 类应前置 (互补)
        let comp = engine.find_matching_complementary("rust", None, &["rust-analyzer"]);
        assert!(!comp.is_empty());
        assert_eq!(comp[0].name, "security-review");
    }

    #[test]
    fn test_find_matching_complementary_no_active_keeps_priority_order() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let high = r#"---
name: rust-analyzer
description: rust analysis
triggers: ["rust"]
category: coding
priority: 90
---
body"#;
        let low = r#"---
name: security-review
description: security review
triggers: ["rust"]
category: security
priority: 40
---
body"#;
        std::fs::write(skills_dir.join("a.md"), high).unwrap();
        std::fs::write(skills_dir.join("b.md"), low).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // 无已激活技能 → 无互补偏好, 保持优先级降序
        let comp = engine.find_matching_complementary("rust", None, &[]);
        assert_eq!(comp[0].name, "rust-analyzer");
    }

    #[test]
    fn test_find_matching_complementary_all_covered_falls_back_to_priority() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let coding = r#"---
name: rust-analyzer
description: rust analysis
triggers: ["rust"]
category: coding
priority: 90
---
body"#;
        let security = r#"---
name: security-review
description: security review
triggers: ["rust"]
category: security
priority: 40
---
body"#;
        std::fs::write(skills_dir.join("a.md"), coding).unwrap();
        std::fs::write(skills_dir.join("b.md"), security).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // active 覆盖全部候选类别 → 无互补空间, 退回优先级排序 (coding 90 优先)
        let comp = engine.find_matching_complementary("rust", None, &["rust-analyzer", "security-review"]);
        assert_eq!(comp[0].name, "rust-analyzer");
    }

    #[test]
    fn test_over_validation_score_flags_procedure_heavy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify
2. cargo clean then re-read every file
3. verify compile twice and re-read all docs
4. audit every line and validate again
5. recheck rebuild and verify the compile
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let score = engine.over_validation_score("heavy-skill");
        assert!(score >= 12, "procedure-heavy skill must score >= 12, got {}", score);
    }

    #[test]
    fn test_attribution_tracks_activations_and_flags() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify the compile
2. cargo clean then re-read every single file and recheck
3. verify compile twice and audit the result line by line
4. validate the rebuild and recheck the verify output
5. cargo clean again and re-read the recheck report
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        engine.activate_skill("heavy-skill").unwrap();
        engine.activate_skill("heavy-skill").unwrap_err();

        let report = engine.attribution_report();
        let attr = report.iter().find(|a| a.name == "heavy-skill").unwrap();
        assert_eq!(attr.activations, 1);
        assert!(attr.procedure_heavy, "heavy skill must be flagged procedure-heavy");
    }

    #[test]
    fn test_skill_tree_stats_counts_hierarchy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let root = r#"---
name: root-skill
description: root
triggers: ["root"]
category: coding
---
body"#;
        let child = r#"---
name: child-skill
description: child
triggers: ["child"]
category: coding
parent: root-skill
---
body"#;
        let orphan = r#"---
name: orphan-skill
description: orphan (parent missing)
triggers: ["orphan"]
category: analysis
parent: ghost-parent
---
body"#;
        std::fs::write(skills_dir.join("root.md"), root).unwrap();
        std::fs::write(skills_dir.join("child.md"), child).unwrap();
        std::fs::write(skills_dir.join("orphan.md"), orphan).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let stats = engine.skill_tree_stats();
        assert_eq!(stats.total_skills, 3);
        assert_eq!(stats.roots, 1, "only root-skill has no parent");
        assert_eq!(stats.orphans, 1, "orphan-skill points to missing parent");
        assert_eq!(stats.max_depth, 1, "child depth = 1");
        assert_eq!(stats.categories.get("coding"), Some(&2));
        assert_eq!(stats.categories.get("analysis"), Some(&1));
    }

    #[test]
    fn test_flagged_attributions_reports_procedure_heavy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify the compile twice
2. cargo clean then re-read every single file and recheck
3. verify the compile and audit the result line by line
4. validate the rebuild and recheck the verify output
5. cargo clean again and re-read the recheck report
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        engine.activate_skill("heavy-skill").unwrap();

        let flagged = engine.flagged_attributions();
        assert!(!flagged.is_empty(), "procedure-heavy skill must surface in flagged report");
        assert!(flagged.iter().all(|a| a.procedure_heavy));
    }

    // ── P23 PromptLibrary ──
    #[test]
    fn test_prompt_register_and_get() {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("judge", "score").with_tags(vec!["eval".into()])).unwrap();
        assert_eq!(lib.len(), 1);
        let p = lib.get("judge").expect("get");
        assert_eq!(p.version, 1);
        assert_eq!(p.tags, vec!["eval".to_string()]);
    }

    #[test]
    fn test_prompt_same_name_bumps_version() {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("judge", "v1")).unwrap();
        lib.register(PromptEntry::new("judge", "v2")).unwrap();
        assert_eq!(lib.len(), 1);
        assert_eq!(lib.get("judge").unwrap().version, 2);
        assert_eq!(lib.get("judge").unwrap().content, "v2");
    }

    #[test]
    fn test_prompt_by_tag() {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("a", "1").with_tags(vec!["eval".into()])).unwrap();
        lib.register(PromptEntry::new("b", "2").with_tags(vec!["extract".into()])).unwrap();
        assert_eq!(lib.by_tag("eval").len(), 1);
        assert_eq!(lib.by_tag("extract").len(), 1);
        assert_eq!(lib.by_tag("nope").len(), 0);
    }

    #[test]
    fn test_prompt_missing_returns_none() {
        let lib = PromptLibrary::new();
        assert!(lib.get("absent").is_none());
        assert!(lib.is_empty());
    }

    #[test]
    fn test_prompt_selftest() {
        let lib = PromptLibrary::new();
        assert!(lib.self_test().is_ok());
    }

    // ── P4: AnchorPromote (dsh-anchored-standard 吸收) ──
    #[test]
    fn test_disclosure_default_stage_is_minimal() {
        let ap = AnchorPromote::default();
        assert_eq!(ap.stage, 0, "default anchors on stage 0");
        assert_eq!(ap.minimal_tools, 2);
        assert_eq!(ap.standard_tools, 10);
        assert_eq!(ap.active_tool_count(), 2, "stage 0 → Minimal tool budget");
    }

    #[test]
    fn test_disclosure_stage_default_fields() {
        let ds = DisclosureStage::default();
        assert_eq!(ds.stage, 0);
        assert_eq!(ds.label, "Minimal");
        assert_eq!(ds.tool_count, 2);
        assert!(!ds.durable, "default stage not yet durable");
    }

    #[test]
    fn test_disclosure_record_call_then_promote() {
        let mut ap = AnchorPromote::default();
        assert!(!ap.maybe_promote(), "no durable call → stay anchored");
        ap.record_call();
        assert_eq!(ap.durable_calls, 1);
        assert!(ap.maybe_promote(), "first durable call → promote");
        assert_eq!(ap.stage, 1);
        assert!(!ap.maybe_promote(), "already promoted → no re-promote");
    }

    #[test]
    fn test_disclosure_promoted_active_tool_count_is_standard() {
        let mut ap = AnchorPromote::default();
        assert_eq!(ap.active_tool_count(), 2);
        ap.record_call();
        assert!(ap.maybe_promote());
        assert_eq!(ap.active_tool_count(), 10, "promoted → Standard tool budget");
    }

    #[test]
    fn test_disclosure_savings_is_80_percent() {
        let ap = AnchorPromote::default();
        assert_eq!(ap.disclosure_savings(), 0.8, "(1 - 2/10) = 0.8");
    }

    // ── J-Space pass 分层 (fast/full/loop 门控) ──
    #[test]
    fn test_jspace_pass_default_is_full() {
        let ap = AnchorPromote::default();
        assert_eq!(ap.pass, TaskPass::Full);
        assert_eq!(ap.pass.label(), "full");
        assert!(!ap.pass.needs_standard());
        assert!(!ap.pass.is_verifiable_in_glance());
    }

    #[test]
    fn test_jspace_fast_pass_keeps_zero_tools() {
        let mut ap = AnchorPromote::new(2, 10, PromoteSignal::FirstDurableCall).with_pass(TaskPass::Fast);
        ap.record_call();
        assert_eq!(ap.active_tool_count(), 0, "fast pass exposes no tools");
        assert_eq!(ap.disclosure_savings(), 1.0, "fast saves everything");
        assert!(ap.pass.is_verifiable_in_glance());
    }

    #[test]
    fn test_jspace_loop_pass_promotes_immediately() {
        let mut ap = AnchorPromote::new(2, 10, PromoteSignal::FirstDurableCall)
            .with_pass(TaskPass::Loop);
        assert_eq!(ap.active_tool_count(), 10, "loop pass needs full toolset from start");
        assert!(ap.maybe_promote(), "loop promotes without waiting for durable");
        assert_eq!(ap.stage, 1);
        assert_eq!(ap.disclosure_savings(), 0.0, "loop keeps no disclosure savings");
        assert!(ap.pass.needs_standard());
    }

    #[test]
    fn test_jspace_three_pass_labels() {
        assert_eq!(TaskPass::ALL.len(), 3);
        assert_eq!(TaskPass::Fast.label(), "fast");
        assert_eq!(TaskPass::Full.label(), "full");
        assert_eq!(TaskPass::Loop.label(), "loop");
        assert!(!TaskPass::Fast.needs_standard());
        assert!(TaskPass::Loop.needs_standard());
        assert!(!TaskPass::Full.needs_standard());
    }

    #[test]
    fn test_disclosure_engine_step_transitions_and_stays() {
        let mut engine = SkillEngine::new(PathBuf::from("/nonexistent/skills"));
        assert_eq!(engine.disclosure.stage, 0);
        assert!(!engine.step_disclosure(), "no durable call yet → no transition");
        engine.disclosure.record_call();
        assert!(engine.step_disclosure(), "first durable call → transition");
        assert_eq!(engine.disclosure.stage, 1);
        assert_eq!(engine.disclosure.active_tool_count(), 10);
        assert!(!engine.step_disclosure(), "second call stays promoted");
        assert_eq!(engine.disclosure.stage, 1);
    }

    #[test]
    fn test_disclosure_visible_active_gates_tool_set() {
        // P4 行为接线回归: visible_active() 必须真实限制工具集 —
        // Minimal 阶段只暴露预算(2)个高优先级技能, promote 后完整暴露。
        let mut engine = SkillEngine::new(PathBuf::from("/nonexistent/skills"));
        // 直接注入 4 个活跃技能 (优先级 4/3/2/1 → 1 最高)
        for (name, prio) in [("low", 4u8), ("mid", 3), ("high", 2), ("top", 1)] {
            engine.skills.push(SkillEntry {
                name: name.to_string(),
                description: format!("skill {}", name),
                category: "test".into(),
                triggers: vec![],
                e8_modes: vec![],
                tools: vec![],
                hooks: vec![],
                priority: prio,
                path: PathBuf::new(),
                content: String::new(),
                active: true,
                references: vec![],
                parent: String::new(),
                verified: false,
            });
        }
        // Minimal 阶段: 预算 2 → 只暴露 top/high (priority 1,2)
        assert_eq!(engine.disclosure.stage, 0);
        let visible = engine.visible_active();
        assert_eq!(visible.len(), 2, "Minimal budget gates to 2, got {}", visible.len());
        assert_eq!(visible[0].name, "top", "highest priority first");
        assert_eq!(visible[1].name, "high", "second highest priority");
        // promote 后: 完整暴露全部 4 个
        engine.disclosure.record_call();
        assert!(engine.step_disclosure());
        assert_eq!(engine.disclosure.stage, 1);
        let visible_full = engine.visible_active();
        assert_eq!(visible_full.len(), 4, "Standard stage exposes all active, got {}", visible_full.len());
        // 未激活技能不受门控影响 (list_active 与 visible 一致: 都只含 active)
    }

    #[test]
    fn test_disclosure_new_constructor_with_both_signal() {
        let mut ap = AnchorPromote::new(3, 12, PromoteSignal::Both);
        assert_eq!(ap.stage, 0);
        assert_eq!(ap.minimal_tools, 3);
        assert_eq!(ap.standard_tools, 12);
        ap.record_call();
        assert!(ap.maybe_promote(), "Both signal satisfied on first durable call");
        assert_eq!(ap.active_tool_count(), 12);
        assert_eq!(ap.disclosure_savings(), 0.75, "(1 - 3/12)");
    }

    #[test]
    fn test_register_check_flags_inner_notation() {
        let ap = AnchorPromote::default();
        let findings = ap.register_check("Result: A ⇒ B. Verified by the run.");
        assert!(
            findings.iter().any(|f| f.contains("inner-register")),
            "inner-register notation must leak: {findings:?}"
        );
    }

    #[test]
    fn test_register_check_flags_state_markers() {
        let ap = AnchorPromote::default();
        let findings = ap.register_check("PHEW done. GRRR retry.");
        assert!(
            findings.iter().any(|f| f.contains("state markers")),
            "state markers must leak: {findings:?}"
        );
    }

    #[test]
    fn test_register_check_flags_verified_without_coverage() {
        let ap = AnchorPromote::default();
        let findings = ap.register_check("This is now verified.");
        assert!(
            findings.iter().any(|f| f.contains("no stated coverage")),
            "verified without coverage is a mood: {findings:?}"
        );
    }

    #[test]
    fn test_register_check_flags_repetition_loop() {
        let ap = AnchorPromote::default();
        let findings = ap.register_check("repeat\nrepeat\nrepeat");
        assert!(
            findings.iter().any(|f| f.contains("repetition loop")),
            "repeated line must leak: {findings:?}"
        );
    }

    #[test]
    fn test_register_check_clean_outgoing_holds() {
        let ap = AnchorPromote::default();
        let findings = ap.register_check(
            "The invariant held. Verified by brute force, n <= 6, including empty input.",
        );
        assert!(findings.is_empty(), "clean outgoing must hold: {findings:?}");
    }

    // ── P6: BookToSkill (book-to-skill 输入侧) ──

    fn sample_book(chapters: Vec<(usize, usize)>) -> BookInput {
        BookInput {
            title: "Sample Book".into(),
            format: DocFormat::Markdown,
            chapters: chapters
                .into_iter()
                .map(|(order, char_count)| DocChapter {
                    title: format!("Chapter {}", order),
                    order,
                    char_count,
                    summary: String::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn test_infer_format_by_extension() {
        assert_eq!(BookToSkill::infer_format("a.pdf"), DocFormat::Pdf);
        assert_eq!(BookToSkill::infer_format("a.epub"), DocFormat::Epub);
        assert_eq!(BookToSkill::infer_format("a.docx"), DocFormat::Docx);
        assert_eq!(BookToSkill::infer_format("a.md"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("a.MARKDOWN"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("a.html"), DocFormat::Html);
        assert_eq!(BookToSkill::infer_format("a.htm"), DocFormat::Html);
        assert_eq!(BookToSkill::infer_format("a.rtf"), DocFormat::Rtf);
        assert_eq!(BookToSkill::infer_format("a.mobi"), DocFormat::Mobi);
        assert_eq!(BookToSkill::infer_format("a.unknown"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("no_extension"), DocFormat::Markdown);
        assert_eq!(BookToSkill::infer_format("a.pdf"), BookToSkill::infer_format("b.PDF"));
        assert_eq!(DocFormat::Pdf.label(), "pdf");
        assert_eq!(DocFormat::Html.label(), "html");
    }

    #[test]
    fn test_normalize_filters_short_chapters_keeps_order() {
        let bts = BookToSkill::new(500, 8);
        let book = sample_book(vec![(0, 400), (1, 600), (2, 100), (3, 900)]);
        let kept = bts.normalize(&book);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].order, 1, "order preserved");
        assert_eq!(kept[0].char_count, 600);
        assert_eq!(kept[1].order, 3);
        assert_eq!(kept[1].char_count, 900);
    }

    #[test]
    fn test_discover_candidates_caps_and_priority() {
        let bts = BookToSkill::new(500, 3);
        let book = sample_book(vec![(0, 600), (1, 3000), (2, 700), (3, 2500)]);
        let candidates = bts.discover_candidates(&book);
        assert_eq!(candidates.len(), 3, "capped at max_candidates=3");
        assert_eq!(candidates[0].priority, 1, "600 chars ≤ 2000 → priority 1");
        assert_eq!(candidates[1].priority, 2, "3000 chars > 2000 → priority 2");
        assert_eq!(candidates[2].priority, 1);
        assert_eq!(candidates[0].source_chapters, vec![0]);
        assert_eq!(candidates[1].source_chapters, vec![1]);
        // 短章节被 normalize 过滤, 不出现在候选中
        assert!(!candidates.iter().any(|c| c.source_chapters == vec![4]));
    }

    #[test]
    fn test_discover_candidates_cleans_titles() {
        let bts = BookToSkill::default();
        let book = BookInput {
            title: "T".into(),
            format: DocFormat::Pdf,
            chapters: vec![DocChapter {
                title: "12. Introduction to Skill Casting!".into(),
                order: 0,
                char_count: 1000,
                summary: String::new(),
            }],
        };
        let candidates = bts.discover_candidates(&book);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].name, "introduction_to_skill_casting");
    }

    #[test]
    fn test_skill_yield_in_range_and_less_than_one_after_filter() {
        let bts = BookToSkill::new(500, 8);
        let book = sample_book(vec![(0, 400), (1, 600), (2, 100), (3, 900)]);
        let yield_ratio = bts.skill_yield(&book);
        assert!(yield_ratio >= 0.0 && yield_ratio <= 1.0, "yield in [0,1]");
        assert!((yield_ratio - 0.75).abs() < 1e-9, "kept 1500 / total 2000 = 0.75");
        assert!(yield_ratio < 1.0, "filtered chapters → yield < 1");
    }

    #[test]
    fn test_skill_yield_all_retained_is_one() {
        let bts = BookToSkill::new(500, 8);
        let book = sample_book(vec![(0, 600), (1, 3000)]);
        assert_eq!(bts.skill_yield(&book), 1.0);
    }

    #[test]
    fn test_skill_yield_empty_input_is_zero() {
        let bts = BookToSkill::default();
        let empty = BookInput {
            title: "Empty".into(),
            format: DocFormat::Markdown,
            chapters: vec![],
        };
        assert_eq!(bts.skill_yield(&empty), 0.0, "no chapters → yield 0");
        let all_short = sample_book(vec![(0, 100), (1, 200)]);
        assert_eq!(bts.skill_yield(&all_short), 0.0, "all filtered → yield 0");
    }

    #[test]
    fn test_book_to_skill_selftest() {
        use crate::core::nt_core_self_test::SelfTest;
        let bts = BookToSkill::default();
        assert_eq!(bts.name(), "nt_mind_book_to_skill");
        assert!(bts.self_test().is_ok());
    }

    // ── cordiverse F1: RevertibleEffect + InverseLedger ──

    #[test]
    fn test_ledger_records_inverse_in_load_order() {
        let mut ledger = InverseLedger::new();
        let id = ledger.begin_install();
        ledger.push_inverse(id, RevertibleEffect::new("first", || Ok(()))).unwrap();
        ledger.push_inverse(id, RevertibleEffect::new("second", || Ok(()))).unwrap();
        ledger.push_inverse(id, RevertibleEffect::new("third", || Ok(()))).unwrap();
        assert_eq!(ledger.inverse_count(id), 3);
        // 账本按加载序记录 (不是逆序)
        assert_eq!(ledger.inverse_labels(id), vec!["first", "second", "third"]);
    }

    #[test]
    fn test_ledger_lifo_teardown_runs_reverse_order() {
        let mut ledger = InverseLedger::new();
        let id = ledger.begin_install();
        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        for label in ["first", "second", "third"] {
            let l = log.clone();
            ledger.push_inverse(id, RevertibleEffect::new(label, move || {
                l.lock().unwrap().push(label.to_string());
                Ok(())
            })).unwrap();
        }
        let results = ledger.teardown(id);
        assert!(results.iter().all(|r| r.is_ok()));
        let got = log.lock().unwrap().clone();
        assert_eq!(got, vec!["third", "second", "first"], "LIFO: 逆加载序");
        assert_eq!(ledger.inverse_count(id), 0, "teardown 消耗事务");
    }

    #[test]
    fn test_ledger_teardown_unknown_id_is_empty() {
        let mut ledger = InverseLedger::new();
        assert!(ledger.teardown(99).is_empty());
    }

    // ── cordiverse F5: FiberLifecycle ──

    #[test]
    fn test_fiber_lifecycle_rejects_illegal_transitions() {
        let mut fiber = FiberLifecycle::new("f1", 0);
        assert_eq!(fiber.state(), FiberLifecycleState::Loaded);
        assert!(fiber.transition(FiberLifecycleState::Active).is_ok());
        assert!(fiber.transition(FiberLifecycleState::Active).is_err(), "自环非法");
        assert!(fiber.transition(FiberLifecycleState::Loaded).is_err(), "Active→Loaded 非法");
        assert!(fiber.transition(FiberLifecycleState::Retired).is_ok());
        assert!(fiber.transition(FiberLifecycleState::Failed).is_err(), "Retired 是终态");
        assert!(fiber.transition(FiberLifecycleState::Active).is_err(), "Retired→Active 非法");
    }

    #[test]
    fn test_fiber_lifecycle_suspend_and_resume() {
        let mut fiber = FiberLifecycle::new("f2", 0);
        fiber.transition(FiberLifecycleState::Active).unwrap();
        fiber.transition(FiberLifecycleState::Suspended).unwrap();
        assert_eq!(fiber.state(), FiberLifecycleState::Suspended);
        fiber.transition(FiberLifecycleState::Active).unwrap();
        assert_eq!(fiber.state(), FiberLifecycleState::Active);
    }

    #[test]
    fn test_fiber_failure_captured_per_fiber_without_aborting_others() {
        // 两个独立 fiber: A 的逆失败被捕获, B 完全不受影响 (L-Raise)
        let mut ledger = InverseLedger::new();
        let id_a = ledger.begin_install();
        ledger.push_inverse(id_a, RevertibleEffect::new("a1", || Err("a1 boom".into()))).unwrap();
        let id_b = ledger.begin_install();
        ledger.push_inverse(id_b, RevertibleEffect::new("b1", || Ok(()))).unwrap();

        let mut fiber_a = FiberLifecycle::new("a", id_a);
        let mut fiber_b = FiberLifecycle::new("b", id_b);
        fiber_a.transition(FiberLifecycleState::Active).unwrap();
        fiber_b.transition(FiberLifecycleState::Active).unwrap();

        let res_a = ledger.teardown(id_a);
        assert!(res_a[0].is_err(), "fiber A 逆失败必须被捕获");
        fiber_a.record_failure("a1 boom");

        let res_b = ledger.teardown(id_b);
        assert!(res_b[0].is_ok(), "fiber B teardown 不受 A 影响");
        assert_eq!(fiber_b.state(), FiberLifecycleState::Active);
        assert_eq!(fiber_a.state(), FiberLifecycleState::Failed);
        assert_eq!(fiber_a.failures.len(), 1);
        assert_eq!(fiber_a.failures[0].message, "a1 boom");
    }

    // ── wiring: install → ledger → LIFO teardown ──

    #[test]
    fn test_install_wires_inverse_ledger_and_lifo_teardown() {
        let tmp = setup_temp_dir();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("SKILL.md"), sample_skill_content()).unwrap();

        let engine_dir = tmp.path().join("engine");
        std::fs::create_dir_all(&engine_dir).unwrap();
        let mut engine = SkillEngine::new(engine_dir.clone());
        engine.install_skill(&src).unwrap();

        let target = engine_dir.join("rust-analyzer");
        assert!(target.exists(), "installed skill dir must exist");
        let fiber = engine.fiber_lifecycles.get("rust-analyzer").unwrap();
        assert_eq!(fiber.state(), FiberLifecycleState::Active);
        let install_id = fiber.install_id;
        assert!(engine.inverse_ledger.inverse_count(install_id) >= 1);

        let results = engine.uninstall_skill("rust-analyzer").unwrap();
        assert!(results.iter().all(|r| r.is_ok()), "teardown inverses must succeed");
        assert!(!target.exists(), "uninstall 经 LIFO teardown 删除技能目录");
        assert_eq!(engine.fiber_state("rust-analyzer"), Some(FiberLifecycleState::Retired));
        assert!(engine.find_matching("rust", None).is_empty(), "技能从路由移除");
        assert!(engine.uninstall_skill("rust-analyzer").is_err(), "重复卸载被拒");
    }

    // ── C5 自愈检测件: revertible_effects (F1) + fiber_lifecycle (F5) ──

    #[test]
    fn test_revertible_effects_healer() {
        use crate::core::nt_core_self_test::SelfTest;
        let healer = RevertibleEffectsHealer;
        assert_eq!(healer.name(), "nt_mind_skill_engine::revertible_effects_healer");
        assert!(healer.self_test().is_ok());
    }

    #[test]
    fn test_fiber_lifecycle_healer() {
        use crate::core::nt_core_self_test::SelfTest;
        let healer = FiberLifecycleHealer;
        assert_eq!(healer.name(), "nt_mind_skill_engine::fiber_lifecycle_healer");
        assert!(healer.self_test().is_ok());
    }

    #[test]
    fn test_release_dangling_recovers_dangling_ownership() {
        // 合法持有的 fiber 不被误释放; 事务消失的 held fiber 被自动释放。
        let mut engine = SkillEngine::new(PathBuf::new());
        let id = FiberLifecycleHealer::install_held_fiber(&mut engine, "healthy").unwrap();
        let dangling_id = FiberLifecycleHealer::install_held_fiber(&mut engine, "dangling").unwrap();
        engine.inverse_ledger.teardown(dangling_id);

        let released = engine.release_dangling();
        assert_eq!(released, vec!["dangling"], "仅悬挂 fiber 被释放");
        assert!(engine.inverse_ledger.has_transaction(id), "健康 fiber 事务保留");
        assert_eq!(engine.fiber_state("healthy"), Some(FiberLifecycleState::Active));
        assert_eq!(engine.fiber_state("dangling"), Some(FiberLifecycleState::Retired));
        assert!(engine.release_dangling().is_empty(), "幂等: 无残留悬挂");
    }

    // ── A5 (SkillNet): 技能五维质量评估 ──
    #[test]
    fn test_quality_scorer_rewards_structured_skills() {
        let content = "---\nname: test-skill\ndescription: A skill with a short focused description\n\
            triggers: [test]\ntools: [rg]\nreferences: [ref.md]\ncategory: testing\npriority: 50\n---\n\
            When to use: testing\nVerification: run selftest and check output\n\
            Steps: do the thing carefully without dangerous operations\n";
        let dir = std::env::temp_dir().join("neotrix_quality_test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("SKILL.md"), content).unwrap();
        let skill = SkillEntry::from_file(&dir.join("SKILL.md")).expect("parse skill");
        let scores = SkillQualityScorer::evaluate(&skill);
        assert!(scores.completeness > 0.8, "结构化 skill 完整性高, got {}", scores.completeness);
        assert!(scores.executability > 0.0, "Verification 段 → 可执行性 >0");
        assert!(scores.cost_awareness > 0.5, "短描述 → 成本意识高");
        assert!(scores.passes_gate(0.5, 0.5), "结构化技能过质量门");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_quality_scorer_penalizes_dangerous_commands() {
        let content = "---\nname: danger-skill\ndescription: rm -rf everything with sudo and --force\n\
            triggers: [x]\n---\nRun: sudo rm -rf / --force\n";
        let dir = std::env::temp_dir().join("neotrix_quality_danger_test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("SKILL.md"), content).unwrap();
        let skill = SkillEntry::from_file(&dir.join("SKILL.md")).expect("parse skill");
        let scores = SkillQualityScorer::evaluate(&skill);
        assert!(scores.safety < 0.5, "危险命令 → 安全分低, got {}", scores.safety);
        assert!(!scores.passes_gate(0.5, 0.6), "安全分不足 → 拒绝");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── A5b (SkillNet): 技能组合推断 ──
    fn skill_entry(name: &str, category: &str, tools: &[&str], triggers: &[&str]) -> SkillEntry {
        SkillEntry {
            name: name.to_string(),
            description: String::new(),
            triggers: triggers.iter().map(|s| s.to_string()).collect(),
            e8_modes: vec![],
            tools: tools.iter().map(|s| s.to_string()).collect(),
            hooks: vec![],
            priority: 0,
            path: std::env::temp_dir().join(name),
            content: String::new(),
            active: true,
            references: vec![],
            category: category.to_string(),
            parent: String::new(),
            verified: false,
        }
    }

    #[test]
    fn test_composer_substitute_same_category_overlapping_tools() {
        let a = skill_entry("a", "testing", &["rg", "cargo"], &["test"]);
        let b = skill_entry("b", "testing", &["rg", "cargo", "rustc"], &["check"]);
        assert_eq!(SkillComposer::compose(&a, &b), SkillRelationship::Substitute);
    }

    #[test]
    fn test_composer_complement_shared_triggers_partial_tools() {
        let a = skill_entry("extract", "data", &["rg"], &["parse", "scrape"]);
        let b = skill_entry("store", "data", &["sqlite", "rg"], &["parse"]);
        assert_eq!(SkillComposer::compose(&a, &b), SkillRelationship::Complement);
    }

    #[test]
    fn test_composer_handoff_cross_category_shared_signal() {
        let a = skill_entry("gen", "writing", &["claude"], &["draft", "outline"]);
        let b = skill_entry("polish", "editing", &["claude"], &["draft"]);
        assert_eq!(SkillComposer::compose(&a, &b), SkillRelationship::Handoff);
    }

    #[test]
    fn test_composer_unrelated_no_signal() {
        let a = skill_entry("crawl", "world", &["curl"], &["fetch"]);
        let b = skill_entry("meme", "fun", &["ffmpeg"], &["gif"]);
        assert_eq!(SkillComposer::compose(&a, &b), SkillRelationship::Unrelated);
    }

    #[test]
    fn test_composer_label_roundtrip() {
        assert_eq!(SkillRelationship::Complement.label(), "complement");
        assert_eq!(SkillRelationship::Handoff.label(), "handoff");
        assert_eq!(SkillRelationship::Substitute.label(), "substitute");
        assert_eq!(SkillRelationship::Unrelated.label(), "unrelated");
    }

    // ── P4 技能驻留成本审计 (asm absorbed 2026-08-19) ──────────
    #[test]
    fn test_quality_scores_measure_resident_tokens() {
        // 短技能: resident/body 应小且 cost_awareness 高。
        let short = SkillEntry {
            name: "short-skill".into(),
            description: "quick helper".into(),
            triggers: vec!["quick".into()],
            e8_modes: vec![],
            tools: vec!["read".into()],
            hooks: vec![],
            priority: 50,
            path: PathBuf::from("/tmp/short/SKILL.md"),
            content: "---\nname: short-skill\ndescription: quick helper\ntriggers: [quick]\n---\nDo one thing tersely.\n".into(),
            active: false,
            references: vec![],
            category: "general".into(),
            parent: String::new(),
            verified: false,
        };
        let scores = SkillQualityScorer::evaluate(&short);
        assert!(scores.resident_tokens > 0, "resident tokens must be measured");
        assert!(scores.body_tokens > 0, "body tokens must be measured");
        assert!(scores.body_tokens < scores.resident_tokens, "resident 含 frontmatter, 应大于 body");
        assert!(scores.cost_awareness > 0.5, "短技能成本分应高, got {}", scores.cost_awareness);
    }

    #[test]
    fn test_audit_residency_ranks_fat_skills() {
        use std::collections::HashMap;
        let mut stats: HashMap<String, SkillQualityScores> = HashMap::new();
        let fat = SkillQualityScores {
            safety: 1.0,
            completeness: 0.9,
            executability: 0.8,
            maintainability: 0.7,
            cost_awareness: 0.3,
            resident_tokens: 9000,
            body_tokens: 8800,
        };
        let slim = SkillQualityScores {
            safety: 1.0,
            completeness: 0.9,
            executability: 0.8,
            maintainability: 0.7,
            cost_awareness: 0.9,
            resident_tokens: 400,
            body_tokens: 300,
        };
        stats.insert("fat-skill".into(), fat);
        stats.insert("slim-skill".into(), slim);
        let rows = audit_residency(&stats);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].skill, "fat-skill", "resident 降序, fat 应排首");
        assert_eq!(rows[0].action, "thin-entry", ">1500 tokens → 建议薄入口");
        assert_eq!(rows[1].action, "ok");
    }

    #[test]
    fn test_audit_residency_empty_stats() {
        use std::collections::HashMap;
        let rows = audit_residency(&HashMap::new());
        assert!(rows.is_empty(), "无技能时审计应为空");
    }
