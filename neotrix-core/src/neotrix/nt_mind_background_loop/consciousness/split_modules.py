#!/usr/bin/env python3
"""Split modules.rs (4307 lines, 176 fn) into 8 domain files + thin shim."""

import re, os

SRC = "/Users/neo/Downloads/neotrix/neotrix-core/src/neotrix/nt_mind_background_loop/consciousness/modules.rs"
DST_DIR = os.path.dirname(SRC)

# ── Read full source ──
with open(SRC) as f:
    text = f.read()
lines = text.split("\n")

# ── Parse function boundaries ──
fn_pattern = re.compile(r'^\s+(pub\s+)?fn\s+(\w+)')
fns = []  # (start_line_1idx, end_line_1idx, name)
for i, line in enumerate(lines):
    m = fn_pattern.match(line)
    if m:
        fns.append((i + 1, 0, m.group(2)))  # 1-indexed

for idx in range(len(fns) - 1):
    fns[idx] = (fns[idx][0], fns[idx + 1][0] - 1, fns[idx][2])
if fns:
    fns[-1] = (fns[-1][0], len(lines) - 1, fns[-1][2])

# ── Classify each function ──
# Domain rules: check name prefixes in order
def classify(name):
    if name == "handle_e8_geometry_tick" or name == "handle_e8_cortical_tick":
        return "e8"
    if name == "handle_ema_jepa_tick":
        return "jepa"
    if name == "handle_null_drift_tick" or name == "handle_adaptive_vsa_tick":
        return "storage"
    if name.startswith("handle_kb_") or name == "handle_kb_tick" or name == "handle_entity_resolver_tick" or name == "handle_hubness_detector_tick" or name == "handle_keyword_lexicon_tick" or name == "handle_fringe_mix_tick" or name == "handle_interaction_trace_tick":
        return "kb"
    if name.startswith("handle_meta_agent_") or name.startswith("handle_sub_agent_") or name.startswith("handle_lead_agent_") or name.startswith("handle_goal_manager_") or name.startswith("handle_permission_") or name.startswith("handle_verify_") or name.startswith("handle_dispatch_pipeline_") or name in ("handle_meta_agent_tick", "handle_skill_health_tick", "handle_ultra_review_tick", "handle_cdp_session_tick", "handle_quant_data_tick", "handle_factor_miner_tick", "handle_osint_tick", "handle_mcp_intel_tick", "handle_remote_host_tick", "handle_security_gate_tick", "handle_browser_mcp_tick", "safety_check_mutation"):
        return "agent"
    if name.startswith("handle_visual_") or name == "handle_motion_synthesizer_tick" or name == "handle_html_presentation_tick" or name == "handle_transcript_analysis_tick":
        return "vision"
    if name.startswith("handle_translate_") or name == "translate" or name == "handle_thdc_tick" or name == "handle_storage_engine_tick":
        return "a2a"
    return "core"

groups = {"core": [], "agent": [], "kb": [], "e8": [], "jepa": [], "storage": [], "vision": [], "a2a": []}
for start, end, name in fns:
    grp = classify(name)
    groups[grp].append((start, end, name))

# ── Which imports are needed for each group? ──
# We scan function bodies for imported symbol usage
import_lines_raw = []
in_impl = False
for i, line in enumerate(lines):
    if line.startswith("use "):
        import_lines_raw.append((i + 1, line))
    elif line.startswith("fn "):
        break

# Extract import names (last segment after :: or {})
imports_info = []  # (line_num, text, names_set)
for line_num, line in import_lines_raw:
    # Extract all names from the use statement
    # e.g., "use crate::core::nt_core_agent::browser_mcp::BrowserMCP;" -> {"BrowserMCP"}
    # e.g., "use crate::core::nt_core_experience::workflow_engine::{OutputMapping, StepResult};" -> {"OutputMapping", "StepResult"}
    # e.g., "use super::types::*;" -> {"*"}
    
    # Find what's imported
    all_imports = set()
    
    # Handle { A, B, C } syntax
    brace_match = re.search(r'\{([^}]+)\}', line)
    if brace_match:
        inner = brace_match.group(1)
        for name in inner.split(','):
            name = name.strip()
            if name:
                all_imports.add(name)
    else:
        # Single import - get the last segment
        parts = line.rstrip(';').split('::')
        if parts:
            last = parts[-1].strip()
            if last and last != '{':
                all_imports.add(last)
    
    imports_info.append((line_num, line, all_imports))

# Now for each group, determine which imports are needed
# by checking if any of the import names appear in function bodies
def symbols_used_in_group(group_fns):
    """Return set of import line indices needed for this group."""
    used_symbols = set()
    for start, end, name in group_fns:
        for i in range(start - 1, min(end, start + 10)):  # Check fn sig + first few lines
            if i < len(lines):
                # Only check for non-self patterns
                used_symbols.update(re.findall(r'\b([A-Z][a-zA-Z0-9_]+)\b', lines[i]))
    return used_symbols

group_imports = {}
for grp_name, grp_fns in groups.items():
    if not grp_fns:
        group_imports[grp_name] = []
        continue
    
    used = symbols_used_in_group(grp_fns)
    
    # Always include super::types::*
    needed = []
    # Add super::types::* (always needed)
    needed.append("use super::types::*;")
    
    # Check each import
    for _, line, names in imports_info:
        # skip super::types::* (already added)
        if line.startswith("use super::"):
            continue
        # Check if any of the imported names appear in used symbols
        if names & used or any(n == "*" for n in names):
            needed.append(line)
    
    group_imports[grp_name] = needed

# ── Output: get function text for each group ──
def get_fn_text(start, end):
    return "\n".join(lines[start - 1 : end])  # start/end are 1-indexed

# ── Write group files ──
file_names = {
    "core": "modules_core.rs",
    "agent": "modules_agent.rs",
    "kb": "modules_kb.rs",
    "e8": "modules_e8.rs",
    "jepa": "modules_jepa.rs",
    "storage": "modules_storage.rs",
    "vision": "modules_vision.rs",
    "a2a": "modules_a2a.rs",
}

for grp_name in groups:
    fname = file_names[grp_name]
    grp_fns = groups[grp_name]
    if not grp_fns:
        # Create empty placeholder with impl block
        with open(os.path.join(DST_DIR, fname), "w") as f:
            f.write(f"""#![allow(unused_imports)]
use super::ConsciousnessIntegration;

impl ConsciousnessIntegration {{
    // No handlers for this group yet
}}
""")
        print(f"{fname}: 0 handlers (placeholder)")
        continue
    
    # Build file content
    parts = []
    parts.append("#![allow(unused_imports)]")
    parts.append("use super::ConsciousnessIntegration;")
    for imp in group_imports.get(grp_name, []):
        parts.append(imp)
    parts.append("")
    parts.append(f"// {grp_name.upper()} handlers extracted from modules.rs")
    parts.append(f"// {len(grp_fns)} handlers")
    parts.append("")
    parts.append("impl ConsciousnessIntegration {")
    
    for idx, (start, end, name) in enumerate(grp_fns):
        fn_text = get_fn_text(start, end)
        # Add a blank line before each function (except the first)
        if idx > 0:
            parts.append("")
        parts.append(fn_text)
    
    parts.append("}")
    parts.append("")
    
    with open(os.path.join(DST_DIR, fname), "w") as f:
        f.write("\n".join(parts) + "\n")
    
    print(f"{fname}: {len(grp_fns)} handlers written")

# ── Rewrite modules.rs as a thin shim ──
# Keep imports, tm_to_str, free functions, plus a small impl block with
# handlers that were not moved (the ones we decided to keep in core were
# moved to modules_core.rs, so modules.rs has nothing left except shim)

# The shim just re-exports the impl blocks from sub-modules
# Actually, with sub-modules, we don't need modules.rs at all for the handlers
# But we need to keep:
# 1. The free function tm_to_str
# 2. Any module-level items

# We'll create a minimal modules.rs that includes the sub-modules' content
# via re-export. Since impl blocks from sub-modules extend the type,
# the original modules.rs isn't needed as a compilation root.

# Instead, we add mod declarations in mod.rs and modules.rs becomes empty.

# Let's write a minimal modules.rs
shim = """// modules.rs — decomposed into 8 domain sub-files
// See: modules_core.rs, modules_agent.rs, modules_kb.rs, modules_e8.rs,
//      modules_jepa.rs, modules_storage.rs, modules_vision.rs, modules_a2a.rs

#![allow(dead_code)]

use super::types::*;

fn tm_to_str(tm: ThinkingMode) -> String {
    match tm {
        ThinkingMode::Fast => "cognitive_load_tick:Fast".to_string(),
        ThinkingMode::Balanced => "cognitive_load_tick:Balanced".to_string(),
        ThinkingMode::Deep => "cognitive_load_tick:Deep".to_string(),
    }
}
"""

with open(SRC, "w") as f:
    f.write(shim)

print(f"\nmodules.rs rewritten as shim ({len(shim.split(chr(10)))} lines)")

# ── Update mod.rs to add the new modules ──
mod_rs_path = os.path.join(DST_DIR, "mod.rs")
with open(mod_rs_path) as f:
    mod_rs = f.read()

new_mods = """mod modules_core;
mod modules_agent;
mod modules_kb;
mod modules_e8;
mod modules_jepa;
mod modules_storage;
mod modules_vision;
mod modules_a2a;
"""

# Insert before the existing mod modules;
if "mod modules;" in mod_rs:
    mod_rs = mod_rs.replace("mod modules;", new_mods + "mod modules;")
elif "pub use types" in mod_rs:
    # Insert before the pub use line
    idx = mod_rs.find("pub use types")
    mod_rs = mod_rs[:idx] + new_mods + "\n" + mod_rs[idx:]

with open(mod_rs_path, "w") as f:
    f.write(mod_rs)

print(f"\nmod.rs updated — added 8 module declarations")
print(f"\n=== Split complete ===")

# Summary
total = sum(len(v) for v in groups.values())
for grp, fns in sorted(groups.items()):
    print(f"  {file_names[grp]}: {len(fns)} handlers")
print(f"  Total: {total} handlers across 8 files + modules.rs shim")
