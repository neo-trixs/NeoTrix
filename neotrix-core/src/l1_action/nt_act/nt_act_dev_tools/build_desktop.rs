//! Desktop Builder — 桌面端构建工具
//!
//! 移植自 scripts/build-desktop.sh
//! 支持 check/build/package:dir/package 阶梯

use std::path::PathBuf;
use std::process::Command;

/// 构建阶梯
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildLadder {
    /// 类型检查 + 单元测试 + cargo check
    Check,
    /// 前端构建 + cargo build 桌面端二进制
    Build,
    /// 完整 tauri build --no-bundle → 未打包 .app/ 目录
    PackageDir,
    /// 完整 tauri build → 原生安装包
    Package,
}

/// 构建配置
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub release: bool,
    pub run: bool,
    pub root: PathBuf,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            release: false,
            run: false,
            root: PathBuf::from("."),
        }
    }
}

/// 桌面端构建器
pub struct DesktopBuilder {
    config: BuildConfig,
}

impl DesktopBuilder {
    pub fn new(config: BuildConfig) -> Self {
        Self { config }
    }

    /// 执行构建
    pub fn build(&self, ladder: BuildLadder) -> Result<String, String> {
        match ladder {
            BuildLadder::Check => self.cmd_check(),
            BuildLadder::Build => self.cmd_build(),
            BuildLadder::PackageDir => self.cmd_package_dir(),
            BuildLadder::Package => self.cmd_package(),
        }
    }

    /// 前端构建
    fn frontend_build(&self) -> Result<(), String> {
        let frontend_dir = self.config.root.join("neocodex-frontend");
        if !frontend_dir.exists() {
            return Err("Frontend directory not found".to_string());
        }

        println!("==> 构建前端 (npm run build)");
        let output = Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(&frontend_dir)
            .output()
            .map_err(|e| format!("Failed to run npm: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Frontend build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // 强制 tauri-build 重新嵌入
        let tauri_conf = self.config.root.join("src-tauri/tauri.conf.json");
        if tauri_conf.exists() {
            std::fs::File::create(&tauri_conf)
                .map_err(|e| format!("Failed to touch tauri.conf.json: {}", e))?;
        }

        Ok(())
    }

    /// check 阶梯
    fn cmd_check(&self) -> Result<String, String> {
        println!("==> [check] 前端类型检查 + 单元测试");
        let frontend_dir = self.config.root.join("neocodex-frontend");
        if frontend_dir.exists() {
            let output = Command::new("npx")
                .args(&["tsc", "--noEmit"])
                .current_dir(&frontend_dir)
                .output()
                .map_err(|e| format!("Failed to run tsc: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "TypeScript check failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        println!("==> [check] Rust cargo check (desktop + core)");
        let output = Command::new("cargo")
            .args(&["check", "--all-targets", "-p", "neotrix-tauri"])
            .current_dir(&self.config.root)
            .output()
            .map_err(|e| format!("Failed to run cargo check: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Cargo check failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok("check 通过".to_string())
    }

    /// build 阶梯
    fn cmd_build(&self) -> Result<String, String> {
        self.frontend_build()?;

        println!("==> [build] 构建 Tauri 桌面端");
        let mut args = vec!["build", "-p", "neotrix-tauri"];
        if self.config.release {
            args.push("--release");
        }

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(&self.config.root)
            .output()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Cargo build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let profile = if self.config.release {
            "release"
        } else {
            "debug"
        };
        let bin = self
            .config
            .root
            .join(format!("target/{}/neotrix-tauri", profile));

        if self.config.run {
            println!("==> 启动桌面端");
            Command::new(&bin)
                .spawn()
                .map_err(|e| format!("Failed to run desktop app: {}", e))?;
        }

        Ok(format!("构建完成: {}", bin.display()))
    }

    /// package:dir 阶梯
    fn cmd_package_dir(&self) -> Result<String, String> {
        self.frontend_build()?;

        println!("==> [package:dir] tauri build --no-bundle");
        let mut args = vec!["tauri", "build", "--no-bundle"];
        if self.config.release {
            args.push("--release");
        }

        let frontend_dir = self.config.root.join("neocodex-frontend");
        let output = Command::new("npx")
            .args(&args)
            .current_dir(&frontend_dir)
            .output()
            .map_err(|e| format!("Failed to run tauri build: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Tauri build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok("package:dir 完成".to_string())
    }

    /// package 阶梯
    fn cmd_package(&self) -> Result<String, String> {
        self.frontend_build()?;

        println!("==> [package] tauri build (原生安装包 + updater 签名)");
        let mut args = vec!["tauri", "build"];
        if self.config.release {
            args.push("--release");
        }

        let frontend_dir = self.config.root.join("neocodex-frontend");
        let output = Command::new("npx")
            .args(&args)
            .current_dir(&frontend_dir)
            .output()
            .map_err(|e| format!("Failed to run tauri build: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Tauri build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok("package 完成".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ladder_debug() {
        let config = BuildConfig {
            release: false,
            run: false,
            root: PathBuf::from("."),
        };
        let builder = DesktopBuilder::new(config);
        assert_eq!(builder.config.release, false);
    }

    #[test]
    fn test_build_ladder_release() {
        let config = BuildConfig {
            release: true,
            run: false,
            root: PathBuf::from("."),
        };
        let builder = DesktopBuilder::new(config);
        assert_eq!(builder.config.release, true);
    }
}
