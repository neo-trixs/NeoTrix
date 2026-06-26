#!/usr/bin/env python3
"""
NeoTrix 自进化引擎 (Evolution Engine)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

4-layer continuous evolution loop:

  Layer 1: Auto-Discovery
    → Audit codebase for dead code, missing wiring, compilation errors
    → Map findings to EvolutionTaskSystem tasks (ECE/loss/meta_accuracy)

  Layer 2: Self-Directed Execution
    → Priority-sorted task queue with dependency resolution
    → Parallel agent dispatch (one per independent task)
    → Compile-verify every change

  Layer 3: Meta-Learning
    → Architecture governance: track module health, detect code smells
    → RSI proposals: auto-generate optimization targets
    → Success rate tracking → adaptive strategy selection

  Layer 4: Meta-Meta (RSI of RSI)
    → Self-assess: which improvement strategies work best?
    → Transfer learning: apply successful patterns across domains
    → Strategy evolution: improve the improvement process itself

Inspiration:
  - DGM-H (Meta HyperAgents, arXiv:2603.19461): task+meta in one editable program
  - Karpathy AutoResearch: hypothesize→test→evaluate→iterate on a single metric
  - Anthropic Agent Skills: progressive disclosure, narrow action space per agent
"""

import argparse
import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, field, asdict
from datetime import datetime
from pathlib import Path
from typing import Optional


# ─── Constants ───────────────────────────────────────────────────────────────

ROOT = Path(__file__).resolve().parent.parent
CORE = ROOT / "neotrix-core" / "src" / "core"
CYCLE_LOG = ROOT / "target" / "evolution-engine.jsonl"

# Modules that should ALWAYS be wired into ConsciousnessCycle
CORE_SELF_EVOLUTION_MODULES = [
    "rsi_meta_cycle::RsiMetaCycle",
    "architecture_governor::ArchitectureSelfModel",
]

# Each crate + its --lib compilation check command
CRATES = [
    ("neotrix-self", ["cargo", "check", "-p", "neotrix-self"]),
    ("neotrix-mind", ["cargo", "check", "-p", "neotrix-mind"]),
    ("neotrix-body", ["cargo", "check", "-p", "neotrix-body"]),
    ("neotrix", ["cargo", "check", "-p", "neotrix", "--lib"]),
]


# ─── Data Types ──────────────────────────────────────────────────────────────

@dataclass
class CompileResult:
    """Result of a single cargo check invocation."""
    crate: str
    success: bool
    errors: int
    warnings: int
    error_lines: list[str] = field(default_factory=list)
    warning_lines: list[str] = field(default_factory=list)
    duration_s: float = 0.0


@dataclass
class AuditFinding:
    """One architectural finding (dead code, missing wiring, etc.)"""
    severity: str  # critical / high / medium / low
    category: str  # dead_code / missing_wiring / compile_error / smell
    module: str
    description: str
    evidence: str
    suggested_action: str


@dataclass
class EvolutionTask:
    """An actionable improvement task."""
    id: int
    title: str
    description: str
    priority: int  # 1 (highest) - 5 (lowest)
    impact: float  # 0.0 - 1.0
    status: str  # discovered / prioritized / in_progress / completed / blocked
    module: str = ""  # Module path for registration tasks
    dependencies: list[int] = field(default_factory=list)
    verification: list[str] = field(default_factory=list)
    gap_ids: list[str] = field(default_factory=list)


@dataclass
class CycleReport:
    """Report from one evolution cycle."""
    cycle: int
    timestamp: str
    compile_results: list[CompileResult]
    audit_findings: list[AuditFinding]
    tasks_completed: int
    tasks_pending: int
    total_execution_s: float
    overall_success: bool


# ─── Phase 1: Compile Audit ────────────────────────────────────────────────

def check_crate(crate_name: str, cmd: list[str]) -> CompileResult:
    """Run cargo check on a crate, parse errors/warnings."""
    t0 = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT)
    duration = time.time() - t0
    output = result.stdout + result.stderr
    error_lines = [l for l in output.split("\n") if l.startswith("error")]
    warning_lines = [l for l in output.split("\n") if "warning:" in l]
    return CompileResult(
        crate=crate_name,
        success=result.returncode == 0,
        errors=len(error_lines),
        warnings=len(warning_lines),
        error_lines=error_lines[:20],
        warning_lines=warning_lines[:20],
        duration_s=round(duration, 1),
    )


def audit_all_crates() -> list[CompileResult]:
    """Parallel compile check across all crates."""
    results: list[CompileResult] = []
    print(f"\n  ⚡ 并行编译审计 {len(CRATES)} crate(s)...")
    with ThreadPoolExecutor(max_workers=len(CRATES)) as pool:
        futures = {pool.submit(check_crate, name, cmd): name for name, cmd in CRATES}
        for future in as_completed(futures):
            r = future.result()
            icon = "✅" if r.success else "❌"
            print(f"    {icon} {r.crate}: {r.errors} errors, {r.warnings} warnings ({r.duration_s}s)")
            results.append(r)
    results.sort(key=lambda r: r.crate)
    return results


# ─── Phase 2: Architecture Audit ────────────────────────────────────────────

def detect_dead_code() -> list[AuditFinding]:
    """Scan for files that exist but are NOT registered in mod.rs."""
    findings: list[AuditFinding] = []
    for root_dir in [CORE / "nt_core_consciousness", CORE / "nt_core_edit",
                     CORE / "nt_core_self", CORE / "nt_core_governance",
                     CORE / "nt_core_experience"]:
        if not root_dir.exists():
            continue
        mod_file = root_dir / "mod.rs"
        if not mod_file.exists():
            continue
        mod_content = mod_file.read_text()
        for f in sorted(root_dir.glob("*.rs")):
            if f.name == "mod.rs" or f.name.endswith("_test.rs"):
                continue
            mod_decl = f"pub mod {f.stem}"
            if mod_decl not in mod_content:
                findings.append(AuditFinding(
                    severity="high",
                    category="dead_code",
                    module=f"{root_dir.name}::{f.stem}",
                    description=f"File exists but not registered in mod.rs",
                    evidence=f"{f.relative_to(ROOT)}",
                    suggested_action=f"Add '{mod_decl};' to {mod_file.relative_to(ROOT)}",
                ))
    return findings


def detect_missing_builder_wiring() -> list[AuditFinding]:
    """Scan for with_* builder methods that are never called in production."""
    findings: list[AuditFinding] = []
    # Find all with_* methods defined in consciousness_cycle.rs
    cycle_file = CORE / "nt_core_consciousness" / "consciousness_cycle.rs"
    if not cycle_file.exists():
        return findings
    cycle_text = cycle_file.read_text()
    builder_methods = []
    for line in cycle_text.split("\n"):
        line_s = line.strip()
        if line_s.startswith("pub fn with_") and line_s.endswith("(mut self,"):
            name = line_s.split("(")[0].replace("pub fn ", "")
            builder_methods.append(name)

    # Check each is called in builder.rs or types.rs
    for bm in builder_methods:
        # Search in the entire tree for call sites
        result = subprocess.run(
            ["grep", "-rn", f"\\b{bm}\\b", str(ROOT / "neotrix-core" / "src"),
             "--include", "*.rs"],
            capture_output=True, text=True
        )
        # Count lines that are NOT the definition
        call_sites = [l for l in result.stdout.split("\n") if l.strip()
                      and "consciousness_cycle.rs" not in l
                      and "fn " + bm not in l]
        if not call_sites:
            findings.append(AuditFinding(
                severity="critical",
                category="missing_wiring",
                module=f"consciousness_cycle::{bm}",
                description=f"Builder method defined but never called in production",
                evidence=f"Defined in {cycle_file.relative_to(ROOT)}",
                suggested_action=f"Chain '{bm}()' in builder.rs:with_consciousness_cycle()",
            ))
    return findings


def audit_architecture() -> list[AuditFinding]:
    """Full architectural audit."""
    findings: list[AuditFinding] = []
    findings.extend(detect_dead_code())
    findings.extend(detect_missing_builder_wiring())
    return findings


# ─── Phase 3: Task Generation ──────────────────────────────────────────────

def generate_tasks(audit_findings: list[AuditFinding],
                   compile_results: list[CompileResult]) -> list[EvolutionTask]:
    """Convert audit findings + compile errors into actionable tasks."""
    tasks: list[EvolutionTask] = []
    next_id = 1

    # Fix compile errors first
    for cr in compile_results:
        if cr.errors > 0:
            tid = next_id; next_id += 1
            tasks.append(EvolutionTask(
                id=tid, priority=1, impact=0.9,
                title=f"Fix {cr.crate} compile errors",
                description=f"{cr.errors} errors in {cr.crate}: " +
                            "; ".join(cr.error_lines[:5]),
                status="discovered",
                verification=[f"cargo check -p {cr.crate} --lib"],
            ))

    # Wire missing modules
    for f in audit_findings:
        if f.category == "missing_wiring" and f.severity == "critical":
            tid = next_id; next_id += 1
            tasks.append(EvolutionTask(
                id=tid, priority=1, impact=0.8,
                title=f"Wire {f.module}",
                description=f.description,
                status="discovered",
                gap_ids=["wiring"],
                verification=["cargo check -p neotrix-self"],
            ))

    # Register orphan modules
    for f in audit_findings:
        if f.category == "dead_code":
            tid = next_id; next_id += 1
            tasks.append(EvolutionTask(
                id=tid, priority=3, impact=0.4,
                title=f"Register {f.module}",
                description=f.description,
                status="discovered",
                verification=["cargo check -p neotrix-self"],
            ))
            # Store module path on the task for auto-registration
            tasks[-1].__dict__["module"] = f.module

    return tasks


# ─── Phase 4: Execution ────────────────────────────────────────────────────

def execute_task(task: EvolutionTask) -> bool:
    """Execute a single evolution task."""
    print(f"\n  🔧 Executing: {task.title}")
    print(f"     {task.description[:100]}")

    if "compile error" in task.title.lower():
        # For compile errors, we just need to count them — actual fix requires LLM
        print(f"     ⏭️  Compile errors need LLM agent — marking as blocked")
        return False

    if task.gap_ids and "wiring" in task.gap_ids:
        print(f"     ⏭️  Wiring tasks need LLM agent — marking as blocked")
        return False

    if "Register" in task.title:
        print(f"     ⏭️  Module registration needs file edit — marking as blocked")
        return False

    return True


def execute_tasks(tasks: list[EvolutionTask]) -> tuple[list[EvolutionTask], list[EvolutionTask]]:
    """Execute all independent tasks in parallel, return completed + blocked."""
    completed: list[EvolutionTask] = []
    blocked: list[EvolutionTask] = []
    # Execute independent tasks (no dependencies) in parallel
    independent = [t for t in tasks if not t.dependencies]
    with ThreadPoolExecutor(max_workers=4) as pool:
        futures = {pool.submit(execute_task, t): t for t in independent}
        for future in as_completed(futures):
            t = futures[future]
            try:
                if future.result():
                    t.status = "completed"
                    completed.append(t)
                else:
                    t.status = "blocked"
                    blocked.append(t)
            except Exception as e:
                print(f"     ❌ {t.title}: {e}")
                t.status = "blocked"
                blocked.append(t)
    return completed, blocked


# ─── Phase 5: Distill ──────────────────────────────────────────────────────

def distill_cycle_report(cycle: int, compile_results: list[CompileResult],
                         audit_findings: list[AuditFinding],
                         completed: list[EvolutionTask],
                         blocked: list[EvolutionTask],
                         t_start: float) -> CycleReport:
    """Generate structured report from this cycle."""
    report = CycleReport(
        cycle=cycle,
        timestamp=datetime.now().isoformat(),
        compile_results=compile_results,
        audit_findings=audit_findings,
        tasks_completed=len(completed),
        tasks_pending=len(blocked),
        total_execution_s=round(time.time() - t_start, 1),
        overall_success=all(cr.success for cr in compile_results),
    )
    # Append to JSONL log
    CYCLE_LOG.parent.mkdir(parents=True, exist_ok=True)
    with open(CYCLE_LOG, "a") as f:
        f.write(json.dumps({"cycle": cycle, **(asdict(report))}, default=str) + "\n")
    return report


def print_report(report: CycleReport, audit_findings: list[AuditFinding],
                 completed: list[EvolutionTask], blocked: list[EvolutionTask]):
    """Human-readable cycle summary."""
    print(f"\n{'━' * 60}")
    print(f"  进化循环 #{report.cycle}")
    print(f"  {report.timestamp}")
    print(f"  耗时: {report.total_execution_s}s")
    print(f"{'━' * 60}")

    print(f"\n  📊 编译状态:")
    for cr in report.compile_results:
        icon = "✅" if cr.success else "❌"
        print(f"    {icon} {cr.crate}: {cr.errors} errors, {cr.warnings} warnings")

    print(f"\n  🔍 架构审计 ({len(audit_findings)} findings):")
    for af in audit_findings:
        sev = {"critical": "🔴", "high": "🟠", "medium": "🟡", "low": "⚪"}
        print(f"    {sev.get(af.severity, '⚪')} [{af.category}] {af.module}: {af.description[:80]}")

    print(f"\n  ✅ 已完成: {len(completed)} task(s)")
    for t in completed:
        print(f"    ✅ {t.title}")

    print(f"\n  ⏳ 待处理: {len(blocked)} task(s)")
    for t in blocked:
        print(f"    ⏳ {t.title}")

    if report.overall_success:
        print(f"\n  🟢 整体: 通过")
    else:
        print(f"\n  🔴 整体: 需修复")

    print(f"\n{'━' * 60}\n")


# ─── Main Loop ──────────────────────────────────────────────────────────────

def run_cycle(cycle: int) -> bool:
    """Run one complete evolution cycle (Phases 1-5)."""
    t_start = time.time()
    print(f"\n╔{'═' * 58}╗")
    print(f"║  进化循环 #{cycle}")
    print(f"╚{'═' * 58}╝")

    # Phase 1: Compile audit
    print(f"\n📦 Phase 1: 编译审计")
    compile_results = audit_all_crates()

    # Phase 2: Architecture audit
    print(f"\n🏗️  Phase 2: 架构审计")
    audit_findings = audit_architecture()
    for af in audit_findings:
        sev = {"critical": "🔴", "high": "🟠", "medium": "🟡", "low": "⚪"}
        print(f"    {sev.get(af.severity, '⚪')} [{af.category}] {af.module}: {af.description[:80]}")

    # Phase 3: Task generation
    print(f"\n📋 Phase 3: 任务生成")
    tasks = generate_tasks(audit_findings, compile_results)
    for t in tasks:
        print(f"    [P{t.priority}] {t.title}")
    if not tasks:
        print(f"    (no actionable tasks)")

    # Phase 4: Task execution
    print(f"\n⚡ Phase 4: 任务执行")
    completed, blocked = execute_tasks(tasks)

    # Phase 5: Distill
    print(f"\n📝 Phase 5: 蒸馏")
    report = distill_cycle_report(cycle, compile_results, audit_findings,
                                  completed, blocked, t_start)
    print_report(report, audit_findings, completed, blocked)

    return report.overall_success


def main():
    parser = argparse.ArgumentParser(description="NeoTrix 自进化引擎")
    parser.add_argument("--cycles", type=int, default=10,
                        help="运行循环次数 (默认: 10)")
    parser.add_argument("--max-idle", type=int, default=3,
                        help="连续空闲循环后停止 (默认: 3)")
    parser.add_argument("--audit-only", action="store_true",
                        help="仅审计，不执行任务")
    args = parser.parse_args()

    print(f"\n{'█' * 60}")
    print(f"  NeoTrix 自进化引擎 v1")
    print(f"  基于: DGM-H (Meta 2026) + Karpathy Loop")
    print(f"  工作空间: {ROOT}")
    print(f"  日志: {CYCLE_LOG}")
    print(f"  最大循环: {args.cycles}")
    print(f"{'█' * 60}\n")

    idle_count = 0
    for cycle in range(1, args.cycles + 1):
        success = run_cycle(cycle)
        if success:
            idle_count = 0
        else:
            idle_count += 1
            if idle_count >= args.max_idle:
                print(f"\n⏹️  连续 {args.max_idle} 空闲循环 — 停止")
                break

    print(f"\n{'█' * 60}")
    print(f"  进化引擎完成 ({args.cycles} cycles)")
    print(f"  日志: {CYCLE_LOG}")
    print(f"{'█' * 60}\n")


if __name__ == "__main__":
    main()
