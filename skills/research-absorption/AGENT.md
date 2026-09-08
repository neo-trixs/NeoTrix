# External Research Absorption Agent

## Purpose
Automated 10000+ cycle research absorption pipeline that searches, reads, extracts, and maps external projects to NeoTrix architecture decisions.

## Pipeline (Per Cycle)

```
1. SEARCH  → Find new projects by category/keyword
2. READ    → Fetch README/docs/papers
3. EXTRACT → Core purpose + problem + solution + evidence
4. MAP     → Map to NeoTrix L0-L6 layers + existing decisions
5. WRITE   → Generate new D-decisions + C-patterns
6. VERIFY  → Check uniqueness + alignment
```

## Categories (Priority Order)

| Priority | Category | Keywords | Target D/C |
|----------|----------|----------|------------|
| P0 | Memory Systems | agent memory, persistent memory, memory consolidation | D126-D134 |
| P0 | Multi-Agent Orchestration | colony, swarm, multi-agent, worker | D121, D128-D130 |
| P0 | Workflow Engines | DAG, workflow, pipeline, execution engine | D120 |
| P1 | Agent Frameworks | ReAct, tool use, agent loop, reasoning | D117-D119 |
| P1 | Code Generation | coding agent, code generation, SWE | D123-D125 |
| P1 | Knowledge Graphs | graph memory, knowledge graph, entity linking | D50-D52 |
| P2 | Security/Safety | guardrails, sandbox, safety, alignment | D59-D61, D80 |
| P2 | Observability | tracing, monitoring, health, metrics | D49, D68-D69 |
| P2 | Browser/Automation | web scraping, browser, automation | D115-D116 |

## Search Strategy

### Sources (Ordered by Signal)
1. **GitHub Trending** (daily/weekly) — highest signal for new projects
2. **arXiv** (cs.AI, cs.CL, cs.SE) — papers with code
3. **Papers With Code** — benchmark leaders
4. **Hacker News** — high-signal discussions
5. **Reddit** (r/MachineLearning, r/LocalLLaMA) — community signal

### Search Queries (Per Category)
```
memory: "agent memory" OR "memory consolidation" OR "episodic memory" stars:>100
orchestration: "multi-agent" OR "colony" OR "swarm" stars:>50
workflow: "DAG execution" OR "workflow engine" OR "pipeline" stars:>100
coding: "coding agent" OR "SWE" OR "code generation" stars:>50
security: "guardrails" OR "sandbox" OR "agent safety" stars:>50
```

## Extraction Template

For each project, extract:
```yaml
project: <name>
stars: <N>
core_purpose: <1 sentence>
problem: <what problem does it solve>
solution: <how does it solve it>
evidence: <key technical details>
neo_trix_mapping: <which L0-L6 layer>
existing_decision: <D## if extends existing>
novel_pattern: <C## if new pattern>
priority: P0/P1/P2
```

## Decision Generation Rules

1. **UNIQUE**: Each D must address a distinct architectural concern
2. **EVIDENCE-BACKED**: Every D must cite specific research
3. **MAPPED**: Every D must map to a NeoTrix module
4. **SYNERGIZED**: Every D must reference existing Ds it collaborates with
5. **ABSORBED**: Use corrected methodology (why→what→how→map)

## Pattern Generation Rules

1. **COMPOSABLE**: Each C must be composable with other Cs
2. **LAYERED**: Each C must map to L0-L6 layers
3. **TESTABLE**: Each C must be verifiable
4. **EVOLVABLE**: Each C must support self-evolution

## Batch Processing

### Cycle Batch Size
- Single cycle: 1 project → 1 D + 1 C
- Batch: 5-10 projects → 5-10 Ds + 5-10 Cs
- Mega batch: 50+ projects → 20-30 Ds + 15-20 Cs

### Deduplication
Before writing, check:
1. Does D## already exist for this concern?
2. Does C## already capture this pattern?
3. Is this a genuine extension or a duplicate?

### Quality Gates
- [ ] Core purpose extracted (not just mechanism)
- [ ] Problem statement clear (what does it solve)
- [ ] Evidence cited (specific numbers/benchmarks)
- [ ] NeoTrix mapping correct (L0-L6)
- [ ] Synergy references included (existing Ds)
- [ ] No duplicates (unique D/C)

## Implementation

### Phase 1: Search (Cycles 1-1000)
- Search all categories systematically
- Build project inventory
- Prioritize by star count + novelty

### Phase 2: Read (Cycles 1001-3000)
- Read top 500 projects by priority
- Extract core purpose + problem + solution
- Map to NeoTrix architecture

### Phase 3: Write (Cycles 3001-5000)
- Generate D-decisions for unique concerns
- Generate C-patterns for reusable patterns
- Update architecture document

### Phase 4: Verify (Cycles 5001-7000)
- Check for duplicates
- Verify synergy references
- Validate quality gates

### Phase 5: Evolve (Cycles 7001-10000+)
- Re-scan for new projects (weekly)
- Update existing Ds with new evidence
- Merge related Ds
- Split overly broad Ds

## Usage

```bash
# Run single absorption cycle
bash skills/research-absorption/run.sh --cycle 1 --category memory

# Run batch of 10 cycles
bash skills/research-absorption/run.sh --batch 10 --category all

# Run full 10000-cycle pipeline
bash skills/research-absorption/run.sh --target 10000 --parallel 4
```

## Output Format

Each cycle produces:
```markdown
### D{N}: **{Title}**
| # | 决策领域 | 问题 | 研究证据 | 架构决策 | 实现位置 |
|---|---------|------|---------|---------|---------|
| D{N} | **{领域}** | {问题}? | {证据} | **{决策}**: {细节} | `nt_{module}::{name}` |
```

## Metrics

Track per cycle:
- Projects searched
- Projects read
- Decisions generated
- Patterns generated
- Duplicates detected
- Quality gate pass rate
