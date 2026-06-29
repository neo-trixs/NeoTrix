use std::collections::HashMap;

use crate::agent::tool::lifecycle::*;

pub struct ReactDoctorTool {
    manifest: ToolManifest,
}

const REACT_PATTERNS: &[(&str, &str, &str, &str)] = &[
    // Hooks rules
    ("hooks-rules", "error",
     r"(useState|useEffect|useMemo|useCallback|useRef|useContext)\s*\([^)]*\)[^;]*\n(?!\s*const |\s*let |\s*var |\s*function |\s*if |\s*for |\s*while |\s*return |\s*\}|\s*//)",
     "Hooks must be called at the top level of a React function component"),

    // Missing useEffect dependencies
    ("missing-deps", "warning",
     r"useEffect\s*\(\s*\([^)]*\)\s*=>\s*\{[^}]*\}[^)]*\)",
     "useEffect without dependency array — will run on every render"),

    // setState in render
    ("setState-in-render", "error",
     r"(setState|dispatch|set[A-Z]\w+)\s*\([^)]*\)\s*(?!;*\s*$)",
     "Calling setState directly in render body causes infinite re-render"),

    // Inline arrow function in JSX props (re-render cause)
    ("inline-handler", "warning",
     r"on[A-Z]\w+\s*=\s*\{\s*\([^)]*\)\s*=>",
     "Inline arrow function in JSX prop creates new closure on every render"),

    // Missing key in lists
    ("missing-key", "warning",
     r"(\.map|\.filter)\s*\([^)]*\)\s*\.\s*map\s*\([^)]*\)|\.map\s*\(\s*\(?\w+\)?\s*=>\s*<[^>]+>(?![\s\S]*?key=)",
     "List items should have a unique 'key' prop for optimal reconciliation"),

    // Using index as key
    ("index-key", "warning",
     r"key\s*=\s*\{\s*index\s*\}",
     "Using array index as key can cause issues with dynamic lists — use a stable identifier"),

    // Large useEffect body
    ("large-effect", "info",
     r"useEffect\s*\(\s*\([^)]*\)\s*=>\s*\{[^}]{300,}\}",
     "useEffect body is very large — consider extracting logic into custom hooks"),

    // Direct DOM manipulation
    ("direct-dom", "warning",
     r"document\.(getElementById|querySelector|createElement)|\.innerHTML\s*=",
     "Direct DOM manipulation in React leads to reconciliation conflicts"),

    // Class component without constructor cleanup
    ("legacy-class", "info",
     r"class\s+\w+\s+extends\s+(React\.)?Component",
     "Class component detected — consider migrating to function component with hooks"),

    // Missing cleanup in useEffect
    ("missing-cleanup", "warning",
     r"useEffect\s*\(\s*\([^)]*\)\s*=>\s*\{[^}]*addEventListener|setInterval|setTimeout|\.subscribe\s*\([^}]*\}(?!\s*\(\)\s*=>|return)",
     "useEffect with subscriptions/timers should return a cleanup function to prevent memory leaks"),

    // useState object instead of individual state
    ("object-state", "info",
     r"const\s+\[\w+,\s*set\w+\]\s*=\s*useState\s*\(\s*\{",
     "Using object with useState can cause unnecessary re-renders — prefer individual state variables or useReducer"),
];

impl ReactDoctorTool {
    pub fn new() -> Self {
        Self {
            manifest: ToolManifest {
                id: "react_doctor".into(),
                name: "React Doctor".into(),
                version: "0.1.0".into(),
                permissions: vec![ToolPermission::FileRead],
                mcp: Some(McpServerDecl {
                    command: "neotrix".to_string(),
                    args: vec!["tool".to_string(), "--run".to_string(), "react_doctor".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("NEOTRIX_TOOL_MODE".to_string(), "mcp".to_string());
                        env
                    },
                }),
                min_runtime: "0.1.0".into(),
                description: "Analyze React component health — hooks rules, missing deps, anti-patterns, and performance issues".into(),
                author: Some("NeoTrix".into()),
            },
        }
    }
}

impl Default for ReactDoctorTool {
    fn default() -> Self {
        Self::new()
    }
}

fn analyze_react_code(code: &str) -> Vec<serde_json::Value> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    for &(id, severity, pattern, message) in REACT_PATTERNS {
        let re = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for (i, line) in lines.iter().enumerate() {
            if re.find(line).is_some() {
                findings.push(serde_json::json!({
                    "id": id,
                    "severity": severity,
                    "line": i + 1,
                    "message": message,
                    "snippet": line.trim(),
                }));
                break;
            }
        }
    }
    findings
}

impl AgentTool for ReactDoctorTool {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }

    fn start(&mut self, _api: ToolApi) -> Result<(), ToolError> {
        // Validate all patterns compile
        for &(id, _, pattern, _) in REACT_PATTERNS {
            if regex::Regex::new(pattern).is_err() {
                return Err(ToolError::Runtime {
                    id: self.id().into(),
                    message: format!("invalid pattern: {}", id),
                });
            }
        }
        Ok(())
    }

    fn execute(&self, ctx: ToolContext) -> Result<ToolOutput, ToolError> {
        let args: serde_json::Value =
            serde_json::from_str(&ctx.input).map_err(|e| ToolError::Runtime {
                id: self.id().into(),
                message: e.to_string(),
            })?;

        let code = args
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or(&ctx.input);

        if code.is_empty() || code.trim().is_empty() {
            let result = serde_json::json!({
                "tool": "react_doctor",
                "status": "no_input",
                "message": "Provide React component source code via 'code' field or raw input",
                "available_checks": REACT_PATTERNS.iter().map(|&(id, sev, _, msg)| {
                    serde_json::json!({ "id": id, "severity": sev, "description": msg })
                }).collect::<Vec<_>>(),
            });
            return Ok(ToolOutput {
                result: serde_json::to_string_pretty(&result).unwrap_or_default(),
                metadata: HashMap::new(),
            });
        }

        let findings = analyze_react_code(code);
        let by_severity = {
            let mut m = HashMap::new();
            for f in &findings {
                let s = f["severity"].as_str().unwrap_or("info");
                *m.entry(s.to_string()).or_insert(0usize) += 1;
            }
            m
        };

        let result = serde_json::json!({
            "tool": "react_doctor",
            "status": "analyzed",
            "total_findings": findings.len(),
            "by_severity": by_severity,
            "findings": findings,
            "component_count": code.matches("function ").count()
                + code.matches("const ").filter(|_| {
                    code.contains("=>")
                }).count(),
        });
        Ok(ToolOutput {
            result: serde_json::to_string_pretty(&result).unwrap_or_default(),
            metadata: HashMap::new(),
        })
    }

    fn stop(&mut self) -> Result<(), ToolError> {
        Ok(())
    }
}
