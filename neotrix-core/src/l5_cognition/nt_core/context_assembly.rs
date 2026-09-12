//! # ContextPipe — 五阶段上下文组装引擎
//!
//! 基于 ContextPipe (arXiv 2609.00749) 设计：将上下文窗口视为"查询预算"，
//! 像数据库查询 EXPLAIN 一样组装上下文。
//!
//! 五阶段流程：Plan → Bind → Optimize → Execute → Feedback

use std::collections::HashMap;

/// 上下文源类型
#[derive(Debug, Clone, Hash, Eq, PartialEq)]

pub enum ContextSource {
    /// KB 搜索结果
    KnowledgeBase,
    /// 会话历史
    History,
    /// 工具输出（沙箱压缩后）
    ToolOutput,
    /// 技能模板
    SkillTemplate,
    /// 系统提示
    SystemPrompt,
}

/// 上下文片段
#[derive(Debug, Clone)]

pub struct ContextFragment {
    /// 来源
    pub source: ContextSource,
    /// 内容
    pub content: String,
    /// Token 数量
    pub token_count: usize,
    /// 相关性分数 0.0-1.0
    pub relevance_score: f64,
    /// 优先级（0=最高）
    pub priority: u32,
}

/// 信息需求（Plan 阶段产出）
#[derive(Debug, Clone)]

pub struct InformationNeed {
    /// 需求描述
    pub description: String,
    /// 预估 token 数
    pub estimated_tokens: usize,
    /// 是否必须包含
    pub required: bool,
}

/// 预算分配（Optimize 阶段产出）
#[derive(Debug, Clone)]

pub struct BudgetAllocation {
    /// 分配来源
    pub source: ContextSource,
    /// 分配的 token 数
    pub allocated_tokens: usize,
    /// 分配理由
    pub reason: String,
}

/// 上下文组装计划
#[derive(Debug)]

pub struct AssemblyPlan {
    /// 信息需求列表
    pub needs: Vec<InformationNeed>,
    /// 预算分配
    pub allocations: Vec<BudgetAllocation>,
    /// 总预算
    pub total_budget: usize,
}

/// 组装结果
#[derive(Debug)]

pub struct AssemblyResult {
    /// 选中的片段
    pub fragments: Vec<ContextFragment>,
    /// 总 token 数
    pub total_tokens: usize,
    /// 预算使用率 0.0-1.0
    pub budget_used: f64,
    /// 需求覆盖比例
    pub coverage: f64,
    /// 综合质量分数
    pub quality_score: f64,
}

/// 上下文组装引擎 — 五阶段上下文管理
#[derive(Debug)]

pub struct ContextAssembler {
    /// 总 token 预算
    budget: usize,
    /// 已用 token
    used_tokens: usize,
    /// 历史组装记录（用于 Feedback 阶段学习）
    history: Vec<AssemblyResult>,
    /// 源优先级权重
    source_weights: HashMap<ContextSource, f64>,
}

impl ContextAssembler {
    /// 创建新的上下文组装器
    pub fn new(budget: usize) -> Self {
        let mut weights = HashMap::new();
        weights.insert(ContextSource::SystemPrompt, 1.0);
        weights.insert(ContextSource::KnowledgeBase, 0.9);
        weights.insert(ContextSource::SkillTemplate, 0.8);
        weights.insert(ContextSource::History, 0.7);
        weights.insert(ContextSource::ToolOutput, 0.6);

        Self {
            budget,
            used_tokens: 0,
            history: Vec::new(),
            source_weights: weights,
        }
    }

    /// Stage 1: Plan — 分解任务为信息需求
    pub fn plan(&self, task: &str) -> Vec<InformationNeed> {
        let mut needs = Vec::new();

        // 基本需求：系统提示总是需要
        needs.push(InformationNeed {
            description: "系统提示与角色定义".to_string(),
            estimated_tokens: 500,
            required: true,
        });

        // 任务上下文
        if !task.is_empty() {
            needs.push(InformationNeed {
                description: format!("任务上下文: {}", task),
                estimated_tokens: task.len() / 4, // rough estimate
                required: true,
            });
        }

        // 关键词触发的额外需求
        let task_lower = task.to_lowercase();
        if task_lower.contains("知识") || task_lower.contains("kb") || task_lower.contains("知识库")
        {
            needs.push(InformationNeed {
                description: "知识库检索结果".to_string(),
                estimated_tokens: 1000,
                required: false,
            });
        }
        if task_lower.contains("历史") || task_lower.contains("会话") || task_lower.contains("之前")
        {
            needs.push(InformationNeed {
                description: "历史会话上下文".to_string(),
                estimated_tokens: 800,
                required: false,
            });
        }
        if task_lower.contains("技能") || task_lower.contains("skill") {
            needs.push(InformationNeed {
                description: "技能模板加载".to_string(),
                estimated_tokens: 600,
                required: false,
            });
        }
        if task_lower.contains("工具") || task_lower.contains("tool") {
            needs.push(InformationNeed {
                description: "工具输出上下文".to_string(),
                estimated_tokens: 500,
                required: false,
            });
        }

        needs
    }

    /// Stage 2: Bind — 将需求映射到数据源
    pub fn bind(&self, needs: &[InformationNeed]) -> Vec<(InformationNeed, Vec<ContextSource>)> {
        needs
            .iter()
            .map(|n| {
                let desc = n.description.as_str();
                let sources = if desc.contains("知识库") || desc.contains("KB") {
                    vec![ContextSource::KnowledgeBase]
                } else if desc.contains("历史") || desc.contains("会话") {
                    vec![ContextSource::History]
                } else if desc.contains("技能") || desc.contains("Skill") {
                    vec![ContextSource::SkillTemplate]
                } else if desc.contains("工具") || desc.contains("Tool") {
                    vec![ContextSource::ToolOutput]
                } else {
                    // 默认：历史 + KB 双源
                    vec![ContextSource::History, ContextSource::KnowledgeBase]
                };
                (n.clone(), sources)
            })
            .collect()
    }

    /// Stage 3: Optimize — 预算分配
    pub fn optimize(
        &self,
        needs: &[(InformationNeed, Vec<ContextSource>)],
    ) -> Vec<BudgetAllocation> {
        let mut allocations = Vec::new();
        let mut remaining = self.budget;

        // 按权重降序排序
        let mut sorted: Vec<_> = needs.to_vec();
        sorted.sort_by(|a, b| {
            let w_a =
                a.1.first()
                    .and_then(|s| self.source_weights.get(s))
                    .unwrap_or(&0.0);
            let w_b =
                b.1.first()
                    .and_then(|s| self.source_weights.get(s))
                    .unwrap_or(&0.0);
            w_b.partial_cmp(w_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (need, sources) in sorted {
            let alloc_tokens = need.estimated_tokens.min(remaining);
            if alloc_tokens > 0 {
                allocations.push(BudgetAllocation {
                    source: sources.first().cloned().unwrap_or(ContextSource::History),
                    allocated_tokens: alloc_tokens,
                    reason: if need.required {
                        "必需".to_string()
                    } else {
                        "可选".to_string()
                    },
                });
                remaining -= alloc_tokens;
            }
        }

        allocations
    }

    /// Stage 4: Execute — 组装上下文
    pub fn execute(
        &mut self,
        allocations: &[BudgetAllocation],
        fragments: Vec<ContextFragment>,
    ) -> AssemblyResult {
        let mut selected = Vec::new();
        let mut total_tokens = 0;

        // 按相关性降序排序，贪心选择
        let mut sorted_fragments = fragments;
        sorted_fragments.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // 跟踪已分配的预算
        let alloc_map: HashMap<&ContextSource, usize> = allocations
            .iter()
            .map(|a| (&a.source, a.allocated_tokens))
            .collect();

        for fragment in sorted_fragments {
            let cap = alloc_map
                .get(&fragment.source)
                .copied()
                .unwrap_or(self.budget);
            let source_used: usize = selected
                .iter()
                .filter(|f: &&ContextFragment| f.source == fragment.source)
                .map(|f| f.token_count)
                .sum();

            if total_tokens + fragment.token_count <= self.budget
                && source_used + fragment.token_count <= cap
            {
                total_tokens += fragment.token_count;
                selected.push(fragment);
            }
        }

        let budget_used = if self.budget > 0 {
            total_tokens as f64 / self.budget as f64
        } else {
            0.0
        };

        // 覆盖率：已选来源占分配来源比例
        let covered_sources: std::collections::HashSet<_> =
            selected.iter().map(|f| &f.source).collect();
        let total_sources: std::collections::HashSet<_> =
            allocations.iter().map(|a| &a.source).collect();
        let coverage = if total_sources.is_empty() {
            1.0
        } else {
            covered_sources.intersection(&total_sources).count() as f64 / total_sources.len() as f64
        };

        let quality_score = budget_used * 0.8 + coverage * 0.2;

        AssemblyResult {
            fragments: selected,
            total_tokens,
            budget_used,
            coverage,
            quality_score,
        }
    }

    /// Stage 5: Feedback — 评估并记录
    pub fn feedback(&mut self, result: AssemblyResult) {
        self.history.push(result);
        // 保留最近 100 条
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    /// 获取历史平均质量
    pub fn avg_quality(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().map(|r| r.quality_score).sum::<f64>() / self.history.len() as f64
    }

    /// 获取预算使用率
    pub(crate) fn _budget_utilization(&self) -> f64 {
        if self.budget == 0 {
            return 0.0;
        }
        self.used_tokens as f64 / self.budget as f64
    }

    /// 获取总预算
    pub fn budget(&self) -> usize {
        self.budget
    }

    /// 获取源权重
    pub fn source_weight(&self, source: &ContextSource) -> f64 {
        self.source_weights.get(source).copied().unwrap_or(0.0)
    }
}
