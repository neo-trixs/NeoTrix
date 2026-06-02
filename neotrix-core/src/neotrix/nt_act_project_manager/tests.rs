use super::*;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn create_temp_project(dir: &Path, files: &[(&str, &str)]) {
    fs::create_dir_all(dir).expect("should create temp project dir");
    for (name, content) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("should create parent dir for file");
        }
        let mut f = fs::File::create(&path).expect("should create temp file");
        write!(f, "{}", content).expect("should write to temp file");
    }
}

#[test]
fn test_project_manager_new() {
    let pm = ProjectManager::new();
    assert!(pm.projects.is_empty());
    assert!(pm.active_id.is_none());
    assert_eq!(pm.recent_projects.len(), 0);
}

#[test]
fn test_open_new_project() {
    let dir = std::env::temp_dir().join("neotrix_test_pm_open");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n")]);
    let mut pm = ProjectManager::new();
    let project = pm.open(&dir).expect("should open new project");
    assert_eq!(project.name, dir.file_name().expect("temp dir should have name").to_str().expect("path should be valid UTF-8"));
    assert!(project.tech_stack.contains(&"rust".to_string()));
    assert!(pm.active_id.is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_open_existing_project() {
    let dir = std::env::temp_dir().join("neotrix_test_pm_open_existing");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "[package]\nname = \"test\"\n")]);
    let mut pm = ProjectManager::new();
    pm.open(&dir).expect("should open project first time");
    let before_count = pm.projects.len();
    pm.open(&dir).expect("should open existing project again");
    assert_eq!(pm.projects.len(), before_count, "should not duplicate");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_tech_stack_rust() {
    let dir = std::env::temp_dir().join("neotrix_test_stack_rust");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "")]);
    let stack = ProjectManager::detect_tech_stack(&dir);
    assert!(stack.contains(&"rust".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_tech_stack_node() {
    let dir = std::env::temp_dir().join("neotrix_test_stack_node");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("package.json", "{}")]);
    let stack = ProjectManager::detect_tech_stack(&dir);
    assert!(stack.contains(&"node".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_tech_stack_python() {
    let dir = std::env::temp_dir().join("neotrix_test_stack_python");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("pyproject.toml", "")]);
    let stack = ProjectManager::detect_tech_stack(&dir);
    assert!(stack.contains(&"python".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_tech_stack_go() {
    let dir = std::env::temp_dir().join("neotrix_test_stack_go");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("go.mod", "")]);
    let stack = ProjectManager::detect_tech_stack(&dir);
    assert!(stack.contains(&"go".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_tech_stack_multiple() {
    let dir = std::env::temp_dir().join("neotrix_test_stack_multi");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", ""), ("package.json", "{\"dependencies\": {\"next\": \"13\"}}")]);
    let stack = ProjectManager::detect_tech_stack(&dir);
    assert!(stack.contains(&"rust".to_string()));
    assert!(stack.contains(&"node".to_string()));
    assert!(stack.contains(&"nextjs".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_switch_project() {
    let dir_a = std::env::temp_dir().join("neotrix_test_switch_a");
    let dir_b = std::env::temp_dir().join("neotrix_test_switch_b");
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
    create_temp_project(&dir_a, &[("Cargo.toml", "")]);
    create_temp_project(&dir_b, &[("package.json", "{}")]);
    let mut pm = ProjectManager::new();
    let a = pm.open(&dir_a).expect("should open project A");
    let id_a = a.id.clone();
    pm.open(&dir_b).expect("should open project B");
    let switched = pm.switch(&id_a).expect("should switch to project A");
    assert_eq!(switched.id, id_a);
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn test_switch_nonexistent() {
    let mut pm = ProjectManager::new();
    let result = pm.switch("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_close() {
    let dir = std::env::temp_dir().join("neotrix_test_close");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "")]);
    let mut pm = ProjectManager::new();
    pm.open(&dir).expect("should open project");
    assert!(pm.active_id.is_some());
    pm.close();
    assert!(pm.active_id.is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_recent() {
    let dir_a = std::env::temp_dir().join("neotrix_test_recent_a");
    let dir_b = std::env::temp_dir().join("neotrix_test_recent_b");
    let dir_c = std::env::temp_dir().join("neotrix_test_recent_c");
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
    let _ = fs::remove_dir_all(&dir_c);
    create_temp_project(&dir_a, &[("Cargo.toml", "")]);
    create_temp_project(&dir_b, &[("Cargo.toml", "")]);
    create_temp_project(&dir_c, &[("Cargo.toml", "")]);
    let mut pm = ProjectManager::new();
    let a = pm.open(&dir_a).expect("should open project A");
    let id_a = a.id.clone();
    pm.open(&dir_b).expect("should open project B");
    let recent = pm.recent();
    assert_eq!(recent.len(), 2);
    let dir_b_canonical = fs::canonicalize(&dir_b).unwrap_or_else(|_| dir_b.clone());
    assert_eq!(recent[0].id, dir_b_canonical.to_string_lossy().to_string());
    pm.switch(&id_a).expect("should switch to project A");
    let recent = pm.recent();
    assert_eq!(recent[0].id, id_a);
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
    let _ = fs::remove_dir_all(&dir_c);
}

#[test]
fn test_recent_max_10() {
    let mut pm = ProjectManager::new();
    for i in 0..15 {
        let dir = std::env::temp_dir().join(format!("neotrix_test_recent_max_{}", i));
        let _ = fs::remove_dir_all(&dir);
        create_temp_project(&dir, &[("Cargo.toml", "")]);
        pm.open(&dir).expect("should open project");
        let _ = fs::remove_dir_all(&dir);
    }
    assert_eq!(pm.recent_projects.len(), 10);
    assert_eq!(pm.recent().len(), 10);
}

#[test]
fn test_all_projects() {
    let dir_a = std::env::temp_dir().join("neotrix_test_all_a");
    let dir_b = std::env::temp_dir().join("neotrix_test_all_b");
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
    create_temp_project(&dir_a, &[("Cargo.toml", "")]);
    create_temp_project(&dir_b, &[("Cargo.toml", "")]);
    let mut pm = ProjectManager::new();
    pm.open(&dir_a).expect("should open project A");
    pm.open(&dir_b).expect("should open project B");
    assert_eq!(pm.all().len(), 2);
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn test_active() {
    let dir = std::env::temp_dir().join("neotrix_test_active");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "")]);
    let mut pm = ProjectManager::new();
    assert!(pm.active().is_none());
    pm.open(&dir).expect("should open project");
    assert!(pm.active().is_some());
    pm.close();
    assert!(pm.active().is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_save_and_load() {
    let dir = std::env::temp_dir().join("neotrix_test_save_load");
    let save_path = dir.join("projects.json");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "")]);
    let mut pm = ProjectManager::new();
    pm.open(&dir).expect("should open project");
    pm.save(&save_path).expect("should save project manager");
    let loaded = ProjectManager::load(&save_path).expect("should load project manager");
    assert_eq!(loaded.projects.len(), 1);
    assert_eq!(loaded.projects[0].name, dir.file_name().expect("temp dir should have name").to_str().expect("path should be valid UTF-8"));
    assert!(loaded.projects[0].tech_stack.contains(&"rust".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_load_nonexistent() {
    let result = ProjectManager::load(Path::new("/nonexistent/path.json"));
    assert!(result.is_err());
}

#[test]
fn test_set_and_get_config() {
    let dir = std::env::temp_dir().join("neotrix_test_config");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "")]);
    let mut pm = ProjectManager::new();
    let p = pm.open(&dir).expect("should open project");
    let id = p.id.clone();
    pm.set_config(&id, "openai", "gpt-4", "You are a helpful assistant.");
    let config = pm.get_config(&id).expect("should get config for project");
    assert_eq!(config, ("openai", "gpt-4", "You are a helpful assistant."));
    assert!(pm.get_config("nonexistent").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_git_is_git_repo() {
    let dir = std::env::temp_dir().join("neotrix_test_git_isrepo");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("should create temp dir");
    assert!(!GitIntegration::is_git_repo(&dir));
    fs::create_dir_all(dir.join(".git")).expect("should create .git dir");
    assert!(GitIntegration::is_git_repo(&dir));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_git_current_branch() {
    let dir = std::env::temp_dir().join("neotrix_test_git_branch");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("should create temp dir");
    let output = Command::new("git")
        .args(["init", &dir.to_string_lossy()])
        .output()
        .expect("git init should start");
    if !output.status.success() {
        eprintln!("git init failed, skipping test");
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.email", "test@test.com"])
        .output()
        .expect("git config email should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
        .output()
        .expect("git config name should start");
    fs::write(dir.join("README.md"), "# test").expect("should write README");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "."])
        .output()
        .expect("git add should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "commit", "-m", "init"])
        .output()
        .expect("git commit should start");
    let branch = GitIntegration::current_branch(&dir).expect("should get current branch");
    assert_eq!(branch, "main");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_git_list_branches() {
    let dir = std::env::temp_dir().join("neotrix_test_git_list");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("should create temp dir");
    let output = Command::new("git")
        .args(["init", &dir.to_string_lossy()])
        .output()
        .expect("git init should start");
    if !output.status.success() {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.email", "test@test.com"])
        .output()
        .expect("git config email should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
        .output()
        .expect("git config name should start");
    fs::write(dir.join("README.md"), "# test").expect("should write README");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "."])
        .output()
        .expect("git add should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "commit", "-m", "init"])
        .output()
        .expect("git commit should start");
    let branches = GitIntegration::list_branches(&dir).expect("should list branches");
    assert!(branches.contains(&"main".to_string()));
    GitIntegration::create_branch(&dir, "feature-x").expect("should create branch");
    let branches = GitIntegration::list_branches(&dir).expect("should list branches");
    assert!(branches.contains(&"feature-x".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_git_switch_branch() {
    let tmp = tempfile::tempdir().expect("should create temp dir");
    let dir = tmp.path().to_path_buf();
    let output = Command::new("git")
        .args(["init", &dir.to_string_lossy()])
        .output()
        .expect("git init should start");
    if !output.status.success() { return; }
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.email", "test@test.com"])
        .output()
        .expect("git config email should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
        .output()
        .expect("git config name should start");
    fs::write(dir.join("README.md"), "# test").expect("should write README");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "."])
        .output()
        .expect("git add should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "commit", "-m", "init"])
        .output()
        .expect("git commit should start");
    GitIntegration::create_branch(&dir, "feature-y").expect("should create branch");
    assert_eq!(GitIntegration::current_branch(&dir).expect("should get current branch"), "feature-y");
    GitIntegration::switch_branch(&dir, "main").expect("should switch branch");
    assert_eq!(GitIntegration::current_branch(&dir).expect("should get current branch"), "main");
}

#[test]
fn test_git_diff_staged() {
    let tmp = tempfile::tempdir().expect("should create temp dir");
    let dir = tmp.path().to_path_buf();
    let output = Command::new("git")
        .args(["init", &dir.to_string_lossy()])
        .output()
        .expect("git init should start");
    if !output.status.success() { return; }
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.email", "test@test.com"])
        .output()
        .expect("git config email should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
        .output()
        .expect("git config name should start");
    fs::write(dir.join("test.txt"), "content").expect("should write test file");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "test.txt"])
        .output()
        .expect("git add should start");
    let diff = GitIntegration::diff_staged(&dir).expect("should get staged diff");
    assert!(diff.contains("test.txt"));
}

#[test]
fn test_git_diff_unstaged() {
    let dir = std::env::temp_dir().join("neotrix_test_git_diff_unstaged");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("should create temp dir");
    let output = Command::new("git")
        .args(["init", &dir.to_string_lossy()])
        .output()
        .expect("git init should start");
    if !output.status.success() {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.email", "test@test.com"])
        .output()
        .expect("git config email should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
        .output()
        .expect("git config name should start");
    fs::write(dir.join("existing.txt"), "original").expect("should write existing file");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "existing.txt"])
        .output()
        .expect("git add should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "commit", "-m", "initial"])
        .output()
        .expect("git commit should start");
    fs::write(dir.join("existing.txt"), "modified").expect("should modify existing file");
    let diff = GitIntegration::diff_unstaged(&dir).expect("should get unstaged diff");
    assert!(diff.contains("existing.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_git_commit() {
    let tmp = tempfile::tempdir().expect("should create temp dir");
    let dir = tmp.path().to_path_buf();
    let output = Command::new("git")
        .args(["init", &dir.to_string_lossy()])
        .output()
        .expect("git init should start");
    if !output.status.success() { return; }
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.email", "test@test.com"])
        .output()
        .expect("git config email should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
        .output()
        .expect("git config name should start");
    fs::write(dir.join("README.md"), "# test").expect("should write README");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "."])
        .output()
        .expect("git add should start");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "commit", "-m", "init"])
        .output()
        .expect("git commit should start");
    fs::write(dir.join("new.txt"), "new content").expect("should write new file");
    Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "new.txt"])
        .output()
        .expect("git add should start");
    GitIntegration::commit(&dir, "add new.txt").expect("should commit");
    let log = Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "log", "--oneline", "-1"])
        .output()
        .expect("git log should start");
    assert!(String::from_utf8_lossy(&log.stdout).contains("add new.txt"));
}

#[test]
fn test_git_generate_commit_message_add() {
    let diff = "diff --git a/src/new.rs b/src/new.rs\nnew file mode 100644\n--- /dev/null\n+++ b/src/new.rs\n@@ -0,0 +1 @@\n+fn foo() {}\n";
    let msg = GitIntegration::generate_commit_message(diff);
    assert!(msg.contains("add:"));
}

#[test]
fn test_git_generate_commit_message_empty() {
    let msg = GitIntegration::generate_commit_message("");
    assert_eq!(msg, "chore: miscellaneous changes");
}

#[test]
fn test_detect_tech_stack_tauri() {
    let dir = std::env::temp_dir().join("neotrix_test_stack_tauri");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Cargo.toml", "[dependencies]\ntauri = \"1\"\n")]);
    let stack = ProjectManager::detect_tech_stack(&dir);
    assert!(stack.contains(&"tauri".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_tech_stack_docker() {
    let dir = std::env::temp_dir().join("neotrix_test_stack_docker");
    let _ = fs::remove_dir_all(&dir);
    create_temp_project(&dir, &[("Dockerfile", "FROM ubuntu")]);
    let stack = ProjectManager::detect_tech_stack(&dir);
    assert!(stack.contains(&"docker".to_string()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_open_nonexistent_path() {
    let mut pm = ProjectManager::new();
    let result = pm.open(Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn test_worktree_add_rejects_on_no_repo() {
    let dir = std::env::temp_dir().join("neotrix_test_worktree_fail");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("should create temp dir");
    let wd = dir.join("worktree-test");
    let result = GitIntegration::worktree_add(&dir, &wd, "some-branch");
    assert!(result.is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_git_current_branch_fails_in_non_repo() {
    let dir = std::env::temp_dir().join("neotrix_test_non_repo");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("should create temp dir");
    let result = GitIntegration::current_branch(&dir);
    assert!(result.is_err());
    let _ = fs::remove_dir_all(&dir);
}
