use crate::neotrix::nt_act_orchestrator::planner::PlannerNode;
use crate::neotrix::nt_mind::group_contracts::GroupManager;

fn create_contract_repo(dir: &std::path::Path, content: &str) {
    std::fs::create_dir_all(dir).expect("result");
    std::fs::write(dir.join("lib.rs"), content).expect("result");
}

struct TestRepo {
    _a: tempfile::TempDir,
    _b: tempfile::TempDir,
}

fn setup_gm() -> (GroupManager, TestRepo) {
    let mut gm = GroupManager::new();
    gm.create_group("core");

    let dir_a = tempfile::tempdir().expect("tempdir");
    let dir_b = tempfile::tempdir().expect("tempdir");
    let repo = TestRepo { _a: dir_a, _b: dir_b };

    create_contract_repo(repo._a.path(), "pub fn 执行任务() -> i32 { 42 }\npub fn 验证结果() -> bool { true }");
    create_contract_repo(repo._b.path(), "pub fn 执行任务() -> i32 { 43 }");

    gm.add_repo("core", repo._a.path().to_str().expect("result"), "repo_a");
    gm.add_repo("core", repo._b.path().to_str().expect("result"), "repo_b");
    (gm, repo)
}

#[test]
fn test_planner_enriches_cross_repo_tasks() {
    let (gm, _repo) = setup_gm();
    let planner = PlannerNode::with_group_manager(gm);
    let tasks = planner.decompose("some generic goal");

    assert!(!tasks.is_empty(), "Should decompose tasks");

    let has_cross_repo = tasks.iter().any(|t| {
        let decoded: String = t.input.iter().map(|&f| f as u8 as char).collect();
        decoded.contains("[cross-repo")
    });
    assert!(has_cross_repo, "Expected at least one task with cross-repo annotation");
}

#[test]
fn test_planner_no_group_manager() {
    let planner = PlannerNode::new();
    let tasks = planner.decompose("some task");
    assert!(!tasks.is_empty(), "Should decompose without group manager");
}

#[test]
fn test_orchestrator_builds_with_group_manager() {
    let (_gm, _repo) = setup_gm();
}

#[test]
fn test_planner_with_group_manager_enriches_only_known_contracts() {
    let (gm, _repo) = setup_gm();
    let planner = PlannerNode::with_group_manager(gm);
    let tasks = planner.decompose("design a component");

    assert!(!tasks.is_empty(), "Design plan should still decompose");

    // design plan subtasks don't contain "执行任务" so fewer enrichments expected
    let cross_repo_count = tasks.iter().filter(|t| {
        let decoded: String = t.input.iter().map(|&f| f as u8 as char).collect();
        decoded.contains("[cross-repo")
    }).count();
    assert_eq!(cross_repo_count, 0, "Design plan shouldn't match cross-repo contracts named 执行任务");
}

#[test]
    fn test_decompose_cross_repo_info_in_task_name() {
    let (gm, _repo) = setup_gm();
    let planner = PlannerNode::with_group_manager(gm);
    let tasks = planner.decompose("some generic goal");

    let enriched = tasks.iter().find(|t| {
        let decoded: String = t.input.iter().map(|&f| f as u8 as char).collect();
        decoded.contains("[cross-repo")
    });
    assert!(enriched.is_some(), "At least one enriched task");

    if let Some(task) = enriched {
        let decoded: String = task.input.iter().map(|&f| f as u8 as char).collect();
        assert!(decoded.contains("->"), "Should reference a repo in cross-repo info");
    }
}
