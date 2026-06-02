# NeoTrix 命名架构迁移映射

## Domain 1 — NT-CORE (`core/` → 文件前缀 `nt_core_`)

| 原路径 | 新路径 |
|--------|--------|
| `core/e8.rs` | `core/nt_core_e8.rs` |
| `core/e8_reasoning.rs` | `core/nt_core_hex.rs` |
| `core/e8_experiment.rs` | `core/nt_core_policy.rs` |
| `core/e8_observer.rs` | `core/nt_core_observer.rs` |
| `core/hypercube/` | `core/nt_core_hcube/` |
| `core/consciousness/` | `core/nt_core_gwt/` |
| `core/capability.rs` | `core/nt_core_cap.rs` |
| `core/memory/` | `core/nt_core_bank/` |
| `core/signal.rs` | `core/nt_core_ssm.rs` |
| `core/metacognition/` | `core/nt_core_meta/` |
| `core/thinking_model/` | `core/nt_core_self/` |
| `core/hypergraph.rs` | `core/nt_core_graph.rs` |
| `core/vl_jepa.rs` | `core/nt_core_jepa.rs` |
| `core/contrastive_abstraction.rs` | `core/nt_core_abstr.rs` |
| `core/cdwm.rs` | `core/nt_core_cdwm.rs` |
| `core/absorb.rs` | `core/nt_core_absorb.rs` |
| `core/accessor.rs` | `core/nt_core_accessor.rs` |
| `core/iteration.rs` | `core/nt_core_iter.rs` |
| `core/iteration_agent.rs` | `core/nt_core_iter_agent.rs` |
| `core/traits.rs` | `core/nt_core_traits.rs` |
| `core/embedding.rs` | `core/nt_core_embed.rs` |
| `core/walsh_memory.rs` | `core/nt_core_walsh.rs` |
| `core/crt_time.rs` | `core/nt_core_crt.rs` |
| `core/kronecker_cleanup.rs` | `core/nt_core_kron.rs` |
| `core/workspace.rs` | `core/nt_core_ws.rs` |
| `core/smart_router.rs` | `core/nt_core_router.rs` |
| `core/whitebox_memory.rs` | `core/nt_core_wbmem.rs` |
| `core/sigreg.rs` | `core/nt_core_sigreg.rs` |
| `core/td_flows.rs` | `core/nt_core_td.rs` |
| `core/connectors.rs` | `core/nt_core_conn.rs` |
| `core/architect_agent/` | `core/nt_core_arch/` |
| `core/epoch/` | `core/nt_core_epoch/` |
| `core/mcp_server.rs` | `core/nt_core_mcp.rs` |
| `core/rkyv_store.rs` | `core/nt_core_rkyv.rs` |
| `core/event.rs` | `core/nt_core_event.rs` |
| `core/latent_predictor.rs` | `core/nt_core_pred.rs` |
| `core/sensory/` | `core/nt_core_sense/` |
| `core/awareness/` | `core/nt_core_aware/` |
| `core/core_interface/` | `core/nt_core_iface/` |
| `core/sigreg.rs` | - |
| `core/mod.rs` | `core/nt_core.rs` |

## Domain 2 — NT-MIND (`neotrix/` → `nt_mind/`)

| 原路径 | 新路径 |
|--------|--------|
| `reasoning_brain/` | `nt_mind/` |
| `reasoning_brain/self_iterating/` | `nt_mind/seal/` |
| `reasoning_brain/brain_core.rs` | `nt_mind/nt_mind_brain.rs` |
| `reasoning_brain/brain_seal.rs` | `nt_mind/nt_mind_strat.rs` |
| `reasoning_brain/brain_absorb.rs` | `nt_mind/nt_mind_absorb.rs` |
| `reasoning_brain/brain_ewc.rs` | `nt_mind/nt_mind_ewc.rs` |
| `reasoning_brain/brain_dgm.rs` | `nt_mind/nt_mind_dgm.rs` |
| `reasoning_brain/brain_impl.rs` | `nt_mind/nt_mind_impl.rs` |
| `reasoning_brain/pipeline.rs` | `nt_mind/seal/nt_mind_pipeline.rs` |
| `reasoning_brain/validation.rs` | `nt_mind/seal/nt_mind_valid.rs` |
| `reasoning_brain/skillopt.rs` | `nt_mind/seal/nt_mind_skill.rs` |
| `reasoning_brain/harness_adapter.rs` | `nt_mind/seal/nt_mind_adapt.rs` |
| `reasoning_brain/aging_monitor.rs` | `nt_mind/seal/nt_mind_age.rs` |
| `reasoning_brain/secret_scanner.rs` | `nt_mind/seal/nt_mind_scan.rs` |
| `reasoning_brain/sia_loop.rs` | `nt_mind/seal/nt_mind_sia.rs` |
| `reasoning_brain/hypercore.rs` | `nt_mind/seal/nt_mind_hmeta.rs` |
| `reasoning_brain/hyperarchive.rs` | `nt_mind/seal/nt_mind_archive.rs` |
| `reasoning_brain/hyperdgm.rs` | `nt_mind/seal/nt_mind_hyperdgm.rs` |
| `reasoning_brain/hyperstage.rs` | `nt_mind/seal/nt_mind_hstage.rs` |
| `reasoning_brain/hyperagents.rs` | `nt_mind/seal/nt_mind_hagent.rs` |
| `reasoning_brain/skill_crystallizer.rs` | `nt_mind/seal/nt_mind_crystal.rs` |
| `reasoning_brain/persist_impl.rs` | `nt_mind/nt_mind_persist.rs` |
| `reasoning_brain/loop_impl/` | `nt_mind/nt_mind_loop/` |
| `reasoning_brain/self_edit/` | `nt_mind/edit/` |
| `reasoning_brain/reasoning_engine/` | `nt_mind/engine/` |
| `reasoning_brain/reasoning_types.rs` | `nt_mind/nt_mind_types.rs` |
| `reasoning_brain/stats.rs` | `nt_mind/nt_mind_stats.rs` |
| `reasoning_brain/model_router.rs` | `nt_mind/nt_mind_router.rs` |
| `reasoning_brain/tier_prompts.rs` | `nt_mind/nt_mind_prompts.rs` |
| `reasoning_brain/lora.rs` | `nt_mind/nt_mind_lora.rs` |
| `reasoning_brain/multi_brain.rs` | `nt_mind/nt_mind_multi.rs` |
| `reasoning_brain/bm25.rs` | `nt_mind/nt_mind_bm25.rs` |
| `reasoning_brain/embedding.rs` | `nt_mind/nt_mind_embed.rs` |
| `reasoning_brain/sleep/` | `nt_mind/nt_mind_sleep/` |
| `reasoning_brain/core/` | `nt_mind/core/` |

Wait, I'm going too deep into the weeds. Let me simplify this - the actual file names in `reasoning_brain/` are different from what the exploration agent listed. Let me check the actual directory structure.

Actually, I already have a thorough listing from the exploration. The neotrix directory under `neotrix-core/src/neotrix/` has:
- `reasoning_brain/` with all those submodules
- `knowledge_base/`
- `crypto_agent/`
- etc.

This is getting really complex. Let me simplify my approach - I'll just start executing with bash commands and fix things as I go. The key insight is that for each rename, I need to:
1. git mv the file
2. Update mod.rs
3. Update all references across the codebase

Let me start with the most critical and self-contained renames first.

Actually, let me take yet another approach. Rather than renaming every single file in the codebase (which would be 100+ file renames), let me focus on the most impactful changes:

1. Rename the key directories that are most visible
2. Rename the key files that are most referenced
3. Update the mod.rs files
4. Use `use` aliases to ease the transition

For a first pass, let me:
1. Keep `core/` directory name as-is (rename internal files with nt_core_ prefix)
2. Keep module names within mod.rs updated
3. Use path attributes where needed

Actually wait - I think the cleanest approach is:

For the top-level entries in `neotrix-core/src/core/mod.rs`, instead of renaming all 42 files physically, I can:
1. Create nt_core_e8.rs as a new file that re-exports everything from the old file
2. Or just rename the files using git mv

Let me just go ahead and execute the git mvs and fix things up. I'll start with the core/ directory since that's the foundation.

Let me write a comprehensive bash script that handles the core directory.<｜end▁of▁thinking｜>Let me start executing systematically. Phase 0 is done (branch created, cargo check passes). Now let me tackle the file renaming domain by domain.

<｜｜DSML｜｜tool_calls>
<｜｜DSML｜｜invoke name="bash">
<｜｜DSML｜｜parameter name="command" string="true"># Phase 1: NT-CORE — Rename files in core/
# Execute all git mv operations for core/ directory
cd neotrix-core/src/core

# Single files
git mv e8.rs nt_core_e8.rs
git mv e8_reasoning.rs nt_core_hex.rs
git mv e8_experiment.rs nt_core_policy.rs
git mv e8_observer.rs nt_core_observer.rs
git mv capability.rs nt_core_cap.rs
git mv signal.rs nt_core_ssm.rs
git mv hypergraph.rs nt_core_graph.rs
git mv vl_jepa.rs nt_core_jepa.rs
git mv contrastive_abstraction.rs nt_core_abstr.rs
git mv cdwm.rs nt_core_cdwm.rs
git mv absorb.rs nt_core_absorb.rs
git mv accessor.rs nt_core_accessor.rs
git mv iteration.rs nt_core_iter.rs
git mv iteration_agent.rs nt_core_iter_agent.rs
git mv traits.rs nt_core_traits.rs
git mv embedding.rs nt_core_embed.rs
git mv walsh_memory.rs nt_core_walsh.rs
git mv crt_time.rs nt_core_crt.rs
git mv kronecker_cleanup.rs nt_core_kron.rs
git mv workspace.rs nt_core_ws.rs
git mv smart_router.rs nt_core_router.rs
git mv whitebox_memory.rs nt_core_wbmem.rs
git mv sigreg.rs nt_core_sigreg.rs
git mv td_flows.rs nt_core_td.rs
git mv connectors.rs nt_core_conn.rs
git mv mcp_server.rs nt_core_mcp.rs
git mv rkyv_store.rs nt_core_rkyv.rs
git mv event.rs nt_core_event.rs
git mv latent_predictor.rs nt_core_pred.rs

# Directories (need to rename internal mod.rs too)
git mv consciousness nt_core_gwt
git mv hypercube nt_core_hcube
git mv memory nt_core_bank
git mv metacognition nt_core_meta
git mv thinking_model nt_core_self
git mv sensory nt_core_sense
git mv awareness nt_core_aware
git mv architect_agent nt_core_arch
git mv epoch nt_core_epoch
git mv core_interface nt_core_iface

echo "Core directory renames complete"
ls -la