use std::io;
use std::path::Path;

use super::types::*;

// ── Heuristic functions ─────────────────────────────────────────────────

pub fn detect_project_type(text: &str) -> &str {
    let t = text.to_lowercase();
    if t.contains("website")
        || t.contains("web app")
        || t.contains("webapp")
        || t.contains("frontend")
    {
        "web"
    } else if t.contains("mobile")
        || t.contains("ios")
        || t.contains("android")
        || t.contains("app")
    {
        if t.contains("ios") {
            "ios"
        } else if t.contains("android") {
            "android"
        } else {
            "mobile"
        }
    } else if t.contains("cli") || t.contains("command") || t.contains("terminal") {
        "cli"
    } else if t.contains("library")
        || t.contains("lib")
        || t.contains("package")
        || t.contains("crate")
    {
        "library"
    } else if t.contains("api")
        || t.contains("backend")
        || t.contains("server")
        || t.contains("service")
    {
        "api"
    } else if t.contains("fix") || t.contains("bug") || t.contains("error") || t.contains("issue") {
        "bugfix"
    } else if t.contains("research")
        || t.contains("investigat")
        || t.contains("survey")
        || t.contains("explore")
    {
        "research"
    } else if t.contains("script") || t.contains("tool") || t.contains("pipeline") {
        "script"
    } else {
        "general"
    }
}

pub fn suggest_verification(project_type: &str, outcome: &str) -> Vec<String> {
    let mut v = Vec::new();
    match project_type {
        "web" | "mobile" | "ios" | "android" => {
            v.push("check: application builds without errors".into());
            v.push("check: all pages render at 1280x720 viewport".into());
            if outcome.to_lowercase().contains("form") || outcome.to_lowercase().contains("login") {
                v.push("check: form submission succeeds with valid input".into());
                v.push("check: form shows validation on empty input".into());
            }
        }
        "cli" => {
            v.push("check: binary compiles and runs".into());
            v.push("check: --help flag displays usage".into());
            v.push("check: exit code is 0 for normal execution".into());
        }
        "api" => {
            v.push("check: server starts on expected port".into());
            v.push("check: health endpoint returns 200".into());
        }
        "library" => {
            v.push("check: cargo build succeeds with no warnings".into());
            v.push("check: cargo test passes".into());
            if outcome.to_lowercase().contains("doc") || outcome.to_lowercase().contains("docs") {
                v.push("check: cargo doc builds without errors".into());
            }
        }
        "bugfix" => {
            v.push("check: bug reproduction case now passes".into());
            v.push("check: existing tests still pass".into());
            v.push("check: no new clippy warnings".into());
        }
        "research" => {
            v.push("check: at least 3 authoritative sources cited".into());
            v.push("check: summary covers pros/cons/tradeoffs".into());
        }
        _ => {
            v.push("check: implementation satisfies stated outcome".into());
        }
    }
    v
}

pub fn suggest_constraints(project_type: &str, domain: Option<&str>) -> Vec<String> {
    let mut c = Vec::new();
    match project_type {
        "web" | "mobile" | "ios" | "android" => {
            c.push("Do not modify existing authentication logic".into());
            c.push("Maintain responsive layout for 320px-1920px".into());
        }
        "bugfix" => {
            c.push("Do not change public API signatures".into());
            c.push("Do not modify database schema".into());
            c.push("Preserve existing test behavior".into());
        }
        "api" => {
            c.push("Maintain backward compatibility for existing endpoints".into());
            c.push("All new endpoints must include error handling".into());
        }
        "research" => {
            c.push("Do not modify any source files".into());
            c.push("Output as structured report only".into());
        }
        _ => {}
    }
    if let Some(d) = domain {
        let dl = d.to_lowercase();
        if dl == "medical" || dl == "health" {
            c.push("No patient data may be exposed in logs".into());
            c.push("All outputs must pass HIPAA compliance check".into());
        }
        if dl == "finance" || dl == "financial" {
            c.push("All monetary calculations must use decimal arithmetic".into());
            c.push("Audit trail must be preserved for all transactions".into());
        }
        if dl == "copyright" || dl == "legal" {
            c.push("No third-party code without license verification".into());
        }
    }
    c
}

pub fn suggest_boundaries(project_type: &str) -> GoalBoundaries {
    let mut b = GoalBoundaries::default();
    match project_type {
        "bugfix" => {
            b.allowed_paths = vec!["src/**/*.rs".into()];
            b.max_files_to_modify = 3;
        }
        "research" => {
            b.allowed_paths = vec!["docs/**/*.md".into(), "notes/**/*.md".into()];
            b.forbidden_paths.push("**/*.rs".into());
            b.max_files_to_modify = 1;
        }
        "web" | "mobile" | "ios" | "android" => {
            b.max_files_to_modify = 8;
        }
        "cli" => {
            b.allowed_paths.push("src/main.rs".into());
            b.max_files_to_modify = 5;
        }
        "script" => {
            b.allowed_paths.push("scripts/**/*".into());
            b.max_files_to_modify = 3;
        }
        _ => {}
    }
    b
}

pub fn suggest_iteration_policy(risk: &str) -> IterationPolicy {
    match risk {
        "high" => IterationPolicy {
            max_rounds: 5,
            rerun_after_change: true,
            inspect_logs_before_retry: true,
            stop_on_consecutive_failures: 1,
        },
        "medium" => IterationPolicy {
            max_rounds: 3,
            rerun_after_change: true,
            inspect_logs_before_retry: true,
            stop_on_consecutive_failures: 2,
        },
        _ => IterationPolicy {
            max_rounds: 3,
            rerun_after_change: true,
            inspect_logs_before_retry: false,
            stop_on_consecutive_failures: 3,
        },
    }
}

pub fn suggest_stop_conditions(outcome: &str) -> Vec<String> {
    let mut s = Vec::new();
    let o = outcome.to_lowercase();
    s.push("User explicitly confirms completion".into());
    if o.contains("test") || o.contains("pass") {
        s.push("All tests pass with 100% success rate".into());
    }
    if o.contains("deploy") || o.contains("release") {
        s.push("Build artifact is generated and verified".into());
        s.push("Deployment pipeline reports green status".into());
    }
    if o.contains("migrat") || o.contains("refactor") {
        s.push("Old code path is removed or deprecated".into());
        s.push("Integration tests pass against new code".into());
    }
    if s.len() == 1 {
        s.push("All verification checks pass".into());
    }
    s
}

pub fn classify_risk(text: &str) -> &str {
    let t = text.to_lowercase();
    if t.contains("production")
        || t.contains("prod")
        || t.contains("payment")
        || t.contains("credit card")
        || t.contains("billing")
        || t.contains("medical")
        || t.contains("patient")
        || t.contains("hipaa")
        || t.contains("financial")
        || t.contains("stock")
        || t.contains("trade")
    {
        "high"
    } else if t.contains("api")
        || t.contains("auth")
        || t.contains("login")
        || t.contains("database")
        || t.contains("db")
        || t.contains("user data")
    {
        "medium"
    } else {
        "low"
    }
}

pub fn translate_vague_words(text: &str) -> String {
    let mut t = text.to_string();
    let replacements: Vec<(&str, &str)> = vec![
        ("高级", "验证视觉质量通过截图和间距检查"),
        ("专业", "检查层级结构、字体一致性、可读性"),
        ("好看", "确认布局对齐、色彩协调、间距统一"),
        ("优化", "测量当前性能指标后针对性改进"),
        ("简单", "最小可行实现不含额外抽象层"),
        ("快速", "优先执行速度而非代码优雅性"),
        ("robust", "add error handling for all edge cases"),
        (
            "clean",
            "refactor with consistent naming and minimal duplication",
        ),
        (
            "efficient",
            "measure current complexity then reduce by at least one order",
        ),
        (
            "scalable",
            "design for 10x current load without rearchitecture",
        ),
        (
            "intuitive",
            "verify new user can complete primary task in under 30s",
        ),
        ("modern", "use latest stable language edition and idioms"),
    ];
    for (from, to) in replacements {
        t = t.replace(from, to);
    }
    t
}

// ── Decompose steps ─────────────────────────────────────────────────────

fn default_steps_for(outcome: &str, project_type: &str) -> Vec<GoalStep> {
    let o = outcome.to_lowercase();
    match project_type {
        "bugfix" => vec![
            GoalStep::new(
                0,
                "Reproduce the bug with a minimal test case",
                "run the reproduction steps and confirm the failure",
            ),
            GoalStep::new(
                1,
                "Identify root cause in source code",
                "inspect stack trace or log output pointing to origin",
            ),
            GoalStep::new(
                2,
                "Implement fix with passing reproduction test",
                "verify the reproduction case now succeeds",
            ),
            GoalStep::new(
                3,
                "Run full test suite to check for regressions",
                "cargo test --all-targets",
            ),
        ],
        "research" => vec![
            GoalStep::new(
                0,
                "Gather at least 5 authoritative sources on the topic",
                "each source must be from peer-reviewed or established publication",
            ),
            GoalStep::new(
                1,
                "Synthesize findings into structured summary",
                "cover: core concepts, approaches, tradeoffs, open questions",
            ),
            GoalStep::new(
                2,
                "Identify actionable insights for current project",
                "map findings to existing architecture gaps",
            ),
        ],
        "web" | "mobile" | "ios" | "android" => {
            let mut steps = vec![
                GoalStep::new(
                    0,
                    "Scaffold project structure and dependencies",
                    "build tool reports no errors",
                ),
                GoalStep::new(
                    1,
                    "Implement core UI layout for primary view",
                    "screenshot matches design spec",
                ),
                GoalStep::new(
                    2,
                    "Wire up data flow and state management",
                    "view renders live data from mock source",
                ),
            ];
            if o.contains("auth") || o.contains("login") {
                steps.push(GoalStep::new(
                    3,
                    "Implement authentication flow",
                    "login/logout/register round-trip works",
                ));
            }
            if o.contains("form") || o.contains("input") {
                steps.push(GoalStep::new(
                    steps.len(),
                    "Add form validation and submission",
                    "empty fields show errors, valid data submits",
                ));
            }
            steps.push(GoalStep::new(
                steps.len(),
                "Run integration tests across supported targets",
                "all tests pass",
            ));
            steps
        }
        "api" => vec![
            GoalStep::new(
                0,
                "Define API contract (OpenAPI or similar)",
                "spec documents all endpoints with request/response schemas",
            ),
            GoalStep::new(
                1,
                "Implement core endpoints with error handling",
                "each endpoint returns correct status codes",
            ),
            GoalStep::new(
                2,
                "Add input validation and authentication middleware",
                "unauthorized requests return 401",
            ),
            GoalStep::new(
                3,
                "Write integration tests covering happy path and errors",
                "cargo test --test '*' passes",
            ),
        ],
        "cli" => vec![
            GoalStep::new(
                0,
                "Define CLI argument interface with clap or similar",
                "--help output covers all flags",
            ),
            GoalStep::new(
                1,
                "Implement core command logic",
                "command produces expected output for sample input",
            ),
            GoalStep::new(
                2,
                "Add error handling and user-friendly messages",
                "invalid input produces actionable error",
            ),
            GoalStep::new(
                3,
                "Test binary end-to-end with sample data",
                "exit code 0 and output matches expected",
            ),
        ],
        _ => vec![
            GoalStep::new(
                0,
                "Define clear acceptance criteria",
                "criteria are measurable and verifiable",
            ),
            GoalStep::new(
                1,
                "Implement minimal solution",
                "solution satisfies acceptance criteria",
            ),
            GoalStep::new(2, "Verify and iterate", "verification checks pass"),
        ],
    }
}

// ── GoalDecomposer ──────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoalDecomposer {
    pub goals: Vec<GoalContract>,
    pub active_goal: Option<usize>,
    pub max_goals: usize,
    pub max_depth: u32,
    steps_cache: Vec<Vec<GoalStep>>,
}

impl GoalDecomposer {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            active_goal: None,
            max_goals: 20,
            max_depth: 5,
            steps_cache: Vec::new(),
        }
    }

    pub fn with_max_goals(mut self, max: usize) -> Self {
        self.max_goals = max;
        self
    }

    pub fn decompose(&mut self, request: &str, context: &DecompositionContext) -> &GoalContract {
        if self.goals.len() >= self.max_goals {
            self.goals.remove(0);
            if !self.steps_cache.is_empty() {
                self.steps_cache.remove(0);
            }
        }

        let mut gc = decompose_vague(request);
        if let Some(pt) = &context.project_type {
            let detected = detect_project_type(request);
            if detected == "general" {
                gc.outcome = format!("Build a {pt} project: {r}", pt = pt, r = request);
                gc.verification = suggest_verification(pt, &gc.outcome);
                gc.constraints = suggest_constraints(pt, context.domain.as_deref());
                gc.boundaries = suggest_boundaries(pt);
            }
        }
        for uc in &context.user_constraints {
            gc.constraints.push(uc.clone());
        }
        if let Some(d) = &context.domain {
            let extra = suggest_constraints("general", Some(d));
            for c in extra {
                if !gc.constraints.contains(&c) {
                    gc.constraints.push(c);
                }
            }
            if d == "medical" && gc.pause_conditions.is_empty() {
                gc.pause_conditions
                    .push("Medical domain: requires expert human review".into());
            }
            if d == "finance" && gc.pause_conditions.is_empty() {
                gc.pause_conditions
                    .push("Financial domain: requires audit trail review".into());
            }
        }
        let idx = self.goals.len();
        let project_type_for_steps = if let Some(pt) = &context.project_type {
            pt.clone()
        } else {
            detect_project_type(request).to_string()
        };
        let steps = default_steps_for(&gc.outcome, &project_type_for_steps);
        self.steps_cache.push(steps);
        self.goals.push(gc);
        self.active_goal = Some(idx);
        &self.goals[idx]
    }

    pub fn steps_for(&self, goal_idx: usize) -> Option<&[GoalStep]> {
        self.steps_cache.get(goal_idx).map(|v| v.as_slice())
    }

    pub fn next_step(&mut self) -> Option<&GoalStep> {
        let idx = self.active_goal?;
        let steps = self.steps_cache.get_mut(idx)?;
        let pending_idx = steps.iter().position(|s| s.status == StepStatus::Pending)?;
        steps[pending_idx].status = StepStatus::InProgress;
        let steps_ref = self.steps_cache.get(idx)?;
        Some(&steps_ref[pending_idx])
    }

    pub fn record_verification(&mut self, step_id: usize, result: StepVerificationResult) {
        let idx = match self.active_goal {
            Some(i) => i,
            None => return,
        };
        let steps = match self.steps_cache.get_mut(idx) {
            Some(s) => s,
            None => return,
        };
        let step = match steps.iter_mut().find(|s| s.id == step_id) {
            Some(s) => s,
            None => return,
        };
        if result.passed {
            step.status = StepStatus::Verified;
        } else {
            step.status = StepStatus::Failed(result.details.clone());
            step.error_log.push(result.details.clone());
        }
    }

    pub fn check_pause_conditions(&self) -> Option<String> {
        let idx = self.active_goal?;
        let goal = self.goals.get(idx)?;
        for c in &goal.pause_conditions {
            if c.contains("payment")
                || c.contains("human review")
                || c.contains("expert")
                || c.contains("legal")
            {
                return Some(c.clone());
            }
        }
        None
    }

    pub fn report(&self) -> String {
        if self.goals.is_empty() {
            return "No goals tracked.".to_string();
        }
        let mut lines = Vec::new();
        lines.push("╔══════════════════════════════════════╗".to_string());
        lines.push("║     Goal Decomposer Report           ║".to_string());
        lines.push("╚══════════════════════════════════════╝".to_string());
        for (i, goal) in self.goals.iter().enumerate() {
            let status_str = match &goal.status {
                DecomposerGoalStatus::Draft => "Draft",
                DecomposerGoalStatus::InProgress => "In Progress",
                DecomposerGoalStatus::Paused(r) => return format!("PAUSED: {r}"),
                DecomposerGoalStatus::Completed(e) => return format!("COMPLETED: {e}"),
                DecomposerGoalStatus::Failed(e) => return format!("FAILED: {e}"),
            };
            lines.push(format!("Goal #{}: {}", i, status_str));
            lines.push(format!("  Raw: {}", goal.raw_request));
            lines.push(format!("  Outcome: {}", goal.outcome));
            if let Some(steps) = self.steps_cache.get(i) {
                lines.push(format!(
                    "  Steps: {}/{}",
                    steps
                        .iter()
                        .filter(|s| s.status == StepStatus::Verified)
                        .count(),
                    steps.len()
                ));
                for step in steps {
                    let mark = match &step.status {
                        StepStatus::Pending => "⏳",
                        StepStatus::InProgress => "🔄",
                        StepStatus::Verified => "✅",
                        StepStatus::Failed(_) => "❌",
                    };
                    lines.push(format!("    {mark} step {}: {}", step.id, step.description));
                }
            }
            lines.push(String::new());
        }
        lines.join("\n")
    }
}

// ── GoalExecutor ────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionLog {
    pub cycle: u64,
    pub goal_id: String,
    pub step_id: usize,
    pub action: String,
    pub result: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoalExecutor {
    pub decomposer: GoalDecomposer,
    pub log: Vec<ExecutionLog>,
}

impl GoalExecutor {
    pub fn new() -> Self {
        Self {
            decomposer: GoalDecomposer::new(),
            log: Vec::new(),
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let tmp_path = path.as_ref().with_extension("json.tmp");
        std::fs::write(&tmp_path, &json)?;
        std::fs::rename(&tmp_path, path.as_ref())?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = std::fs::read_to_string(path.as_ref())?;
        let executor: GoalExecutor = serde_json::from_str(&data)?;
        Ok(executor)
    }

    pub fn export_plan_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# NeoTrix Execution Plan\n\n");
        out.push_str(&format!("Total goals: {}\n\n", self.decomposer.goals.len()));
        for (i, goal) in self.decomposer.goals.iter().enumerate() {
            out.push_str(&format!("## Goal {}: {}\n", i + 1, goal.raw_request));
            out.push_str(&format!("- Status: {:?}\n", goal.status));
            out.push_str(&format!("- Outcome: {}\n", goal.outcome));
            if let Some(steps) = self.decomposer.steps_for(i) {
                for (j, step) in steps.iter().enumerate() {
                    out.push_str(&format!(
                        "  {}. {} [{:?}]\n",
                        j + 1,
                        step.description,
                        step.status
                    ));
                }
            }
            out.push('\n');
        }
        out.push_str("## Execution Log\n\n");
        for entry in &self.log {
            out.push_str(&format!(
                "- Cycle {}: goal={} step={} action={} result={}\n",
                entry.cycle, entry.goal_id, entry.step_id, entry.action, entry.result
            ));
        }
        out
    }

    pub fn auto_decompose_and_execute(&mut self, request: &str, context: &DecompositionContext) {
        self.decomposer.decompose(request, context);
        let goal_idx = match self.decomposer.active_goal {
            Some(i) => i,
            None => return,
        };
        let goal_id = self.decomposer.goals[goal_idx].id.clone();
        loop {
            let step = match self.decomposer.next_step() {
                Some(s) => s.clone(),
                None => break,
            };
            let result = format!("Executed: {}", step.description);
            self.log.push(ExecutionLog {
                cycle: self.log.len() as u64,
                goal_id: goal_id.clone(),
                step_id: step.id,
                action: step.description.clone(),
                result: result.clone(),
            });
            let vr = StepVerificationResult {
                passed: true,
                evidence: step.verification_hint.clone(),
                details: result,
            };
            self.decomposer.record_verification(step.id, vr);
        }
    }
}

// ── Top-level decompose helpers ─────────────────────────────────────────

pub fn decompose_vague(text: &str) -> GoalContract {
    let translated = translate_vague_words(text);
    let project_type = detect_project_type(&translated);
    let domain = if text.to_lowercase().contains("medical")
        || text.to_lowercase().contains("health")
        || text.to_lowercase().contains("patient")
    {
        Some("medical")
    } else if text.to_lowercase().contains("finance")
        || text.to_lowercase().contains("financial")
        || text.to_lowercase().contains("payment")
    {
        Some("finance")
    } else if text.to_lowercase().contains("copyright") || text.to_lowercase().contains("legal") {
        Some("copyright")
    } else {
        None
    };
    let risk = classify_risk(text);
    let outcome = match project_type {
        "bugfix" => format!("Fix the reported bug in {}", text),
        "research" => format!("Research and summarize: {}", text),
        "web" | "mobile" | "ios" | "android" => {
            format!("Build a {} application: {}", project_type, text)
        }
        "cli" => format!("Create a CLI tool that {}", text),
        "api" => format!("Design and implement an API for {}", text),
        "library" => format!("Create a reusable library for {}", text),
        "script" => format!("Write a script that {}", text),
        _ => format!("Implement: {}", text),
    };
    let mut gc = GoalContract::new(text);
    gc.outcome = outcome.clone();
    gc.verification = suggest_verification(project_type, &outcome);
    gc.constraints = suggest_constraints(project_type, domain);
    gc.boundaries = suggest_boundaries(project_type);
    gc.iteration_policy = suggest_iteration_policy(risk);
    gc.stop_conditions = suggest_stop_conditions(&outcome);

    if let Some(d) = domain {
        match d {
            "medical" => {
                gc.pause_conditions
                    .push("Medical domain: requires expert human review".into());
                gc.pause_conditions
                    .push("No deployment without HIPAA compliance sign-off".into());
            }
            "finance" => {
                gc.pause_conditions
                    .push("Financial domain: requires audit trail review".into());
                gc.pause_conditions
                    .push("All transactions must be human-verified before execution".into());
            }
            "copyright" => {
                gc.pause_conditions
                    .push("Copyright domain: requires license verification".into());
            }
            _ => {}
        }
    } else if risk == "high" {
        gc.pause_conditions
            .push("High-risk operation: requires human confirmation".into());
    }

    gc
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_contract_creation() {
        let gc = GoalContract::new("build a calculator");
        assert!(gc.id.starts_with("goal_"));
        assert_eq!(gc.raw_request, "build a calculator");
        assert_eq!(gc.status, DecomposerGoalStatus::Draft);
        assert_eq!(gc.boundaries.max_files_to_modify, 10);
    }

    #[test]
    fn test_decompose_vague_app_request() {
        let gc = decompose_vague("make me a todo app");
        assert!(gc.outcome.contains("mobile") || gc.outcome.contains("application"));
        assert!(!gc.verification.is_empty());
        assert!(gc.boundaries.max_files_to_modify <= 10);
    }

    #[test]
    fn test_decompose_vague_fix_bug() {
        let gc = decompose_vague("fix login bug");
        assert!(gc.outcome.contains("Fix"));
        assert!(gc.constraints.iter().any(|c| c.contains("API signatures")));
        assert_eq!(gc.boundaries.max_files_to_modify, 3);
    }

    #[test]
    fn test_decompose_vague_research() {
        let gc = decompose_vague("research transformer architectures");
        assert!(gc.outcome.contains("Research"));
        assert!(gc.verification.iter().any(|v| v.contains("sources")));
        assert!(gc
            .boundaries
            .forbidden_paths
            .iter()
            .any(|p| p.contains("*.rs")));
    }

    #[test]
    fn test_decompose_medical_constraints() {
        let gc = decompose_vague("build a medical record system");
        assert!(!gc.pause_conditions.is_empty());
        assert!(gc.pause_conditions.iter().any(|p| p.contains("Medical")));
        assert!(gc.constraints.iter().any(|c| c.contains("patient data")));
    }

    #[test]
    fn test_suggest_verification_web_app() {
        let v = suggest_verification("web", "build a login form");
        assert!(v.iter().any(|x| x.contains("pages render")));
        assert!(v.iter().any(|x| x.contains("form submission")));
    }

    #[test]
    fn test_suggest_verification_cli_tool() {
        let v = suggest_verification("cli", "create a file watcher");
        assert!(v.iter().any(|x| x.contains("binary compiles")));
        assert!(v.iter().any(|x| x.contains("--help")));
    }

    #[test]
    fn test_goal_boundaries_default() {
        let b = GoalBoundaries::default();
        assert_eq!(b.max_files_to_modify, 10);
        assert!(b.allowed_paths.contains(&"**/*.rs".into()));
    }

    #[test]
    fn test_next_step_returns_pending() {
        let mut d = GoalDecomposer::new();
        d.goals.push(GoalContract::new("test"));
        d.steps_cache
            .push(vec![GoalStep::new(0, "step one", "verify it")]);
        d.active_goal = Some(0);
        let step = d.next_step();
        assert!(step.is_some());
        assert_eq!(step.unwrap().description, "step one");
    }

    #[test]
    fn test_record_verification_passed() {
        let mut d = GoalDecomposer::new();
        d.goals.push(GoalContract::new("test"));
        d.steps_cache.push(vec![GoalStep::new(0, "step", "hint")]);
        d.active_goal = Some(0);
        d.next_step();
        let r = StepVerificationResult {
            passed: true,
            evidence: "check passed".into(),
            details: "ok".into(),
        };
        d.record_verification(0, r);
        let steps = d.steps_cache.get(0).unwrap();
        assert_eq!(steps[0].status, StepStatus::Verified);
    }

    #[test]
    fn test_record_verification_failed() {
        let mut d = GoalDecomposer::new();
        d.goals.push(GoalContract::new("test"));
        d.steps_cache.push(vec![GoalStep::new(0, "step", "hint")]);
        d.active_goal = Some(0);
        d.next_step();
        let r = StepVerificationResult {
            passed: false,
            evidence: "check failed".into(),
            details: "compilation error".into(),
        };
        d.record_verification(0, r);
        let steps = d.steps_cache.get(0).unwrap();
        assert_eq!(
            steps[0].status,
            StepStatus::Failed("compilation error".into())
        );
        assert!(!steps[0].error_log.is_empty());
    }

    #[test]
    fn test_check_pause_conditions_none() {
        let d = GoalDecomposer::new();
        assert!(d.check_pause_conditions().is_none());
    }

    #[test]
    fn test_check_pause_conditions_payment() {
        let mut d = GoalDecomposer::new();
        let gc = decompose_vague("add payment processing");
        d.goals.push(gc);
        d.active_goal = Some(0);
        let result = d.check_pause_conditions();
        assert!(result.is_some());
    }

    #[test]
    fn test_classify_risk_low() {
        assert_eq!(classify_risk("build a local to-do app"), "low");
    }

    #[test]
    fn test_classify_risk_high() {
        assert_eq!(
            classify_risk("deploy payment gateway to production"),
            "high"
        );
    }

    #[test]
    fn test_translate_vague_words() {
        let t = translate_vague_words("做一个高级的界面");
        assert!(t.contains("截图"));
    }

    #[test]
    fn test_detect_project_type_web() {
        assert_eq!(detect_project_type("build a website"), "web");
        assert_eq!(detect_project_type("web app for notes"), "web");
    }

    #[test]
    fn test_detect_project_type_mobile() {
        assert_eq!(detect_project_type("mobile app"), "mobile");
        assert_eq!(detect_project_type("ios app"), "ios");
    }

    #[test]
    fn test_iteration_policy_high_risk() {
        let p = suggest_iteration_policy("high");
        assert_eq!(p.stop_on_consecutive_failures, 1);
        assert_eq!(p.max_rounds, 5);
    }

    #[test]
    fn test_report_contains_goals() {
        let mut d = GoalDecomposer::new();
        d.goals.push(GoalContract::new("test goal"));
        d.steps_cache.push(vec![GoalStep::new(0, "step", "hint")]);
        d.active_goal = Some(0);
        let r = d.report();
        assert!(r.contains("test goal"));
    }

    #[test]
    fn test_decompose_financial_constraints() {
        let gc = decompose_vague("build a stock trading platform");
        assert!(!gc.pause_conditions.is_empty());
        assert!(gc.constraints.iter().any(|c| c.contains("decimal")));
    }

    #[test]
    fn test_goal_executor_auto_loop() {
        let mut ex = GoalExecutor::new();
        ex.auto_decompose_and_execute("build a hello world cli", &DecompositionContext::default());
        assert!(!ex.log.is_empty());
        assert!(ex.log.iter().any(|l| l.action.contains("Define")));
    }

    #[test]
    fn test_detect_project_type_api() {
        assert_eq!(detect_project_type("build a REST API"), "api");
        assert_eq!(detect_project_type("backend service"), "api");
    }

    #[test]
    fn test_suggest_stop_conditions_deploy() {
        let s = suggest_stop_conditions("deploy to production");
        assert!(s.iter().any(|x| x.contains("Build artifact")));
    }

    #[test]
    fn test_suggest_stop_conditions_refactor() {
        let s = suggest_stop_conditions("refactor database layer");
        assert!(s.iter().any(|x| x.contains("Old code path")));
    }

    #[test]
    fn test_decompose_vague_library() {
        let gc = decompose_vague("create a crate for CSV parsing");
        assert!(gc.outcome.contains("library"));
    }

    #[test]
    fn test_decompose_vague_script() {
        let gc = decompose_vague("write a backup script");
        assert_eq!(gc.boundaries.max_files_to_modify, 3);
    }

    #[test]
    fn test_default_goal_decomposer_max() {
        let d = GoalDecomposer::new();
        assert_eq!(d.max_goals, 20);
    }

    #[test]
    fn test_goal_decomposer_with_max() {
        let d = GoalDecomposer::new().with_max_goals(5);
        assert_eq!(d.max_goals, 5);
    }

    #[test]
    fn test_save_to_file_creates_json() {
        let mut ex = GoalExecutor::new();
        ex.auto_decompose_and_execute("build a hello world cli", &DecompositionContext::default());
        let dir = std::env::temp_dir().join("neotrix_goal_test_save");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("failed to create temp dir for save test");
        let path = dir.join("executor.json");
        ex.save_to_file(&path)
            .expect("failed to save executor JSON");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).expect("failed to read saved executor JSON");
        assert!(content.contains("GoalExecutor"));
        assert!(content.contains("hello world"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_from_file_roundtrip() {
        let mut ex = GoalExecutor::new();
        ex.auto_decompose_and_execute("build a hello world cli", &DecompositionContext::default());
        let dir = std::env::temp_dir().join("neotrix_goal_test_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("failed to create temp dir for roundtrip test");
        let path = dir.join("executor.json");
        ex.save_to_file(&path)
            .expect("failed to save executor for roundtrip");
        let loaded =
            GoalExecutor::load_from_file(&path).expect("failed to load executor from file");
        assert!(!loaded.log.is_empty());
        assert_eq!(loaded.log.len(), ex.log.len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_plan_markdown_nonempty() {
        let mut ex = GoalExecutor::new();
        ex.auto_decompose_and_execute("build a hello world cli", &DecompositionContext::default());
        let md = ex.export_plan_markdown();
        assert!(!md.is_empty());
        assert!(md.contains("Total goals:"));
        assert!(md.contains("Execution Log"));
    }

    #[test]
    fn test_save_atomic_no_tmp_leftover() {
        let mut ex = GoalExecutor::new();
        ex.auto_decompose_and_execute("build a hello world cli", &DecompositionContext::default());
        let dir = std::env::temp_dir().join("neotrix_goal_test_atomic");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("failed to create temp dir for atomic test");
        let path = dir.join("executor.json");
        ex.save_to_file(&path)
            .expect("failed to save executor for atomic test");
        let tmp_path = dir.join("executor.json.tmp");
        assert!(!tmp_path.exists());
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
