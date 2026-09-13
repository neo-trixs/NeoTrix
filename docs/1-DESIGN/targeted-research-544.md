# Internal Dispatch Routes — 4-Module Expansion (544)

## Summary

Expanded `dispatch_internal_capability` in `nt_core_consciousness_core.rs` with 16 new CAPABILITY_ROUTES entries and 8 new match arms covering 4 previously-unwired priority modules.

## Changes

**File:** `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`

### CAPABILITY_ROUTES (16 new entries)

| Keyword | capability_tag | Domain | Specialist | Module |
|---------|---------------|--------|------------|--------|
| 进化 | `seal_iterate` | NT-MIND | KnowledgeIntegrator | nt_core_seal |
| 迭代 | `seal_iterate` | NT-MIND | KnowledgeIntegrator | nt_core_seal |
| 蒸馏 | `seal_distill` | NT-MIND | KnowledgeIntegrator | nt_core_seal |
| 自我评估 | `self_model_tick` | NT-CORE | ReflectionEngine | nt_core_self |
| 能力评估 | `self_model_tick` | NT-CORE | ReflectionEngine | nt_core_self |
| 认知健康 | `metacog_evaluate` | NT-CORE | ReflectionEngine | nt_core_self |
| 元观察 | `meta_observe` | NT-META | MetaCognitionAnalyst | nt_meta |
| 质量扫描 | `sentrux_scan` | NT-META | MetaCognitionAnalyst | nt_meta |
| 代码质量 | `sentrux_scan` | NT-META | MetaCognitionAnalyst | nt_meta |
| 构建健康 | `build_watchdog` | NT-META | MetaCognitionAnalyst | nt_meta |
| 构建检查 | `build_watchdog` | NT-META | MetaCognitionAnalyst | nt_meta |
| 安全审计 | `shield_audit` | NT-SHIELD | RiskAssessor | nt_shield |
| 攻击检测 | `shield_audit` | NT-SHIELD | RiskAssessor | nt_shield |
| 漏洞扫描 | `agentic_scan` | NT-SHIELD | RiskAssessor | nt_shield |
| 安全扫描 | `agentic_scan` | NT-SHIELD | RiskAssessor | nt_shield |

### Dispatch Arms (8 new match arms)

| capability_tag | Function Called | Source Module |
|----------------|---------------|---------------|
| `seal_iterate` | `EvolutionLoop::new().run_cycle(None, None)` | `l5_cognition::nt_mind::evolution::evolution_loop` |
| `seal_distill` | `EvolutionLoop::new().run_cycle(None, None)` (reads `new_patterns`/`suggestions`) | same |
| `self_model_tick` | `SelfModel::new().tick(workspace_signal, load_delta, meta_alarm)` | `core::nt_core_self::self_model` |
| `metacog_evaluate` | `CognitiveEvaluator::new().evaluate(&SiliconSelfModel::default())` | `core::nt_core_self::metacognitive_evaluator` |
| `meta_observe` | `MetaObserver::new(config).observe(&snapshot)` | `l6_meta::memory::meta_observer` |
| `sentrux_scan` | `SentruxSensor::new().scan(&path)` | `l6_meta::coordination::nt_meta_sentrux` |
| `build_watchdog` | `BuildWatchdog::new(config).check_health()` | `l6_meta::coordination::nt_meta_build_watchdog` |
| `shield_audit` | `global_shield().lock().security_audit(&input)` | `cli::shield_enforcer` |
| `agentic_scan` | `AgenticScanner::new(config).recon_scan(&target)` | `l3_embodiment::nt_shield::nt_shield_agentic_scan` |

### Design Decisions

1. **SEAL pipeline via EvolutionLoop** — The real `SelfIteratingBrain` requires complex cortex initialization. `EvolutionLoop` provides a synchronous, no-dependency entry point that scans the project, finds issues, and suggests fixes. For the `seal_distill` route, the same cycle is run but output focuses on `new_patterns`/`suggestions` (distillation artifacts).

2. **Self model feeds from consciousness snapshot** — `self_model_tick` reads `coherence`/`weighted_fog_sum`/`mars_system1_activations` from the current consciousness core snapshot, creating a feedback loop where the self model is calibrated against the system's own perception.

3. **Meta observer uses live snapshot** — `meta_observe` calls `status()` to get the latest consciousness snapshot, then runs the meta observer on it. This gives a meta-level quality check of the system's own observation.

4. **Shield uses global singleton** — `shield_audit` acquires the global `Mutex<ShieldEnforcer>` via `global_shield()`, ensuring the same enforcer state is shared across all callers.

5. **Sentrux scan defaults to cwd** — When no path is provided in the task summary, the scan targets the current working directory (project root).

## Module Coverage

| Module | Routes Added | Keywords |
|--------|-------------|----------|
| nt_core_seal | 3 | 进化, 迭代, 蒸馏 |
| nt_core_self | 3 | 自我评估, 能力评估, 认知健康 |
| nt_meta | 5 | 元观察, 质量扫描, 代码质量, 构建健康, 构建检查 |
| nt_shield | 4 | 安全审计, 攻击检测, 漏洞扫描, 安全扫描 |
| **Total new** | **15** | |

**Total CAPABILITY_ROUTES:** 72 entries | **Total dispatch match arms:** 26 (including `_` catch-all)

## Import Path Fixes (this session)

3 broken import paths were corrected in the existing dispatch arms:
- `pdf_image_stats`: `crate::neotrix::pdf_image_stats` → `crate::neotrix::nt_file_ability::pdf::pdf_image_extract::pdf_image_stats`
- `PdfImageExtractConfig`/`extract_pdf_images`: same module path correction
- `SiliconSelfModel`: `silicon_self_model` → `silicon_self`
