# targeted-research-701 — Internal Dispatch Route Wiring

## Date: 2026-09-13

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop in `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`.

## Routes Added

| # | Capability Tag | Domain | Specialist | Purpose |
|---|---------------|--------|------------|---------|
| 1 | `adversarial_router` | NT-SHIELD | RiskAssessor | Adversarial request detection — probes/blocks adversarial LLM inputs |
| 2 | `kb_dependency_graph` | NT-MEMORY | KnowledgeRetriever | KB dependency tracking graph — diff/impact/cycle detection/visualization |
| 3 | `experience_regression` | NT-MIND | MetaCognitionAnalyst | experience-tree regression testing — full/delta/smoke scope |
| 4 | `knowledge_compilation` | NT-MIND | KnowledgeIntegrator | Knowledge compilation pipeline — distill/merge/optimize phases |

## CAPABILITY_ROUTES Entries (5 each, 20 total)

Each route includes both Chinese and English keywords for natural language matching:

- `adversarial_router`: 对抗路由, 请求检测, adversarial, 攻击路由
- `kb_dependency_graph`: 知识库依赖图, kb依赖图, 知识依赖, dep_kb
- `experience_regression`: 经验回归测试, 经验树回归, exp_regression, 回归验证
- `knowledge_compilation`: 知识编译, 知识管线, compile_knowledge, 编译知识

## Match Arms

Each arm opens KB, reads stats, determines sub-mode from summary keywords, reads/updates relevant KV counters, and returns a formatted status string.

## Pre-existing Build Errors

Two unrelated errors in `nt_memory_lifecycle.rs` and `nt_memory_pack.rs` (missing `nt_memory_brain` / `nt_memory_pack_chunked` modules) prevent full compilation. These are pre-existing and unrelated to this change.
