# Iteration Batch 717 — Knowledge Management, Internal Docs & Knowledge Base Defects

**Date:** 2026-09-06  
**Research Sources:** 5 deep searches across knowledge management, internal docs, knowledge base, OKF spec, and code decision memory  
**Prior Batch:** 716 (security: LLM Firewall, east-west monitoring, AI agent security, microsegmentation, NDR)  

---

## Research Summary

### Sources Cited

| # | Source | URL | Domain |
|---|--------|-----|--------|
| S1 | Google OKF v0.2 SPEC | https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md | Knowledge format standard |
| S2 | Microsoft LLM Wiki | https://github.com/microsoft/llmwiki | VS Code wiki with MCP |
| S3 | Karpathy LLM Wiki Pattern | https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f | Foundational pattern |
| S4 | code-wiki | https://github.com/tuandm/code-wiki | Decision memory for codebases |
| S5 | DecisionGraph | https://github.com/hieuchaydi/DecisionGraph | Engineering decision memory + guardrails |
| S6 | doc-wiki (narailabs) | https://github.com/narailabs/doc-wiki | Enterprise LLM Wiki for coding agents |
| S7 | Focowiki | https://github.com/farozerolabs/focowiki | OKF-style enterprise knowledge base |
| S8 | OpenWiki (LangChain) | https://github.com/langchain-ai/openwiki | CLI agent wiki with OKF output |
| S9 | Kherad | https://github.com/mohammadmaso/kherad | Git-backed wiki + AI drafting |
| S10 | Wiki KB | https://github.com/SonicBotMan/wiki-kb | MCP knowledge base with compiled truth + timeline |
| S11 | llm-wiki-engine (Rust) | https://github.com/geronimo-iia/llm-wiki-engine | Headless wiki engine, MCP + ACP |
| S12 | Archiva | https://github.com/Jalkarna/archiva | Git-native decision memory |
| S13 | Wiki Forge | https://github.com/thevrus/Wiki-Forge | Decision provenance engine |
| S14 | OpenKB | https://github.com/VectifyAI/OpenKB | Reasoning-based retrieval KB |
| S15 | DocuWriter.ai | https://www.docuwriter.ai/posts/make-a-wiki | Auto-generating project wikis |
| S16 | OpenKnowledge OKF Plugin | https://openknowledge.ai/blog/open-knowledge-format-okf-plugin-linter | OKF linter + MCP audit |
| S17 | Sahaj 5-Framework Wiki | https://www.sahaj.ai/stitching-five-documentation-frameworks-into-one-coherent-engineering-wiki/ | Multi-framework wiki stitching |
| S18 | Factory AutoWiki | https://factory.ai/news/wiki | Documentation as build artifact |
| S19 | MeteorOps Repo Wiki | https://meteorops.com/blog/how-to-build-a-repo-wiki-engineers-actually-use | Repo wiki best practices |
| S20 | Docsio Engineering Docs | https://docsio.co/blog/engineering-documentation | 2026 engineering docs guide |
| S21 | llm-wiki-flywheel | https://github.com/Asun28/llm-wiki-flywheel | Automated wiki with trust scoring |
| S22 | decision-memory | https://github.com/RheaPatel/decision-memory | Scoped decision memory for code |
| S23 | OpenKnowledge Codebase Wiki | https://openknowledge.ai/docs/workflows/codebase-wiki | Agent-authored codebase maps |

---

## NEW Defects Identified

### DEFECT 717-01: No OKF-Native Knowledge Representation Layer (CRITICAL)
**Severity:** CRITICAL  
**Gap:** NeoTrix KB (SQLite) has no support for Google's Open Knowledge Format (OKF v0.2), which is rapidly becoming the vendor-neutral standard for AI agent knowledge. OKF bundles are plain markdown + YAML frontmatter with trust tiers, provenance, lifecycle, and attested computation fields. NeoTrix's KB uses custom schema without interoperability with the emerging ecosystem (Focowiki, OpenKB, OpenWiki, doc-wiki all produce OKF bundles).  
**Evidence:** S1 (OKF SPEC), S7 (Focowiki OKF), S8 (OpenWiki OKF output), S16 (OKF plugin/linter)  
**Impact:** NeoTrix cannot import/export knowledge bundles from/to other AI agent systems. Knowledge is locked in proprietary SQLite format. External agents cannot consume NeoTrix's knowledge without custom adapters.  
**Fix Required:**
- Add OKF v0.2 frontmatter support to KB nodes (type, title, description, resource, tags, timestamp, trust tiers, lifecycle status)
- Implement OKF bundle export/import as a KB operation
- Add `index.md` and `log.md` generation for any KB namespace
- Support `generated`, `verified`, `sources` provenance fields
- Add `stale_after` and lifecycle status tracking

---

### DEFECT 717-02: No "Why" Decision Memory System (HIGH)
**Severity:** HIGH  
**Gap:** NeoTrix has no system to capture and surface the *rationale* behind architectural decisions. Code-wiki, DecisionGraph, Archiva, Wiki Forge, and decision-memory all focus on capturing "why" (decisions, gotchas, constraints, superseded approaches) — information that leaves when engineers leave. NeoTrix captures *what* (code, KB nodes) but not *why*.  
**Evidence:** S4 (code-wiki: "captures decisions, superseded approaches, gotchas"), S5 (DecisionGraph: "engineering decision memory"), S12 (Archiva: "git-native decision memory"), S13 (Wiki Forge: "decision provenance engine"), S22 (decision-memory: "scoped decision memory")  
**Impact:** When NeoTrix's codebase evolves, agents cannot understand why patterns were chosen. Re-evaluation of already-resolved decisions wastes context. Agents may reintroduce patterns that were explicitly rejected.  
**Fix Required:**
- Add `DecisionRecord` type to KB with fields: id, summary, rationale, scope (file globs), tags, author, source, confidence, status (active/superseded/archived), supersededBy
- Implement `.decisions/` directory structure (markdown files, version-controlled)
- Add MCP tools: `query_decisions`, `record_decision`, `list_decisions`, `check_guardrail`
- Implement supersession chains (new decision replaces old with link)
- Add code-anchored rationale (decisions attached to specific functions/classes, not just files)

---

### DEFECT 717-03: No Knowledge Compounding / Accumulation Across Sessions (HIGH)
**Severity:** HIGH  
**Gap:** NeoTrix's KB stores data but does not implement the "compile-at-write-time" pattern (Karpathy LLM Wiki). Knowledge should compound: each ingest updates, cross-links, and strengthens existing wiki pages rather than creating isolated entries. Current KB is retrieve-at-query-time (RAG-style), not compile-at-write-time (DKR-style).  
**Evidence:** S3 (Karpathy: "knowledge compounds with every addition"), S11 (llm-wiki-engine: "Dynamic Knowledge Repository"), S21 (llm-wiki-flywheel: "compile, don't retrieve"), S10 (Wiki KB: "compiled truth + timeline")  
**Impact:** Every session starts from near-zero. Cross-references must be rebuilt each time. Knowledge doesn't improve over time. The KB accumulates data but not *understanding*.  
**Fix Required:**
- Implement "wiki compilation" pipeline: on ingest, LLM reads existing pages, updates cross-links, notes contradictions, strengthens synthesis
- Add retroactive wikilink injection (new topic auto-linked into existing pages)
- Implement incremental re-compilation (SHA-256 change detection for source files)
- Add knowledge graph with typed relationships (not just flat nodes)
- Track `confidence_score` and `last-verified` per wiki page

---

### DEFECT 717-04: No Trust/Provenance Tracking for KB Entries (HIGH)
**Severity:** HIGH  
**Gap:** NeoTrix KB entries have no trust tiers, provenance signals, or verification status. OKF v0.2 defines `generated`, `verified`, `sources` (with per-source credibility signals), and trust tiers. Wiki KB has "compiled truth + timeline" where timeline (evidence) always wins over summary. llm-wiki-flywheel has Bayesian trust scoring.  
**Evidence:** S1 (OKF §5: provenance/trust/lifecycle), S10 (Wiki KB: "timeline wins over truth"), S21 (llm-wiki-flywheel: "Bayesian trust scoring, contradiction detection")  
**Impact:** Users cannot distinguish verified knowledge from agent-hallucinated content. No audit trail for who/what confirmed a claim. Stale or incorrect knowledge persists without detection.  
**Fix Required:**
- Add `generated_by` (agent/model), `verified_by` (human/agent), `trust_tier` (unverified/verified/authoritative) to KB entries
- Implement `sources` array with per-source credibility signals (fetch_method, source_age, citation_count)
- Add `stale_after` timestamp for lifecycle tracking
- Implement contradiction detection on ingest (new fact vs existing fact)
- Add `wiki_review` quality gate: promote draft → active only when confidence threshold met

---

### DEFECT 717-05: No Knowledge Gap Detection (MEDIUM)
**Severity:** MEDIUM  
**Gap:** NeoTrix has no proactive mechanism to detect missing knowledge. Multiple tools (llm-wiki, Wiki KB, OpenKB, llm-wiki-flywheel) implement gap analysis: the system hypothesizes questions users might ask, attempts to answer them, identifies where knowledge is missing, and suggests what documents to create next.  
**Evidence:** S10 (Wiki KB: "proactive gap analysis — system guesses questions, self-answers, identifies missing info"), S14 (OpenKB: "Skill Factory — distills redistributable skills from wiki"), S21 (llm-wiki-flywheel: "kb evolve — coverage gaps, connection opportunities")  
**Impact:** Knowledge base grows reactively. Important architectural context is never captured because no one asks about it. Teams discover knowledge gaps only during incidents.  
**Fix Required:**
- Implement proactive gap analysis: generate candidate questions across 9 angles (what/how/why/compare/best_practice/pitfall/integration/perf/security)
- For each candidate, attempt retrieval + LLM synthesis, then judge as answerable/partial/no
- Aggregate frequency-ranked suggestions for new documents
- Add `kb evolve` command: coverage gaps, connection opportunities, missing page types, disconnected graph components

---

### DEFECT 717-06: No Staleness Detection / Knowledge Drift (MEDIUM)
**Severity:** MEDIUM  
**Gap:** NeoTrix KB entries have no staleness detection. When source code changes, related KB entries become stale without notification. code-wiki, Wiki Forge, and Archiva all implement drift detection: track source code fingerprints, and when code changes, flag wiki pages whose source has moved.  
**Evidence:** S4 (code-wiki: "content-hash drift detection flags pages whose source code has moved"), S13 (Wiki Forge: "when code changes, detects drift and rewrites only what changed"), S12 (Archiva: "stale decisions when code fingerprints change")  
**Impact:** Agents read stale documentation and make incorrect assumptions. KB trust degrades silently over time.  
**Fix Required:**
- Track `code_paths` (specific files/dirs) per wiki page in frontmatter
- On code changes (git hook or CI), re-verify wiki pages against current code
- Update `last-verified` and `confidence_score` on re-verification
- Add `wiki_lint --fix` to re-verify stale topics
- Implement `kb_detect_drift` MCP tool

---

### DEFECT 717-07: No Structured Documentation Framework Stitching (MEDIUM)
**Severity:** MEDIUM  
**Gap:** NeoTrix has no standardized top-level documentation structure for internal engineering docs. Sahaj's 5-framework stitching approach shows how to combine Diataxis (development), C4+arc42 (architecture), GitLab Handbook (team), cheat-sheet pattern (ops), and Cagan (discovery) into a coherent wiki with 8 top-level sections. Each section has a clear home for every page.  
**Evidence:** S17 (Sahaj: "8 top-level sections — Discovery, Architecture, Development, Team, Operations, Release, Support, Functional")  
**Impact:** Internal docs are disorganized. New engineers cannot find what they need. Documentation lives in random locations without consistent structure.  
**Fix Required:**
- Define 8 top-level wiki sections: Discovery, Architecture, Development, Team, Operations, Release, Support, Functional
- Each section gets a framework convention (Diataxis for Dev, C4+arc42 for Architecture, etc.)
- Implement "every page has exactly one obvious home" routing
- Add section-level frontmatter templates

---

### DEFECT 717-08: No Cross-Service/Ecosystem Knowledge Integration (MEDIUM)
**Severity:** MEDIUM  
**Gap:** NeoTrix KB is code-only. Enterprise wiki tools (doc-wiki, Focowiki, AutoWiki) integrate external ecosystem knowledge: Jira tickets, Confluence pages, GitHub issues, database schemas, AWS/GCP configs, and team-specific artifacts. doc-wiki's "ecosystem-aware wiki" sees cross-service dependencies when multiple repos are present.  
**Evidence:** S6 (doc-wiki: "Jira/Confluence/GitHub/Notion/Linear/AWS/GCP route through one connector planner"), S18 (AutoWiki: "documentation as build artifact — refreshed on every push")  
**Impact:** NeoTrix's knowledge is incomplete — it knows the code but not the surrounding ecosystem (tickets, decisions, infrastructure).  
**Fix Required:**
- Implement connector abstraction for external services (Jira, GitHub, Confluence, Notion)
- Add DB schema ingestion (ORM entity → table mapping)
- Support multi-repo awareness (cross-service dependency detection)
- Add CI/CD integration for auto-refresh on push

---

### DEFECT 717-09: No Decision Guardrails / Pre-Change Safety Checks (MEDIUM)
**Severity:** MEDIUM  
**Gap:** NeoTrix has no mechanism to check if a proposed code change conflicts with existing architectural decisions before execution. DecisionGraph implements guardrails: before a refactor, query the decision memory for conflicts. Archiva has `ghost_check` for stale/orphaned decisions.  
**Evidence:** S5 (DecisionGraph: "pre-change guardrails — query decision memory before risky refactor"), S12 (Archiva: "ghost_check — stale decisions when code fingerprints change")  
**Impact:** Agents may make changes that violate architectural decisions without knowing. No safety net for decision-aware refactoring.  
**Fix Required:**
- Implement `check_guardrail` MCP tool: given a proposed change, query decision memory for conflicts
- Add `ghost_check`: detect decisions whose scope files have changed significantly
- Surface advisory warnings (not blocking) when editing files with active decisions
- Integrate with NT-SHIELD for safety validation

---

### DEFECT 717-10: No Knowledge-as-Build-Artifact Pipeline (LOW)
**Severity:** LOW  
**Gap:** NeoTrix documentation is manually maintained. Factory AutoWiki and OpenWiki demonstrate "documentation as build artifact": wiki generation runs in CI/CD on every push, producing versioned documentation alongside code. Wiki changes appear in PR diffs.  
**Evidence:** S18 (AutoWiki: "documentation as a build artifact — produced automatically, versioned alongside code"), S8 (OpenWiki: "CI workflow auto-opens PR with documentation updates")  
**Impact:** Documentation falls behind code. No version history of doc changes. Docs cannot be reviewed in PRs.  
**Fix Required:**
- Add CI workflow: on push to main, regenerate affected wiki pages
- Wiki changes committed to repo (or opened as PR)
- Version wiki by commit hash and timestamp
- Implement incremental regeneration (diff from last commit, only update affected pages)

---

### DEFECT 717-11: No Multi-Framework Documentation Integration (LOW)
**Severity:** LOW  
**Gap:** NeoTrix uses only one documentation approach internally. The Sahaj approach shows how to stitch 5+ frameworks (Diataxis, C4, arc42, GitLab Handbook, cheat-sheet) into a single coherent wiki, with each framework covering its strength area. NeoTrix's AGENTS.md references rules but not a structured documentation framework.  
**Evidence:** S17 (Sahaj: "stitching five-plus frameworks into one coherent top-level shape")  
**Impact:** Documentation quality varies by author. No consistent structure across different doc types.  
**Fix Required:**
- Adopt Diataxis for Development docs (tutorials, how-to, reference, explanation)
- Adopt C4+arc42 for Architecture docs
- Add "cheat-sheet pattern" for Operations (one-pager per service)
- Document the framework choices explicitly

---

### DEFECT 717-12: No LLM-Context-Optimized Knowledge Export (LOW)
**Severity:** LOW  
**Gap:** NeoTrix KB has no `llms.txt` / `llms-full.txt` export format. The llms.txt standard (https://llmstxt.org/) defines priority-tiered, token-budgeted knowledge export for LLM consumption. llm-wiki-flywheel, llm-wiki-engine, and OpenKB all emit `llms.txt` + `llms-full.txt` for agent consumption.  
**Evidence:** S3 (Karpathy/llms.txt standard), S21 (llm-wiki-flywheel: "kb publish Tier-1 builders"), S11 (llm-wiki-engine: "wiki_export to llms.txt, llms-full, or JSON")  
**Impact:** External agents cannot efficiently consume NeoTrix's knowledge. No standardized way to share knowledge with other AI systems.  
**Fix Required:**
- Implement `llms.txt` export (index + summaries, ~4KB per page)
- Implement `llms-full.txt` export (full content of all pages)
- Add `graph.jsonld` export (knowledge graph as JSON-LD)
- Make export command available via MCP: `wiki_export --format={llms|llms-full|json|markdown}`

---

### DEFECT 717-13: No Quality Gate / Lint System for KB Health (LOW)
**Severity:** LOW  
**Gap:** NeoTrix KB has no health-check or linting system. Multiple tools implement lint: dead links, orphan pages, stubs, low-confidence pages, broken cross-references, wikilink cycles, duplicate slugs. Wiki KB's quality gate (`wiki_review`) improved page quality from 13% to 100% passing.  
**Evidence:** S10 (Wiki KB: "wiki_review — draft → active only when summary ≥50 chars, facts ≥2"), S21 (llm-wiki-flywheel: "kb lint — dead links, orphans, staleness, stubs, frontmatter, source coverage, wikilink cycles, duplicate slugs, low-trust pages")  
**Impact:** KB quality degrades silently. No mechanism to detect or fix common issues.  
**Fix Required:**
- Implement `wiki_lint` with severity tiers (ERROR/WARNING/INFO)
- Check: orphan pages, broken cross-references, missing frontmatter, stale pages, low confidence, wikilink cycles, duplicate slugs
- Add `wiki_lint --fix` for auto-repair of common issues
- Integrate lint into SEAL pipeline as Phase-0 health check

---

## Summary Table

| Defect ID | Severity | Category | Summary |
|-----------|----------|----------|---------|
| 717-01 | CRITICAL | Knowledge Format | No OKF v0.2 native knowledge representation |
| 717-02 | HIGH | Decision Memory | No "why" decision memory system |
| 717-03 | HIGH | Knowledge Compounding | No compile-at-write-time pattern |
| 717-04 | HIGH | Trust/Provenance | No trust tiers or verification tracking |
| 717-05 | MEDIUM | Gap Detection | No proactive knowledge gap analysis |
| 717-06 | MEDIUM | Staleness | No knowledge drift detection |
| 717-07 | MEDIUM | Doc Structure | No multi-framework documentation stitching |
| 717-08 | MEDIUM | Ecosystem Integration | No external service knowledge connectors |
| 717-09 | MEDIUM | Safety | No decision guardrails for refactoring |
| 717-10 | LOW | CI/CD | No documentation-as-build-artifact pipeline |
| 717-11 | LOW | Framework | No Diataxis/C4/arc42 framework adoption |
| 717-12 | LOW | Export | No llms.txt / standardized knowledge export |
| 717-13 | LOW | Quality | No KB health lint system |

---

## Cross-Reference with Prior Batches

| Prior Batch | Topic | Relation to 717 |
|-------------|-------|-----------------|
| 716 | LLM Firewall | DEFECT 717-01 (OKF) — firewall can inspect OKF bundles for trust signals |
| 716 | AI Agent Security | DEFECT 717-04 (Trust) — trust tiers enable agent-specific access control |
| 716 | East-West Monitoring | DEFECT 717-09 (Guardrails) — decision memory can inform network-level guardrails |
| 715 | Knowledge Lifecycle | DEFECT 717-06 (Staleness) — lifecycle tracking complements retention policies |

---

## Priority Order for Implementation

1. **DEFECT 717-01** — OKF format (unblocks ecosystem interoperability)
2. **DEFECT 717-02** — Decision memory (captures institutional knowledge)
3. **DEFECT 717-03** — Knowledge compounding (makes KB improve over time)
4. **DEFECT 717-04** — Trust/provenance (makes KB trustworthy)
5. **DEFECT 717-05** — Gap detection (makes KB proactive)
6. **DEFECT 717-06** — Staleness detection (keeps KB current)
7. **DEFECT 717-09** — Guardrails (safety for refactoring)
8. **DEFECT 717-13** — Lint system (KB health monitoring)
