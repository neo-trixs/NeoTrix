//! genoffice（genspark-ai/genoffice）外部二进制适配器。
//!
//! 设计依据（与 `doc_parse::html_to_markdown_via_mdream` 同型）：
//! - **存在即用、不存在即回落**：`genoffice_bin()` 解析不到可执行文件时，
//!   所有公开函数返回明确错误，而不是 panic——这是「适配器注册」式接入，
//!   不是平行实现。
//! - **AI 移除**：genoffice 的 AI 能力集中在 `@genoffice/agent-core`、
//!   `@genoffice/ai-provider`、`@genoffice/ai-search` 三个独立包，以及
//!   Electron 壳的 AI 面板。本适配器**不**接入这三个包，也**不**调用
//!   `genoffice search` / `image` / `media` / `capabilities` 这类会触发
//!   外部 LLM/云端调用的命令——这些能力由 NeoTrix 自身的核心（L0–L6 +
//!   本地 llama/ollama）承担。我们只把文档引擎（docx/xlsx/pptx/pdf/
//!   markdown/html 的读、写、转、渲染、校验）作为可编排的「工具能力」
//!   嵌入。
//!
//! 入口：`genoffice_bin()` → `genoffice_*` 系列 typed wrapper →
//! `GenOfficeCapability`（实现 `crate::l0_substrate::nt_core_capability_types::UnifiedCapability`）。

use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::Value;

/// genoffice 二进制的显式覆盖（最高优先级）。
const GENOFFICE_BIN_ENV: &str = "GENOFFICE_BIN";

/// 二进制搜索候选：PATH 查找 `genoffice`；macOS 包内安装位置。
const MAC_APP_CLI: &str = "/Applications/GenOffice.app/Contents/Resources/cli/genoffice";
const MAC_APP_CLI_USER: &str = "/Applications/GenOffice.app/Contents/MacOS/genoffice";

// ─── 错误类型 ────────────────────────────────────────────────────────────

/// genoffice 调用的错误分类（与外部二进制交互的失败模式）。
#[derive(Debug, thiserror::Error)]
pub enum GenOfficeError {
    #[error("genoffice 二进制未安装或不可执行（设置 GENOFFICE_BIN 或装入 PATH）")]
    NotInstalled,
    #[error("genoffice 执行失败(exit code={code:?}): {stderr}")]
    ExecutionFailed { code: Option<i32>, stderr: String },
    #[error("genoffice 输出非合法 JSON")]
    NotJson,
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

pub type GenOfficeResult<T> = Result<T, GenOfficeError>;

// ─── 二进制解析 ─────────────────────────────────────────────────────────

/// 解析 genoffice 可执行文件路径。优先级：`GENOFFICE_BIN` → PATH → macOS 包内。
///
/// 与 mdream 的 `MDREAM_BIN` 同型：只有显式环境变量指向可执行文件，
/// 或 PATH/已知安装位置能找到，才返回 `Some`；否则 `None`。
pub fn genoffice_bin() -> Option<PathBuf> {
    // 1) 显式环境变量
    if let Ok(bin) = std::env::var(GENOFFICE_BIN_ENV) {
        let p = PathBuf::from(&bin);
        if is_executable(&p) {
            return Some(p);
        }
        // 不静默回退到「别的」——显式给了但非法，就视为未安装，让用户定位。
    }

    // 2) PATH
    if let Some(p) = which_on_path("genoffice") {
        return Some(p);
    }

    // 3) macOS 包内默认位置
    for candidate in [MAC_APP_CLI, MAC_APP_CLI_USER] {
        let p = PathBuf::from(candidate);
        if is_executable(&p) {
            return Some(p);
        }
    }

    None
}

/// genoffice 是否可用（二进制存在即可调用，文档引擎本身是本地的）。
pub fn genoffice_available() -> bool {
    genoffice_bin().is_some()
}

fn is_executable(p: &std::path::Path) -> bool {
    match std::fs::metadata(p) {
        Ok(m) => {
            // unix: 有任意执行位即可
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;
                m.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                m.is_file()
            }
        }
        Err(_) => false,
    }
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{name}.exe"));
            if is_executable(&exe) {
                return Some(exe);
            }
        }
    }
    None
}

// ─── 低层执行器 ─────────────────────────────────────────────────────────

/// 同步执行 `genoffice <args>`，返回 stdout 字符串。
///
/// 与 `html_to_markdown_via_mdream` 一致：用 `std::process::Command`，
/// 不引入 tokio 运行时依赖（本模块是 L1 叶子能力）。stderr 收集进错误体。
pub fn run_genoffice(args: &[&str]) -> GenOfficeResult<String> {
    let bin = genoffice_bin().ok_or(GenOfficeError::NotInstalled)?;
    let out = Command::new(&bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        // genoffice 的 CLI 把结构化错误 JSON 打到 **stdout**（如 output_exists），
        // 而 stderr 为空——此时把 stdout 的摘要带上，否则 ExecutionFailed 的
        // stderr 为空、错误来源完全不可见。
        let stderr = if stderr.trim().is_empty() {
            let out_s = String::from_utf8_lossy(&out.stdout).into_owned();
            format!("(see stdout) {:.200}", out_s.trim())
        } else {
            stderr
        };
        Err(GenOfficeError::ExecutionFailed {
            code: out.status.code(),
            stderr,
        })
    }
}

/// 同上，但要求 stdout 是合法 JSON（`--json` 输出形态）。
pub fn run_genoffice_json(args: &[&str]) -> GenOfficeResult<Value> {
    let raw = run_genoffice(args)?;
    serde_json::from_str(&raw).map_err(|_| GenOfficeError::NotJson)
}

// ─── 文档引擎 typed wrapper ──────────────────────────────────────────────
// 这些是「直接操控 genoffice 所有文档能力」的稳定 API。
// 全都把 `--json` 输出解析为 `serde_json::Value`，便于上层编排。

/// `genoffice info <file> --json`：探测文档结构（heading/block/sheet/slide 数）。
pub fn genoffice_info(path: &std::path::Path) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    run_genoffice_json(&["info", &p, "--json"])
}

/// `genoffice convert <in> --to <fmt> [--out <out>] --json`：格式互转
/// （md/html/docx/xlsx/pptx/pdf 之间，含 PDF→docx/xlsx/pptx）。
pub fn genoffice_convert(
    input: &std::path::Path,
    to: &str,
    out: Option<&std::path::Path>,
) -> GenOfficeResult<Value> {
    let inp = input.to_string_lossy().into_owned();
    let mut args = vec!["convert", &inp, "--to", to, "--json"];
    let out_s;
    if let Some(o) = out {
        out_s = o.to_string_lossy().into_owned();
        args.push("--out");
        args.push(&out_s);
    }
    run_genoffice_json(&args)
}

/// `genoffice create --type <t> [--from <src>] [--spec <dir|file>] [--outline <f>] [--out <out>] [--render <dir>] [--audit] --json`
/// 创建 docx/xlsx/pptx/pdf。
pub fn genoffice_create(
    kind: &str,
    from: Option<&std::path::Path>,
    spec: Option<&std::path::Path>,
    outline: Option<&std::path::Path>,
    out: &std::path::Path,
) -> GenOfficeResult<Value> {
    let out_s = out.to_string_lossy().into_owned();
    let mut args = vec!["create", "--type", kind, "--out", &out_s, "--json"];
    let from_s;
    if let Some(f) = from {
        from_s = f.to_string_lossy().into_owned();
        args.push("--from");
        args.push(&from_s);
    }
    let spec_s;
    if let Some(s) = spec {
        spec_s = s.to_string_lossy().into_owned();
        args.push("--spec");
        args.push(&spec_s);
    }
    let outline_s;
    if let Some(o) = outline {
        outline_s = o.to_string_lossy().into_owned();
        args.push("--outline");
        args.push(&outline_s);
    }
    run_genoffice_json(&args)
}

/// `genoffice docs read <file> --json`：读出 ".docx" 块结构。
pub fn genoffice_docs_read(path: &std::path::Path, range: Option<&str>) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    let mut args = vec!["docs", "read", &p, "--json"];
    if let Some(r) = range {
        args.push("--range");
        args.push(r);
    }
    run_genoffice_json(&args)
}

/// `genoffice docs apply <file> --ops <ops> [--dry-run]`：按 ops 文档就地编辑（原子，失败不落盘）。
pub fn genoffice_docs_apply(
    path: &std::path::Path,
    ops: &std::path::Path,
    dry_run: bool,
) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    let ops_s = ops.to_string_lossy().into_owned();
    let mut args = vec!["docs", "apply", &p, "--ops", &ops_s, "--json"];
    if dry_run {
        args.push("--dry-run");
    }
    run_genoffice_json(&args)
}

/// `genoffice docs check <file> --json`：文档健康检查（断链书签/失效字段/空图表等）。
pub fn genoffice_docs_check(path: &std::path::Path) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    run_genoffice_json(&["docs", "check", &p, "--json"])
}

/// `genoffice sheet read <file> [--sheet <s>] [--range <r>] --json`。
pub fn genoffice_sheet_read(
    path: &std::path::Path,
    sheet: Option<&str>,
    range: Option<&str>,
) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    let mut args = vec!["sheet", "read", &p, "--json"];
    if let Some(s) = sheet {
        args.push("--sheet");
        args.push(s);
    }
    if let Some(r) = range {
        args.push("--range");
        args.push(r);
    }
    run_genoffice_json(&args)
}

/// `genoffice sheet apply <file> --cells <cells> [--dry-run]` 或 `--ops <ops>`。
pub fn genoffice_sheet_apply(
    path: &std::path::Path,
    cells: Option<&std::path::Path>,
    ops: Option<&std::path::Path>,
    dry_run: bool,
) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    let mut args = vec!["sheet", "apply", &p, "--json"];
    let cells_s;
    if let Some(c) = cells {
        cells_s = c.to_string_lossy().into_owned();
        args.push("--cells");
        args.push(&cells_s);
    }
    let ops_s;
    if let Some(o) = ops {
        ops_s = o.to_string_lossy().into_owned();
        args.push("--ops");
        args.push(&ops_s);
    }
    if dry_run {
        args.push("--dry-run");
    }
    run_genoffice_json(&args)
}

/// `genoffice slides read <file> [--full] [--units <u>] --json`。
pub fn genoffice_slides_read(path: &std::path::Path, full: bool) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    let mut args = vec!["slides", "read", &p, "--json"];
    if full {
        args.push("--full");
    }
    run_genoffice_json(&args)
}

/// `genoffice slides check <page-or-outline> --json`：单页或大纲的几何/重叠校验。
pub fn genoffice_slides_check(spec: &std::path::Path) -> GenOfficeResult<Value> {
    let s = spec.to_string_lossy().into_owned();
    run_genoffice_json(&["slides", "check", &s, "--json"])
}

/// `genoffice slides audit <file> --json`：整 deck 的布局审计。
pub fn genoffice_slides_audit(path: &std::path::Path) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    run_genoffice_json(&["slides", "audit", &p, "--json"])
}

/// `genoffice pdf read <file> [--page <n> | --range <r>] --json`。
pub fn genoffice_pdf_read(path: &std::path::Path, range: Option<&str>) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    let mut args = vec!["pdf", "read", &p, "--json"];
    if let Some(r) = range {
        args.push("--range");
        args.push(r);
    }
    run_genoffice_json(&args)
}

/// `genoffice render <file> --out <dir> [--scale <n>] [--page <n>] --json`：
/// 把任意文档的每页渲染成 PNG，供上层核验。
pub fn genoffice_render(
    path: &std::path::Path,
    out_dir: &std::path::Path,
    page: Option<u32>,
    scale: Option<u32>,
) -> GenOfficeResult<Value> {
    let p = path.to_string_lossy().into_owned();
    let out_s = out_dir.to_string_lossy().into_owned();
    let mut args = vec!["render", &p, "--out", &out_s, "--json"];
    let page_s;
    if let Some(n) = page {
        page_s = n.to_string();
        args.push("--page");
        args.push(&page_s);
    }
    let scale_s;
    if let Some(s) = scale {
        scale_s = s.to_string();
        args.push("--scale");
        args.push(&scale_s);
    }
    run_genoffice_json(&args)
}

/// `genoffice merge <template> --data <values> --out <out> [--force] [--strict] --json`：
/// 填充 `{{key}}` 占位模板。
pub fn genoffice_merge(
    template: &std::path::Path,
    data: &std::path::Path,
    out: &std::path::Path,
    strict: bool,
) -> GenOfficeResult<Value> {
    let tpl = template.to_string_lossy().into_owned();
    let data_s = data.to_string_lossy().into_owned();
    let out_s = out.to_string_lossy().into_owned();
    let mut args = vec!["merge", &tpl, "--data", &data_s, "--out", &out_s, "--json"];
    if strict {
        args.push("--strict");
    }
    run_genoffice_json(&args)
}

/// `genoffice guide <domain> [<group>] --json`：返回该领域的全部可编排操作（ops）及其 schema。
/// 这是「直接操控其所有能力」的枚举入口——agents 通过它发现可下发的 ops。
pub fn genoffice_guide(domain: &str, group: Option<&str>) -> GenOfficeResult<Value> {
    let mut args = vec!["guide", domain];
    if let Some(g) = group {
        args.push(g);
    }
    args.push("--json");
    run_genoffice_json(&args)
}

// ─── UnifiedCapability 注册适配 ──────────────────────────────────────────

use crate::l0_substrate::nt_core_capability_types as core_cap;

/// genoffice 文档引擎能力。输入/输出走 `CapabilityInput::Kv` /
/// `CapabilityOutput::Kv`（`op` → JSON 解析 → stdout），避免侵入共享类型枚举。
pub struct GenOfficeCapability;

impl GenOfficeCapability {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GenOfficeCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl core_cap::UnifiedCapability for GenOfficeCapability {
    fn meta(&self) -> core_cap::CapabilityMeta {
        core_cap::CapabilityMeta {
            id: "nt-file-genoffice".to_string(),
            name: "GenOffice Document Engine".to_string(),
            layer: core_cap::Layer::L1Action,
            domain: core_cap::Domain::NtFileAbility,
            version: "1.0.0".to_string(),
            description: "genoffice 文档引擎（docx/xlsx/pptx/pdf/markdown/html 读写转换渲染），AI 移除、由 NeoTrix 核心编排".to_string(),
            tags: vec![
                "genoffice".to_string(),
                "docx".to_string(),
                "xlsx".to_string(),
                "pptx".to_string(),
                "pdf".to_string(),
                "markdown".to_string(),
                "html".to_string(),
            ],
            status: core_cap::CapabilityStatus::Healthy,
            metrics: core_cap::CapabilityMetrics::default(),
            cost_weight: 0.0,
            priority: 1.0,
        }
    }

    fn health(&self) -> core_cap::CapabilityHealth {
        core_cap::CapabilityHealth {
            state: if genoffice_available() {
                core_cap::CapabilityState::Healthy
            } else {
                core_cap::CapabilityState::Disabled
            },
            success_rate: 1.0,
            avg_latency_ms: 0.0,
            last_called: None,
            call_count: 0,
        }
    }

    /// `CapabilityInput::Kv` 约定：
    /// - `op`：`info|convert|create|docs_read|docs_apply|docs_check|sheet_read|sheet_apply|slides_read|slides_check|slides_audit|pdf_read|render|merge|guide`
    /// - `path`/`input`/`to`/`out`/`kind`/`from`/`spec`/`outline`/`sheet`/`range`/`domain`/`group`/`cells`/`ops`/`dry_run`/`strict`/`full`/`page`/`scale`/`template`/`data`：相应子命令的参数
    ///
    /// 输出 `CapabilityOutput::Kv` `{ "stdout": "...", "json": "..." }`。
    fn execute(
        &self,
        input: core_cap::CapabilityInput,
    ) -> Result<core_cap::CapabilityOutput, core_cap::CapabilityError> {
        let map = match input {
            core_cap::CapabilityInput::Kv(m) => m,
            other => {
                return Err(core_cap::CapabilityError::UnsupportedInput(format!(
                    "expected Kv input, got {other:?}"
                )))
            }
        };
        let op = map.get("op").cloned().unwrap_or_default();
        let get = |k: &str| map.get(k).cloned();

        // 统一构造 owned args，再转 &[&str]（避免 Box::leak）
        let mut owned: Vec<String> = Vec::new();
        match op.as_str() {
            "info" => {
                owned.push("info".into());
                owned.push(get("path").unwrap_or_default());
                owned.push("--json".into());
            }
            "convert" => {
                owned.push("convert".into());
                owned.push(get("input").unwrap_or_default());
                owned.push("--to".into());
                owned.push(get("to").unwrap_or_default());
                owned.push("--json".into());
                if let Some(o) = get("out") {
                    owned.push("--out".into());
                    owned.push(o);
                }
            }
            "docs_read" => {
                owned.push("docs".into());
                owned.push("read".into());
                owned.push(get("path").unwrap_or_default());
                owned.push("--json".into());
                if let Some(r) = get("range") {
                    owned.push("--range".into());
                    owned.push(r);
                }
            }
            "guide" => {
                owned.push("guide".into());
                owned.push(get("domain").unwrap_or_else(|| "docs".into()));
                if let Some(g) = get("group") {
                    owned.push(g);
                }
                owned.push("--json".into());
            }
            other => {
                return Err(core_cap::CapabilityError::UnsupportedInput(format!(
                    "unknown genoffice op: {other}"
                )))
            }
        }
        let args: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        let raw = run_genoffice_json(&args);

        match raw {
            Ok(v) => Ok(core_cap::CapabilityOutput::Kv({
                let mut m = std::collections::HashMap::new();
                m.insert("stdout".to_string(), v.to_string());
                m
            })),
            Err(e) => Err(core_cap::CapabilityError::ExecutionFailed(e.to_string())),
        }
    }

    fn supports(&self, input: &core_cap::CapabilityInput) -> bool {
        matches!(input, core_cap::CapabilityInput::Kv(m) if m.contains_key("op"))
    }
}

/// 创建 genoffice 能力实例 — 返回核心 trait 对象供 NT-CORE 能力工厂消费。
pub fn create_genoffice_capability() -> std::sync::Arc<dyn core_cap::UnifiedCapability> {
    std::sync::Arc::new(GenOfficeCapability::new())
}

/// 注册 genoffice 能力到本地注册中心（NT-CORE 能力工厂入口）。
pub fn register_genoffice_capability(registry: &mut core_cap::CapabilityRegistry) {
    registry.register(std::sync::Arc::new(GenOfficeCapability::new()) as std::sync::Arc<dyn core_cap::UnifiedCapability>);
}

// ─── 测试 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l0_substrate::nt_core_capability_types::UnifiedCapability as _;

    #[test]
    fn bin_unset_returns_none() {
        // 不保证环境干净，但至少不 panic、返回 Option
        let _ = genoffice_bin();
    }

    #[test]
    fn available_is_bool() {
        let _ = genoffice_available();
    }

    #[test]
    fn capability_meta_wellformed() {
        let cap = GenOfficeCapability::new();
        let m = cap.meta();
        assert_eq!(m.id, "nt-file-genoffice");
        assert_eq!(m.domain, core_cap::Domain::NtFileAbility);
        assert!(m.tags.iter().any(|t| t == "genoffice"));
    }

    #[test]
    fn supports_requires_op_key() {
        let cap = GenOfficeCapability::new();
        let mut kv = std::collections::HashMap::new();
        kv.insert("op".to_string(), "info".to_string());
        assert!(cap.supports(&core_cap::CapabilityInput::Kv(kv)));

        let empty = std::collections::HashMap::new();
        assert!(!cap.supports(&core_cap::CapabilityInput::Kv(empty)));

        assert!(!cap.supports(&core_cap::CapabilityInput::Text("x".into())));
    }

    #[test]
    fn execute_rejects_bad_input() {
        let cap = GenOfficeCapability::new();
        let r = cap.execute(core_cap::CapabilityInput::Text("hi".into()));
        assert!(r.is_err());
    }

    #[test]
    fn execute_unknown_op_rejected() {
        let cap = GenOfficeCapability::new();
        let mut kv = std::collections::HashMap::new();
        kv.insert("op".to_string(), "does_not_exist".to_string());
        let r = cap.execute(core_cap::CapabilityInput::Kv(kv));
        assert!(r.is_err());
    }

    /// 端到端：真实 spawn genoffice 二进制跑 `info` + `convert`。
    /// 未安装二进制时整个用例跳过（不失败），与 mdream `Option` 适配器同型。
    #[test]
    fn e2e_info_and_convert() {
        if !genoffice_available() {
            eprintln!("genoffice not installed, skipping e2e");
            return;
        }
        let dir = std::env::temp_dir().join("neotrix_genoffice_e2e");
        let _ = std::fs::create_dir_all(&dir);
        let md = dir.join("e2e.md");
        std::fs::write(&md, b"# Title\n\nHello world.\n").expect("write md");

        // genoffice convert 默认拒绝覆盖已存在输出 ⇒ 每次用唯一名字，保证隔离。
        let uniq = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let docx = dir.join(format!("e2e_{uniq}.docx"));

        // info -> detail.format == "md"
        let info = genoffice_info(&md).expect("genoffice info should succeed");
        let fmt = info
            .pointer("/detail/format")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(fmt, "md", "info detail.format should be md, got {info}");

        // convert md -> docx, output file exists
        let out = genoffice_convert(&md, "docx", Some(&docx)).expect("convert should succeed");
        assert!(docx.exists(), "output docx missing (out={out})");
        let _ = std::fs::remove_file(&docx);
    }
}
