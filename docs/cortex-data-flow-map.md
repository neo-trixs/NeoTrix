# Cortex (外置大脑) Data-Flow Lineage Map

> Evidence-first map of the current external-brain (`/Volumes/NeoTrixBrain`, 114 GB) data flow in NeoTrix.
> Every claim cites `file:line`. No logic was changed; this is documentation only.

## 0. Scope & shared language

- **NT-MEMORY** owns the live KB — a SQLite store at `~/.neotrix/knowledge.db` (the "live KB").
- **NT-CORE / E8 Hexagram** consumes the external brain's causal graph via the abduction bridge.
- **SEAL Pipeline** (NT-MIND) is the evolution loop that distills/absorption-writes into the live KB.
- **External brain (外置大脑)** = the mounted volume `/Volumes/NeoTrixBrain` (constant `CORTEX_ROOT`, defined identically in `cortex_cmds.rs:17`, `nt_memory_resource_ingest.rs:397`, and read at `e8_abduction_bridge.rs:31`). It holds:
  - `cortex-archive/{zim,pmtiles,wikipedia}` — 114 GB offline archives.
  - `working/causal_graph.json` — exported E8 causal graph (`CORTEX_CAUSAL`, `cortex_cmds.rs:18`).
  - `knowledge-archive-corpus-20260825.db` — 68 GB cold corpus copy (superset snapshot of live KB).

**Bottom line:** the external brain is today a **read-only + cold-archive sink**. There is **no reverse flow** (evolved knowledge, embeddings, or SEAL absorption output is never written back to it). See §3.

---

## 1. End-to-end data flow (as currently implemented)

```
                 EXTERNAL BRAIN (/Volumes/NeoTrixBrain)                 LIVE KB (~/.neotrix/knowledge.db)
┌─────────────────────────────────────────────────────┐        ┌──────────────────────────────────────────┐
│ cortex-archive/{zim,pmtiles,wikipedia}               │        │ nodes (node_type, metadata JSON,           │
│ working/causal_graph.json  ───────────────┐          │        │   source_episode, supersedes, tier)        │
│ knowledge-archive-corpus-20260825.db      │          │        │ kv_store (namespace='experience')          │
└─────────────────────────────────────────────────────┘        │ edges                                         │
            │ (mount required)                                   │                                               │
            │ ① INGEST/REGISTER                                  │ ④ SEAL ABSORPTION (write)                    │
            ▼                                                    │    neotrix-experience bin → kv_store          │
 register_cortex_brain() ── upsert_cortex_node() ──▶  INSERT     │    (experience.rs:6,230)                    │
 (nt_memory_resource_ingest.rs:426)   (rs:401)       cortex_brain│    run_seal_loop → SelfIteratingBrain        │
            │                                   nodes            │    (seal_loop.rs:280,641)                    │
            │ ② CONSUME (causal graph)                            │         │                                     │
            ▼                                                    │         ▼                                     │
 E8AbductionBridge::new() ── load_cortex_causal_graph() ─▶       │   (stays in live KB; NO write-back)         │
 (e8_abduction_bridge.rs:31)      (abduction/mod.rs:66)          │                                               │
            │                                   merged into      │  ③ corpus cold copy (one-way, local→ext):    │
            ▼                                   abduction engine │    migrate_cortex_corpus() (rs:504)          │
 E8 abduction cycle (predict / hypothesize / explain)            │    → knowledge-archive-corpus-*.db on ext    │
                                                                  │                                               │
            └──────────────  GAP: nothing writes back ───────────┘
```

### ① Ingestion → Registration (`cortex_source://` nodes)

- `register_cortex_brain(conn, root)` scans `cortex-archive/{zim,pmtiles,wikipedia}` + `working/causal_graph.json` and upserts lightweight **`cortex_brain`** registry nodes so the volume is *discoverable and connected* (Dark Forest: connect, don't bloat). `nt_memory_resource_ingest.rs:426-468`.
- Per populated archive sub-dir it creates a `cortex_source://<sub>` node via `register_archive_dir` (`nt_memory_resource_ingest.rs:766-798`); the causal graph becomes `cortex_source://causal_graph` (rs:438-445); the 68 GB corpus becomes `cortex_source://corpus-archive` (rs:458-465).
- `upsert_cortex_node` (`nt_memory_resource_ingest.rs:401-418`) writes `nodes (id,node_type='cortex_brain',title,content,url,created_at,updated_at,data_tier='cache',tier='warm',metadata)`. The `kind` discriminator (`cortex_archive_dir` / `cortex_causal_graph` / `corpus_cold_archive`) lives **inside `metadata`** because the `nodes` table has no `kind` column (rs:399-400).
- **T3 production wiring:** `register_cortex_brain` is called from `ingest_session_resources` at startup (`nt_memory_resource_ingest.rs:389-391`), so registration runs in production, not just on `/cortex register`. It no-ops when the volume is unmounted (rs:427-429).
- CLI surface: `/cortex status` counts `SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'` (`cortex_cmds.rs:89-97`).

### ② Consumption — E8 abduction reads `causal_graph.json`

- On construction, `E8AbductionBridge::new` checks `working/causal_graph.json` and, if present, calls `abductive_engine.load_cortex_causal_graph` (`e8_abduction_bridge.rs:31-36`). Missing file is non-fatal — engine degrades to built-in graph only (rs:28-30).
- `load_cortex_causal_graph` (`abduction/mod.rs:66-81`) parses via `CausalGraph::from_cortex_json` and **remaps indices** so external nodes/edges merge without clobbering local inference state. Returns edge count merged.
- `from_cortex_json` (`abduction/causal_graph.rs:157-193`) expects schema `{nodes:[{id}], links:[{s,t,v}], ts}`; external nodes get default confidence `0.8` (un-observed external claim) and edges are labelled `"causal"` (rs:151-153).
- CLI surface: `/cortex causal` loads and summarizes the graph via `CausalGraph::from_cortex_json` (`cortex_cmds.rs:196-216`).
- The merged graph then drives abduction: `record_transition_with_abduction` → `run_abduction_cycle` (`e8_abduction_bridge.rs:68-110`, `abduction/mod.rs:171-240`).

### ③ Cold corpus copy (one-way, local → external)

- `migrate_cortex_corpus` (`nt_memory_resource_ingest.rs:504-545`) copies the local 68 GB `knowledge-archive-corpus-20260825.db` onto the external brain as **cold storage**. Idempotent (skips if complete copy exists, rs:514-521), space-guarded (rs:522-528), **resumable on volume drop** via `copy_resumable` with a `<dest>.prog` sidecar (rs:551-611). Source is preserved (redundancy).
- `corpus_archive_path` (rs:472-486) prefers the external copy, falls back to local `~/.neotrix` copy. The corpus is a **superset snapshot** of the live KB and is explicitly **not merged** into the warm live KB (rs:447-449, `Dark Forest: connect, don't bloat`).
- CLI surface: `/cortex corpus status|migrate [--force]` (`cortex_cmds.rs:128-176`).

### ④ Evolution — SEAL absorption / distillation writes to LIVE KB only

- The SEAL loop runs via `SelfIteratingBrain::run_seal_loop` → `close_iteration_loop` (`seal_loop.rs:280`, `641-733`). The acceptance gate persists the candidate behavior; rejection rolls back (`_snapshot_restore`, rs:728).
- Distilled/absorption experience is persisted to the live KB by the `neotrix-experience` binary into `kv_store` namespace `experience` (`experience.rs:6`, `:230` INSERT). High-signal findings are promoted to `experience_targets` immediately (rs:1144-1151) and auto-distilled past a threshold (rs:1248-1267).
- The internal reasoning brain also has a `brain_write_back` (`nt_io_user_avatar.rs:458`, exposed as Tauri `brain_cmds.rs:93`) — but this writes to the **in-memory / `~/.neotrix` ReasoningBrain**, **not** the external brain volume.

---

## 2. Concrete gaps for evolutionary iteration

All gaps are evidence-backed by the read-only constant usage in §1.

**G1 — No reverse flow (write-back).** Every `grep` of `/Volumes/NeoTrixBrain` shows only reads (`register_*`, `load_cortex_causal_graph`, `from_cortex_json`, corpus migrations) and one cold-copy *out* (`migrate_cortex_corpus`). There is **no code path** that writes evolved knowledge, VSA/KB embeddings, SEAL absorption output, or `experience` namespace content back to the volume. The external brain is a **frozen snapshot + read source**.

**G2 — No delta / sync / versioning.** `migrate_cortex_corpus` is a full-file copy with a `.prog` checkpoint (resumable) but no content delta, no schema version, and no `ts`/hash comparison beyond byte-size equality (rs:514-521). `from_cortex_json` reads a `ts` field (`causal_graph.rs:160-161`) but never compares or records it, so staleness vs. the live KB is undetectable.

**G3 — No lineage tracking.** `cortex_brain` nodes carry only `kind` + `path` + `files`/`bytes` in `metadata` (`nt_memory_resource_ingest.rs:783-787`, `433-437`, `452-457`). There is **no field** linking a `cortex_source` node or any live KB node to:
  - which SEAL absorption cycle produced/updated it,
  - which external-brain snapshot it was derived from,
  - provenance (`source_episode` exists on normal nodes, rs:248, but `cortex_brain` upserts set it `NULL`, rs:413).
  The live KB's `nodes` table *does* support `source_episode` and `supersedes` (`nt_memory_resource_ingest.rs:247-248`; schema in `nt_memory_setting_consistency.rs:155-189`) — but `cortex_brain` nodes don't use them.

**G4 — External brain is inert between mounts.** `register_cortex_brain` and `load_cortex_causal_graph` both no-op/skip when unmounted (`nt_memory_resource_ingest.rs:427-429`; `e8_abduction_bridge.rs:32`). There is no background reconciliation that would push live-KB deltas to the volume when it re-mounts.

**G5 — Orphan reclaim is one-directional and corpus-dependent.** `prune_cortex_orphans` (rs:675-738) only deletes live KB nodes whose ZIM backing vanished from the volume; it never updates or version-stamps the external archive. It relies on the 68 GB corpus as the superset backing store (rs:687-706) — a fragile cross-file contract with no checksum.

---

## 3. Proposed lineage / versioning strategy

### 3.1 Provenance model (additive, no logic change required to land)

Extend the `metadata` JSON on `cortex_brain` nodes (and optionally normal `nodes`) with a `lineage` block:

```jsonc
{
  "kind": "cortex_causal_graph",
  "path": "/Volumes/NeoTrixBrain/working/causal_graph.json",
  "loaded_by": "E8AbductionBridge",
  "lineage": {
    "external_snapshot_ts": 1700000000,     // from causal_graph.json 'ts' (causal_graph.rs:160)
    "external_sha256": "<hash of file at load>",
    "last_synced_at": 1700000001,
    "last_seal_cycle": "NNN",               // ties to experience namespace cycle (experience.rs hub)
    "direction": "in",                      // in=consumed; out=written-back
    "kb_nodes_derived": 1234
  }
}
```

This reuses the existing `metadata` column (no schema migration) and the existing `source_episode`/`supersedes` fields for normal nodes. A `corpus_cold_archive` node gains `lineage.external_sha256` + `lineage.last_seal_cycle` so a future `/cortex sync` can detect drift (G2/G3).

### 3.2 Make the external brain an active evolution partner (bidirectional sync boundary)

Introduce a **sync boundary** module (new, additive) `nt_memory_cortex_sync` that, when the volume is mounted, performs:

1. **In-bound (already exists):** register + load causal graph (today's §1.①②).
2. **Out-bound (new):** serialize a *delta* of the live KB since last sync:
   - new/updated `nodes` + `edges` (filter by `updated_at > lineage.last_synced_at`),
   - new `experience` namespace entries (SEAL absorption cycles, `experience.rs:230`),
   - new VSA/KB embeddings if/when persisted externally.
   Write them to `working/causal_graph.json` (augmented) **and/or** a new `working/nt_cortex_delta.jsonl` append-only log.
3. **Idempotent + resumable:** mirror `copy_resumable`'s `.prog` + readback-verify pattern (`nt_memory_resource_ingest.rs:551-611`) so a dropped volume never corrupts the external archive.
4. **Version gate:** refuse out-bound sync unless `external_snapshot_ts`/`sha256` matches what was last loaded (prevents writing onto a swapped/older external brain — closes G2).

### 3.3 Incremental rollout (low-risk, Dark-Forest-safe)

- **Phase 0 (no risk):** Add `lineage` fields to `upsert_cortex_node` + `register_cortex_brain` outputs (read-only metadata; today's registration still works). Land `external_sha256`/`ts` capture in `from_cortex_json` (`causal_graph.rs:157`).
- **Phase 1:** New read-only `/cortex lineage` subcommand reporting `last_synced_at` / `last_seal_cycle` / `external_sha256` for every `cortex_brain` node (observability only).
- **Phase 2:** Implement `nt_memory_cortex_sync::export_delta` (out-bound) gated behind a `cortex_sync` feature flag, writing `working/nt_cortex_delta.jsonl`. Dry-run first (`/cortex sync --dry-run`), matching the existing `--force` convention (`cortex_cmds.rs:40,143`).
- **Phase 3:** Wire export into `close_iteration_loop` (`seal_loop.rs:641`) as an optional post-accept hook (only when volume mounted + version matches) — makes SEAL absorption *flow back* to the external brain.
- **Phase 4:** Bidirectional merge — on mount, `register_cortex_brain` consumes `nt_cortex_delta.jsonl` back into the live KB (closing the loop; today only the static `causal_graph.json` is consumed).

### 3.4 Why this matters

Today the external brain is a **passive cold mirror** (one corpus copy + a read-only causal graph). Closing G1–G3 turns it into a **durable evolution partner**: SEAL cycles leave an auditable trail on the volume, the live KB can be reconstructed from it, and the 114 GB archive becomes the *source of truth across sessions* rather than dead weight — satisfying Dark Forest (connect, don't bloat) and the project's pointer-conservation rule (lineage lives in KB, not AGENTS.md).

---

## 4. Evidence index (file:line)

| Claim | Citation |
|---|---|
| `CORTEX_ROOT` constant | `cortex_cmds.rs:17`, `nt_memory_resource_ingest.rs:397` |
| `CORTEX_CAUSAL` path | `cortex_cmds.rs:18` |
| live KB path `~/.neotrix/knowledge.db` | `cortex_cmds.rs:20-23`, `nt_core_state.rs:33`, `nt_core_consciousness_core.rs:443-449` |
| `register_cortex_brain` | `nt_memory_resource_ingest.rs:426-468` |
| `upsert_cortex_node` (metadata-driven `kind`) | `nt_memory_resource_ingest.rs:401-418` |
| T3 wiring (startup call) | `nt_memory_resource_ingest.rs:389-391` |
| `/cortex status` count | `cortex_cmds.rs:89-97` |
| E8 loads causal graph | `e8_abduction_bridge.rs:31-36` |
| `load_cortex_causal_graph` (index remap) | `abduction/mod.rs:66-81` |
| `from_cortex_json` schema + confidence 0.8 | `abduction/causal_graph.rs:157-193` |
| `/cortex causal` | `cortex_cmds.rs:196-216` |
| `migrate_cortex_corpus` (one-way cold copy) | `nt_memory_resource_ingest.rs:504-545` |
| `corpus_archive_path` (superset, no merge) | `nt_memory_resource_ingest.rs:472-486`, `447-449` |
| `copy_resumable` (`.prog` checkpoint) | `nt_memory_resource_ingest.rs:551-611` |
| SEAL loop + accept/rollback | `seal_loop.rs:280`, `641-733` |
| experience → `kv_store` `experience` ns | `experience.rs:6`, `:230` |
| internal `brain_write_back` (not external) | `nt_io_user_avatar.rs:458`, `brain_cmds.rs:93` |
| `prune_cortex_orphans` (one-directional) | `nt_memory_resource_ingest.rs:675-738` |
| `/Volumes/NeoTrixBrain` only read/copied-out | grep: 10 matches, all read/migrate (§2 G1) |
