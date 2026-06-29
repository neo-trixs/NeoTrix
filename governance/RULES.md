# Operational Rules
> Rules that NeoTrix follows and can self-modify.

## Rule 1: Session Log Archiving
**Trigger**: AGENTS.md exceeds 500 lines
**Action**: Extract session logs to sessions/ directory
**Authority**: Autonomous (no approval needed)

## Rule 2: Dead Code Annotation
**Trigger**: Dispatch arm found with zero callers for 30+ days
**Action**: Append `// DEAD` comment
**Authority**: Autonomous (no approval needed)

## Rule 3: #[serial] Audit
**Trigger**: New OnceLock<Mutex<>> singleton added
**Action**: Verify #[serial] is present in test module
**Authority**: Review required (must pass PR)

## Rule 4: Decision Log Entry
**Trigger**: Any architecture change affecting >2 files
**Action**: Add D-NNN entry to DECISION_LOG.md
**Authority**: Autonomous (approval not required)

## Rule 5: Spec Drift Detection
**Trigger**: Subsystem API changes without corresponding spec update
**Action**: Flag drift in spec_status, add `// OUTDATED` annotation to spec YAML header
**Authority**: Autonomous (warning only, no block)

## Rule 6: Governance Self-Review
**Trigger**: 30 days since last RULES.md modification
**Action**: Scan governance/ for outdated rules, propose amendments
**Authority**: Review required (must pass PR)
