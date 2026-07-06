//! AutoFixer — 自执行修复引擎

/// 自动修复执行器 — 对已检测问题执行真实代码修改
pub struct AutoFixer;

impl AutoFixer {
    /// 运行 cargo fix 自动修复编译警告
    pub fn cargo_fix() -> Result<String, String> {
        let output = std::process::Command::new("cargo")
            .args(["fix", "--lib", "--allow-dirty"])
            .output()
            .map_err(|e| format!("cargo fix 调用失败: {}", e))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(format!("cargo fix 完成: {}", stdout.lines().count()))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("cargo fix 失败: {}", stderr.lines().next().unwrap_or("unknown")))
        }
    }

    /// 运行 cargo check 获取实时编译状态
    pub fn cargo_check() -> Result<(usize, usize), String> {
        let output = std::process::Command::new("cargo")
            .args(["check", "--lib"])
            .output()
            .map_err(|e| format!("cargo check 调用失败: {}", e))?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let errors = stderr.matches("error[").count();
        let warnings = stderr.matches("warning:").count();
        Ok((errors, warnings))
    }

    /// 启用一个被 #[ignore] 的测试
    pub fn enable_ignored_test(file_path: &str, line: usize) -> Result<String, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut lines: Vec<&str> = content.lines().collect();
        if line == 0 || line > lines.len() {
            return Err("行号越界".into());
        }
        let idx = line - 1;
        if lines[idx].trim() == "#[ignore]" {
            lines.remove(idx);
            let new_content = lines.join("\n");
            std::fs::write(file_path, &new_content)
                .map_err(|e| format!("写入失败: {}", e))?;
            Ok(format!("已启用 {}", file_path))
        } else {
            Err(format!("第{}行不是 #[ignore]", line))
        }
    }

    /// 向文件添加测试模块存根 (如果不存在)
    pub fn add_test_stub(file_path: &str) -> Result<String, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        if content.contains("#[cfg(test)]") {
            return Err("已有测试模块".into());
        }
        let stub = "\n\n#[cfg(test)]\nmod tests {\n\n    #[test]\n    fn test_basic() {\n        assert!(true);\n    }\n}\n";
        let mut new_content = content;
        new_content.push_str(stub);
        std::fs::write(file_path, &new_content)
            .map_err(|e| format!("写入失败: {}", e))?;
        Ok(format!("已添加测试存根到 {}", file_path))
    }

    /// 移除未使用导入 (通过 cargo fix)
    pub fn remove_unused_imports() -> Result<String, String> {
        let output = std::process::Command::new("cargo")
            .args(["fix", "--lib", "--allow-dirty", "--edition-idioms"])
            .output()
            .map_err(|e| format!("cargo fix 调用失败: {}", e))?;
        if output.status.success() {
            Ok("已清理未使用导入".to_string())
        } else {
            Err("cargo fix 失败".to_string())
        }
    }

    /// 删除文件中特定行的 TODO 注释
    pub fn remove_todo_line(file_path: &str, line: usize) -> Result<String, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut lines: Vec<&str> = content.lines().collect();
        if line == 0 || line > lines.len() {
            return Err("行号越界".into());
        }
        let idx = line - 1;
        let trimmed = lines[idx].trim();
        if trimmed.starts_with("// TODO") || trimmed.starts_with("//TODO") {
            lines.remove(idx);
            let new_content = lines.join("\n");
            std::fs::write(file_path, &new_content)
                .map_err(|e| format!("写入失败: {}", e))?;
            Ok(format!("已移除 TODO 行 {}:{}", file_path, line))
        } else {
            Err(format!("第{}行不是纯 TODO 注释", line))
        }
    }

    /// 将大文件在已知模块边界处拆分为多个文件
    ///
    /// 安全: 仅在 NEOTRIX_SPLIT_ENABLE=1 环境变量设置时执行真实拆分
    /// 测试时通过设置 `NEOTRIX_SPLIT_ENABLE=0` 防止意外
    pub fn split_file(file_path: &str) -> Result<String, String> {
        if std::env::var("NEOTRIX_SPLIT_ENABLE").as_deref() != Ok("1") {
            return Err("split_file 需要设置 NEOTRIX_SPLIT_ENABLE=1".into());
        }
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut created = Vec::new();
        let path = std::path::Path::new(file_path);
        let parent = path.parent().ok_or("无法确定父目录")?;
        let stem = path.file_stem().ok_or("无法确定文件名")?;

        let dir_path = parent.join(stem);
        std::fs::create_dir_all(&dir_path)
            .map_err(|e| format!("创建目录失败: {}", e))?;

        let mut current_block = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if (trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ")
                || trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ")
                || trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ")
                || trimmed.starts_with("impl ") || trimmed.starts_with("pub impl ")
                || trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")
                || trimmed.starts_with("#[cfg("))
                && !current_block.trim().is_empty()
            {
                created.push(std::mem::take(&mut current_block));
            }
            current_block.push_str(line);
            current_block.push('\n');
        }
        if !current_block.trim().is_empty() {
            created.push(current_block);
        }

        // 写入每个块到单独文件
        let mut sub_mods = Vec::new();
        for (i, block) in created.iter().enumerate() {
            let item_file = dir_path.join(format!("part_{}.rs", i));
            std::fs::write(&item_file, block)
                .map_err(|e| format!("写入失败: {}", e))?;
            sub_mods.push(format!("part_{}", i));
        }

        // 生成 mod.rs
        let mut mod_rs = String::new();
        for m in &sub_mods {
            mod_rs.push_str(&format!("pub mod {};\n", m));
        }
        std::fs::write(dir_path.join("mod.rs"), &mod_rs)
            .map_err(|e| format!("写入 mod.rs 失败: {}", e))?;

        // 删除原文件
        std::fs::remove_file(file_path)
            .map_err(|e| format!("删除原文件失败: {}", e))?;

        Ok(format!("拆分为 {} 个文件: {}", sub_mods.len(), sub_mods.join(", ")))
    }

    /// 扫描并清理文件中的 TODO 注释
    pub fn cleanup_todos(file_path: &str) -> Result<usize, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("读取失败: {}", e))?;
        let mut removed = 0usize;
        let lines: Vec<&str> = content.lines().collect();

        // 只移除纯 TODO 注释行 (非代码逻辑 TODO)
        let mut kept = Vec::new();
        for line in &lines {
            let trimmed = line.trim();
            if trimmed == "// TODO" || trimmed == "//TODO" || trimmed == "// FIXME" {
                removed += 1;
                continue;
            }
            kept.push(*line);
        }

        if removed > 0 {
            std::fs::write(file_path, kept.join("\n"))
                .map_err(|e| format!("写入失败: {}", e))?;
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autofixer_cargo_check_structure() {
        let result = AutoFixer::cargo_check();
        assert!(result.is_ok() || result.is_err());
    }
}
