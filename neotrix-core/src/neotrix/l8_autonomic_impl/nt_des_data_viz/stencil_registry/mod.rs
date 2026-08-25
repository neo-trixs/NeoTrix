//! Stencil Registry — markdown-viewer/skills 9514 stencils 吸收
//! 
//! 60 类别 mxgraph stencil 索引，支持 PlantUML 渲染

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Stencil 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stencil {
    pub category: String,
    pub name: String,
    pub alias: String,
    pub label: String,
    pub needs_fill_color: bool,
    pub file_path: Option<PathBuf>,
}

/// Stencil 类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StencilCategory {
    pub name: String,
    pub count: usize,
    pub needs_fill_color_count: usize,
    pub stencils: Vec<Stencil>,
}

/// Stencil 注册表
#[derive(Debug)]
pub struct StencilRegistry {
    categories: HashMap<String, StencilCategory>,
    all_stencils: HashMap<String, Stencil>, // "category::name" -> Stencil
}

impl StencilRegistry {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
            all_stencils: HashMap::new(),
        }
    }

    /// 从 markdown-viewer 目录加载
    pub fn load_from_markdown_viewer(&mut self, base_path: &PathBuf) -> Result<(), String> {
        let stencils_dir = base_path.join("uml").join("stencils");
        if !stencils_dir.exists() {
            return Err("Stencils directory not found".to_string());
        }

        // 预定义的类别映射 (来自 README.md)
        let category_files = vec![
            ("alibaba_cloud", 310), ("android", 49), ("archimate", 10), ("archimate3", 42),
            ("arrows", 34), ("atlassian", 27), ("aws", 99), ("aws2", 250), ("aws3", 293),
            ("aws3d", 67), ("aws4", 1034), ("azure", 89), ("basic", 30), ("bootstrap", 4),
            ("bpmn", 183), ("cabinets", 57), ("cisco", 296), ("cisco_safe", 485),
            ("cisco19", 233), ("citrix", 97), ("citrix2", 126), ("dfd", 6),
            ("eip", 42), ("electrical", 642), ("floorplan", 75), ("flowchart", 35),
            ("fluid_power", 246), ("gcp", 66), ("gcp2", 322), ("gmdl", 104),
            ("ibm", 8), ("ibm_cloud", 110), ("infographic", 26), ("ios7", 168),
            ("kubernetes", 41), ("kubernetes2", 39), ("lean_mapping", 38), ("mockup", 104),
            ("mscae", 368), ("networks", 58), ("networks2", 115), ("office", 449),
            ("openstack", 18), ("pid", 479), ("pid2inst", 24), ("pid2misc", 10),
            ("pid2valves", 34), ("rack", 487), ("rackGeneral", 6), ("salesforce", 96),
            ("sap", 1), ("signs", 369), ("sitemap", 50), ("sysml", 29), ("uml25", 6),
            ("veeam", 333), ("veeam2", 247), ("vvd", 94), ("webicons", 176), ("weblogos", 178),
        ];

        for (name, count) in category_files {
            let file_path = stencils_dir.join(format!("{}.md", name));
            if file_path.exists() {
                let category = self.parse_stencil_file(&file_path, name, count)?;
                self.categories.insert(name.to_string(), category.clone());
                for stencil in &category.stencils {
                    self.all_stencils.insert(format!("{}::{}", name, stencil.name), stencil.clone());
                }
            }
        }

        Ok(())
    }

    fn parse_stencil_file(&self, path: &PathBuf, category: &str, expected_count: usize) -> Result<StencilCategory, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut stencils = Vec::new();
        
        // 解析 markdown 表格格式
        for line in content.lines() {
            if line.starts_with("| ") && line.contains(" | ") && !line.contains("---") && !line.contains("Category") {
                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    let name = parts[1].to_string();
                    let alias = format!("{}_{}", category, name.replace('-', "_"));
                    let label = parts[2].to_string();
                    let needs_fill = parts.get(3).map(|s| !s.trim().is_empty()).unwrap_or(false);
                    
                    stencils.push(Stencil {
                        category: category.to_string(),
                        name: name.clone(),
                        alias,
                        label,
                        needs_fill_color: needs_fill,
                        file_path: Some(path.clone()),
                    });
                }
            }
        }

        Ok(StencilCategory {
            name: category.to_string(),
            count: stencils.len(),
            needs_fill_color_count: stencils.iter().filter(|s| s.needs_fill_color).count(),
            stencils,
        })
    }

    /// 搜索 stencil
    pub fn search(&self, query: &str) -> Vec<&Stencil> {
        let query_lower = query.to_lowercase();
        self.all_stencils.values()
            .filter(|s| s.name.to_lowercase().contains(&query_lower) || s.label.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// 按类别获取
    pub fn by_category(&self, category: &str) -> Option<&StencilCategory> {
        self.categories.get(category)
    }

    /// 获取所有类别
    pub fn categories(&self) -> Vec<&StencilCategory> {
        self.categories.values().collect()
    }

    /// 生成 PlantUML stencil 引用代码
    pub fn plantuml_ref(&self, category: &str, name: &str, label: &str, alias: &str) -> String {
        format!("mxgraph.{}.{} \"{}\" as {}", category, name, label, alias)
    }

    /// 统计信息
    pub fn stats(&self) -> StencilRegistryStats {
        let total_stencils: usize = self.categories.values().map(|c| c.count).sum();
        let total_categories = self.categories.len();
        
        StencilRegistryStats {
            total_stencils,
            total_categories,
            by_category: self.categories.iter()
                .map(|(k, v)| (k.clone(), v.count))
                .collect(),
        }
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let mut registry = StencilRegistry::new();
        
        // Test with minimal mock data
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let stencils_dir = temp_dir.path().join("uml").join("stencils");
        std::fs::create_dir_all(&stencils_dir).map_err(|e| e.to_string())?;
        
        // Create a mock stencil file
        std::fs::write(
            stencils_dir.join("test.md"),
            "| Name | Label | Needs Fill |\n|------|-------|------------|\n| test-icon | Test Icon | yes |\n| another | Another Icon | no |\n"
        ).map_err(|e| e.to_string())?;
        
        let category = registry.parse_stencil_file(&stencils_dir.join("test.md"), "test", 2)?;
        assert_eq!(category.name, "test");
        assert_eq!(category.count, 2);
        assert_eq!(category.stencils[0].name, "test-icon");
        assert!(category.stencils[0].needs_fill_color);
        assert!(!category.stencils[1].needs_fill_color);
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StencilRegistryStats {
    pub total_stencils: usize,
    pub total_categories: usize,
    pub by_category: HashMap<String, usize>,
}

impl Default for StencilRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = StencilRegistry::new();
        assert_eq!(registry.categories.len(), 0);
    }

    #[test]
    fn test_parse_stencil_file() {
        let registry = StencilRegistry::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("test.md");
        std::fs::write(&file, "| Name | Label | Needs Fill |\n|------|-------|------------|\n| icon1 | Icon 1 | yes |\n| icon2 | Icon 2 | no |\n").unwrap();
        
        let category = registry.parse_stencil_file(&file, "test", 2).unwrap();
        assert_eq!(category.count, 2);
        assert!(category.stencils[0].needs_fill_color);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(StencilRegistry::self_test().is_ok());
    }
}