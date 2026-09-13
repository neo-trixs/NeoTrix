# Targeted Research 696 — Consciousness Core Dispatch Routes

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop. Each route includes CAPABILITY_ROUTES keyword entries (6 per route) and a dedicated match arm in `dispatch_internal_capability`.

## New Routes

| Route | Capability Tag | Domain | Specialist | Purpose |
|-------|---------------|--------|------------|---------|
| `honeyroute` | `honeyroute` | NT-SHIELD | RiskAssessor | Adversarial LLM detection — honeypot probe tracking, attack interception |
| `knowledge_reasoning_sep` | `knowledge_reasoning_sep` | NT-MIND | KnowledgeIntegrator | Knowledge/reasoning separation — decompose, partition, classify knowledge vs reasoning artifacts |
| `dependency_graph` | `dependency_graph` | NT-MEMORY | KnowledgeRetriever | KB dependency tracking — graph traversal, impact analysis, cycle detection |
| `regression_enrichment` | `regression_enrichment` | NT-MIND | MetaCognitionAnalyst | Experience-tree regression enrichment — backfill, propagate, enrich regression test knowledge |

## Keyword Mappings (per route)

### honeyroute
- `honeyroute`, `蜜罐路由`, `对抗检测`, `llm对抗`, `honey`, `adversarial`

### knowledge_reasoning_sep
- `knowledge_reasoning_sep`, `知识推理分离`, `推理分离`, `知识推理拆分`, `reasoning_sep`, `kr_sep`

### dependency_graph
- `dependency_graph`, `依赖图`, `依赖追踪`, `kb依赖`, `dep_graph`, `依赖关系`

### regression_enrichment
- `regression_enrichment`, `回归富化`, `经验回归富化`, `回归增强`, `enrichment`, `经验富化`

## Dispatch Behavior

Each match arm opens `KnowledgeBase`, reads relevant stats/counters, determines mode from instruction keywords, and returns a formatted status string. All routes are KB-backed and report node/edge/kv counts.

## Verification

- `cargo check -p neotrix --lib` — passed
- 42 new route occurrences across CAPABILITY_ROUTES + match arms
