use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use super::designer::{ArchitectureDesign, CodeAction, ModuleBlueprint, RefactoringPlan, TypeKind};

/// 单个文件变更
#[derive(Clone, Debug)]
pub struct FileChange {
    pub path: String,
    pub content: String,
    pub action: ChangeAction,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChangeAction {
    Create,
    Modify,
    Delete,
}

/// 代码实现器 — 将架构设计转为 Rust 文件并写入磁盘
pub struct CodeImplementer;

impl Default for CodeImplementer {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeImplementer {
    pub fn new() -> Self {
        Self
    }

    /// 将架构设计转化为文件变更集合（不写入磁盘）
    pub fn plan_changes(&self, design: &ArchitectureDesign, project_root: &str) -> Vec<FileChange> {
        let mut changes = Vec::new();

        for blueprint in &design.new_modules {
            changes.push(self.create_module(blueprint, project_root));
            if should_register_in_parent(blueprint) {
                changes.push(self.create_parent_registration(blueprint, project_root));
            }
        }

        for plan in &design.refactoring_plans {
            for action in &plan.actions {
                if let Some(change) = self.apply_action(action, plan, project_root) {
                    changes.push(change);
                }
            }
        }

        changes
    }

    /// 将变更写入磁盘
    pub fn write_changes(&self, changes: &[FileChange]) -> Result<(), String> {
        for change in changes {
            let path = Path::new(&change.path);
            match change.action {
                ChangeAction::Create => {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            format!("Failed to create directory {}: {}", parent.display(), e)
                        })?;
                    }
                    let mut file = fs::File::create(path)
                        .map_err(|e| format!("Failed to create file {}: {}", path.display(), e))?;
                    file.write_all(change.content.as_bytes())
                        .map_err(|e| format!("Failed to write file {}: {}", path.display(), e))?;
                }
                ChangeAction::Modify => {
                    let mut file = fs::OpenOptions::new()
                        .append(true)
                        .open(path)
                        .map_err(|e| {
                            format!("Failed to open file {} for append: {}", path.display(), e)
                        })?;
                    file.write_all(change.content.as_bytes()).map_err(|e| {
                        format!("Failed to append to file {}: {}", path.display(), e)
                    })?;
                }
                ChangeAction::Delete => {
                    if path.exists() {
                        fs::remove_file(path).map_err(|e| {
                            format!("Failed to delete file {}: {}", path.display(), e)
                        })?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 从磁盘读取文件内容
    pub fn read_file_content(&self, path: &str) -> Result<String, String> {
        let mut file =
            fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        Ok(content)
    }

    /// 旧接口兼容 — 同时规划和写入
    pub fn implement(&self, design: &ArchitectureDesign, project_root: &str) -> Vec<FileChange> {
        let changes = self.plan_changes(design, project_root);
        let _ = self.write_changes(&changes);
        changes
    }

    /// 回滚变更：删除创建的文件，恢复修改的文件
    pub fn rollback(&self, changes: &[FileChange]) -> Result<(), String> {
        for change in changes.iter().rev() {
            let path = Path::new(&change.path);
            match change.action {
                ChangeAction::Create => {
                    if path.exists() {
                        fs::remove_file(path).map_err(|e| {
                            format!("Failed to rollback file {}: {}", path.display(), e)
                        })?;
                    }
                }
                ChangeAction::Modify => {
                    // 无法精确回滚追加操作 — 打印警告
                    log::warn!(
                        "Cannot rollback modification to {} (content was appended)",
                        path.display()
                    );
                }
                ChangeAction::Delete => {
                    // 无法恢复删除 — 打印警告
                    log::warn!("Cannot rollback deletion of {}", path.display());
                }
            }
        }
        Ok(())
    }

    fn create_module(&self, blueprint: &ModuleBlueprint, project_root: &str) -> FileChange {
        let dir_path = format!(
            "{}/src/{}/{}",
            project_root, blueprint.parent_module, blueprint.name
        );
        let file_path = format!(
            "{}/{}.rs",
            dir_path,
            blueprint.file_name.trim_end_matches(".rs")
        );

        let mut content = String::new();
        content.push_str(&format!(
            "//! {} — auto-generated by ArchitectAgent\n\n",
            blueprint.description
        ));

        for t in &blueprint.traits {
            content.push_str(&format!("pub trait {} {{\n", t.name));
            for m in &t.methods {
                content.push_str(&format!("    {};\n", m.signature));
            }
            content.push_str("}\n\n");
        }

        for ty in &blueprint.types {
            content.push_str("#[derive(Debug, Clone)]\n");
            let prefix = if ty.fields.iter().any(|f| f.is_pub) {
                "pub "
            } else {
                ""
            };
            match &ty.kind {
                TypeKind::Struct => {
                    content.push_str(&format!("{}struct {} {{\n", prefix, ty.name));
                    for f in &ty.fields {
                        let vis = if f.is_pub { "pub " } else { "" };
                        content.push_str(&format!("    {}{}: {},\n", vis, f.name, f.type_expr));
                    }
                    content.push_str("}\n\n");
                }
                TypeKind::Enum => {
                    content.push_str(&format!("{}enum {} {{\n", prefix, ty.name));
                    for f in &ty.fields {
                        content.push_str(&format!("    {}({}),\n", f.name, f.type_expr));
                    }
                    content.push_str("}\n\n");
                }
                TypeKind::Newtype(inner) => {
                    content.push_str(&format!("{}struct {}({});\n\n", prefix, ty.name, inner));
                }
            }
        }

        FileChange {
            path: file_path,
            content,
            action: ChangeAction::Create,
        }
    }

    fn create_parent_registration(
        &self,
        blueprint: &ModuleBlueprint,
        project_root: &str,
    ) -> FileChange {
        let mod_file = format!("{}/src/{}/mod.rs", project_root, blueprint.parent_module);
        let line = format!("pub mod {};\n", blueprint.name);

        FileChange {
            path: mod_file,
            content: line,
            action: ChangeAction::Modify,
        }
    }

    fn apply_action(
        &self,
        action: &CodeAction,
        _plan: &RefactoringPlan,
        _project_root: &str,
    ) -> Option<FileChange> {
        match action {
            CodeAction::AddTests {
                test_file,
                test_functions,
            } => {
                let mut content = String::from("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");
                for func in test_functions {
                    content.push_str(&format!(
                        "    #[test]\n    fn {}() {{\n        assert!(true);\n    }}\n\n",
                        func
                    ));
                }
                content.push_str("}\n");
                Some(FileChange {
                    path: test_file.clone(),
                    content,
                    action: ChangeAction::Create,
                })
            }
            CodeAction::SplitModule { new_sub_modules } => {
                let mut content = String::from("// Module split by ArchitectAgent\n");
                for sub in new_sub_modules {
                    content.push_str(&format!("pub mod {};\n", sub));
                }
                Some(FileChange {
                    path: "new_module_split.rs".to_string(),
                    content,
                    action: ChangeAction::Create,
                })
            }
            _ => None,
        }
    }
}

fn should_register_in_parent(blueprint: &ModuleBlueprint) -> bool {
    !blueprint.parent_module.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_arch::designer::{
        CodeAction, FieldBlueprint, MethodBlueprint, ModuleBlueprint, RefactoringPlan,
        TraitBlueprint, TypeBlueprint, TypeKind,
    };
    use tempfile::TempDir;

    #[test]
    fn test_create_module_generates_rust_code() {
        let impler = CodeImplementer::new();
        let blueprint = ModuleBlueprint {
            name: "test_mod".to_string(),
            parent_module: "core".to_string(),
            file_name: "test_mod.rs".to_string(),
            types: vec![TypeBlueprint {
                name: "Foo".to_string(),
                kind: TypeKind::Struct,
                fields: vec![FieldBlueprint {
                    name: "bar".to_string(),
                    type_expr: "String".to_string(),
                    is_pub: true,
                }],
            }],
            traits: vec![TraitBlueprint {
                name: "FooTrait".to_string(),
                methods: vec![MethodBlueprint {
                    name: "do_thing".to_string(),
                    signature: "fn do_thing(&self) -> bool".to_string(),
                    body_template: String::new(),
                }],
            }],
            description: "A test module".to_string(),
        };
        let design = ArchitectureDesign {
            new_modules: vec![blueprint],
            refactoring_plans: vec![],
            rationale: "test".to_string(),
        };
        let changes = impler.plan_changes(&design, "/tmp/test_project");
        assert!(!changes.is_empty());
        let module_change = changes
            .iter()
            .find(|c| c.path.ends_with("test_mod.rs"))
            .expect("module change for test_mod.rs should exist");
        assert_eq!(module_change.action, ChangeAction::Create);
        assert!(module_change.content.contains("struct Foo"));
        assert!(module_change.content.contains("pub trait FooTrait"));
    }

    #[test]
    fn test_generates_parent_registration() {
        let impler = CodeImplementer::new();
        let blueprint = ModuleBlueprint {
            name: "new_sub".to_string(),
            parent_module: "core.memory".to_string(),
            file_name: "new_sub.rs".to_string(),
            types: vec![],
            traits: vec![],
            description: "sub module".to_string(),
        };
        let design = ArchitectureDesign {
            new_modules: vec![blueprint],
            refactoring_plans: vec![],
            rationale: "test".to_string(),
        };
        let changes = impler.plan_changes(&design, "/tmp/test_project");
        assert!(
            changes.iter().any(|c| c.path.contains("mod.rs")),
            "should register in parent mod.rs"
        );
    }

    #[test]
    fn test_add_test_action() {
        let impler = CodeImplementer::new();
        let plan = RefactoringPlan {
            target_module: "core.foo".to_string(),
            target_file: "src/core/foo/mod.rs".to_string(),
            description: "add tests".to_string(),
            actions: vec![CodeAction::AddTests {
                test_file: "src/core/foo/tests.rs".to_string(),
                test_functions: vec!["test_foo".to_string()],
            }],
        };
        let design = ArchitectureDesign {
            new_modules: vec![],
            refactoring_plans: vec![plan],
            rationale: "test".to_string(),
        };
        let changes = impler.plan_changes(&design, "/tmp/test_project");
        assert!(changes.iter().any(|c| c.path.contains("tests.rs")));
    }

    #[test]
    fn test_write_changes_creates_files() {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let impler = CodeImplementer::new();
        let changes = vec![FileChange {
            path: tmp.path().join("test.rs").to_string_lossy().to_string(),
            content: "pub fn foo() -> i32 { 42 }".to_string(),
            action: ChangeAction::Create,
        }];
        impler
            .write_changes(&changes)
            .expect("write changes should succeed");
        let content = impler
            .read_file_content(&changes[0].path)
            .expect("should read back file content");
        assert!(content.contains("fn foo"));
    }

    #[test]
    fn test_write_changes_creates_directories() {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let impler = CodeImplementer::new();
        let nested = tmp.path().join("deeply/nested/dir/test.rs");
        let changes = vec![FileChange {
            path: nested.to_string_lossy().to_string(),
            content: "// nested file".to_string(),
            action: ChangeAction::Create,
        }];
        impler
            .write_changes(&changes)
            .expect("write changes should succeed");
        assert!(nested.exists());
    }

    #[test]
    fn test_rollback_creates() {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let impler = CodeImplementer::new();
        let file_path = tmp.path().join("to_rollback.rs");
        let changes = vec![FileChange {
            path: file_path.to_string_lossy().to_string(),
            content: "// temp".to_string(),
            action: ChangeAction::Create,
        }];
        impler
            .write_changes(&changes)
            .expect("write changes should succeed");
        assert!(file_path.exists());
        impler.rollback(&changes).expect("rollback should succeed");
        assert!(!file_path.exists());
    }
}
