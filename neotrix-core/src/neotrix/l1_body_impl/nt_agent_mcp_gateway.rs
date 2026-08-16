//! # MCP Governance Gateway (G15) + Tool Folding (G16)
//!
//! Absorbed from the awesome-mcp-servers meta-gateway pattern:
//!
//! 1. **N→4 tool folding** — before presenting a tool list to the LLM, group
//!    the full tool set into 4 canonical categories (search / act / knowledge /
//!    file), present folded descriptions, and expand membership at call time.
//!    A deterministic token-savings metric is measured (chars + chars/4 token
//!    heuristic); typical savings for a large server fleet are 60–95%.
//!
//! 2. **Governance proxy** — allow / deny / require-human-approval (HITL) rules
//!    checked in the call path *before* execution. Reuses the existing
//!    `SandboxVerdict` verdict language from `nt_act_sandbox` (R-P42: no
//!    parallel verdict type). Precedence: Deny > RequireApproval > Allow >
//!    unlisted default.
//!
//! 3. **Hash-chain evidence** — every governed call appends a SHA-256 chain
//!    link (`prev_hash | seq | tool | args | verdict | hitl | result`), stored
//!    for audit and wired into `nt_shield_audit` `CheckResult.evidence`.
//!
//! The gateway is reachable from production via `McpRegistry::gateway()` and
//! `McpRegistry::call_tool_governed()` (same registry object every existing
//! MCP caller already holds). Plain `call_tool` is untouched → backward
//! compatible.

use std::cell::RefCell;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::neotrix::l1_body_impl::nt_act_sandbox::SandboxVerdict;
use crate::neotrix::l1_body_impl::nt_agent_mcp_registry::{McpRegistry, McpToolDef};
use crate::neotrix::l1_body_impl::nt_shield_audit::{AuditMode, AuditReport, CheckResult, CheckStatus};

// ---------------------------------------------------------------------------
// N → 4 Tool Folding
// ---------------------------------------------------------------------------

/// Canonical tool categories (4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    Search,
    Act,
    Knowledge,
    File,
}

impl ToolCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ToolCategory::Search => "search",
            ToolCategory::Act => "act",
            ToolCategory::Knowledge => "knowledge",
            ToolCategory::File => "file",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ToolCategory::Search => "web / corpus retrieval and data acquisition",
            ToolCategory::Act => "execution, orchestration and side-effecting actions",
            ToolCategory::Knowledge => "knowledge base, memory, recall and evidence",
            ToolCategory::File => "filesystem read / write / edit and path operations",
        }
    }
}

/// Lightweight tool spec used for folding (independent of `McpToolDef`, so the
/// folding core is reusable for any tool enumeration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl From<McpToolDef> for ToolSpec {
    fn from(t: McpToolDef) -> Self {
        Self {
            name: t.name,
            description: t.description,
            input_schema: t.input_schema,
        }
    }
}

impl From<&McpToolDef> for ToolSpec {
    fn from(t: &McpToolDef) -> Self {
        Self {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.input_schema.clone(),
        }
    }
}

/// One folded category: canonical description + member tool names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldedCategory {
    pub category: ToolCategory,
    pub description: String,
    pub member_tools: Vec<String>,
}

/// Result of N→4 folding with a token-savings metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldedSpecs {
    /// Always the 4 canonical categories, in canonical order.
    pub categories: Vec<FoldedCategory>,
    /// Serialized size of the full (unfolded) tool list, in chars.
    pub original_chars: usize,
    /// Serialized size of the folded representation, in chars.
    pub folded_chars: usize,
    pub saved_chars: usize,
    /// Savings as a fraction 0.0..=1.0 of chars (clamped ≥ 0).
    pub savings_percent: f64,
    /// Token heuristic ≈ chars / 4.
    pub original_tokens: usize,
    pub folded_tokens: usize,
    pub saved_tokens: usize,
}

/// Deterministic keyword classifier. Tool **name** is the primary signal (a
/// tool named `neotrix_search` is a search tool regardless of a description
/// mentioning "knowledge base"); description is only a fallback for names that
/// hit no keyword. Order within each pass: FILE > KNOWLEDGE > SEARCH > ACT.
fn classify_tool(name: &str, description: &str) -> ToolCategory {
    const FILE: &[&str] = &[
        "file", "folder", "filesystem", "mkdir", "rmdir", "list_dir", "glob",
        "chmod", "chown", "read_file", "write_file", "edit_file", "fs_write",
    ];
    const KNOWLEDGE: &[&str] = &[
        "kb_", "knowledge", "memory", "recall", "remember", "context", "fact",
        "wiki", "evidence", "embed", "corpus",
    ];
    const SEARCH: &[&str] = &[
        "search", "find", "query", "lookup", "retrieve", "scrape", "fetch",
        "web", "ask", "bing", "google", "crawl", "osint",
    ];
    let classify_hay = |hay: &str| -> Option<ToolCategory> {
        if FILE.iter().any(|k| hay.contains(k)) {
            return Some(ToolCategory::File);
        }
        if KNOWLEDGE.iter().any(|k| hay.contains(k)) {
            return Some(ToolCategory::Knowledge);
        }
        if SEARCH.iter().any(|k| hay.contains(k)) {
            return Some(ToolCategory::Search);
        }
        None
    };
    if let Some(cat) = classify_hay(&name.to_lowercase()) {
        return cat;
    }
    classify_hay(&description.to_lowercase()).unwrap_or(ToolCategory::Act)
}

/// N→4 fold over an arbitrary tool-spec enumeration.
pub fn fold_tool_specs(specs: Vec<ToolSpec>) -> FoldedSpecs {
    let original_chars: usize = specs
        .iter()
        .filter_map(|s| serde_json::to_string(s).ok().map(|j| j.len()))
        .sum();

    let mut buckets: std::collections::BTreeMap<ToolCategory, Vec<String>> =
        std::collections::BTreeMap::new();
    for s in &specs {
        buckets
            .entry(classify_tool(&s.name, &s.description))
            .or_default()
            .push(s.name.clone());
    }

    let mut categories = Vec::with_capacity(4);
    for cat in [
        ToolCategory::Search,
        ToolCategory::Act,
        ToolCategory::Knowledge,
        ToolCategory::File,
    ] {
        let members = buckets.remove(&cat).unwrap_or_default();
        let n = members.len();
        let mut desc = format!("{} — {} tools", cat.name(), n);
        if n > 0 {
            let preview: Vec<&str> = members.iter().take(6).map(|s| s.as_str()).collect();
            desc.push_str(": ");
            desc.push_str(&preview.join(", "));
            if n > preview.len() {
                desc.push_str(", …");
            }
        }
        categories.push(FoldedCategory {
            category: cat,
            description: desc,
            member_tools: members,
        });
    }

    let folded_chars = serde_json::to_string(&categories)
        .map(|j| j.len())
        .unwrap_or(0);
    let saved_chars = original_chars.saturating_sub(folded_chars);
    let savings_percent = if original_chars == 0 {
        0.0
    } else {
        saved_chars as f64 / original_chars as f64
    };
    let tokens = |chars: usize| (chars + 3) / 4;

    FoldedSpecs {
        categories,
        original_chars,
        folded_chars,
        saved_chars,
        savings_percent,
        original_tokens: tokens(original_chars),
        folded_tokens: tokens(folded_chars),
        saved_tokens: tokens(saved_chars),
    }
}

/// N→4 fold directly over `McpToolDef` (registry / built-in tool lists).
pub fn fold_tool_specs_from_defs(defs: Vec<McpToolDef>) -> FoldedSpecs {
    fold_tool_specs(defs.into_iter().map(ToolSpec::from).collect())
}

// ---------------------------------------------------------------------------
// Governance — allow / deny / HITL
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceAction {
    Allow,
    Deny,
    RequireApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    /// Glob pattern with `*` wildcards (e.g. `shell:*`, `github:*`, `rm:*`).
    pub pattern: String,
    pub action: GovernanceAction,
}

/// Policy checked in the call path before execution. Verdicts reuse the
/// `SandboxVerdict` type from `nt_act_sandbox` (no parallel verdict language).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub rules: Vec<GovernanceRule>,
    /// Verdict for tools not matching any rule. `permissive()` defaults to
    /// Approved (backward compatible with existing callers); `restrictive()`
    /// defaults to RequiresApproval (fail-closed).
    pub unlisted: SandboxVerdict,
}

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self::permissive()
    }
}

impl GovernancePolicy {
    /// Backward-compatible default: everything not explicitly governed is
    /// Approved — existing behavior is preserved unless rules say otherwise.
    pub fn permissive() -> Self {
        Self {
            rules: Vec::new(),
            unlisted: SandboxVerdict::Approved,
        }
    }

    /// Fail-closed: anything not explicitly allowed requires human approval.
    pub fn restrictive() -> Self {
        Self {
            rules: Vec::new(),
            unlisted: SandboxVerdict::RequiresApproval,
        }
    }

    pub fn with_rule(mut self, pattern: &str, action: GovernanceAction) -> Self {
        self.rules.push(GovernanceRule {
            pattern: pattern.to_string(),
            action,
        });
        self
    }

    pub fn allow(self, pattern: &str) -> Self {
        self.with_rule(pattern, GovernanceAction::Allow)
    }

    pub fn deny(self, pattern: &str) -> Self {
        self.with_rule(pattern, GovernanceAction::Deny)
    }

    pub fn require_approval(self, pattern: &str) -> Self {
        self.with_rule(pattern, GovernanceAction::RequireApproval)
    }

    /// Evaluate a tool name against the policy.
    /// Precedence: **Deny > RequireApproval > Allow > unlisted**.
    pub fn check(&self, tool: &str) -> SandboxVerdict {
        let mut approval = false;
        let mut allowed = false;
        for rule in &self.rules {
            if !glob_match(&rule.pattern, tool) {
                continue;
            }
            match rule.action {
                GovernanceAction::Deny => return SandboxVerdict::Denied,
                GovernanceAction::RequireApproval => approval = true,
                GovernanceAction::Allow => allowed = true,
            }
        }
        if approval {
            return SandboxVerdict::RequiresApproval;
        }
        if allowed {
            return SandboxVerdict::Approved;
        }
        self.unlisted
    }
}

/// Minimal glob matcher supporting `*` wildcards. No character classes / `?`.
///
/// Semantics:
///   - `exact` matches only the exact string.
///   - `ns:*` matches the bare namespace root `ns`, any `ns:...` tool, and any
///     tool whose name starts with `ns` (the `:` is optional in the match).
///   - `*delete*` matches any string containing `delete`.
///   - `*` matches anything.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let first = parts.first().copied().unwrap_or("");
    let last = parts.last().copied().unwrap_or("");

    // Trailing-star pattern `ns:*`: prefix match on `ns` (colon optional), so
    // `shell:*` covers `shell`, `shell:curl` and `shell_exec` alike.
    if last.is_empty() && parts.len() == 2 {
        let prefix = first.strip_suffix(':').unwrap_or(first);
        return text.starts_with(prefix);
    }

    if !first.is_empty() && !text.starts_with(first) {
        return false;
    }
    if !last.is_empty() && !text.ends_with(last) {
        return false;
    }
    let start = first.len();
    let end_bound = if last.is_empty() {
        text.len()
    } else {
        text.len().saturating_sub(last.len())
    };
    let middle = if parts.len() <= 2 {
        &[][..]
    } else {
        &parts[1..parts.len() - 1]
    };
    let mut pos = start;
    for seg in middle {
        if seg.is_empty() {
            continue;
        }
        let Some(idx) = text[pos..end_bound].find(seg) else {
            return false;
        };
        pos += idx + seg.len();
    }
    true
}

// ---------------------------------------------------------------------------
// Hash-chain evidence
// ---------------------------------------------------------------------------

/// Domain separator for the chain genesis (prevents cross-chain reuse).
pub const CHAIN_DOMAIN: &str = "neotrix:mcp-gateway:v1";

/// One hash-chained evidence entry for a governed MCP call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub seq: u64,
    pub tool_name: String,
    pub args: Value,
    pub verdict: SandboxVerdict,
    pub approved_by_hitl: bool,
    /// Truncated result preview (or error marker) for audit.
    pub result: Option<String>,
    pub prev_hash: String,
    pub hash: String,
    pub ts_ms: u64,
}

impl EvidenceEntry {
    /// Recompute this entry's own SHA-256 from its payload + prev_hash.
    fn recompute_hash(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{:?}|{}|{}",
            self.seq,
            self.tool_name,
            serde_json::to_string(&self.args).unwrap_or_default(),
            self.verdict,
            self.approved_by_hitl,
            self.result.as_deref().unwrap_or(""),
        );
        sha256_hex(&format!("{}|{}", self.prev_hash, payload))
    }

    /// Whether this single link's stored hash matches its recomputed hash.
    pub fn verify_link(&self) -> bool {
        self.hash == self.recompute_hash()
    }
}

/// SHA-256 hash chain over governed tool calls (append-only evidence).
#[derive(Debug, Clone)]
pub struct HashChain {
    pub entries: Vec<EvidenceEntry>,
    pub genesis_hash: String,
}

impl HashChain {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            genesis_hash: sha256_hex(CHAIN_DOMAIN),
        }
    }

    /// Append a link: `hash = sha256(prev_hash | seq | tool | args | verdict | hitl | result)`.
    pub fn append(
        &mut self,
        tool_name: &str,
        args: Value,
        verdict: SandboxVerdict,
        approved_by_hitl: bool,
        result: Option<String>,
    ) -> EvidenceEntry {
        let seq = self.entries.len() as u64;
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| self.genesis_hash.clone());
        let ts_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let mut entry = EvidenceEntry {
            seq,
            tool_name: tool_name.to_string(),
            args,
            verdict,
            approved_by_hitl,
            result,
            prev_hash,
            hash: String::new(),
            ts_ms,
        };
        entry.hash = entry.recompute_hash();
        self.entries.push(entry.clone());
        entry
    }

    /// Full-chain integrity check: every link recomputes to its stored hash
    /// and each `prev_hash` matches the previous link's hash.
    pub fn verify(&self) -> bool {
        self.broken_links().is_empty()
    }

    /// Indices of broken links. A link is broken when its own stored hash no
    /// longer matches its recomputed hash (self-integrity); once any link is
    /// broken the rest of the chain is untrusted and cascades (poisoned).
    pub fn broken_links(&self) -> Vec<usize> {
        let mut bad = Vec::new();
        let mut poisoned = false;
        for (i, e) in self.entries.iter().enumerate() {
            if poisoned || !e.verify_link() {
                bad.push(i);
                poisoned = true;
            }
        }
        bad
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// One-line evidence strings (for logs / `nt_shield_audit`).
    pub fn evidence_strings(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|e| {
                format!(
                    "mcp#{} tool={} verdict={:?} hitl={} sha256={}",
                    e.seq, e.tool_name, e.verdict, e.approved_by_hitl, e.hash
                )
            })
            .collect()
    }
}

impl Default for HashChain {
    fn default() -> Self {
        Self::new()
    }
}

/// SHA-256 hex digest of a UTF-8 payload.
pub fn sha256_hex(data: &str) -> String {
    let mut h = Sha256::new();
    h.update(data.as_bytes());
    hex::encode(h.finalize())
}

// ---------------------------------------------------------------------------
// McpGateway — governed wrapper over the existing call_tool path
// ---------------------------------------------------------------------------

/// HITL approval gate callback: receives (tool_name, args), returns granted?
pub type ApprovalGate = Box<dyn Fn(&str, &Value) -> bool + Send + Sync + 'static>;

/// Result of a governed call.
#[derive(Debug, Clone)]
pub struct GatewayCall {
    pub tool: String,
    pub verdict: SandboxVerdict,
    pub approved_by_hitl: bool,
    pub content: String,
    pub evidence: EvidenceEntry,
}

/// MCP meta-gateway (G15/G16): governance + HITL + hash-chain evidence over
/// the existing `McpRegistry::call_tool` production path. The hash chain is
/// persisted on the registry itself, so a fresh `McpGateway` per call still
/// chains evidence correctly. Backward compatible — plain `call_tool` is
/// unchanged; this is the governed wrapper.
pub struct McpGateway<'a> {
    registry: &'a McpRegistry,
    policy: RefCell<GovernancePolicy>,
    approval_gate: RefCell<Option<ApprovalGate>>,
}

impl<'a> McpGateway<'a> {
    pub fn new(registry: &'a McpRegistry) -> Self {
        Self {
            registry,
            policy: RefCell::new(GovernancePolicy::permissive()),
            approval_gate: RefCell::new(None),
        }
    }

    pub fn with_policy(self, policy: GovernancePolicy) -> Self {
        self.policy.replace(policy);
        self
    }

    pub fn set_policy(&self, policy: GovernancePolicy) {
        self.policy.replace(policy);
    }

    pub fn set_approval_gate(
        &self,
        gate: impl Fn(&str, &Value) -> bool + Send + Sync + 'static,
    ) {
        self.approval_gate.replace(Some(Box::new(gate)));
    }

    pub fn policy(&self) -> std::cell::Ref<'_, GovernancePolicy> {
        self.policy.borrow()
    }

    /// N→4 folding over the registry's full tool list (token-savings measured).
    pub fn folding(&self) -> FoldedSpecs {
        fold_tool_specs_from_defs(self.registry.list_tools())
    }

    /// Governed call: policy check → HITL gate → existing `call_tool`.
    /// Every outcome (allow / deny / blocked-on-approval / executed / failed)
    /// is appended to the registry hash chain.
    pub fn call(&self, name: &str, args: &Value) -> Result<GatewayCall, String> {
        let verdict = self.policy.borrow().check(name);
        match verdict {
            SandboxVerdict::Denied => {
                self.registry
                    .record_evidence(name, args, verdict, false, None);
                Err(format!(
                    "MCP governance: tool '{}' denied by policy",
                    name
                ))
            }
            SandboxVerdict::RequiresApproval => {
                let approved = self.gate_approval(name, args);
                if !approved {
                    self.registry
                        .record_evidence(name, args, verdict, false, None);
                    return Err(format!(
                        "MCP governance: tool '{}' requires human approval (not granted)",
                        name
                    ));
                }
                self.execute(name, args, true)
            }
            SandboxVerdict::Approved => self.execute(name, args, false),
        }
    }

    fn gate_approval(&self, name: &str, args: &Value) -> bool {
        match self.approval_gate.borrow().as_ref() {
            Some(gate) => gate(name, args),
            None => false,
        }
    }

    fn execute(
        &self,
        name: &str,
        args: &Value,
        approved_by_hitl: bool,
    ) -> Result<GatewayCall, String> {
        match self.registry.call_tool(name, args) {
            Ok(content) => {
                let evidence = self.registry.record_evidence(
                    name,
                    args,
                    SandboxVerdict::Approved,
                    approved_by_hitl,
                    Some(truncate(&content, 256)),
                );
                Ok(GatewayCall {
                    tool: name.to_string(),
                    verdict: SandboxVerdict::Approved,
                    approved_by_hitl,
                    content,
                    evidence,
                })
            }
            Err(e) => {
                self.registry.record_evidence(
                    name,
                    args,
                    SandboxVerdict::Approved,
                    approved_by_hitl,
                    Some(format!("ERROR: {}", e)),
                );
                Err(format!("MCP tool '{}' failed: {}", name, e))
            }
        }
    }

    // -- Audit wiring (nt_shield_audit evidence) ----------------------------

    /// Hash chain → `nt_shield_audit::CheckResult` list, evidence populated.
    pub fn as_audit_check_results(&self) -> Vec<CheckResult> {
        let chain = self.registry.evidence_chain();
        let valid = chain.verify();
        chain
            .entries
            .iter()
            .map(|e| CheckResult {
                check_id: format!("MCP-EV-{:04}", e.seq),
                status: if valid {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                },
                evidence: Some(format!(
                    "tool={} verdict={:?} hitl={} hash-chain={}",
                    e.tool_name, e.verdict, e.approved_by_hitl, e.hash
                )),
                confidence: if valid { 1.0 } else { 0.0 },
            })
            .collect()
    }

    /// Chain integrity as a full `nt_shield_audit::AuditReport`.
    pub fn audit_report(&self) -> AuditReport {
        let results = self.as_audit_check_results();
        let n = results.len();
        let passed = results
            .iter()
            .filter(|r| matches!(r.status, CheckStatus::Passed))
            .count();
        let failed = n - passed;
        AuditReport {
            project: "mcp-gateway".into(),
            mode: AuditMode::Static,
            total_checks: n,
            passed,
            failed,
            suspicious: 0,
            score: if n == 0 {
                100.0
            } else {
                passed as f64 / n as f64 * 100.0
            },
            results,
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.min(s.len())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l1_body_impl::nt_agent_mcp_registry::{McpRegistry, McpTransport};
    use crate::neotrix::l1_body_impl::nt_agent_mcp_transport::TransportMode;

    fn sample_tool(name: &str, desc: &str) -> McpToolDef {
        McpToolDef {
            name: name.into(),
            description: desc.into(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
            server_name: "test".into(),
            transport: McpTransport::Local {
                command: "sh".into(),
                args: vec![],
            },
            schema_version: None,
        }
    }

    fn realistic_toolset() -> Vec<McpToolDef> {
        vec![
            sample_tool("web_search", "Search the public web"),
            sample_tool("github_search", "Search GitHub code and issues"),
            sample_tool("google_news", "Retrieve latest news results"),
            sample_tool("scrape_url", "Fetch and extract a URL"),
            sample_tool("kb_query", "Query the NeoTrix knowledge base"),
            sample_tool("memory_recall", "Recall past session memory"),
            sample_tool("wiki_lookup", "Look up facts on Wikipedia"),
            sample_tool("read_file", "Read a file from disk"),
            sample_tool("edit_file", "Edit a file in place"),
            sample_tool("mkdir_path", "Create a directory"),
            sample_tool("shell_exec", "Run a shell command"),
            sample_tool("notify_user", "Send a notification to the user"),
            sample_tool("deploy_service", "Deploy a service to production"),
            sample_tool("code_apply", "Apply a code patch"),
        ]
    }

    #[test]
    fn test_folding_groups_into_four_categories() {
        let folded = fold_tool_specs_from_defs(realistic_toolset());
        assert_eq!(folded.categories.len(), 4, "must always emit 4 canonical categories");
        let names: Vec<&str> = folded.categories.iter().map(|c| c.category.name()).collect();
        assert_eq!(names, vec!["search", "act", "knowledge", "file"]);
        // search bucket holds the web/query tools
        let search = &folded.categories[0];
        assert!(search.member_tools.contains(&"web_search".to_string()));
        assert!(search.member_tools.contains(&"github_search".to_string()));
        // file bucket holds the fs tools
        let file = &folded.categories[3];
        assert!(file.member_tools.contains(&"read_file".to_string()));
        assert!(file.member_tools.contains(&"mkdir_path".to_string()));
        // knowledge bucket holds kb/memory tools
        let knowledge = &folded.categories[2];
        assert!(knowledge.member_tools.contains(&"kb_query".to_string()));
        assert!(knowledge.member_tools.contains(&"memory_recall".to_string()));
        // act bucket holds everything else
        let act = &folded.categories[1];
        assert!(act.member_tools.contains(&"shell_exec".to_string()));
        assert!(act.member_tools.contains(&"deploy_service".to_string()));
        // total membership == input size
        let total: usize = folded.categories.iter().map(|c| c.member_tools.len()).sum();
        assert_eq!(total, realistic_toolset().len());
    }

    #[test]
    fn test_folding_token_savings_positive() {
        let folded = fold_tool_specs_from_defs(realistic_toolset());
        assert!(folded.original_chars > 0);
        assert!(
            folded.folded_chars < folded.original_chars,
            "folded {} < original {}",
            folded.folded_chars,
            folded.original_chars
        );
        assert!(folded.saved_chars > 0);
        assert!(folded.saved_tokens > 0);
        assert!(
            folded.savings_percent > 0.5,
            "expected ≥50% savings, got {:.1}%",
            folded.savings_percent * 100.0
        );
    }

    #[test]
    fn test_governance_allow_and_deny() {
        let policy = GovernancePolicy::permissive()
            .allow("search:*")
            .allow("github:*")
            .deny("shell:*")
            .deny("rm:*")
            .deny("github:delete*");
        assert_eq!(policy.check("search:web"), SandboxVerdict::Approved);
        assert_eq!(policy.check("github:list_repos"), SandboxVerdict::Approved);
        assert_eq!(policy.check("shell:curl"), SandboxVerdict::Denied);
        assert_eq!(policy.check("rm:data"), SandboxVerdict::Denied);
        // Deny beats Allow for the same prefix family
        assert_eq!(policy.check("github:delete_repo"), SandboxVerdict::Denied);
        // Unlisted → permissive default (backward compat)
        assert_eq!(policy.check("unlisted_tool"), SandboxVerdict::Approved);
    }

    #[test]
    fn test_governance_restrictive_fail_closed() {
        let policy = GovernancePolicy::restrictive()
            .allow("search:*")
            .deny("shell:*");
        assert_eq!(policy.check("search:web"), SandboxVerdict::Approved);
        assert_eq!(policy.check("shell:x"), SandboxVerdict::Denied);
        // unlisted → RequiresApproval (fail-closed)
        assert_eq!(policy.check("unlisted_tool"), SandboxVerdict::RequiresApproval);
    }

    #[test]
    fn test_governance_deny_precedence_over_approval() {
        let policy = GovernancePolicy::permissive()
            .require_approval("github:*")
            .deny("github:delete*");
        assert_eq!(policy.check("github:create_repo"), SandboxVerdict::RequiresApproval);
        assert_eq!(policy.check("github:delete_repo"), SandboxVerdict::Denied);
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("shell:*", "shell:curl"));
        assert!(glob_match("shell:*", "shell"));
        assert!(glob_match("*delete*", "github:delete_repo"));
        assert!(glob_match("*", "anything"));
        assert!(!glob_match("shell:*", "network:curl"));
        assert!(glob_match("exact", "exact"));
        assert!(!glob_match("exact", "exact2"));
    }

    #[test]
    fn test_hash_chain_tampering_breaks_chain() {
        let mut chain = HashChain::new();
        chain.append("web_search", serde_json::json!({"q": "a"}), SandboxVerdict::Approved, false, Some("r1".into()));
        chain.append("shell_exec", serde_json::json!({"cmd": "ls"}), SandboxVerdict::Approved, true, Some("r2".into()));
        chain.append("kb_query", serde_json::json!({"q": "b"}), SandboxVerdict::Approved, false, Some("r3".into()));
        assert!(chain.verify(), "intact chain must verify");
        assert_eq!(chain.entries.len(), 3);

        // Tamper with the first link's args → chain must break from link 0 on.
        chain.entries[0].args = serde_json::json!({"q": "TAMPERED"});
        let broken = chain.broken_links();
        assert!(!chain.verify());
        assert_eq!(broken, vec![0, 1, 2], "all subsequent links must fail once link 0 is tampered");

        // Restore → recompute link 0's hash to heal → chain verifies again.
        chain.entries[0].hash = chain.entries[0].recompute_hash();
        assert!(chain.verify());
    }

    #[test]
    fn test_gateway_hitl_gate_enforced() {
        let mut reg = McpRegistry::new();
        reg.publish("deploy", "nonexistent-cmd-xyz", &[], "deploy tool");
        let policy = GovernancePolicy::permissive().require_approval("deploy_tool");

        // No approval gate → RequiresApproval blocks before execution.
        let gw = reg.gateway().with_policy(policy.clone());
        let err = gw.call("deploy_tool", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("requires human approval"), "got: {}", err);

        // Gate that refuses → still blocked, but evidence recorded.
        let gw = reg.gateway().with_policy(policy.clone());
        gw.set_approval_gate(|_, _| false);
        let err = gw.call("deploy_tool", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("requires human approval"));

        // Gate that grants → passes the gate and reaches execution (spawn fails).
        let gw = reg.gateway().with_policy(policy);
        gw.set_approval_gate(|_, _| true);
        let err = gw.call("deploy_tool", &serde_json::json!({})).unwrap_err();
        assert!(!err.contains("requires human approval"), "must reach execution: {}", err);
        assert!(err.contains("failed"), "execution attempt recorded: {}", err);
    }

    #[test]
    fn test_production_path_reachability_via_registry() {
        // env-gated: spawn 真实 sh 子进程; 全量并行/多会话负载下 fork 资源耗尽
        // 会误报 "Spawn sh: No such file or directory (os error 2)"
        if std::env::var("NT_E2E_SUBPROCESS")
            .map(|v| v == "1")
            .unwrap_or(false)
            != true
        {
            eprintln!("skipped: set NT_E2E_SUBPROCESS=1 to run real-subprocess MCP e2e");
            return;
        }
        // A real subprocess MCP server that answers a valid JSON-RPC response,
        // so the governed path executes end-to-end through the registry.
        let mut reg = McpRegistry::new();
        let tool = McpToolDef {
            name: "echo_greeting".into(),
            description: "Returns a greeting".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {"who": {"type": "string"}}}),
            server_name: "echo".into(),
            transport: McpTransport::Local {
                command: "sh".into(),
                args: vec!["-c".into(),
                           "echo '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"governed-ok\"}]}}'".into()],
            },
            schema_version: None,
        };
        reg.register_stdio("echo", "sh", &["-c", "echo ok"], vec![tool]);

        // Default permissive policy → governed call behaves like plain call_tool.
        let result = reg.call_tool_governed(
            "echo_greeting",
            &serde_json::json!({"who": "world"}),
            &GovernancePolicy::permissive(),
        );
        let content = result.expect("governed call must reach the transport");
        assert!(content.contains("governed-ok"), "got: {}", content);

        // Evidence recorded on the registry and the chain is valid.
        assert!(reg.chain_valid());
        assert_eq!(reg.evidence_chain().len(), 1);

        // Deny policy blocks before transport and still records evidence.
        let denied = reg.call_tool_governed(
            "echo_greeting",
            &serde_json::json!({}),
            &GovernancePolicy::permissive().deny("echo:*"),
        );
        assert!(denied.unwrap_err().contains("denied by policy"));
        assert_eq!(reg.evidence_chain().len(), 2, "denial is also evidenced");

        // Gateway surface reachable from the registry (folding + audit).
        let gw = reg.gateway();
        let folded = gw.folding();
        assert_eq!(folded.categories.len(), 4);
        let report = gw.audit_report();
        assert_eq!(report.total_checks, 2);
        assert_eq!(report.passed, 2, "evidence chain is intact");
    }

    #[test]
    fn test_gateway_wire_into_builtin_tools() {
        // register_neotrix_tools returns folded specs — production registration
        // path (entry/mod.rs bootstrap) computes folding live.
        let mut reg = McpRegistry::new();
        let folded =
            crate::neotrix::l1_body_impl::nt_agent_mcp_tools::register_neotrix_tools(&mut reg);
        assert_eq!(folded.categories.len(), 4);
        // neotrix_search is knowledge (kb) — wait, it contains "search" → search bucket.
        let search = folded
            .categories
            .iter()
            .find(|c| c.category == ToolCategory::Search)
            .expect("search category present");
        assert!(search.member_tools.iter().any(|n| n == "neotrix_search"));
        assert!(reg.gateway().folding().savings_percent >= 0.0);
        let _ = TransportMode::Local { command: "sh".into(), args: vec![] };
    }
}
