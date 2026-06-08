source_id: autoscientists
url: https://github.com/mims-harvard/AutoScientists
category: consciousness_evolution
analyzed: true
depth: full (README + runbook.md + LAUNCH.md)
status: crawled
priority: high
neotrix_phase: 1.1, 1.4

## Key Patterns Adopted
- Hook template pattern (runbook.md + LAUNCH.md)
- Meta-improvement every 3 cycles (self-editing ROLE files)
- Pure coordinator principle (orchestrator never runs experiments)
- Stagnation detection (0 KEEP in last 10 → stop)
- Champion promotion as single source of truth

## Crawl Plan
- [x] README.md
- [x] runbook.md (orchestrator base program)
- [x] task-autoresearch/LAUNCH.md (hook implementations)
- [] task-autoresearch/TASK.md (transport error)
- [] system/reference/SKILL.md
- [] system/templates/HEARTBEAT.md
- [] launch.py
- [] paper: arxiv.org/abs/2605.28655

## Relevance to NeoTrix
Directly maps to pipeline architecture (Phase 1.1) and meta-cognitive loop (Phase 1.4).
