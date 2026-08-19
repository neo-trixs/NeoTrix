//! RecipeRefactor — 声明式重构引擎 (OpenRewrite 吸收, R-P79 代码级接线)
//!
//! 参考: OpenRewrite 声明式 recipe 编码重构规则, 一次定义跨项目/多接收者执行。
//! 本模块提供确定性、零 LLM 依赖的重构原语:
//!   - `Recipe` = 一组有序变换步骤 (替换/删除/插入/正则重写)
//!   - `apply_to` 对文本/文件内容执行并报告每个步骤的命中数
//!   - `dry_run` 预览不落盘
//!
//! R-P42: 作为 nt_act_code 子系统节点接入 (SelfCode 家族), 不建平行重构系统。
//! 纯确定性: 无网络 / 无 tokio / 无文件 IO (内容由调用方传入)。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 重构错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeError {
    /// 步骤替换文本在目标中不存在。
    PatternNotFound(String),
    /// 正则编译失败。
    InvalidRegex(String),
    /// 步骤序号越界/缺失。
    BadStepIndex(String),
}

impl std::fmt::Display for RecipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatternNotFound(s) => write!(f, "recipe step pattern not found: {s}"),
            Self::InvalidRegex(s) => write!(f, "invalid regex: {s}"),
            Self::BadStepIndex(s) => write!(f, "bad step index: {s}"),
        }
    }
}

impl std::error::Error for RecipeError {}

/// 单步变换动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeStep {
    /// 字面替换: 将 `from` 全部替换为 `to`。
    Replace { from: String, to: String },
    /// 正则替换 (全局): 将匹配正则的全部替换为 `to` (支持 `$1` 反向引用)。
    RegexReplace { pattern: String, to: String },
    /// 删除所有匹配字面串的行 (含行尾)。
    DeleteLines { containing: String },
}

/// 单条重构规则。
#[derive(Debug, Clone)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub steps: Vec<RecipeStep>,
}

/// 步骤执行结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StepResult {
    pub hits: u64,
}

/// 执行结果汇总。
#[derive(Debug, Clone, Default)]
pub struct RecipeResult {
    pub recipe: String,
    pub step_results: Vec<StepResult>,
    pub transformed: bool,
    pub hit_total: u64,
    pub output: String,
}

impl RecipeResult {
    /// 变换后的完整内容。
    pub fn output(&self) -> &str {
        &self.output
    }
}

/// 重构引擎 — 执行 Recipe 序列。
#[derive(Debug, Default)]
pub struct RecipeRefactor {
    applied: AtomicU64,
    registry: Arc<std::sync::RwLock<HashMap<String, Recipe>>>,
}

impl RecipeRefactor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个可复用 recipe。
    pub fn register(&self, recipe: Recipe) {
        if let Ok(mut m) = self.registry.write() {
            m.insert(recipe.name.clone(), recipe);
        }
    }

    /// 按名取 recipe。
    pub fn get(&self, name: &str) -> Option<Recipe> {
        self.registry
            .read()
            .ok()
            .and_then(|m| m.get(name).cloned())
    }

    pub fn list(&self) -> Vec<String> {
        self.registry
            .read()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// 执行 recipe (dry_run=true 时返回预览但不累计 applied 计数)。
    pub fn apply(
        &self,
        recipe: &Recipe,
        content: &str,
        dry_run: bool,
    ) -> Result<RecipeResult, RecipeError> {
        let mut buf = content.to_string();
        let mut step_results = Vec::with_capacity(recipe.steps.len());
        for (i, step) in recipe.steps.iter().enumerate() {
            let res = match step {
                RecipeStep::Replace { from, to } => {
                    if !buf.contains(from.as_str()) {
                        return Err(RecipeError::PatternNotFound(format!(
                            "step[{i}] from={from:?}"
                        )));
                    }
                    let hits = buf.matches(from.as_str()).count() as u64;
                    buf = buf.replace(from.as_str(), to.as_str());
                    StepResult { hits }
                }
                RecipeStep::RegexReplace { pattern, to } => {
                    let re = regex::Regex::new(pattern)
                        .map_err(|_| RecipeError::InvalidRegex(pattern.clone()))?;
                    let hits = re.find_iter(&buf).count() as u64;
                    buf = re.replace_all(&buf, to.as_str()).into_owned();
                    StepResult { hits }
                }
                RecipeStep::DeleteLines { containing } => {
                    let hits = buf.lines().filter(|l| l.contains(containing.as_str())).count() as u64;
                    buf = buf
                        .lines()
                        .filter(|l| !l.contains(containing.as_str()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    StepResult { hits }
                }
            };
            step_results.push(res);
        }
        let hit_total = step_results.iter().map(|s| s.hit_total()).sum();
        let transformed = hit_total > 0 && buf != content;
        if !dry_run && transformed {
            self.applied.fetch_add(1, Ordering::Relaxed);
        }
        Ok(RecipeResult {
            recipe: recipe.name.clone(),
            step_results,
            transformed,
            hit_total,
            output: buf,
        })
    }

    /// 累积执行次数 (行为指标)。
    pub fn applied_count(&self) -> u64 {
        self.applied.load(Ordering::Relaxed)
    }
}

impl StepResult {
    pub fn hit_total(&self) -> u64 {
        self.hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_replace_steps_execute_in_order() {
        let engine = RecipeRefactor::new();
        let recipe = Recipe {
            name: "rename_fn".to_string(),
            description: "示例: 函数改名".to_string(),
            steps: vec![
                RecipeStep::Replace {
                    from: "fn compute_()".to_string(),
                    to: "fn calc()".to_string(),
                },
                RecipeStep::RegexReplace {
                    pattern: r"compute_(\w+)".to_string(),
                    to: "calc_$1".to_string(),
                },
            ],
        };
        let src = "fn compute_() {}\nlet x = compute_foo();";
        let res = engine.apply(&recipe, src, false).expect("applies");
        assert!(res.transformed);
        assert_eq!(res.hit_total, 2);
        assert_eq!(engine.applied_count(), 1);
    }

    #[test]
    fn delete_lines_removes_containing() {
        let recipe = Recipe {
            name: "drop_debug".to_string(),
            description: "删除调试行".to_string(),
            steps: vec![RecipeStep::DeleteLines {
                containing: "eprintln!".to_string(),
            }],
        };
        let src = "a\neprintln!(\"x\");\nb\neprintln!(\"y\");";
        let res = RecipeRefactor::new().apply(&recipe, src, false).expect("applies");
        assert_eq!(res.step_results[0].hits, 2);
        assert!(!res.output().contains("eprintln!"));
    }

    #[test]
    fn pattern_not_found_errors() {
        let recipe = Recipe {
            name: "bad".to_string(),
            description: "缺失模式".to_string(),
            steps: vec![RecipeStep::Replace {
                from: "does_not_exist_xyz".to_string(),
                to: "x".to_string(),
            }],
        };
        let res = RecipeRefactor::new().apply(&recipe, "hello", false);
        assert!(matches!(res, Err(RecipeError::PatternNotFound(_))));
    }

    #[test]
    fn dry_run_does_not_count_apply() {
        let engine = RecipeRefactor::new();
        let recipe = Recipe {
            name: "preview".to_string(),
            description: "预览".to_string(),
            steps: vec![RecipeStep::Replace {
                from: "a".to_string(),
                to: "b".to_string(),
            }],
        };
        engine.apply(&recipe, "cat", true).expect("dry run");
        assert_eq!(engine.applied_count(), 0);
    }

    #[test]
    fn recipe_registry_roundtrip() {
        let engine = RecipeRefactor::new();
        engine.register(Recipe {
            name: "r1".to_string(),
            description: "d1".to_string(),
            steps: vec![],
        });
        assert_eq!(engine.list(), vec!["r1".to_string()]);
        assert!(engine.get("r1").is_some());
    }
}
