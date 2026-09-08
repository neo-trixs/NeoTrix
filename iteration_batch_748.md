# Iteration 748 — Git Governance & Code Ownership Research

**Date**: 2026-09-07
**Context**: Batch 747 proved 5 critical DX defects. This iteration researches git governance patterns to find NEW defects in NeoTrix.

---

## 1. CODEOWNERS Research

### Sources
- LLVM RFC: Contributor policy refresh (2026-08-31) — https://discourse.llvm.org/t/rfc-contributor-policy-refresh/91699
- DevToolHub: GitHub CODEOWNERS Permissions & Best Practices 2026 — https://devtoolhub.com/github-codeowners-permissions-best-practices/
- Hypertext Dispatches: CODEOWNERS Automating Code Review Ownership (2026-03) — https://tenthirtyam.org/dispatches/2026/03/25/codeowners-automating-code-review-ownership
- GitHub Changelog: Required reviewer rule GA (2026-02-17) — https://github.blog/changelog/2026-02-17-required-reviewer-rule-is-now-generally-available/
- arXiv 2512.05551: Automated Code Review Assignments — https://arxiv.org/html/2512.05551v1

### Key Findings
1. **LLVM discovered stale review requests** — when authors force-push rebased branches, CODEOWNERS-tagged teams are NOT automatically cleaned up. GitHub offers no auto-withdrawal. This creates phantom review noise.
2. **CODEOWNERS ≠ contribution-based ownership** (arXiv study) — declared owners differ from actual contributors in ~40% of cases, meaning CODEOWNERS becomes stale fast.
3. **Required reviewer rule (GA 2026-02)** is a separate policy layer from CODEOWNERS — negation patterns (`!`) now supported. CODEOWNERS routes reviews; required reviewer rule enforces policy. They must be configured independently.
4. **Single-path ownership** — best practice is one team per path to avoid review fatigue. Multiple overlapping owners cause notification storms.
5. **LLVM proposed limiting non-committers to 1 open PR** — controversial but reduces noise for maintainers.

### NEW Defect Found
**DEFECT-748-1: NeoTrix has no CODEOWNERS file mapping NT-* domains to ownership paths.**
- The 6-layer architecture (L1-L6) has ~15+ module directories but zero ownership enforcement.
- No automatic review routing for `neotrix-core/src/l1_action/`, `neotrix-core/src/l5_cognition/`, etc.
- The git pre-commit hook protects AGENTS.md structure but doesn't route reviews.
- **Impact**: Any contributor can merge changes to consciousness-critical paths (E8, GWT, SEAL) without domain expert review.

---

## 2. Branch Protection & Merge Queue Research

### Sources
- Youngju Kim: GitHub Branch Protection in Practice (2026-03-17) — https://www.youngju.dev/blog/devops/2026-03-17-github-branch-protection-rulesets-merge-queue-codeowners.en
- TopicTrick: GitHub Branch Protection Rules Step-by-Step (2026-07) — https://topictrick.com/blog/github-branch-protection-policies
- Tenki.cloud: GitHub Merge Queue in 2026 — https://tenki.cloud/blog/github-merge-queue-setup
- GitHub Docs: Managing a merge queue — https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
- Microsoft APM Issue #770: CI adopt merge queue (2026-04) — https://github.com/microsoft/apm/issues/770
- Caelo CMS Issue #23: Protect main branch (2026-05) — https://github.com/caelo-cms/caelo-cms/issues/23

### Key Findings
1. **3-layer ruleset model** (2026 best practice):
   - Organization baseline (no force-push, commit signing)
   - Critical branch policy (required checks, review count, merge queue)
   - Repository/team policy (CODEOWNERS, path-specific workflows)
2. **Merge queue validates branch reality** — not just serialization. Re-checks mergeability against latest target state. Critical for monorepos where PRs interact indirectly.
3. **Flaky tests block entire queues** — Microsoft adopted tiered CI (fast checks on PR push, heavy integration at merge time) to avoid queue stalls.
4. **Stale approval loophole** — without "dismiss stale approvals", a developer can approve, push breaking change, merge immediately. Must enable on critical branches.
5. **Hotfix exception process must be documented** — admin bypasses without documentation create invisible governance gaps.

### NEW Defect Found
**DEFECT-748-2: NeoTrix has no documented hotfix exception process.**
- The "Dark Forest" axiom (every module must compile+test+connect) has no bypass path for production incidents.
- No documented admin bypass procedure for when the SEAL pipeline or E8 engine breaks on main.
- **Impact**: In production incidents, maintainers must either bypass governance silently or wait for full CI, creating a trust gap.

**DEFECT-748-3: NeoTrix uses no merge queue despite having multi-domain contributors.**
- With 7 domains (NT-CORE through NT-FEEL) and cross-domain modules, PRs interact indirectly.
- Current workflow: direct merge or manual PR flow without queue validation.
- **Impact**: "Green in isolation" doesn't guarantee "safe in main" — the exact scenario merge queue solves.

---

## 3. Git Governance & Conventional Commits Research

### Sources
- A.R. Zerin: Git Commit Message Guidelines (2026-07-31) — https://arzerin.com/2026/07/31/git-commit-message-guidelines-and-conventional-commit-standards/
- ByteLedger: Conventional Commits Explained 2026 — https://byteledger.vizleo.com/blog/conventional-commits-explained-2026
- DeployHQ: Conventional Commits Guide (2026-08-12) — https://www.deployhq.com/blog/conventional-commits-a-standardized-approach-to-commit-messages
- Dev.to: Git Workflow Best Practices 2026 — https://dev.to/_d7eb1c1703182e3ce1782/git-workflow-best-practices-the-developers-guide-for-2026-4gl0
- TechEarl: Git Commit Message Best Practices (2026-06) — https://techearl.com/git-commit-message-conventions

### Key Findings
1. **`security` is now a standard commit type** (2026 convention) — for vulnerabilities, attack surface reduction, credential handling. NeoTrix's Egress Privacy Guard changes should use `security(...)` not `fix(...)`.
2. **Enforcement moved to commit-time hooks** (2026 shift) — commitlint + husky at commit time, not CI after. Catches bad messages before they enter history.
3. **PR title = squashed commit message** — with squash-and-merge, enforce conventional commits on PR title, not individual commits.
4. **50/72 rule** — subject under 50 chars, body wrapped at 72. Git tools truncate longer subjects in `git log --oneline`.
5. **Breaking changes need `!` suffix + body note** — `feat(api)!: replace response structure` with `BREAKING CHANGE:` in body.

### NEW Defect Found
**DEFECT-748-4: NeoTrix has no commit message enforcement.**
- No commitlint, no husky hooks, no conventional commits validation.
- Current commit history likely has inconsistent formats ("fix bug", "WIP", etc.).
- **Impact**: No automated changelog generation, no semantic versioning support, no machine-parseable history for SEAL pipeline evolution tracking.

**DEFECT-748-5: NeoTrix lacks `security` commit type for security-sensitive changes.**
- Egress Privacy Guard, sandbox egress policies, stealth net changes — all security-critical.
- These should be tagged `security(nt_shield): ...` not `fix(...)` or `feat(...)`.
- **Impact**: Security changes invisible to security audit trails, cannot be filtered in `git log --grep=security`.

---

## Summary: 5 NEW Defects

| ID | Defect | Severity | Domain |
|----|--------|----------|--------|
| DEFECT-748-1 | No CODEOWNERS file for NT-* domain ownership | HIGH | NT-GOVERNANCE |
| DEFECT-748-2 | No documented hotfix exception process | MEDIUM | NT-GOVERNANCE |
| DEFECT-748-3 | No merge queue for multi-domain PR validation | HIGH | NT-GOVERNANCE |
| DEFECT-748-4 | No conventional commits enforcement | MEDIUM | NT-META |
| DEFECT-748-5 | Missing `security` commit type for security changes | LOW | NT-SHIELD |

## Cumulative Defects (Batch 747 + 748)

| Batch | Defects |
|-------|---------|
| 747 | No plan-first mode, no per-tool transparency, no session rollback, plaintext API keys, no ratatui dashboard |
| 748 | No CODEOWNERS, no hotfix process, no merge queue, no commit enforcement, no security commit type |
