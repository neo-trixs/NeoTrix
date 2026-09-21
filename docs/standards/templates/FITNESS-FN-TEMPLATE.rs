//! TEMPLATE — do NOT compile standalone.
//! Paste into `neotrix-core/src/l5_cognition/nt_core_arch_fitness.rs`,
//! then register in `arch_fitness_tests()` and add a unit test in `mod tests`.
//! Rule: NTS-B05 (taxonomy header), NTS-B06 (allowlist MUST expire), R-P246.

use crate::l0_substrate::nt_core_self_test::SelfTest;

// ─────────────────────────────────────────────────────────────
// <Name>Fitness — one-line responsibility
// Taxonomy: scope=<atomic|holistic> cadence=<triggered|continual|temporal>
//           result=<static|dynamic> invocation=<automated>
// Allowlist: <none | path-prefixes with EXPIRY date, e.g. "EXPIRES 2026-12-31">
// ─────────────────────────────────────────────────────────────

/// What architectural characteristic this guard protects, and why.
/// Current baseline: <numbers from first run, e.g. "0 violations on 2026-09-21">.
pub struct NewGuardFitness;

impl SelfTest for NewGuardFitness {
    fn name(&self) -> &str {
        "arch_fitness_new_guard"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        // 1. Scan read-only (never modify code here).
        // 2. Collect violations as `path:line | detail` strings.
        // 3. Empty => Ok(()); else Err(vec![summary, violations...]) capped at 20 lines.
        let violations: Vec<String> = Vec::new();
        if violations.is_empty() {
            Ok(())
        } else {
            let mut msg = vec![format!(
                "violation count {} (<what must hold>)",
                violations.len()
            )];
            msg.extend(violations.iter().take(20).cloned());
            Err(msg)
        }
    }
}
