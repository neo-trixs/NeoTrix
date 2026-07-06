#!/usr/bin/env bash
# enforce-neotrix-layer-deps.sh
#
# Enforces the NeoTrix 9-Layer Architecture dependency rules:
#   Upper layers can import from lower layers.
#   Lower layers must NOT import from upper layers.
#
# Primary check (strict): files in core/l{N}_*/ directories
#   → all use crate::core::l{M}_* imports must have M <= N
#
# Secondary check (lenient): files elsewhere with prefix-based layer mapping
#   → warn if a module imports from a higher layer
#
# Flat core/nt_core_* files are skipped (in transition).
# Infrastructure modules (no layer assigned) are skipped in the check.

set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRC_DIR="$PROJECT_ROOT/neotrix-core/src"

if [ ! -d "$SRC_DIR" ]; then
  echo "ERROR: source directory not found at $SRC_DIR" >&2
  exit 1
fi

if ! command -v rg &>/dev/null; then
  echo "ERROR: ripgrep (rg) is required but not found in PATH." >&2
  echo "       Install it with: brew install ripgrep  (macOS)" >&2
  echo "       or: cargo install ripgrep" >&2
  exit 1
fi

# ─── Helper: emulate two independent associative arrays ──────────────────────

_make_map() {
  local __name="$1"
  eval "${__name}_keys=()"
  eval "${__name}_vals=()"
}

_put() {
  local __map="$1" __key="$2" __val="$3"
  eval "${__map}_keys[\${#${__map}_keys[@]}]=\"\$__key\""
  eval "${__map}_vals[\${#${__map}_vals[@]}]=\"\$__val\""
}

_get() {
  local __map="$1" __key="$2"
  eval 'local __n="${#'"${__map}"'_keys[@]}"'
  local __i
  for (( __i=0; __i < __n; __i++ )); do
    eval 'local __k="${'"${__map}_keys[$__i]"'}"'
    if [ "$__k" = "$__key" ]; then
      eval 'echo "${'"${__map}_vals[$__i]"'}"'
      return 0
    fi
  done
  return 1
}

# First-match prefix lookup (for import mapping)
_get_prefix() {
  local __map="$1" __name="$2"
  eval 'local __n="${#'"${__map}"'_keys[@]}"'
  local __i
  for (( __i=0; __i < __n; __i++ )); do
    eval 'local __k="${'"${__map}_keys[$__i]"'}"'
    case "$__name" in
      "${__k}"*) 
        eval 'echo "${'"${__map}_vals[$__i]"'}"'
        return 0
        ;;
    esac
  done
  return 1
}

# ─── Directory → layer mapping (core/l{N}_*/) ───────────────────────────────

_make_map dir

_put dir l0_substrate 0
_put dir l1_body 1
_put dir l2_perception 2
_put dir l3_memory 3
_put dir l4_cognition 4
_put dir l5_consciousness 5
_put dir l6_self 6
_put dir l7_capability 7
_put dir l8_autonomic 8
_put dir l9_transcendent 9

_put dir l1_body_impl 1
_put dir l2_world_impl 2
_put dir l3_memory_impl 3
_put dir l8_autonomic_impl 8
_put dir l9_transcendent_impl 9

# ─── Module prefix → layer mapping (longest prefix FIRST) ───────────────────

_make_map pfx

# Layer 9 — Transcendent
_put pfx nt_core_knowledge_gap 9
_put pfx nt_core_observer_error 9
_put pfx nt_core_observer 9
_put pfx nt_core_meta 9
_put pfx nt_mind_consciousness_gold_standard 9
_put pfx nt_mind_consciousness_monitor 9

# Layer 8 — Autonomic
_put pfx nt_mind_background_loop 8
_put pfx nt_mind_background_config 8
_put pfx nt_mind_autofixer 8
_put pfx nt_mind_evolution_loop 8
_put pfx nt_mind_evolution_daemon 8
_put pfx nt_mind_self_diagnose 8
_put pfx nt_mind_benchmark 8
_put pfx nt_mind_cleanup 8
_put pfx nt_mind_distiller 8
_put pfx nt_mind_scheduler 8
_put pfx nt_mind_topic_aggregator 8
_put pfx nt_mind_awakening 8
_put pfx nt_mind_ingestion 8
_put pfx nt_mind_absorb 8
_put pfx nt_mind 8

# Layer 7 — Capability
_put pfx nt_cap_ 7
_put pfx l7_capability 7

# Layer 6 — Self
_put pfx nt_core_self 6
_put pfx nt_core_intra_reflection 6

# Layer 5 — Consciousness
_put pfx nt_core_gwt 5
_put pfx nt_core_reson 5
_put pfx nt_core_iit_phi 5
_put pfx nt_core_fep_iit 5
_put pfx nt_core_signal 5
_put pfx nt_core_event 5
_put pfx nt_core_event_bus 5
_put pfx nt_core_consciousness 5

# Layer 4 — Cognition
_put pfx nt_core_e8_vsa 4
_put pfx nt_core_e8 4
_put pfx nt_core_hex 4
_put pfx nt_core_policy 4
_put pfx nt_core_sae_bridge 4
_put pfx nt_core_sae 4
_put pfx nt_core_prm 4
_put pfx nt_core_kernel 4
_put pfx nt_core_code_query 4
_put pfx nt_core_parallel 4
_put pfx nt_core_abstr 4
_put pfx nt_core_cdwm 4
_put pfx nt_core_graph 4
_put pfx nt_core_crt 4
_put pfx nt_core_sigreg 4
_put pfx nt_core_td 4
_put pfx nt_core_fep 4
_put pfx nt_core_walsh 4
_put pfx nt_core_kron 4
_put pfx nt_core_hot_ast 4
_put pfx nt_core_source_edit 4
_put pfx nt_core_mcp 4
_put pfx nt_core_context 4
_put pfx nt_core_router 4
_put pfx nt_core_cap 4
_put pfx nt_core_aura 4
_put pfx nt_core_aware 4

# Layer 3 — Memory
_put pfx nt_memory_kb 3
_put pfx nt_memory_knowledge_populator 3
_put pfx nt_memory 3
_put pfx nt_core_hcube 3
_put pfx nt_core_bank 3
_put pfx nt_core_ssm 3
_put pfx nt_core_negentropy 3
_put pfx nt_core_vector_store 3
_put pfx nt_core_experience 3
_put pfx nt_core_epoch 3

# Layer 2 — Perception
_put pfx nt_world_model_v2 2
_put pfx nt_world_model 2
_put pfx nt_world_jepa 2
_put pfx nt_world_e8 2
_put pfx nt_world_pred_hcube 2
_put pfx nt_world_pred 2
_put pfx nt_world_infer 2
_put pfx nt_world_browse_auto 2
_put pfx nt_world_browse 2
_put pfx nt_world_scrape 2
_put pfx nt_world_sense 2
_put pfx nt_world_crawl 2
_put pfx nt_world_search 2
_put pfx nt_world_vision 2
_put pfx nt_world_journal_index 2
_put pfx nt_world_pet 2
_put pfx nt_world_code_search 2
_put pfx nt_world 2
_put pfx nt_core_jepa 2
_put pfx nt_core_sense 2

# Layer 1 — Body
_put pfx nt_io_logging 1
_put pfx nt_io_http_factory 1
_put pfx nt_io_mention 1
_put pfx nt_io_neotrix_interface 1
_put pfx nt_io_push_channel 1
_put pfx nt_io_standalone 1
_put pfx nt_io_telemetry 1
_put pfx nt_io_user_avatar 1
_put pfx nt_io_avatar_channel 1
_put pfx nt_io_lsp 1
_put pfx nt_io_hotreload 1
_put pfx nt_io_notify 1
_put pfx nt_io_server 1
_put pfx nt_io_remote 1
_put pfx nt_io_web 1
_put pfx nt_io_proxy_server 1
_put pfx nt_io_proxy 1
_put pfx nt_io_plugin 1
_put pfx nt_io_provider 1
_put pfx nt_io_constrained 1
_put pfx nt_io 1
_put pfx nt_shield_audit 1
_put pfx nt_shield_sentry 1
_put pfx nt_shield_sandbox_entry 1
_put pfx nt_shield_sandbox 1
_put pfx nt_shield_prompt 1
_put pfx nt_shield_manager 1
_put pfx nt_shield_stealth_net 1
_put pfx nt_shield 1
_put pfx nt_act_code 1
_put pfx nt_act_goal 1
_put pfx nt_act_gram 1
_put pfx nt_act_spear 1
_put pfx nt_act_autonomy 1
_put pfx nt_act_voice 1
_put pfx nt_act_crypto 1
_put pfx nt_act_earn 1
_put pfx nt_act_social 1
_put pfx nt_act_sync 1
_put pfx nt_act_sub_agent_middleware 1
_put pfx nt_act_orchestrator 1
_put pfx nt_act_project_manager 1
_put pfx nt_act_remote_control 1
_put pfx nt_act 1
_put pfx nt_agent_protocol 1
_put pfx nt_agent_subagent 1
_put pfx nt_agent_mcp_discovery 1
_put pfx nt_agent_mcp_tools 1
_put pfx nt_agent_mcp_adapter 1
_put pfx nt_agent_mcp_auth 1
_put pfx nt_agent_mcp_transport 1
_put pfx nt_agent_mod 1
_put pfx nt_agent_orchestrator 1
_put pfx nt_agent 1
_put pfx nt_tools 1

# Layer 0 — Substrate
_put pfx nt_core_deploy_cache 0
_put pfx nt_core_deploy 0

# ─── Helpers ──────────────────────────────────────────────────────────────────

# Determine a file's layer from its path.
path_to_layer() {
  fpath="$1"

  # 1) Direct parent directory name (core/l{N}_*/ or neotrix/l{N}_impl/)
  parent="$(basename "$(dirname "$fpath")" 2>/dev/null)"
  result="$(_get dir "$parent")" && { echo "$result"; return 0; }

  # 2) Grandparent directory name (for files 2 levels deep inside layer dirs)
  grandparent="$(basename "$(dirname "$(dirname "$fpath")")" 2>/dev/null)"
  result="$(_get dir "$grandparent")" && { echo "$result"; return 0; }

  # 3) For mod.rs, try parent directory as prefix
  filename="$(basename "$fpath" .rs)"
  if [ "$filename" = "mod" ]; then
    result="$(_get_prefix pfx "$parent")" && { echo "$result"; return 0; }
  fi

  # 4) Prefix-based mapping on the filename
  result="$(_get_prefix pfx "$filename")" && { echo "$result"; return 0; }

  return 1
}

# Determine layer from a crate import path segment.
import_to_layer() {
  ipath="$1"
  rel="${ipath#crate::}"

  first="${rel%%::*}"
  rest="${rel#*::}"
  second="${rest%%::*}"

  # case: crate::core::l{N}_* or crate::neotrix::l{N}_impl
  result="$(_get dir "$second")" && { echo "$result"; return 0; }

  # case: crate::core::nt_* or crate::neotrix::nt_*
  if [ "$first" = "core" ] || [ "$first" = "neotrix" ]; then
    case "$second" in
      nt_*|l7_*)
        result="$(_get_prefix pfx "$second")" && { echo "$result"; return 0; }
        ;;
    esac
  fi

  return 1
}

# True if the file lives inside core/l{N}_*/ or neotrix/l{N}_impl/
is_in_layer_dir() {
  fpath="$1"
  parent="$(basename "$(dirname "$fpath")" 2>/dev/null)"
  _get dir "$parent" >/dev/null 2>&1
}

# True if file is a flat core/nt_core_* file (in transition)
is_flat_core_file() {
  fpath="$1"
  dir_name="$(basename "$(dirname "$fpath")" 2>/dev/null)"
  filename="$(basename "$fpath" .rs)"
  [ "$dir_name" = "core" ] && [ "${filename#nt_}" != "$filename" ]
}

# ─── Known in-migration exceptions ────────────────────────────────────────────
# Some modules are documented as "in migration" — source path doesn't match
# target layer yet. These exceptions track the architecture migration plan.

# Format: "src_layer:imported_module_prefix"
# Example: "3:crate::core::nt_core_consciousness::vsa_tag" means
#   L3 is allowed to import from vsa_tag (which lives in consciousness/L5
#   but is being migrated to L3)

KNOWN_CROSS_LAYER_IMPORTS=(
  # L3→L5: negentropy.rs uses GWT resonance for semantic entropy sensing
  "3:crate::core::nt_core_gwt::resonance::"
  # L3→L5: negentropy.rs uses consciousness stream for temporal coherence
  "3:crate::core::nt_core_consciousness::ConsciousnessStream"
  # L3→L5: negentropy.rs uses IIT phi calculator for phi sensor
  "3:crate::neotrix::nt_core_iit_phi::"
  # L1→L4: io_standalone + agent use kernel for reasoning
  "1:crate::neotrix::nt_core_kernel::"
  # L1→L2: io_standalone + agent use world_browse for web interaction
  "1:crate::neotrix::nt_world_browse::"
  # L1→L5: io_standalone uses signal vector type
  "1:crate::neotrix::nt_core_signal::"
  # L1→L8: io_proxy_server uses mind_benchmark for reporting
  "1:crate::neotrix::nt_mind_benchmark::"
  # L1→L3: io_proxy_server uses bank for reasoning memory
  "1:crate::core::nt_core_bank::"
  # L4→L9: L4 cognition imports L9 observer PRM (process reward model)
  "4:crate::core::nt_core_observer::"
  # L3→L5: l3_memory re-exports VSA tagging types from consciousness
  "3:crate::core::nt_core_consciousness::vsa_tag::"
  # L3→L5: l3_memory re-exports source hierarchy types from consciousness
  "3:crate::core::nt_core_consciousness::source_hierarchy::"
  # L3→L5: l3_memory re-exports authority types from consciousness
  "3:crate::core::nt_core_consciousness::authority::"
  # L3→L5: GWTQ needs SpecialistType for consciousness-aware query routing
  "3:crate::core::nt_core_gwt::module_def::SpecialistType"
  # L3→L4: GWTQ needs ReasoningHexagram for E8-mode query filtering
  "3:crate::core::nt_core_hex::ReasoningHexagram"
  # L4→L5: Kernel uses Vector type from signal for state representation
  "4:crate::neotrix::nt_core_signal::Vector"
  # L8→L9: background_loop observes L9 knowledge gap detector for autonomic triggering
  "8:crate::core::nt_core_meta::knowledge_gap_detector::"
  # L8→L9: background_loop uses consciousness gold standard from L9 for quality metric
  "8:crate::neotrix::nt_mind_consciousness_gold_standard::"
  # L8→L9: background_loop uses consciousness monitor from L9 for state tracking
  "8:crate::neotrix::nt_mind_consciousness_monitor::"
  # L8→L9: distillation uses L9 meta for self-model observations
  "8:crate::core::nt_core_meta::"
  # L8→L9: evolution_seed uses L9 planner for goal decomposition
  "8:crate::core::nt_core_meta::planner::"
  # L8→L9: evolution_seed uses L9 debt severity for self-model health
  "8:crate::core::nt_core_meta::self_model::DebtSeverity"
  # L8→L9: evolution_seed uses L9 weakness detection
  "8:crate::core::nt_core_meta::weakness::"
)

is_known_cross_layer_import() {
  local src_layer="$1" import_path="$2"
  local entry src_prefix
  for entry in "${KNOWN_CROSS_LAYER_IMPORTS[@]}"; do
    src_prefix="${entry%%:*}"
    [ "$src_prefix" != "$src_layer" ] && continue
    case "$import_path" in
      "${entry#*:}"*) return 0 ;;
    esac
  done
  return 1
}

# ─── Main ────────────────────────────────────────────────────────────────────

main() {
  scanned=0
  warnings=0
  strict_warnings=0
  flat_skipped=0

  echo "=== NeoTrix 9-Layer Dependency Check ==="
  echo ""

  while IFS= read -r -d '' rs_file; do
    rel_path="${rs_file#$PROJECT_ROOT/}"

    # Skip target/ dirs
    case "$rel_path" in */target/*) continue ;; esac

    # Skip flat core/nt_core_* files
    if is_flat_core_file "$rs_file"; then
      flat_skipped=$(( flat_skipped + 1 ))
      continue
    fi

    src_layer="$(path_to_layer "$rs_file")" || true
    [ -z "$src_layer" ] && continue

    [ "$src_layer" = "999" ] && continue

    scanned=$(( scanned + 1 ))

    in_strict=false
    is_in_layer_dir "$rs_file" && in_strict=true

    # rg -n on single file: "LINE_NUM:CONTENT"
    while IFS= read -r rg_line; do
      # Parse line number and content
      line_num="${rg_line%%:*}"
      line="${rg_line#*:}"

      # Only process lines with "use crate::"
      case "$line" in *"use crate::"*) ;; *) continue ;; esac

      # Normalize: remove "pub " and "use " prefixes
      stripped="$line"
      case "$stripped" in "pub "*) stripped="${stripped#pub }" ;; esac
      case "$stripped" in "use "*) stripped="${stripped#use }" ;; esac

      # Must start with "crate::"
      case "$stripped" in "crate::"*) ;; *) continue ;; esac

      # Extract clean import path (strip braces, 'as', semicolon)
      import_path="${stripped%%\{*}"
      import_path="${import_path%%as *}"
      import_path="${import_path%%;*}"
      import_path="$(printf '%s' "$import_path" | xargs)"

      tgt_layer="$(import_to_layer "$import_path")" || continue
      [ -z "$tgt_layer" ] && continue
      [ "$tgt_layer" = "999" ] && continue

      if [ "$tgt_layer" -gt "$src_layer" ] 2>/dev/null; then
        # Skip known in-migration exceptions
        if is_known_cross_layer_import "$src_layer" "$import_path"; then
          continue
        fi

        abs_rel="${rel_path#neotrix-core/src/}"

        if $in_strict; then
          strict_warnings=$(( strict_warnings + 1 ))
          printf "  [STRICT] %s:%s\n" "$abs_rel" "$line_num"
        else
          warnings=$(( warnings + 1 ))
          printf "  [WARN]   %s:%s\n" "$abs_rel" "$line_num"
        fi
        printf "           src=L%s  tgt=L%s  =>  %s\n" "$src_layer" "$tgt_layer" "$import_path"
      fi
    done < <(rg -nF 'use crate::' "$rs_file" 2>/dev/null || true)

  done < <(find "$SRC_DIR" -name '*.rs' -type f -print0)

  total_violations=$(( warnings + strict_warnings ))

  echo ""
  echo "=== Summary ==="
  printf "  Files scanned:                %d\n" "$scanned"
  printf "  Flat core/nt_core_* skipped:   %d\n" "$flat_skipped"
  printf "  Potential violations:         %d\n" "$total_violations"
  if [ "$strict_warnings" -gt 0 ]; then
    printf "    of which in strict mode:    %d\n" "$strict_warnings"
  fi
  if [ "$warnings" -gt 0 ]; then
    printf "    of which in lenient mode:    %d\n" "$warnings"
  fi
  echo ""
  echo "NOTE: For now this is informational only (exit 0)."
  echo "      Strict violations = files in core/l{N}_*/ directories."
  echo "      Warnings = files outside layer dirs (not yet migrated)."
  echo ""

  exit 0
}

main "$@"
