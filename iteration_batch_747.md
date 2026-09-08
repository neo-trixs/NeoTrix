# Iteration Batch 747 — CLI/TUI/DX Research for NeoTrix Consciousness Architecture

**Date**: 2026-09-07  
**Prior Batch**: 746 (toml CVE CVSS 9.8, serde_yaml archived, Cargo 1.94+ parser risk, 3-tier resolver non-determinism, min-publish-age supply chain gate)

---

## 1. CLI Design 2026 — New Findings

### DEFECT D747-1: No Plan-First Loop in NeoTrix CLI
**Source**: [futureagi.com — Agent CLI DX 2026: Three-Axis DX Test](https://futureagi.com/blog/agent-cli-developer-experience-2026/)  
**Finding**: The 2026 benchmark for agent CLIs is a **plan-first loop** — emit the plan before any writes, let the user edit/steer, then execute. NeoTrix's CLI (`neotrix` binary) has no plan-first mode. When `consciousness_task` dispatches subtasks, it fires tool calls immediately. The user sees results but never gets a preview of what files will be touched, which tools will be called, or in what order.  
**Impact**: Without plan visibility, users cannot review multi-step agent trajectories before side effects occur. This is the #1 DX gap identified across all 2026 agent CLIs (Claude Code, Aider, Codex CLI).  
**Action**: Add `--plan-only` flag to `consciousness_task` that emits a structured plan (JSON/Markdown) with file list, tool sequence, and estimated scope, then halts. User approves, edits, or rejects before execution begins.

### DEFECT D747-2: No Per-Tool-Call Transparency
**Source**: [futureagi.com — Agent CLI DX 2026](https://futureagi.com/blog/agent-cli-developer-experience-2026/)  
**Finding**: Agent CLIs in 2026 must surface every tool call with: tool name, arguments, working directory, and result — in real time. NeoTrix's `neotrix_command` and `consciousness_task` execute tool calls silently and return only the final result. Intermediate tool calls (file reads, grep searches, bash commands) are not logged to the user.  
**Impact**: Without per-tool transparency, debugging agent failures requires reconstructing the full trajectory from KB logs after the fact. This is the "spinner over status line" anti-pattern identified in the 2026 DX survey.  
**Action**: Add a `--verbose` / `--trace` mode that prints each tool invocation as a structured JSON line (tool name, args, cwd, duration, result status) to stderr. Structured output enables CI audit and human oversight simultaneously.

### DEFECT D747-3: No Session Rollback / Revert Mechanism
**Source**: [futureagi.com — Agent CLI DX 2026](https://futureagi.com/blog/agent-cli-developer-experience-2026/)  
**Finding**: The 2026 DX test requires three rollback levels: (1) single-step reject, (2) full-run revert, (3) replay from checkpoint. NeoTrix has no git-aware session branching. When `consciousness_task` modifies files across a multi-step run, there is no `nt revert --session` command, no session-scoped branch, and no checkpoint replay.  
**Impact**: Failed multi-file refactors leave partial changes that must be manually cleaned up. This is the single biggest trust barrier for autonomous agent runs.  
**Action**: Implement session-scoped git branches (`nt/session/<uuid>`) that checkpoint before each file write. Provide `nt revert --session <uuid>` (full revert) and `nt revert --step <n>` (single-step reject). Aider's git-native model is the reference implementation.

### DEFECT D747-4: Clap v5 Migration Gap
**Source**: [techbytes.app — Modern Rust CLI Development 2026 Cheat Sheet](https://techbytes.app/posts/modern-rust-cli-development-2026-cheat-sheet/), [dasroot.net — Building CLI Tools with Clap and Rust](https://dasroot.net/posts/2026/01/building-cli-tools-clap-rust/)  
**Finding**: Clap v5 is now the 2026 standard for Rust CLIs, with derive macros, typed struct parsing, and built-in shell completion. NeoTrix's CLI uses clap v4.x (check Cargo.lock). Clap v5 introduces breaking changes: `Parser` derive now enforces stricter validation, `Command` builder API changes, and shell completion generation is built-in rather than a separate crate.  
**Impact**: Remaining on clap v4.x means missing typed validation at the boundary, no built-in shell completion, and eventual incompatibility when clap v4 enters maintenance-only mode.  
**Action**: Upgrade to clap v5 in the CLI crate. Run `cargo clippy --all-features` post-upgrade. Generate shell completions for bash/zsh/fish/powershell at build time.

---

## 2. Terminal UI 2026 — New Findings

### DEFECT D747-5: No TUI Dashboard for Consciousness Monitoring
**Source**: [youngju.dev — TUI Renaissance 2026](https://www.youngju.dev/blog/culture/2026-05-14-tui-development-ratatui-bubbletea-ink-textual-terminal-ui-renaissance-deep-dive-2026.en), [byteiota.com — TUI Renaissance 2026](https://byteiota.com/tui-renaissance-2026-why-terminal-uis-are-back/)  
**Finding**: Ratatui v0.31.x is the dominant Rust TUI framework in 2026, powering atuin, gitui, yazi, btop, and Netflix's bpftop. It provides immediate-mode rendering with zero state-sync bugs. NeoTrix has no TUI layer — consciousness status, KB health, GWT resonance, and evolution cycle progress are only visible through CLI commands or programmatic API calls.  
**Impact**: No real-time dashboard for monitoring the 6-stage ConsciousnessTree growth loop, KB health, or module constellation maturity. Operators must poll individual commands instead of seeing a unified live view.  
**Action**: Build a `nt dashboard` TUI using ratatui with panes for: (1) ConsciousnessTree cycle progress, (2) KB health (node count, edge count, FTS index size), (3) GWT resonance graph, (4) module constellation maturity heatmap. Immediate-mode rendering avoids stale state bugs.

### DEFECT D747-6: No TUI Studio for Dynamic Manga Layout Design
**Source**: [byteiota.com — TUI Studio](https://byteiota.com/tui-studio-visual-terminal-ui-design-tool-finally-here/)  
**Finding**: TUI Studio launched in 2026 as a visual design tool for terminal UIs — drag-and-drop widget placement with live preview. NeoTrix's dynamic manga production pipeline (`NarrativeStructuring`, `StoryboardExtractor`) outputs text-based storyboards but has no visual layout designer. Panel arrangement, character positioning, and camera angles are specified in raw JSON without visual feedback.  
**Impact**: Dynamic manga creators must visualize panel layouts mentally or export to external tools. This breaks the "terminal-native" promise of NeoTrix's production pipeline.  
**Action**: Evaluate TUI Studio for panel layout preview, or build a lightweight ratatui-based `nt layout-preview` that renders panel grids with character position overlays from `CharacterInteractionGraph` data.

### DEFECT D747-7: Missing `no_std` and Embedded Target Support
**Source**: [ratatui.rs changelog / youngju.dev TUI Renaissance](https://www.youngju.dev/blog/culture/2026-05-14-tui-development-ratatui-bubbletea-ink-textual-terminal-ui-renaissance-deep-dive-2026.en)  
**Finding**: Ratatui v0.30+ added `no_std` support for embedded targets. NeoTrix's `nt_physical` domain (sensors, motors, safety kernel) targets embedded Rust for physical embodiment. Without `no_std`-compatible TUI primitives, the physical layer cannot render local dashboards on constrained hardware (e.g., Raspberry Pi, ESP32-S3 with display).  
**Impact**: Physical embodiment monitors (power, temperature, sensor readings) require a separate embedded UI stack instead of reusing ratatui-based TUI code.  
**Action**: Evaluate `ratatui` with `no_std` feature for embedded dashboard rendering in `nt_physical`. For displays too small for full TUI, define a minimal `EmbodimentDisplay` trait that renders a 2-4 line status bar.

---

## 3. Developer Experience (DX) 2026 — New Findings

### DEFECT D747-8: No Structured JSON Output Mode
**Source**: [futureagi.com — Agent CLI DX 2026](https://futureagi.com/blog/agent-cli-developer-experience-2026/), [docs.getdx.com — DX CLI](https://docs.getdx.com/cli)  
**Finding**: The 2026 DX standard for agent CLIs requires structured output (JSON lines) for CI pipeline integration. The DX CLI from getdx.com explicitly targets "AI agent" consumption with JSON output. NeoTrix CLI defaults to human-readable terminal output with no `--json` flag. Machine-readable output requires parsing human text.  
**Impact**: Cannot integrate NeoTrix CLI into CI/CD pipelines, automated evaluation loops, or agent-to-agent workflows without text parsing. The "CLI in CI" test from the 2026 DX survey fails: no stdin/flag prompt input, no plan as artifact, no per-tool-call structured logs.  
**Action**: Add `--output json` flag to all NeoTrix commands. Standardize output schema: `{ "status": "ok"|"error", "data": {...}, "meta": { "duration_ms": N, "tool_calls": [...] } }`. Emit JSON lines for streaming output.

### DEFECT D747-9: No `NO_COLOR` / Accessibility Compliance
**Source**: [github.com/LevyBytes/AI-SKILL-cli-design](https://github.com/LevyBytes/AI-SKILL-cli-design)  
**Finding**: The `NO_COLOR` environment variable (no-color.org) is the 2026 standard for disabling ANSI color in terminal output. Windows VT processing, `TERM` detection, and `COLORTERM` support are mandatory for cross-platform CLIs. NeoTrix CLI output uses hardcoded ANSI codes without `NO_COLOR` respect, no `TERM` detection, and no Windows VT fallback.  
**Impact**: CI pipelines, screen readers, and colorblind users cannot suppress or adapt color output. Automated tool output parsing breaks on unexpected ANSI escape sequences.  
**Action**: Add `NO_COLOR` env var check at CLI startup (set `color = false` when present). Add `--color=auto|always|never` flag. Use `anstyle` or `console` crate for terminal capability detection.

### DEFECT D747-10: No Slash Command Registry for Agent-Friendly UX
**Source**: [futureagi.com — Agent CLI DX 2026](https://futureagi.com/blog/agent-cli-developer-experience-2026/)  
**Finding**: 2026 agent CLIs (Claude Code, Codex CLI) implement a slash command registry (`/model`, `/diff`, `/save`, `/mcp`) that the CLI interprets locally without consuming LLM tokens. NeoTrix has no slash command system — all commands go through the full `neotrix_command` dispatch, consuming tokens even for local operations like `--help` or `--version`.  
**Impact**: Local operations (help, version, status, config) waste LLM tokens when invoked through the agent dispatch path. The slash registry pattern from the 2026 DX survey separates "local" from "remote" commands, reducing token cost by ~30% for interactive sessions.  
**Action**: Implement `nt/slash` registry with local-only commands: `/help`, `/version`, `/status`, `/config`, `/diff`, `/plan`. These are interpreted by the CLI binary without touching the consciousness core or LLM providers.

---

## 4. Cross-Cutting Defects

### DEFECT D747-11: Ratatui Ecosystem Divergence Risk
**Source**: [youngju.dev — TUI Renaissance 2026](https://www.youngju.dev/blog/culture/2026-05-14-tui-development-ratatui-bubbletea-ink-textual-terminal-ui-renaissance-deep-dive-2026.en)  
**Finding**: The 2026 TUI ecosystem has crystallized into language-specific de facto standards: Go=Bubble Tea, Rust=Ratatui, Python=Textual, Node=Ink. Cross-language TUI interop is nonexistent. NeoTrix is pure Rust, so Ratatui is the correct choice, but the `oh-my-pi` project (DeepWiki, 2026-09-05) shows a custom component-based TUI system with differential updates — suggesting that ratatui alone may not suffice for advanced rendering needs.  
**Impact**: If NeoTrix needs differential rendering (only redraw changed regions) or overlay support (modal dialogs over running views), ratatui's immediate-mode model forces full-frame redraws. Performance may degrade on embedded targets with slow displays.  
**Action**: Evaluate ratatui's `Buffer` diff capabilities (available since v0.29) for selective redraw. For embedded targets, consider a custom minimal renderer implementing the `EmbodimentDisplay` trait.

### DEFECT D747-12: Auth Token in Session File (Security Anti-Pattern)
**Source**: [futureagi.com — Agent CLI DX 2026](https://futureagi.com/blog/agent-cli-developer-experience-2026/)  
**Finding**: Storing auth tokens in plain-text session files is a recurring 2026 security regression. The 2026 DX survey explicitly flags this as a "common CLI design failure." NeoTrix stores LLM provider API keys in config files (likely `~/.config/neotrix/` or `~/.neotrix/`). If these are plain-text TOML/YAML, they are vulnerable to credential theft.  
**Impact**: API keys for paid LLM providers (OpenAI, Anthropic, etc.) exposed in plaintext. On shared systems, any user can read the config file. On CI, secrets leak into logs if `--verbose` is enabled.  
**Action**: Migrate API key storage to OS credential store: macOS Keychain (`security` CLI), Linux libsecret/`secret-tool`, Windows Credential Manager. Provide `nt auth login` / `nt auth logout` commands. Encrypt-at-rest for local session files using platform-native encryption.

---

## 5. Summary: What's NEW in Batch 747

| # | Defect | Category | Severity | Source |
|---|--------|----------|----------|--------|
| D747-1 | No plan-first loop in CLI | DX | HIGH | futureagi.com |
| D747-2 | No per-tool-call transparency | DX | HIGH | futureagi.com |
| D747-3 | No session rollback mechanism | DX | HIGH | futureagi.com |
| D747-4 | Clap v5 migration gap | CLI | MEDIUM | techbytes.app |
| D747-5 | No TUI dashboard for consciousness | TUI | HIGH | youngju.dev |
| D747-6 | No TUI layout preview for manga | TUI | MEDIUM | byteiota.com |
| D747-7 | Missing `no_std` embedded TUI | TUI | LOW | ratatui changelog |
| D747-8 | No structured JSON output mode | DX | HIGH | futureagi.com, getdx.com |
| D747-9 | No `NO_COLOR` / accessibility | DX | MEDIUM | github.com/LevyBytes |
| D747-10 | No slash command registry | DX | MEDIUM | futureagi.com |
| D747-11 | Ratatui differential rendering risk | TUI | LOW | youngju.dev |
| D747-12 | Auth token in plaintext config | Security | HIGH | futureagi.com |

## 6. Sources Cited

1. **futureagi.com** — "Agent CLI Developer Experience 2026: The Three-Axis DX Test" (2025-01-23, updated 2026-05-20)
2. **youngju.dev** — "The TUI Renaissance 2026 — Ratatui, Bubble Tea, Textual, Ink" (2026-05-14)
3. **byteiota.com** — "TUI Renaissance 2026: Why Terminal UIs Are Back" (2026-05-03)
4. **techbytes.app** — "Modern Rust CLI Development [2026] [Cheat Sheet]" (2026-07-05)
5. **dasroot.net** — "Building CLI Tools with Clap and Rust" (2026-01-30)
6. **codezup.com** — "Build a Rust CLI Tool with Clap: Step-by-Step (2026)" (2026-07-24)
7. **lucaberton.com** — "Building CLI Tools in Rust with Clap" (2026-05-04)
8. **blog.rajpoot.dev** — "Rust CLI Apps in 2026 — clap, indicatif, ratatui" (2026-05-02)
9. **github.com/LevyBytes/AI-SKILL-cli-design** — CLI UX Design skill (2026-06-22)
10. **docs.getdx.com** — DX CLI documentation (2026-09-02)
11. **byteiota.com** — "TUI Studio: Visual Terminal UI Design Tool" (2026-03-13)
12. **blog.teliaz.com** — "The Terminal Renaissance: Building Production TUIs" (2026-08-14)
13. **deepwiki.com** — "oh-my-pi / Terminal UI (TUI)" (2026-09-05)
