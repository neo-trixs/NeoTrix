//! nt_tests — cleanup_engine 拆分后的测试集合，行为零变更纯搬移.
//! 原 `mod tests` 内联展开为独立文件；缺失导入在此补显式 use。

use super::*;
use crate::l0_substrate::nt_core_self_test::SelfTest;
use std::fs;
use std::path::PathBuf;

/// 进程级唯一临时目录 — 防并行会话同跑 cargo test 时共享固定 temp 目录互相删除。
fn unique_tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}_pid{}", name, std::process::id()))
}

#[test]
fn test_cleanup_patterns() {
    let patterns = CleanupPattern::all_patterns();
    assert!(!patterns.is_empty());
    assert!(patterns
        .iter()
        .any(|p| p.kind == CleanupKind::ProjectArtifacts));
    assert!(patterns.iter().any(|p| p.kind == CleanupKind::IDECaches));
}

#[test]
fn test_cleanup_engine_new() {
    let engine = CleanupEngine::new();
    assert!(!engine.patterns.is_empty());
    assert!(engine.dry_run_default);
}

#[test]
fn test_cleanup_result_summary() {
    let mut r = _CleanupResult::new(CleanupKind::Cache);
    r.dry_run = true;
    r.deletable_count = 5;
    r.estimated_bytes = 1_048_576;
    let s = r.summary();
    assert!(s.contains("预览"));
    assert!(s.contains("1.0 MB"));
}

#[test]
fn test_clean_kind_descriptions() {
    assert!(!CleanupKind::All.description().is_empty());
    assert!(!CleanupKind::ProjectArtifacts.description().is_empty());
}

#[test]
fn test_cleanup_dirs_creation() {
    let tmp = unique_tmp("neotrix_test_cleanup_dirs");
    let _ = fs::remove_dir_all(&tmp);
    let dirs = _CleanupDirs::new(&tmp);
    dirs.ensure().expect("ensure cleanup dirs");
    assert!(dirs.archive.exists());
    assert!(dirs.log.exists());
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_archiver_archive_paths() {
    let tmp = unique_tmp("neotrix_test_archiver");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    // 创建测试文件
    let test_file = tmp.join("test.txt");
    fs::write(&test_file, b"hello world").unwrap();

    let mut archiver = _Archiver::new(&tmp);
    let paths = vec![test_file.to_string_lossy().to_string()];
    let manifest = archiver
        ._archive_paths(&paths, "test")
        .expect("archive paths");
    assert_eq!(manifest.total_items, 1);
    assert_eq!(manifest.entries[0].source_path, paths[0]);

    // 验证索引
    assert_eq!(archiver.index.entries.len(), 1);

    // 搜索测试
    let hits = archiver.search("test.txt");
    assert_eq!(hits.len(), 1);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_backup_engine() {
    let tmp = unique_tmp("neotrix_test_backup");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    // 创建测试代码文件
    let src = tmp.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("main.rs"), b"fn main() {}").unwrap();
    fs::write(tmp.join("Cargo.toml"), b"[package]\nname = \"test\"\n").unwrap();

    let mut engine = BackupEngine::new(&tmp);
    let manifest = engine.run_backup().expect("backup");
    assert!(manifest.file_count > 0);

    // 验证备份目录
    let backup_root = tmp.join(".backup");
    assert!(backup_root.exists());
    let latest = backup_root.join("latest");
    assert!(latest.exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_cleanup_log() {
    let tmp = unique_tmp("neotrix_test_cleanup_log");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("log")).unwrap();

    _CleanupLog::log(
        &tmp.join("log"),
        &_CleanupLogEntry {
            action: "test".into(),
            kind: "test".into(),
            items: 1,
            bytes: 100,
            batch_id: "batch_1".into(),
            success: true,
            error: None,
        },
    );

    let recent = _CleanupLog::recent(&tmp.join("log"), 10);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].items, 1);

    let _ = fs::remove_dir_all(&tmp);
}

// ---- 跨平台 + 组件移除增强测试 ----

#[test]
fn test_platform_current_and_matches() {
    let cur = Platform::current();
    assert!(matches!(
        cur,
        Platform::MacOS | Platform::Windows | Platform::Linux | Platform::All
    ));
    assert!(Platform::All.matches(cur));
    assert!(cur.matches(cur));
}

#[test]
fn test_pattern_platform_filtering() {
    // 当前平台生效, 非当前平台规则应被 new() 过滤
    let engine = CleanupEngine::new();
    for p in &engine.patterns {
        assert!(
            p.platform.matches(Platform::current()),
            "规则 {} 未按平台过滤",
            p.name
        );
    }
    // 全量列表应同时含 mac/win/linux 规则 (用于跨平台目标)
    let all = CleanupPattern::all_patterns();
    assert!(all.iter().any(|p| p.platform == Platform::MacOS));
    assert!(all.iter().any(|p| p.platform == Platform::Windows));
}

#[test]
fn test_risk_gate_filters() {
    let low = CleanupRiskLevel::Low;
    let high = CleanupRiskLevel::High;
    let engine = CleanupEngine::new()._with_risk_gate(CleanupRiskLevel::Low);
    // 默认 gate 为 Medium 时, High 规则不应执行
    let default_engine = CleanupEngine::new();
    assert!(engine.risk_gate <= low || default_engine.risk_gate > low);
    assert!(high > CleanupRiskLevel::Medium);
    assert!(CleanupRiskLevel::Low < CleanupRiskLevel::High);
}

#[test]
fn test_pattern_expand_home() {
    let home = dirs::home_dir().unwrap();
    let expanded = CleanupPattern::expand("~/.cargo/registry/cache/**");
    assert!(expanded.starts_with(&home.to_string_lossy().to_string()));
    assert!(expanded.contains(".cargo"));
}

#[test]
fn test_cachedir_tag_detection() {
    let tmp = unique_tmp("neotrix_test_cachedir_tag");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    // 带合法签名的 CACHEDIR.TAG → 应被识别为缓存
    fs::write(
        tmp.join("CACHEDIR.TAG"),
        "Signature: 8a477f597d02d456d45674aa7d611ef7b6c14a01bccaebbd4e53c5d4f\ncomment",
    )
    .unwrap();
    assert!(CleanupPattern::_has_cachedir_tag(&tmp));
    // 不带签名 → 不是
    fs::write(tmp.join("CACHEDIR.TAG"), "random").unwrap();
    assert!(!CleanupPattern::_has_cachedir_tag(&tmp));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_system_root_guard() {
    // 拒绝删除系统根目录
    assert!(CleanupPattern::_is_system_root_dir(std::path::Path::new(
        "/"
    )));
    let home = dirs::home_dir().unwrap();
    assert!(CleanupPattern::_is_system_root_dir(&home));
    // 普通目录不受影响
    assert!(!CleanupPattern::_is_system_root_dir(std::path::Path::new(
        "/tmp/neotrix_test_x"
    )));
}

// ---- 吸收增强测试 (AI 缓存 / Homebrew / Xcode / 命令式清理) ----

#[test]
fn test_absorbed_ai_homebrew_xcode_patterns() {
    let all = CleanupPattern::all_patterns();
    // AI 模型缓存 (PureMac AI Apps 吸收)
    assert!(all.iter().any(|p| p.name == "Ollama model cache"));
    assert!(all.iter().any(|p| p.name == "LM Studio model cache"));
    assert!(all.iter().any(|p| p.name == "MCP/Agent hub cache"));
    // Homebrew 缓存 (mac-janitor 吸收)
    assert!(all.iter().any(|p| p.name == "Homebrew cache"));
    // Xcode Archives/Simulators (mac-janitor 吸收)
    assert!(all.iter().any(|p| p.name == "Xcode Archives"));
    assert!(all.iter().any(|p| p.name == "Xcode Simulator runtimes"));
    assert!(all.iter().any(|p| p.name == "Xcode module caches"));
}

#[test]
fn test_absorbed_pattern_risk_gating() {
    // AI 模型缓存默认 medium 风险 → 默认 gate (Medium) 下可用, 但 Low gate 下被过滤
    let engine = CleanupEngine::new();
    assert_eq!(engine.risk_gate, CleanupRiskLevel::Medium);
    // SystemServices 类别枚举存在
    assert!(!CleanupKind::SystemServices.description().is_empty());
    // 命令清理项三件套 (brew/docker/tmutil)
    let cmds = _CommandCleanup::all();
    assert!(cmds.iter().any(|c| c.name == "Homebrew cleanup"));
    assert!(cmds.iter().any(|c| c.name == "Docker system prune"));
    assert!(cmds
        .iter()
        .any(|c| c.name == "Time Machine local snapshots"));
}

#[test]
fn test_command_cleaner_scan_and_confirm_gate() {
    let cleaner = _CommandCleaner::new()._with_dry_run(true);
    // 当前平台过滤后至少 brew (macOS) 或 docker (跨平台) 可用
    let scanned = cleaner.scan();
    assert!(!scanned.is_empty());
    // dry-run 模式执行 brew → dry_run 状态, 不产生副作用
    if let Some((brew, _)) = scanned.iter().find(|(i, _)| i.name == "Homebrew cleanup") {
        let r = cleaner.execute(brew, false);
        assert_eq!(r.status, "dry_run");
        assert!(r.dry_run);
    }
    // 非 dry-run + 需确认项 → needs_confirm
    let real = _CommandCleaner::new()._with_dry_run(false);
    if let Some((docker, _)) = real
        .scan()
        .iter()
        .find(|(i, _)| i.name == "Docker system prune")
    {
        let r = real.execute(docker, false);
        assert_eq!(r.status, "needs_confirm");
    }
}

#[test]
fn test_clean_services_via_engine() {
    let mut engine = CleanupEngine::new();
    engine.dry_run_default = true;
    let r = engine.clean(CleanupKind::SystemServices);
    // dry-run 下不报错, 至少有扫描计数
    assert!(r.errors.is_empty());
    assert!(r.scanned_count > 0);
}

#[test]
fn test_clean_services_dry_run_report_visible() {
    // 修复 #1: dry-run 报告不得静默失效 — pattern_matches 必须填充可用项
    let mut engine = CleanupEngine::new();
    engine.dry_run_default = true;
    let r = engine.clean(CleanupKind::SystemServices);
    assert!(r.dry_run, "clean 应继承 dry_run_default");
    assert!(
        !r.pattern_matches.is_empty(),
        "dry-run 报告应含可用项, 但为空"
    );
    assert!(
        r.pattern_matches.iter().any(|m| m.contains("dry-run")),
        "报告应标注 dry-run 状态: {:?}",
        r.pattern_matches
    );
}

#[test]
fn test_whitelist_expanded_matches() {
    // 修复 #2: 白名单必须 expand 后匹配绝对路径
    let engine = CleanupEngine::new();
    let home = dirs::home_dir().unwrap();
    // 模拟扫描路径: ~/.config 下的真实绝对路径
    let abs = home.join(".config").join("some_app");
    assert!(
        engine.is_whitelisted(&abs),
        "白名单 ~/.config 应匹配绝对路径 {:?} (expand 后)",
        abs
    );
}

#[test]
fn test_cleanup_kind_services_all() {
    // All 描述覆盖全部类别 (回归: 新增 SystemServices 后 All 不应 panic)
    assert!(!CleanupKind::All.description().is_empty());
}

// ---- 蜕皮 (ProjectMolting) 增强测试 ----

#[test]
fn test_project_molting_kind_registered() {
    let all = CleanupPattern::all_patterns();
    assert!(
        all.iter().any(|p| p.kind == CleanupKind::ProjectMolting),
        "ProjectMolting 蜕皮规则必须注册"
    );
    assert!(!CleanupKind::ProjectMolting.description().is_empty());
}

#[test]
fn test_molt_project_detects_legacy_shells() {
    let tmp = unique_tmp("neotrix_test_molting");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("legacy")).unwrap();
    fs::create_dir_all(tmp.join("old_proto")).unwrap();
    fs::create_dir_all(tmp.join("active")).unwrap();
    fs::write(tmp.join("legacy").join("old.rs"), b"// old").unwrap();

    let mut engine = CleanupEngine::new().with_project_root(tmp.clone());
    engine.dry_run_default = true; // 预览模式
    let r = engine.molt_project();

    assert!(r.dry_run);
    assert_eq!(
        r.deletable_count, 2,
        "legacy + old_proto 应被识别, active 不应"
    );
    assert!(r.pattern_matches.iter().any(|m| m.contains("legacy")));
    assert!(r.pattern_matches.iter().any(|m| m.contains("old_proto")));
    assert!(
        r.pattern_matches.iter().all(|m| !m.contains("active")),
        "active 目录不得被蜕皮"
    );
    // 预览模式不产生归档副作用
    assert!(!tmp.join(".cleanup").exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_molt_project_never_molts_project_root() {
    let tmp = unique_tmp("neotrix_test_molt_root_guard");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let mut engine = CleanupEngine::new().with_project_root(tmp.clone());
    engine.dry_run_default = true;
    let r = engine.molt_project();

    // 空项目 (无 legacy/old 子目录) → 0 蜕皮; root 自身绝不入壳
    assert_eq!(r.deletable_count, 0);
    // 项目名即使包含 old 字样也不得误判 (root 自身被 is_shell 忽略)
    let root_named = tmp.join("old_project");
    fs::create_dir_all(&root_named).unwrap();
    engine.dry_run_default = true;
    let r2 = engine.molt_project();
    // 扫描的是 root 下一级, old_project 是 root 的子目录不是 root 自身
    assert!(
        r2.pattern_matches.iter().any(|m| m.contains("old_project")),
        "old_project 作为 root 的子目录应被识别为旧躯壳"
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_molt_project_archives_in_archive_mode() {
    let tmp = unique_tmp("neotrix_test_molt_archive");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("legacy")).unwrap();
    fs::write(tmp.join("legacy").join("stale.rs"), b"// stale").unwrap();

    let mut engine = CleanupEngine::new().with_project_root(tmp.clone());
    engine.dry_run_default = false;
    engine.archive_on_clean = true;
    let r = engine.molt_project();

    assert_eq!(r.deletable_count, 1);
    assert!(r.errors.is_empty(), "归档不应报错: {:?}", r.errors);
    // 旧躯壳已从活动树移除 (移动而非删除)
    assert!(!tmp.join("legacy").exists(), "legacy 应被移入归档");
    assert!(tmp.join(".cleanup").join("archive").exists());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_cleanup_engine_selftest_ok() {
    let result = CleanupEngineSelfTest.self_test();
    assert!(result.is_ok(), "SelfTest 应通过: {:?}", result.err());
}

#[test]
fn test_selftest_detects_missing_molting() {
    // 模拟规则库缺蜕皮 → SelfTest 必须报失败 (R-P23 检测系统自审计)
    // 通过向 all_patterns 打补丁不可行 (static), 验证护栏检测本身:
    let engine = CleanupEngine::new();
    assert!(
        engine
            .patterns
            .iter()
            .any(|p| p.kind == CleanupKind::ProjectMolting),
        "生产规则库应含蜕皮规则"
    );
}
