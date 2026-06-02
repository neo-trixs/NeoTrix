//! SelfEvolver - 自我进化器
//! 从外部链接/信息中自动提取知识，自我迭代优化 ReasoningBrain

use crate::neotrix::nt_core_error::{NeoTrixError, NeoTrixResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;
use serde::{Deserialize, Serialize};

use super::self_edit::MicroEdit;
use super::self_iterating::ReasoningBrain;
use super::memory::{ReasoningBank, ReasoningMemory};
use super::core::*;

/// 三流分析结果（Skill_Seekers 风格）
#[derive(Debug, Clone, Default)]
pub struct ThreeStreamAnalysis {
    /// 代码流: API surface, module graph, deps
    pub code_insights: Vec<String>,
    pub code_edits: Vec<MicroEdit>,
    /// 文档流: mental model, architecture patterns
    pub docs_insights: Vec<String>,
    pub docs_edits: Vec<MicroEdit>,
    /// 洞察流: cross-references, gaps, conflicts
    pub cross_references: Vec<String>,
    pub gap_analysis: Vec<String>,
}

/// One row in the comparison matrix
#[derive(Debug, Clone)]
pub struct ComparedItem {
    pub name: String,
    pub dimension_scores: HashMap<String, f64>,
    pub evidence: Vec<String>,
}

/// Gap status between us and competitor
#[derive(Debug, Clone, PartialEq)]
pub enum GapStatus {
    Has,
    Missing,
    BothMissing,
    BothPresent,
}

/// A single gap analysis row
#[derive(Debug, Clone)]
pub struct GapRow {
    pub dimension: String,
    pub our_status: GapStatus,
    pub their_status: GapStatus,
    pub impact: f64,
    pub recommendation: Option<String>,
}

/// Full comparison matrix output
#[derive(Debug, Clone)]
pub struct ComparisonMatrix {
    pub dimensions: Vec<String>,
    pub items: Vec<ComparedItem>,
    pub gap_analysis: Vec<GapRow>,
}

impl Default for ComparisonMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparisonMatrix {
    pub fn new() -> Self {
        Self {
            dimensions: Vec::new(),
            items: Vec::new(),
            gap_analysis: Vec::new(),
        }
    }
}

/// SelfEvolver 主结构体
pub struct SelfEvolver {
    pub brain: ReasoningBrain,
    pub reasoning_bank: ReasoningBank,  // 注意：store() 需要 &mut self
    pub work_dir: PathBuf,
}

impl SelfEvolver {
    /// 创建新的 SelfEvolver
    pub fn new(brain: ReasoningBrain, reasoning_bank: ReasoningBank, work_dir: PathBuf) -> Self {
        Self { brain, reasoning_bank, work_dir }
    }
    
    /// 检测是否为 URL
    pub fn is_url(text: &str) -> bool {
        Url::parse(text).is_ok() && (text.starts_with("http://") || text.starts_with("https://"))
    }
    
    /// 从 URL 进化（主入口）
    pub fn evolve_from_url(&mut self, url: &str) -> NeoTrixResult<f64> {
        println!("🌐 evolve_from_url: {}", url);
        // 1. 信息获取
        let local_path = self.fetch_information(url)?;
        println!("📁 Local path: {}", local_path.display());
        
        // 2. 深度分析
        let analysis = self.analyze_local_path(&local_path)?;
        println!("📊 Analysis: type={}, confidence={}, edits_count={}", 
                 analysis.item_type, analysis.confidence, analysis.suggested_edits.len());
        
        // 3. 生成 MicroEdit 序列
        let micro_edits = self.generate_micro_edits(&analysis)?;
        println!("🔧 Micro edits count: {}", micro_edits.len());
        for (i, edit) in micro_edits.iter().enumerate() {
            println!("  Edit {}: {:?}", i, edit);
        }
        
        // 4. 应用到 ReasoningBrain
        println!("🧠 Before apply: quality_gates={:.3}", self.brain.capability.arr[IDX_QUALITY_GATES]);
        for edit in &micro_edits {
            self.apply_micro_edit(edit);
        }
        println!("🧠 After apply: quality_gates={:.3}", self.brain.capability.arr[IDX_QUALITY_GATES]);
        
        // 5. 保存到 brain.json
        let reward = self.calculate_reward(&analysis);
        println!("💾 Saving brain.json...");
        self.brain.save()?;
        println!("✅ Saved brain.json, reward={:.3}", reward);
        
        // 6. 存储到 ReasoningBank
        let memory = ReasoningMemory {
            id: uuid::Uuid::new_v4().to_string(),
            task_description: format!("Evolve from: {}", analysis.source_url),
            task_type: crate::neotrix::nt_world_model::TaskType::CodeAnalysis,
            micro_edits: micro_edits.clone(),
            reward,
            reward_source: crate::neotrix::nt_mind::core::RewardSource::External,
            success: reward > 0.5,
            timestamp: chrono::Utc::now().timestamp(),
            embedding: None,
            tier: crate::neotrix::nt_mind::memory::MemoryTier::Semantic,
            lifecycle: crate::neotrix::nt_mind::memory::MemoryLifecycle::new(reward),
            t3_views: crate::core::nt_core_bank::T3Views::new(),
        };
        self.reasoning_bank.store(memory);
        
        Ok(reward)
    }
    
    /// 信息获取（克隆 GitHub repo 或爬取网页）
    fn fetch_information(&self, url: &str) -> NeoTrixResult<PathBuf> {
        let parsed = Url::parse(url).map_err(|e| NeoTrixError::Network(format!("无效URL: {}", e)))?;
        
        if parsed.domain() == Some("github.com") {
            let path_segments: Vec<&str> = parsed.path().trim_matches('/').split('/').collect();
            let repo_name = if path_segments.len() >= 2 {
                format!("{}-{}", path_segments[path_segments.len()-2], path_segments[path_segments.len()-1])
            } else {
                path_segments.last().unwrap_or(&"unknown").to_string()
            };
            let target_dir = self.work_dir.join(repo_name);
            
            if !target_dir.exists() {
                if url.contains(';') || url.contains('`') || url.contains('$') || url.contains('|') || url.contains('&') {
                    return Err(NeoTrixError::from("URL 包含非法字符"));
                }
                let target_str = target_dir.to_str().ok_or_else(|| NeoTrixError::from("路径不是有效 UTF-8"))?;
                let status = Command::new("git")
                    .args(["clone", "--depth=1", "--", url, target_str])
                    .status()
                    .map_err(NeoTrixError::Io)?;
                if !status.success() {
                    return Err(NeoTrixError::from("Git 克隆失败"));
                }
            }
            Ok(target_dir)
        } else {
            let content = Self::fetch_http(url)?;
            let file_name = parsed.path()
                .trim_end_matches('/')
                .split('/')
                .next_back()
                .filter(|s: &&str| !s.is_empty())
                .unwrap_or("page");
            let target_file = self.work_dir.join(format!("{}.html", file_name));
            std::fs::write(&target_file, content)?;
            Ok(target_file)
        }
    }
    
    /// 使用 reqwest 获取 HTTP 内容（替代 curl 子进程，防注入）
    fn fetch_http(url: &str) -> NeoTrixResult<String> {
        let parsed = Url::parse(url).map_err(|_| NeoTrixError::from("无效 URL"))?;
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(NeoTrixError::from("仅支持 http/https"));
        }
        let agent = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; NeoTrix/1.0)")
            .build()
            .map_err(|e| NeoTrixError::Network(format!("创建 HTTP 客户端失败: {}", e)))?;
        let resp = agent.get(url)
            .send()
            .map_err(|e| NeoTrixError::Network(format!("请求失败: {}", e)))?;
        resp.text().map_err(|e| NeoTrixError::Network(format!("读取响应失败: {}", e)))
    }

    /// 深度分析（三流并行：Code + Docs + Insights）
    fn analyze_local_path(&self, path: &PathBuf) -> NeoTrixResult<AnalysisResult> {
        let mut item_type = "unknown".to_string();
        let mut confidence = 0.7;
        
        if path.is_dir() {
            item_type = "repo".to_string();
            
            // 三流并行分析
            let streams = self.analyze_three_streams(path)?;
            
            // 置信度 = 流的覆盖度（在 move 之前检查）
            let has_code = !streams.code_insights.is_empty();
            let has_docs = !streams.docs_insights.is_empty();
            let has_xref = !streams.cross_references.is_empty();
            let stream_count = [has_code, has_docs, has_xref].iter().filter(|&&b| b).count();
            confidence = 0.5 + (stream_count as f64 * 0.15).min(0.45);
            
            // 合并三个流的结果
            let mut all_insights = streams.code_insights.clone();
            all_insights.extend(streams.docs_insights);
            all_insights.extend(streams.cross_references);
            
            let mut all_edits = streams.code_edits.clone();
            all_edits.extend(streams.docs_edits);
            
            // 添加 NormalizeVector
            all_edits.push(MicroEdit::NormalizeVector);
            
            return Ok(AnalysisResult {
                source_url: path.to_string_lossy().to_string(),
                item_type,
                algebraic_insights: all_insights,
                suggested_edits: all_edits,
                impact_weights: vec![],
                confidence,
            });
        }
        
        // 文件分析（回退到单流文档分析）
        if path.is_file() {
            item_type = "file".to_string();
            if let Some(ext) = path.extension() {
                if ext == "md" || ext == "html" {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let (edits, ins) = Self::analyze_docs_stream(&content);
                        let mut all = edits.clone();
                        all.push(MicroEdit::NormalizeVector);
                        return Ok(AnalysisResult {
                            source_url: path.to_string_lossy().to_string(),
                            item_type,
                            algebraic_insights: ins,
                            suggested_edits: all,
                            impact_weights: vec![],
                            confidence: 0.8,
                        });
                    }
                }
            }
        }
        
        Ok(AnalysisResult {
            source_url: path.to_string_lossy().to_string(),
            item_type,
            algebraic_insights: vec![],
            suggested_edits: vec![MicroEdit::NormalizeVector],
            impact_weights: vec![],
            confidence,
        })
    }

    /// 三流分析核心：Code + Docs + Insights（Skill_Seekers 风格）
    fn analyze_three_streams(&self, repo_path: &Path) -> NeoTrixResult<ThreeStreamAnalysis> {
        let mut analysis = ThreeStreamAnalysis::default();
        
        // Stream 1: Code Analysis
        self.analyze_code_stream(repo_path, &mut analysis)?;
        
        // Stream 2: Docs Analysis  
        let docs_paths = self.find_docs(repo_path);
        for doc_path in docs_paths {
            if let Ok(content) = std::fs::read_to_string(&doc_path) {
                let (edits, insights) = Self::analyze_docs_stream(&content);
                analysis.docs_insights.extend(insights);
                analysis.docs_edits.extend(edits);
            }
        }
        
        // Stream 3: Insights (cross-reference code + docs)
        let analysis_clone = analysis.clone();
        self.analyze_insights_stream(&analysis_clone, &mut analysis);
        
        Ok(analysis)
    }

    /// Code Stream: 扫描源码提取模块结构、API surface、依赖
    fn analyze_code_stream(&self, repo_path: &Path, analysis: &mut ThreeStreamAnalysis) -> NeoTrixResult<()> {
        // 1. 检测包管理器文件
        let cargo_path = repo_path.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                analysis.code_insights.push("Rust project (Cargo.toml)".to_string());
                if content.contains("tokio") {
                    analysis.code_edits.push(MicroEdit::AdjustDimension("inference_depth".to_string(), 0.08));
                    analysis.code_insights.push("Async runtime: tokio".to_string());
                }
                if content.contains("wgpu") || content.contains("vulkano") {
                    analysis.code_edits.push(MicroEdit::AdjustDimension("experimental".to_string(), 0.07));
                }
            }
        }
        
        let pkg_path = repo_path.join("package.json");
        if pkg_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                analysis.code_insights.push("Node.js project".to_string());
                if content.contains("tailwindcss") {
                    analysis.code_edits.push(MicroEdit::AdjustDimension("tailwind_proficiency".to_string(), 0.10));
                }
                if content.contains("react-aria") || content.contains("radix-ui") {
                    analysis.code_edits.push(MicroEdit::AdjustDimension("react_aria_usage".to_string(), 0.10));
                }
            }
        }
        
        // 2. 扫描 src/ 目录统计模块数
        let src_path = repo_path.join("src");
        if src_path.is_dir() {
            let mut module_count = 0;
            if let Ok(entries) = std::fs::read_dir(&src_path) {
                for entry in entries.flatten() {
                    if entry.path().extension().is_some_and(|e| e == "rs" || e == "ts" || e == "tsx" || e == "py") {
                        module_count += 1;
                    }
                }
            }
            analysis.code_insights.push(format!("Source modules: {}", module_count));
            if module_count > 20 {
                analysis.code_edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.10));
            }
            if module_count == 0 {
                analysis.code_insights.push("No source files found in src/".to_string());
            }
        }
        
        // 3. 检测 Python 项目
        if repo_path.join("requirements.txt").exists() || repo_path.join("setup.py").exists() || repo_path.join("pyproject.toml").exists() {
            analysis.code_insights.push("Python project".to_string());
        }
        
        Ok(())
    }
    
    /// Docs Stream: 深度文档分析（增强版原 analyze_skill_system）
    fn analyze_docs_stream(content: &str) -> (Vec<MicroEdit>, Vec<String>) {
        let mut edits = vec![];
        let mut insights = vec![];
        let c = content.to_lowercase();
        
        // CodeStable
        if c.contains("human-in-the-loop") || c.contains("人在环") {
            edits.push(MicroEdit::AdjustDimension("quality_gates".to_string(), 0.15));
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.12));
            insights.push("CodeStable: 人在环设计".to_string());
        }
        
        // Agent skills
        if c.contains("agent-skills") || (c.contains("production-grade") && c.contains("skill")) {
            edits.push(MicroEdit::AdjustDimension("quality_gates".to_string(), 0.18));
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.15));
            insights.push("agent-skills: 生产级技能系统".to_string());
        }
        
        // MCP / Copilot
        if c.contains("mcp") || c.contains("copilot") {
            edits.push(MicroEdit::AdjustDimension("semantic_layer".to_string(), 0.12));
            edits.push(MicroEdit::AdjustDimension("ai_native_states".to_string(), 0.10));
            insights.push("MCP/Copilot: AI工具集成".to_string());
        }
        
        // Multi-model / design tools
        if c.contains("multi-model") || c.contains("claude design") {
            edits.push(MicroEdit::AdjustDimension("figma_integration".to_string(), 0.15));
            edits.push(MicroEdit::AdjustDimension("ai_native_states".to_string(), 0.12));
            insights.push("Multi-model: 设计工具集成".to_string());
        }
        
        // Security focus
        if c.contains("nt_shield") || c.contains("oauth") || c.contains("jwt") || c.contains("authentication") {
            edits.push(MicroEdit::AdjustDimension("quality_gates".to_string(), 0.08));
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.06));
            insights.push("Security: 认证授权机制".to_string());
        }
        
        // Mobile
        if c.contains("ios") || c.contains("android") || c.contains("mobile") {
            edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.09));
            insights.push("Mobile: 跨平台移动支持".to_string());
        }
        
        // Database
        if c.contains("sql") || c.contains("database") || c.contains("postgres") || c.contains("redis") {
            edits.push(MicroEdit::AdjustDimension("synthesis".to_string(), 0.06));
            insights.push("Database: 数据持久化层".to_string());
        }
        
        // Design system
        if c.contains("design system") || c.contains("design token") {
            edits.push(MicroEdit::AdjustDimension("compound_composition".to_string(), 0.09));
            edits.push(MicroEdit::AdjustDimension("semantic_layer".to_string(), 0.07));
            insights.push("Design System: 设计Token体系".to_string());
        }
        
        // CLI tools
        if c.contains("cli") && (c.contains("command") || c.contains("terminal")) {
            edits.push(MicroEdit::AdjustDimension("domain_specificity".to_string(), 0.08));
            insights.push("CLI: 命令行工具".to_string());
        }
        
        // Testing
        if c.contains("test") && (c.contains("tdd") || c.contains("coverage") || c.contains("pytest")) {
            edits.push(MicroEdit::AdjustDimension("verification".to_string(), 0.10));
            insights.push("Testing: 测试驱动开发".to_string());
        }
        
        (edits, insights)
    }
    
    /// 查找文档文件
    fn find_docs(&self, repo_path: &Path) -> Vec<PathBuf> {
        let mut docs = Vec::new();
        let readme_candidates = ["README.md", "README.en.md", "README.zh-CN.md", "README.txt"];
        for name in &readme_candidates {
            let p = repo_path.join(name);
            if p.exists() { docs.push(p); break; }
        }
        let docs_dir = repo_path.join("docs");
        if docs_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&docs_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().is_some_and(|e| e == "md") {
                        docs.push(p);
                    }
                }
            }
        }
        docs
    }
    
    /// Insights Stream: 交叉验证 Code + Docs 发现矛盾/差距
    fn analyze_insights_stream(&self, current: &ThreeStreamAnalysis, out: &mut ThreeStreamAnalysis) {
        // 1. 文档描述了但代码没有的功能
        if current.docs_insights.is_empty() && !current.code_insights.is_empty() {
            out.cross_references.push("Doc gap: 项目有代码但缺少文档".to_string());
        }
        
        // 2. 代码中没有的文档声称功能
        if !current.docs_insights.is_empty() && current.code_insights.is_empty() {
            out.cross_references.push("Code gap: 文档描述的功能在代码中未找到".to_string());
        }
        
        // 3. 测试覆盖分析
        if current.code_insights.iter().any(|i| i.contains("Source modules")) {
            out.cross_references.push("Code analysis complete: module structure detected".to_string());
        }
        
        // 4. 多语言项目
        let has_rust = current.code_insights.iter().any(|i| i.contains("Rust"));
        let has_python = current.code_insights.iter().any(|i| i.contains("Python"));
        let has_node = current.code_insights.iter().any(|i| i.contains("Node"));
        
        if [has_rust, has_python, has_node].iter().filter(|&&b| b).count() > 1 {
            out.gap_analysis.push("Multi-language: 跨语言项目需要统一API规范".to_string());
        }
    }
    // ---- end of analyze_insights_stream ----

    /// 生成 MicroEdit 序列
    fn generate_micro_edits(&self, analysis: &AnalysisResult) -> NeoTrixResult<Vec<MicroEdit>> {
        Ok(analysis.suggested_edits.clone())
    }
    
    /// 应用 MicroEdit 到 ReasoningBrain
    fn apply_micro_edit(&mut self, edit: &MicroEdit) {
        match edit {
            MicroEdit::AdjustDimension(dim, delta) => {
                if let Some(idx) = CapabilityVector::index_from_name(dim) {
                    let val = &mut self.brain.capability.arr[idx];
                    let old_val = *val;
                    *val = (*val + delta).clamp(0.0, 1.0);
                    println!("🔧 AdjustDimension: dim={}, idx={}, old={:.3}, delta={:.3}, new={:.3}", dim, idx, old_val, delta, *val);
                } else {
                    println!("❌ AdjustDimension: unknown dim={}", dim);
                }
            }
            MicroEdit::NormalizeVector => {
                println!("🔧 NormalizeVector: before quality_gates={:.3}", self.brain.capability.arr[IDX_QUALITY_GATES]);
                self.brain.capability.normalize();
                println!("🔧 NormalizeVector: after quality_gates={:.3}", self.brain.capability.arr[IDX_QUALITY_GATES]);
            }
            _ => {}
        }
    }

    /// 计算奖励（基于分析置信度和改进程度）
    fn calculate_reward(&self, analysis: &AnalysisResult) -> f64 {
        analysis.confidence * 0.9
    }

    /// 从分析结果生成 MCP 工具配置
    pub fn generate_mcp_tools(&self, analysis: &AnalysisResult) -> Vec<String> {
        let mut tools = Vec::new();
        for insight in &analysis.algebraic_insights {
            let tool_name = format!("apply_{}_insight", analysis.item_type.replace('-', "_"));
            let config = serde_json::json!({
                "tool": tool_name,
                "source": analysis.source_url,
                "insight": insight,
                "action": "apply_capability_adjustment",
                "params": {
                    "dimensions": analysis.suggested_edits.iter().filter_map(|e| {
                        if let MicroEdit::AdjustDimension(dim, _) = e {
                            Some(dim)
                        } else { None }
                    }).collect::<Vec<&String>>(),
                    "confidence": analysis.confidence,
                }
            });
            tools.push(serde_json::to_string_pretty(&config).unwrap_or_default());
        }
        tools
    }
}

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub source_url: String,
    pub item_type: String,
    pub algebraic_insights: Vec<String>,
    pub suggested_edits: Vec<MicroEdit>,
    pub impact_weights: Vec<(String, f64)>,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
use std::path::PathBuf;
    use crate::neotrix::nt_mind::self_evolver::ComparedItem;
    use crate::neotrix::nt_mind::self_evolver::ComparisonMatrix;
    use crate::neotrix::nt_mind::self_evolver::GapRow;
    use crate::neotrix::nt_mind::self_evolver::GapStatus;
    use crate::neotrix::nt_mind::self_evolver::SelfEvolver;
    use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;
    use crate::neotrix::nt_mind::memory::ReasoningBank;
    #[test]
    fn test_is_url() {
        assert!(SelfEvolver::is_url("https://github.com/foo/bar"));
        assert!(SelfEvolver::is_url("http://example.com"));
        assert!(!SelfEvolver::is_url("not a url"));
        assert!(!SelfEvolver::is_url("README.md"));
    }
    
    #[test]
    fn test_evolve_all_repos() {
        let brain = ReasoningBrain::new();
        let bank = ReasoningBank::new(100);
        let work_dir = PathBuf::from("/Users/neo/.neotrix/work");
        let mut evolver = SelfEvolver::new(brain, bank, work_dir);
        
        let urls = vec![
            "https://github.com/liuzhengdongfortest/CodeStable",
            "https://github.com/addyosmani/agent-skills",
            "https://github.com/jackwener/OpenCLI",
            "https://github.com/warpdotdev/oz-skills",
            "https://github.com/OpenCoworkAI/open-codesign",
            "https://github.com/CopilotKit/CopilotKit",
        ];
        
        for url in urls {
            match evolver.evolve_from_url(url) {
                Ok(reward) => println!("✅ Evolved from {}: reward={:.3}", url, reward),
                Err(e) => println!("❌ Failed to evolve from {}: {}", url, e),
            }
        }
    }

    #[test]
    fn test_compared_item_creation() {
        let mut scores = std::collections::HashMap::new();
        scores.insert("nt_shield".to_string(), 0.9);
        scores.insert("performance".to_string(), 0.7);
        let item = ComparedItem {
            name: "test-repo".to_string(),
            dimension_scores: scores,
            evidence: vec!["strong nt_shield".to_string()],
        };
        assert_eq!(item.name, "test-repo");
        assert!((item.dimension_scores.get("nt_shield").copied().unwrap_or(0.0) - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_gap_status_partial_eq() {
        assert_eq!(GapStatus::Has, GapStatus::Has);
        assert_ne!(GapStatus::Has, GapStatus::Missing);
    }

    #[test]
    fn test_comparison_matrix_new() {
        let m = ComparisonMatrix::new();
        assert!(m.dimensions.is_empty());
        assert!(m.items.is_empty());
        assert!(m.gap_analysis.is_empty());
    }

    #[test]
    fn test_gap_row_creation() {
        let gap = GapRow {
            dimension: "nt_shield".to_string(),
            our_status: GapStatus::Missing,
            their_status: GapStatus::Has,
            impact: 0.8,
            recommendation: Some("Add nt_shield audit".to_string()),
        };
        assert_eq!(gap.dimension, "nt_shield");
        assert!(gap.impact > 0.5);
        assert!(gap.recommendation.is_some());
    }

    #[test]
    fn test_moscow_class_default() {
        let _ = ComparisonMatrix::new();
    }
}
