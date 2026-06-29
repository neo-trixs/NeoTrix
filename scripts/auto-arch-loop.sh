#!/usr/bin/env bash
set -euo pipefail

# Auto-Architecture-Loop: Bottleneck Analysis + Gap-Filling
# Each iteration: audit → analyze → fix → verify → distill
# Usage: ./scripts/auto-arch-loop.sh [iteration_count=10]

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ITERATIONS="${1:-10}"
LOGDIR="$ROOT/.arch-loop"
mkdir -p "$LOGDIR"

echo "=== NeoTrix Auto-Architecture Loop ==="
echo "Root: $ROOT"
echo "Iterations: $ITERATIONS"
echo "Logdir: $LOGDIR"
echo ""

# ─── Phase 1: Module Declaration Audit ────────────────────────────────
audit_module_declarations() {
    echo "[AUDIT] Module declaration completeness..."
    local issues=0
    local report="$LOGDIR/audit-modules.txt"
    > "$report"

    local CORE_MOD="$ROOT/neotrix-core/src/core/mod.rs"
    local NEOTRIX_MOD="$ROOT/neotrix-core/src/neotrix/mod.rs"
    local AGENT_MOD="$ROOT/neotrix-core/src/agent/mod.rs"

    # Cross-reference declared vs existing dirs
    for modfile in "$CORE_MOD" "$NEOTRIX_MOD" "$AGENT_MOD"; do
        local dirname
        dirname=$(dirname "$modfile")
        echo "=== $modfile ===" >> "$report"
        while IFS= read -r line; do
            if [[ $line =~ ^(pub(\(crate\))?\ )?mod\ ([a-zA-Z_][a-zA-Z0-9_]*)\; ]]; then
                local modname="${BASH_REMATCH[3]}"
                if [[ ! -d "$dirname/$modname" && ! -f "$dirname/${modname}.rs" ]]; then
                    echo "  MISSING: $modname (declared in $modfile, no file/dir)" >> "$report"
                    ((issues++))
                fi
            fi
        done < "$modfile"
    done

    # Cross-reference files on disk vs declared
    find "$ROOT/neotrix-core/src" -name "*.rs" | while IFS= read -r f; do
        local relpath="${f#$ROOT/neotrix-core/src/}"
        local dirpart
        dirpart=$(dirname "$relpath")
        local filepart
        filepart=$(basename "$f" .rs)

        # mod.rs is always valid
        [[ "$filepart" == "mod" ]] && continue

        # Skip files in agent/tool/ — planned plugin system, not yet registered
        [[ "$relpath" == agent/tool/* ]] && continue

        # Check if this path is reachable via some mod.rs chain
        local parent_mod="$ROOT/neotrix-core/src/$dirpart/mod.rs"
        if [[ -f "$parent_mod" ]]; then
            if ! grep -Eq "^((pub )?mod |#\\[cfg)" "$parent_mod" 2>/dev/null; then
                # If parent mod.rs has NO module declarations at all, skip (likely a lib entry point)
                continue
            fi
            if ! grep -Eq "^(pub(\(crate\))? )?mod $filepart" "$parent_mod" 2>/dev/null; then
                # Double-check: maybe declared in a nested mod.rs that recursively includes it
                # Skip common known patterns
                [[ "$relpath" == cli/* ]] && continue
                [[ "$relpath" == entry/* ]] && continue
                [[ "$relpath" == server/* ]] && continue
                [[ "$relpath" == neotrix/lib.rs ]] && continue
                [[ "$relpath" == neotrix/nt_shield_stealth_net/circuit_isolation.rs ]] && continue
                echo "  ORPHAN_FILE: $relpath (exists on disk but not declared in $dirpart/mod.rs)" >> "$report"
                ((issues++))
            fi
        fi
    done

    echo "$issues" > "$LOGDIR/.module_issues"
    echo "[AUDIT] Module issues: $issues"
    cat "$report"
    return "$issues"
}

# ─── Phase 2: Dependency / Dead Code Audit ────────────────────────────
audit_dependency() {
    echo "[AUDIT] Dependency analysis..."
    local issues=0
    local report="$LOGDIR/audit-deps.txt"
    > "$report"

    # Check for self-referencing modules (importing core from neotrix and vice-versa at wrong levels)
    # Check for modules declared in mod.rs but never imported elsewhere
    local CORE_MOD="$ROOT/neotrix-core/src/core/mod.rs"

    # grep all modules declared in core/mod.rs
    while IFS= read -r line; do
        if [[ $line =~ ^pub\ mod\ ([a-zA-Z_][a-zA-Z0-9_]*)\; ]]; then
            local modname="${BASH_REMATCH[1]}"
            local modpath="crate::core::$modname"
            # Count references outside of core/mod.rs
            local refcount
            refcount=$(grep -r "$modpath" "$ROOT/neotrix-core/src" --include="*.rs" | grep -v "core/mod.rs" | wc -l | tr -d ' ')
            if [[ "$refcount" -eq 0 ]]; then
                echo "  ORPHAN_MODULE: $modname ($modpath) — declared but zero external references" >> "$report"
                ((issues++))
            fi
        fi
    done < "$CORE_MOD"

    # Also check neotrix/mod.rs
    local NEOTRIX_MOD="$ROOT/neotrix-core/src/neotrix/mod.rs"
    while IFS= read -r line; do
        if [[ $line =~ ^pub\ mod\ ([a-zA-Z_][a-zA-Z0-9_]*)\; ]]; then
            local modname="${BASH_REMATCH[1]}"
            local modpath="crate::neotrix::$modname"
            local refcount
            refcount=$(grep -r "$modpath" "$ROOT/neotrix-core/src" --include="*.rs" | grep -v "neotrix/mod.rs" | wc -l | tr -d ' ')
            if [[ "$refcount" -eq 0 ]]; then
                echo "  ORPHAN_MODULE: $modname ($modpath) — declared but zero external references" >> "$report"
                ((issues++))
            fi
        fi
    done < "$NEOTRIX_MOD"

    echo "$issues" > "$LOGDIR/.dep_issues"
    echo "[AUDIT] Dependency issues: $issues"
    cat "$report"
    return "$issues"
}

# ─── Phase 3: Compilation Health Check ────────────────────────────────
audit_compilation() {
    echo "[AUDIT] Compilation health..."
    local issues=0
    local report="$LOGDIR/audit-compile.txt"
    > "$report"

    # Count errors and warnings for each crate
    for crate in neotrix nt-lang; do
        if cargo check -p "$crate" --lib 2>&1 | tee "$LOGDIR/check-$crate.log" | grep -q "error\["; then
            local err_count
            err_count=$(grep -c "error\[" "$LOGDIR/check-$crate.log" || true)
            echo "  ERRORS: $crate has $err_count errors" >> "$report"
            ((issues+=err_count))
        fi
        local warn_count
        warn_count=$(grep -c "warning:" "$LOGDIR/check-$crate.log" || true)
        if [[ "$warn_count" -gt 0 ]]; then
            echo "  WARNINGS: $crate has $warn_count warnings" >> "$report"
        fi
    done

    echo "$issues" > "$LOGDIR/.compile_issues"
    echo "[AUDIT] Compilation issues: $issues"
    cat "$report"
    return "$issues"
}

# ─── Phase 4: Runtime Wiring Gap Analysis ────────────────────────────
audit_runtime_wiring() {
    echo "[AUDIT] Runtime wiring analysis..."
    local issues=0
    local report="$LOGDIR/audit-wiring.txt"
    > "$report"

    # Check BackgroundLoop ticker-handler completeness
    local RUN_RS="$ROOT/neotrix-core/src/neotrix/nt_mind_background_loop/run.rs"
    if [[ -f "$RUN_RS" ]]; then
        # Find all ticker → handler dispatches
        while IFS= read -r dispatch; do
            # Extract handler name
            if [[ $dispatch =~ \.tick\(\)\ =\>\ self\.(.*)\(\)\.await ]]; then
                local handler="${BASH_REMATCH[1]}"
                if ! grep -q "async fn $handler" "$RUN_RS"; then
                    echo "  MISSING_HANDLER: ticker dispatches to $handler but no impl found" >> "$report"
                    ((issues++))
                fi
            fi
        done < <(grep "self\." "$RUN_RS" | grep "\.tick()" || true)
    fi

    # Check for duplicate pipeline (core.rs vs run.rs)
    local CORE_RS="$ROOT/neotrix-core/src/neotrix/nt_mind_background_loop/consciousness/core.rs"
    if [[ -f "$CORE_RS" ]]; then
        local run_batch_count
        run_batch_count=$(grep -c "handle_consciousness_batch" "$RUN_RS" || true)
        local core_batch_count
        core_batch_count=$(grep -c "fn handle_consciousness_batch" "$CORE_RS" || true)
        if [[ "$core_batch_count" -gt 0 && "$run_batch_count" -gt 0 ]]; then
            echo "  DUPLICATE: handle_consciousness_batch in both run.rs and consciousness/core.rs" >> "$report"
            ((issues++))
        fi
    fi

    echo "$issues" > "$LOGDIR/.wiring_issues"
    echo "[AUDIT] Wiring issues: $issues"
    cat "$report"
    return "$issues"
}

# ─── Main Loop ────────────────────────────────────────────────────────
for ((iter=1; iter<=ITERATIONS; iter++)); do
    echo ""
    echo "╔══════════════════════════════════════════════════╗"
    echo "║        AUTO-ARCH LOOP — Iteration $iter/$ITERATIONS         ║"
    echo "╚══════════════════════════════════════════════════╝"

    # Phase 1: Audit
    echo "─── Phase 1: Architecture Audit ───"
    audit_module_declarations || true
    audit_dependency || true
    audit_compilation || true
    audit_runtime_wiring || true

    # Aggregate issues
    total=$(( $(cat "$LOGDIR/.module_issues" 2>/dev/null || echo 0) +
              $(cat "$LOGDIR/.dep_issues" 2>/dev/null || echo 0) +
              $(cat "$LOGDIR/.compile_issues" 2>/dev/null || echo 0) +
              $(cat "$LOGDIR/.wiring_issues" 2>/dev/null || echo 0) ))

    echo ""
    echo "─── Total issues found: $total ───"

    if [[ "$total" -eq 0 ]]; then
        echo "✓ No issues — iteration $iter clean."
        # Even on clean, do a final verify
        cargo check -p neotrix --lib 2>&1 | tail -3
        continue
    fi

    # Phase 2: Generate todo from audit findings
    echo "─── Phase 2: Todo Generation ───"
    TODO_FILE="$LOGDIR/todo-iter$iter.txt"
    > "$TODO_FILE"
    for audit_file in "$LOGDIR"/audit-*.txt; do
        if [[ -f "$audit_file" && -s "$audit_file" ]]; then
            cat "$audit_file" >> "$TODO_FILE"
        fi
    done
    echo "Todo: $(wc -l < "$TODO_FILE") items"

    # Phase 3: Verification checkpoint
    echo "─── Phase 3: Compilation Verify ───"
    cargo check -p neotrix --lib 2>&1 | tail -3

    echo "─── Iteration $iter complete ───"
done

echo ""
echo "=== Auto-Arch-Loop Complete: $ITERATIONS iterations ==="
echo "Reports: $LOGDIR/"
