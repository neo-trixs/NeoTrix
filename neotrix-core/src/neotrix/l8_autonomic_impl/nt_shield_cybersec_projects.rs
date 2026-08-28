//! NT-SHIELD Cybersecurity Projects — 安全能力目录
//!
//! 吸收源: github.com/CarterPerez-dev/Cybersecurity-Projects
//! 能力: 网络安全项目集合 → 分类安全项目/工具 stub + 目录结构。
//!
//! 这是对吸收源的模式吸收 (C1 成熟度): trait + 分类目录模型 stub，
//! 按领域分类列举安全项目 (CTF/取证/防御/红队/密码学/Web 安全等)。
//! 编译通过即可 (R-P1: unsafe 禁用)。

use crate::core::nt_core_self_test::SelfTest;

/// 安全项目领域分类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityCategory {
    /// 捕获旗帜 / 竞赛训练
    Ctf,
    /// 数字取证
    Forensics,
    /// 防御 / 蓝队
    Defense,
    /// 红队 / 渗透
    RedTeam,
    /// 密码学
    Cryptography,
    /// Web 安全
    Web,
    /// 其他 / 未分类
    Other,
}

impl SecurityCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityCategory::Ctf => "ctf",
            SecurityCategory::Forensics => "forensics",
            SecurityCategory::Defense => "defense",
            SecurityCategory::RedTeam => "red_team",
            SecurityCategory::Cryptography => "cryptography",
            SecurityCategory::Web => "web",
            SecurityCategory::Other => "other",
        }
    }

    /// 全部已知分类 (用于目录遍历/校验)。
    pub fn all() -> &'static [SecurityCategory] {
        &[
            SecurityCategory::Ctf,
            SecurityCategory::Forensics,
            SecurityCategory::Defense,
            SecurityCategory::RedTeam,
            SecurityCategory::Cryptography,
            SecurityCategory::Web,
            SecurityCategory::Other,
        ]
    }
}

/// 单个安全能力条目 (stub)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityProject {
    pub name: String,
    pub category: SecurityCategory,
    pub description: String,
}

impl SecurityProject {
    pub fn new(name: &str, category: SecurityCategory, description: &str) -> Self {
        Self {
            name: name.to_string(),
            category,
            description: description.to_string(),
        }
    }
}

/// 安全能力目录核心 trait — 分类登记/检索安全项目。
pub trait SecurityCatalog {
    /// 登记一个安全项目。
    fn register(&mut self, project: SecurityProject);
    /// 按分类检索项目列表。
    fn by_category(&self, cat: &SecurityCategory) -> Vec<&SecurityProject>;
    /// 目录中项目总数。
    fn count(&self) -> usize;
}

/// 默认安全能力目录实现: 内存分类索引。
#[derive(Debug, Default)]
pub struct CyberSecCatalog {
    projects: Vec<SecurityProject>,
}

impl CyberSecCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// 种子目录: 从吸收源归纳的代表性安全项目分类。
    pub fn with_seed() -> Self {
        let mut c = Self::new();
        c.register(SecurityProject::new(
            "CTF-Playbook",
            SecurityCategory::Ctf,
            "竞赛训练剧本与解题框架集",
        ));
        c.register(SecurityProject::new(
            "Forensic-Toolkit",
            SecurityCategory::Forensics,
            "内存/磁盘/网络取证工具集",
        ));
        c.register(SecurityProject::new(
            "Blue-Defense",
            SecurityCategory::Defense,
            "日志监控与入侵检测基线",
        ));
        c.register(SecurityProject::new(
            "Red-Operations",
            SecurityCategory::RedTeam,
            "授权渗透与攻击面枚举",
        ));
        c.register(SecurityProject::new(
            "Crypto-Lab",
            SecurityCategory::Cryptography,
            "加解密与协议分析实验",
        ));
        c.register(SecurityProject::new(
            "Web-Guard",
            SecurityCategory::Web,
            "Web 漏洞扫描与加固",
        ));
        c
    }
}

impl SecurityCatalog for CyberSecCatalog {
    fn register(&mut self, project: SecurityProject) {
        self.projects.push(project);
    }

    fn by_category(&self, cat: &SecurityCategory) -> Vec<&SecurityProject> {
        self.projects.iter().filter(|p| &p.category == cat).collect()
    }

    fn count(&self) -> usize {
        self.projects.len()
    }
}

impl SelfTest for CyberSecCatalog {
    fn name(&self) -> &str {
        "nt_shield_cybersec_projects"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if SecurityCategory::all().is_empty() {
            failures.push("SecurityCategory::all() is empty".to_string());
        }
        if self.count() == 0 {
            failures.push("catalog has no seeded projects".to_string());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_registers_and_counts() {
        let mut c = CyberSecCatalog::new();
        c.register(SecurityProject::new("X", SecurityCategory::Web, "demo"));
        assert_eq!(c.count(), 1);
    }

    #[test]
    fn catalog_filters_by_category() {
        let c = CyberSecCatalog::with_seed();
        let web = c.by_category(&SecurityCategory::Web);
        assert_eq!(web.len(), 1);
        assert_eq!(web[0].name, "Web-Guard");
    }

    #[test]
    fn category_str_roundtrip() {
        for cat in SecurityCategory::all() {
            assert!(!cat.as_str().is_empty());
        }
        assert_eq!(CyberSecCatalog::with_seed().count(), 6);
    }
}
