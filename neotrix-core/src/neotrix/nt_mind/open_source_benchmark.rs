use super::core::CapabilityVector;
use super::self_edit::MicroEdit;
use crate::neotrix::nt_expert_routing::TaskType;

pub struct BenchmarkReport {
    pub project_name: String,
    pub relevance_score: f64,
    pub gap_areas: Vec<(usize, f64, String)>,
    pub suggested_capability_delta: Vec<(usize, f64)>,
    pub summary: String,
}

struct ProjectProfile {
    name: &'static str,
    domain: &'static str,
    task_types: &'static [TaskType],
    capability_profile: &'static [(usize, f64)],
    url: &'static str,
    relevance: f64,
}

static BENCHMARK_PROJECTS: &[ProjectProfile] = &[
    ProjectProfile {
        name: "Obscura",
        domain: "anti-detection-http",
        task_types: &[TaskType::CodeAnalysis, TaskType::CodeGeneration],
        capability_profile: &[(1, 0.95), (3, 0.90), (5, 0.88), (7, 0.85)],
        url: "https://github.com/Obscura",
        relevance: 0.9,
    },
    ProjectProfile {
        name: "ProxyChains-NG",
        domain: "proxy-chain",
        task_types: &[TaskType::Security, TaskType::CodeAnalysis],
        capability_profile: &[(2, 0.92), (4, 0.90), (6, 0.85), (10, 0.88)],
        url: "https://github.com/rofl0r/proxychains-ng",
        relevance: 0.85,
    },
    ProjectProfile {
        name: "curl-impersonate",
        domain: "tls-fingerprint",
        task_types: &[TaskType::CodeAnalysis, TaskType::Security],
        capability_profile: &[(0, 0.96), (8, 0.92), (11, 0.90)],
        url: "https://github.com/lwthiker/curl-impersonate",
        relevance: 0.88,
    },
    ProjectProfile {
        name: "MemOS",
        domain: "memory-management",
        task_types: &[TaskType::Design, TaskType::Planning],
        capability_profile: &[(12, 0.94), (14, 0.92), (15, 0.90), (18, 0.88)],
        url: "https://github.com/ai-forever/mem-os",
        relevance: 0.82,
    },
    ProjectProfile {
        name: "dbskill",
        domain: "multi-dim-retrieval",
        task_types: &[
            TaskType::CodeAnalysis,
            TaskType::CodeGeneration,
            TaskType::Planning,
        ],
        capability_profile: &[(13, 0.93), (16, 0.91), (17, 0.89)],
        url: "https://github.com/dbskill",
        relevance: 0.80,
    },
    ProjectProfile {
        name: "Mamba SSM",
        domain: "selective-state-space",
        task_types: &[TaskType::CodeGeneration, TaskType::General],
        capability_profile: &[(9, 0.95), (19, 0.92), (20, 0.90)],
        url: "https://github.com/state-spaces/mamba",
        relevance: 0.85,
    },
    ProjectProfile {
        name: "HeroUI",
        domain: "ui-design",
        task_types: &[TaskType::Design, TaskType::UIDesign],
        capability_profile: &[(0, 0.98), (1, 0.95)],
        url: "https://heroui.com",
        relevance: 0.92,
    },
    ProjectProfile {
        name: "V2Ray / Xray",
        domain: "proxy-core",
        task_types: &[TaskType::CodeAnalysis, TaskType::Security],
        capability_profile: &[(2, 0.96), (4, 0.94), (6, 0.92), (10, 0.90)],
        url: "https://github.com/xtls/xray-core",
        relevance: 0.95,
    },
    ProjectProfile {
        name: "arti (Tor Rust)",
        domain: "anon-network",
        task_types: &[TaskType::Security, TaskType::CodeGeneration],
        capability_profile: &[(5, 0.93), (7, 0.91)],
        url: "https://gitlab.torproject.org/tpo/core/arti",
        relevance: 0.87,
    },
    ProjectProfile {
        name: "Clash Meta",
        domain: "rule-based-proxy",
        task_types: &[TaskType::CodeAnalysis, TaskType::Security],
        capability_profile: &[(3, 0.95), (8, 0.93)],
        url: "https://github.com/MetaCubeX/Clash.Meta",
        relevance: 0.90,
    },
];

fn match_keyword(task: &str, task_type: &TaskType) -> Vec<(&'static ProjectProfile, usize)> {
    let task_lower = task.to_lowercase();
    let keywords = [
        "proxy",
        "tor",
        "dns",
        "tls",
        "fingerprint",
        "stealth",
        "memory",
        "ui",
        "design",
        "nt_shield",
        "rule",
        "mcp",
        "agent",
        "ssm",
        "routing",
        "chain",
        "evolve",
        "absorb",
        "rotation",
        "bandit",
        "select",
        "privacy",
        "detect",
        "circuit",
        "tunnel",
        "http",
        "socks",
        "vpn",
        "crypto",
        "anonym",
    ];
    BENCHMARK_PROJECTS
        .iter()
        .filter(|p| p.task_types.contains(task_type))
        .map(|p| {
            let score = keywords
                .iter()
                .filter(|&kw| {
                    task_lower.contains(kw)
                        || p.domain.contains(kw)
                        || p.name.to_lowercase().contains(kw)
                })
                .count()
                + if p.task_types.contains(task_type) {
                    2
                } else {
                    0
                };
            (p, score)
        })
        .filter(|(_, score)| *score > 0)
        .collect()
}

fn match_by_task_type(task_type: &TaskType) -> Vec<&'static ProjectProfile> {
    BENCHMARK_PROJECTS
        .iter()
        .filter(|p| p.task_types.contains(task_type))
        .collect()
}

pub struct OpenSourceBenchmarker;

impl OpenSourceBenchmarker {
    pub fn new() -> Self {
        Self
    }

    pub fn benchmark(
        &self,
        task: &str,
        task_type: TaskType,
        current_capability: &CapabilityVector,
    ) -> BenchmarkReport {
        let matched = match_keyword(task, &task_type);
        let default = match_by_task_type(&task_type);
        let target = matched
            .first()
            .map(|(p, _)| *p)
            .or_else(|| default.first().copied());
        match target {
            Some(p) => self.report_for(p, current_capability),
            None => BenchmarkReport {
                project_name: "none".into(),
                relevance_score: 0.0,
                gap_areas: Vec::new(),
                suggested_capability_delta: Vec::new(),
                summary: "No relevant open-source projects found".into(),
            },
        }
    }

    pub fn benchmark_top3(
        &self,
        task: &str,
        task_type: TaskType,
        current_capability: &CapabilityVector,
    ) -> Vec<BenchmarkReport> {
        let mut matched = match_keyword(task, &task_type);
        matched.sort_by_key(|b| std::cmp::Reverse(b.1));
        let mut result: Vec<&ProjectProfile> =
            matched.into_iter().take(3).map(|(p, _)| p).collect();
        if result.len() < 3 {
            let seen: std::collections::HashSet<&str> = result.iter().map(|p| p.name).collect();
            for p in match_by_task_type(&task_type) {
                if result.len() >= 3 {
                    break;
                }
                if !seen.contains(p.name) {
                    result.push(p);
                }
            }
        }
        result
            .into_iter()
            .map(|p| self.report_for(p, current_capability))
            .collect()
    }

    pub fn find_url(&self, task: &str, task_type: &TaskType) -> Option<&'static str> {
        match_keyword(task, task_type).first().map(|(p, _)| p.url)
    }

    fn report_for(&self, project: &ProjectProfile, current: &CapabilityVector) -> BenchmarkReport {
        let mut gap_areas = Vec::new();
        let mut deltas = Vec::new();
        for &(idx, target_val) in project.capability_profile {
            if idx >= current.arr().len() {
                continue;
            }
            let current_val = current.arr()[idx];
            if target_val > current_val + 0.05 {
                let gap = target_val - current_val;
                let dim_name = Self::dim_name(idx);
                gap_areas.push((
                    idx,
                    gap,
                    format!(
                        "{}: {:.2} vs {:.2} ({})",
                        dim_name, target_val, current_val, project.name
                    ),
                ));
                deltas.push((idx, gap * 0.5));
            }
        }
        let summary = if gap_areas.is_empty() {
            format!(
                "Capability matched/exceeds {} in all dimensions",
                project.name
            )
        } else {
            let dims: Vec<String> = gap_areas
                .iter()
                .map(|(i, _, _)| Self::dim_name(*i))
                .collect();
            format!(
                "{} gaps vs {} ({}): cap uplift {:.2}",
                gap_areas.len(),
                project.name,
                dims.join(", "),
                deltas.iter().map(|(_, d)| d).sum::<f64>()
            )
        };
        BenchmarkReport {
            project_name: project.name.to_string(),
            relevance_score: project.relevance,
            gap_areas,
            suggested_capability_delta: deltas,
            summary,
        }
    }

    fn dim_name(idx: usize) -> String {
        match idx {
            0 => "compound_composition",
            1 => "tailwind",
            2 => "accessibility",
            3 => "react_aria",
            4 => "semantic_layer",
            5 => "verification",
            6 => "quality_gates",
            7 => "video_rendering",
            8 => "secret_detection",
            9 => "nt_shield_audit",
            10 => "anti_detection",
            11 => "react_lint",
            12 => "vector_design_canvas",
            13 => "agent_trading",
            14 => "esp32_firmware",
            15 => "figma_integration",
            16 => "celer_filtering",
            17 => "vulnerability_knowledge",
            18 => "web_scraping",
            19 => "health_scoring",
            20 => "mcp_design_tools",
            _ => return format!("dim_{}", idx),
        }
        .to_string()
    }

    pub fn generate_edits_from_reports(reports: &[BenchmarkReport]) -> Vec<MicroEdit> {
        let mut edits = Vec::new();
        let mut applied = std::collections::HashSet::new();
        for report in reports {
            for (idx, delta, _) in &report.gap_areas {
                if applied.insert(*idx) {
                    edits.push(MicroEdit::AdjustDimension(idx.to_string(), *delta));
                }
            }
        }
        if !edits.is_empty() {
            edits.push(MicroEdit::NormalizeVector);
        }
        edits
    }
}

impl Default for OpenSourceBenchmarker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_by_task_type() {
        let matches = match_by_task_type(&TaskType::Security);
        assert!(matches.len() >= 3);
    }

    #[test]
    fn test_matches_ui_design() {
        let matches = match_by_task_type(&TaskType::UIDesign);
        assert!(matches.iter().any(|p| p.name == "HeroUI"));
    }

    #[test]
    fn test_benchmarker_top3() {
        let b = OpenSourceBenchmarker::new();
        let cap = CapabilityVector::default();
        let reports = b.benchmark_top3("stealth proxy with memory", TaskType::CodeAnalysis, &cap);
        assert!(!reports.is_empty());
    }

    #[test]
    fn test_find_url() {
        let b = OpenSourceBenchmarker::new();
        let url = b.find_url("tor", &TaskType::Security);
        assert!(url.is_some());
    }
}
