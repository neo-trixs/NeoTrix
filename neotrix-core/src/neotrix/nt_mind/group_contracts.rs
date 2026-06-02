//! Group / Contract 系统 — 多仓库公共 API 契约管理
//! 借鉴 GitNexus group 系统思想：跨仓库 API 可见性 + 一致性追踪

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;

/// 合约类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContractType {
    Function,
    Struct,
    Trait,
    ApiEndpoint,
    Event,
    Config,
}

impl ContractType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContractType::Function => "fn",
            ContractType::Struct => "struct",
            ContractType::Trait => "trait",
            ContractType::ApiEndpoint => "api_endpoint",
            ContractType::Event => "event",
            ContractType::Config => "config",
        }
    }

    pub fn from_repr(s: &str) -> Option<Self> {
        match s {
            "fn" => Some(ContractType::Function),
            "struct" => Some(ContractType::Struct),
            "trait" => Some(ContractType::Trait),
            "api_endpoint" => Some(ContractType::ApiEndpoint),
            "event" => Some(ContractType::Event),
            "config" => Some(ContractType::Config),
            _ => None,
        }
    }
}

/// 单个 API 契约
#[derive(Debug, Clone)]
pub struct ApiContract {
    pub name: String,
    pub contract_type: ContractType,
    /// 函数签名或类型的完整声明文本
    pub signature: String,
    /// 源文件路径
    pub file_path: String,
    /// 所属仓库名
    pub repo_name: String,
    /// 版本标记（可选）
    pub version: String,
}

/// 仓库组 — 一组关联的仓库及其暴露的契约
#[derive(Debug, Clone)]
pub struct RepositoryGroup {
    pub name: String,
    pub repos: Vec<String>,
    /// repo_name → 该仓库的契约列表
    pub contracts: HashMap<String, Vec<ApiContract>>,
}

/// 匹配类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchType {
    Exact,
    Similar,
    Substring,
}

/// 跨仓库匹配结果
#[derive(Debug, Clone)]
pub struct ContractMatchResult {
    pub matched_contract: ApiContract,
    pub from_repo: String,
    pub to_repo: String,
    pub match_type: MatchType,
}

/// 组管理器
#[derive(Debug, Clone)]
pub struct GroupManager {
    groups: HashMap<String, RepositoryGroup>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// 创建一个新组（如果不存在）
    pub fn create_group(&mut self, name: &str) {
        self.groups.entry(name.to_string()).or_insert(RepositoryGroup {
            name: name.to_string(),
            repos: Vec::new(),
            contracts: HashMap::new(),
        });
    }

    /// 向组中添加仓库，同时提取其契约
    pub fn add_repo(&mut self, group_name: &str, repo_path: &str, repo_name: &str) {
        let group = self.groups.entry(group_name.to_string()).or_insert(RepositoryGroup {
            name: group_name.to_string(),
            repos: Vec::new(),
            contracts: HashMap::new(),
        });

        if !group.repos.contains(&repo_name.to_string()) {
            group.repos.push(repo_name.to_string());
        }

        let contracts = Self::extract_contracts(repo_path, repo_name);
        group.contracts.insert(repo_name.to_string(), contracts);
    }

    /// 从 rust 源文件中提取公共 API 契约
    pub fn extract_contracts(repo_path: &str, repo_name: &str) -> Vec<ApiContract> {
        let path = Path::new(repo_path);
        if !path.exists() {
            return Vec::new();
        }

        let fn_re = Regex::new(r"pub\s+fn\s+(\w+)").expect("hardcoded regex is valid");
        let struct_re = Regex::new(r"pub\s+struct\s+(\w+)").expect("hardcoded regex is valid");
        let trait_re = Regex::new(r"pub\s+(?:unsafe\s+)?trait\s+(\w+)").expect("hardcoded regex is valid");
        let enum_re = Regex::new(r"pub\s+enum\s+(\w+)").expect("hardcoded regex is valid");
        let sig_re = Regex::new(r"fn\s+\w+\([^)]*\)\s*(->\s*[\w:<>, \n\t]+)?").expect("hardcoded regex is valid");

        let mut contracts = Vec::new();
        let entries = Self::walk_rs_files(path);
        for entry in entries {
            let content = match fs::read_to_string(&entry) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // pub fn
            for cap in fn_re.captures_iter(&content) {
                let name = cap[1].to_string();
                let sig = sig_re.captures(&content[cap.get(0).expect("capture group 0 exists").start()..])
                    .map(|m| m[0].trim().to_string())
                    .unwrap_or_else(|| format!("pub fn {}", name));
                let file_path = entry.to_string_lossy().to_string();
                contracts.push(ApiContract {
                    name: name.clone(),
                    contract_type: ContractType::Function,
                    signature: sig,
                    file_path: file_path.clone(),
                    repo_name: repo_name.to_string(),
                    version: String::new(),
                });
            }

            // pub struct
            for cap in struct_re.captures_iter(&content) {
                let name = cap[1].to_string();
                let file_path = entry.to_string_lossy().to_string();
                contracts.push(ApiContract {
                    name: name.clone(),
                    contract_type: ContractType::Struct,
                    signature: format!("pub struct {}", name),
                    file_path: file_path.clone(),
                    repo_name: repo_name.to_string(),
                    version: String::new(),
                });
            }

            // pub trait
            for cap in trait_re.captures_iter(&content) {
                let name = cap[1].to_string();
                let file_path = entry.to_string_lossy().to_string();
                contracts.push(ApiContract {
                    name: name.clone(),
                    contract_type: ContractType::Trait,
                    signature: format!("pub trait {}", name),
                    file_path: file_path.clone(),
                    repo_name: repo_name.to_string(),
                    version: String::new(),
                });
            }

            // pub enum
            for cap in enum_re.captures_iter(&content) {
                let name = cap[1].to_string();
                let file_path = entry.to_string_lossy().to_string();
                contracts.push(ApiContract {
                    name: name.clone(),
                    contract_type: ContractType::Struct,
                    signature: format!("pub enum {}", name),
                    file_path,
                    repo_name: repo_name.to_string(),
                    version: String::new(),
                });
            }
        }

        contracts
    }

    /// 跨仓库匹配同名契约
    pub fn match_cross_repo(&self, name: &str) -> Vec<ContractMatchResult> {
        let mut results = Vec::new();
        let mut all_repos: Vec<(String, Vec<ApiContract>)> = Vec::new();

        for group in self.groups.values() {
            for (repo_name, contracts) in &group.contracts {
                all_repos.push((repo_name.clone(), contracts.clone()));
            }
        }

        for i in 0..all_repos.len() {
            for j in (i + 1)..all_repos.len() {
                let (repo_a, contracts_a) = &all_repos[i];
                let (repo_b, contracts_b) = &all_repos[j];

                for ca in contracts_a {
                    if ca.name == name {
                        for cb in contracts_b {
                            if cb.name == name {
                                results.push(ContractMatchResult {
                                    matched_contract: cb.clone(),
                                    from_repo: repo_a.clone(),
                                    to_repo: repo_b.clone(),
                                    match_type: MatchType::Exact,
                                });
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// 重新提取组内所有仓库的契约
    pub fn sync_group(&mut self, name: &str, repo_paths: &HashMap<String, String>) {
        if let Some(group) = self.groups.get(name) {
            let repos = group.repos.clone();
            for repo_name in &repos {
                if let Some(repo_path) = repo_paths.get(repo_name) {
                    let contracts = Self::extract_contracts(repo_path, repo_name);
                    if let Some(g) = self.groups.get_mut(name) {
                        g.contracts.insert(repo_name.clone(), contracts);
                    }
                }
            }
        }
    }

    /// 按名称或签名子串搜索契约
    pub fn query_group(&self, group_name: &str, query: &str) -> Vec<&ApiContract> {
        let mut results = Vec::new();
        let q = query.to_lowercase();

        if let Some(group) = self.groups.get(group_name) {
            for contracts in group.contracts.values() {
                for c in contracts {
                    if c.name.to_lowercase().contains(&q)
                        || c.signature.to_lowercase().contains(&q)
                    {
                        results.push(c);
                    }
                }
            }
        }

        results
    }

    pub fn get_group(&self, name: &str) -> Option<&RepositoryGroup> {
        self.groups.get(name)
    }

    pub fn groups(&self) -> &HashMap<String, RepositoryGroup> {
        &self.groups
    }

    fn walk_rs_files(dir: &Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        if !dir.is_dir() {
            if dir.extension().is_some_and(|e| e == "rs") {
                files.push(dir.to_path_buf());
            }
            return files;
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return files,
        };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.is_dir() {
                // skip hidden dirs and target
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" {
                    continue;
                }
                files.extend(Self::walk_rs_files(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
        files
    }
}

impl Default for GroupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestDir {
        path: std::path::PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!("neotrix_test_{}", n));
            std::fs::create_dir_all(&path).expect("failed to create temp test directory");
            TestDir { path }
        }

        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn create_test_rust_file(dir: &TestDir, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut file = std::fs::File::create(&path).expect("failed to create test rust file");
        write!(file, "{}", content).expect("failed to write test file content");
        path
    }

    fn source_a() -> &'static str {
        r#"
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub struct User {
    pub id: u64,
    pub name: String,
}

pub trait Greeter {
    fn greet(&self) -> String;
}

pub enum Status {
    Active,
    Inactive,
}

fn internal_helper() -> u32 { 42 }
"#
    }

    fn source_b() -> &'static str {
        r#"
pub fn hello(name: &str) -> String {
    format!("Hi, {}!", name)
}

pub struct User {
    pub id: u64,
    pub email: String,
}

pub trait Serializable {
    fn to_json(&self) -> String;
}
"#
    }

    #[test]
    fn test_extract_contracts_from_source() {
        let dir = TestDir::new();
        create_test_rust_file(&dir, "lib.rs", source_a());

        let contracts = GroupManager::extract_contracts(
            dir.path().to_str().expect("test dir path is valid utf-8"),
            "repo_a",
        );

        assert_eq!(contracts.len(), 4, "should find 4 pub items (fn, struct, trait, enum)");

        let names: Vec<&str> = contracts.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"User"));
        assert!(names.contains(&"Greeter"));
        assert!(names.contains(&"Status"));

        let hello = contracts.iter().find(|c| c.name == "hello").expect("hello contract should exist");
        assert_eq!(hello.contract_type, ContractType::Function);
        assert!(hello.signature.contains("->"));

        let user = contracts.iter().find(|c| c.name == "User").expect("User contract should exist");
        assert_eq!(user.contract_type, ContractType::Struct);
    }

    #[test]
    fn test_extract_contracts_empty_repo() {
        let dir = TestDir::new();
        let contracts = GroupManager::extract_contracts(
            dir.path().to_str().expect("test dir path is valid utf-8"),
            "empty",
        );
        assert!(contracts.is_empty());
    }

    #[test]
    fn test_extract_contracts_non_existent_path() {
        let contracts = GroupManager::extract_contracts("/tmp/non_existent_path_xyz", "ghost");
        assert!(contracts.is_empty());
    }

    #[test]
    fn test_add_repo_and_query() {
        let dir_a = TestDir::new();
        let dir_b = TestDir::new();
        create_test_rust_file(&dir_a, "lib.rs", source_a());
        create_test_rust_file(&dir_b, "mod.rs", source_b());

        let mut manager = GroupManager::new();
        manager.add_repo("core", dir_a.path().to_str().expect("test dir a path is valid utf-8"), "repo_a");
        manager.add_repo("core", dir_b.path().to_str().expect("test dir b path is valid utf-8"), "repo_b");

        let results = manager.query_group("core", "hello");
        assert_eq!(results.len(), 2, "both repos have hello");

        let results_struct = manager.query_group("core", "User");
        assert_eq!(results_struct.len(), 2, "both repos have User");

        let results_trait = manager.query_group("core", "Greeter");
        assert_eq!(results_trait.len(), 1, "only repo_a has Greeter");
    }

    #[test]
    fn test_match_cross_repo() {
        let dir_a = TestDir::new();
        let dir_b = TestDir::new();
        create_test_rust_file(&dir_a, "lib.rs", source_a());
        create_test_rust_file(&dir_b, "mod.rs", source_b());

        let mut manager = GroupManager::new();
        manager.add_repo("core", dir_a.path().to_str().expect("test dir a path is valid utf-8"), "repo_a");
        manager.add_repo("core", dir_b.path().to_str().expect("test dir b path is valid utf-8"), "repo_b");

        let matches = manager.match_cross_repo("hello");
        assert_eq!(matches.len(), 1, "one cross-repo match for hello");
        assert_eq!(matches[0].match_type, MatchType::Exact);
        assert_eq!(matches[0].matched_contract.name, "hello");

        let matches_none = manager.match_cross_repo("Greeter");
        assert!(matches_none.is_empty(), "Greeter only in repo_a");
    }

    #[test]
    fn test_query_group_empty() {
        let manager = GroupManager::new();
        assert!(manager.query_group("non_existent", "anything").is_empty());
    }

    #[test]
    fn test_query_by_signature() {
        let dir = TestDir::new();
        create_test_rust_file(&dir, "lib.rs", source_a());

        let mut manager = GroupManager::new();
        manager.add_repo("core", dir.path().to_str().expect("test dir path is valid utf-8"), "repo_a");

        let results = manager.query_group("core", "String");
        assert!(!results.is_empty(), "should match signature containing String");
    }

    #[test]
    fn test_sync_group() {
        let dir_a = TestDir::new();
        create_test_rust_file(&dir_a, "lib.rs", source_a());

        let mut manager = GroupManager::new();
        manager.add_repo("core", dir_a.path().to_str().expect("test dir a path is valid utf-8"), "repo_a");

        let mut paths = HashMap::new();
        paths.insert("repo_a".to_string(), dir_a.path().to_str().expect("test dir path is valid utf-8").to_string());
        manager.sync_group("core", &paths);

        let results = manager.query_group("core", "hello");
        assert_eq!(results.len(), 1, "after sync, hello still present");
    }

    #[test]
    fn test_multiple_groups() {
        let dir = TestDir::new();
        create_test_rust_file(&dir, "lib.rs", source_a());

        let mut manager = GroupManager::new();
        manager.add_repo("group_a", dir.path().to_str().expect("test dir path is valid utf-8"), "repo_x");
        manager.add_repo("group_b", dir.path().to_str().expect("test dir path is valid utf-8"), "repo_x");

        assert_eq!(manager.groups().len(), 2);
        assert_eq!(manager.query_group("group_a", "hello").len(), 1);
        assert_eq!(manager.query_group("group_b", "hello").len(), 1);
    }

    #[test]
    fn test_contract_type_display() {
        assert_eq!(ContractType::Function.to_string(), "fn");
        assert_eq!(ContractType::Struct.to_string(), "struct");
        assert_eq!(ContractType::Trait.to_string(), "trait");
    }

    #[test]
    fn test_contract_type_from_repr() {
        assert_eq!(ContractType::from_repr("fn"), Some(ContractType::Function));
        assert_eq!(ContractType::from_repr("struct"), Some(ContractType::Struct));
        assert_eq!(ContractType::from_repr("unknown"), None);
    }

    #[test]
    fn test_walk_rs_files_nested() {
        let dir = TestDir::new();
        let nested = dir.path().join("nested");
        std::fs::create_dir_all(&nested).expect("failed to create nested test dir");
        create_test_rust_file(&dir, "lib.rs", source_a());
        let sub_path = nested.join("sub.rs");
        let mut file = std::fs::File::create(&sub_path).expect("failed to create sub test file");
        write!(file, "pub fn sub() {{}}").expect("failed to write sub test file");

        let contracts = GroupManager::extract_contracts(
            dir.path().to_str().expect("test dir path is valid utf-8"),
            "nested",
        );
        let names: Vec<&str> = contracts.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"sub"));
    }

    #[test]
    fn test_duplicate_repo_not_added_twice() {
        let dir = TestDir::new();
        create_test_rust_file(&dir, "lib.rs", source_a());

        let mut manager = GroupManager::new();
        manager.add_repo("core", dir.path().to_str().expect("test dir path is valid utf-8"), "repo_a");
        manager.add_repo("core", dir.path().to_str().expect("test dir path is valid utf-8"), "repo_a");

        let group = manager.get_group("core").expect("core group should have been created");
        assert_eq!(group.repos.len(), 1, "should not duplicate repo name");
    }
}
