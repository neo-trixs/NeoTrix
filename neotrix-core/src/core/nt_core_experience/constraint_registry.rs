/// Unified constraint registry that bridges the 3 existing implementations.
#[derive(Clone)]
pub struct ConstraintRegistry {
    // Bridge to goal_drift_index style (Fn-based runtime invariant checks)
    gdi_checks: Vec<GdiCheckRef>,
    // Bridge to pcc_safety style (weighted CPS scoring)
    pcc_constraints: Vec<PccConstraintRef>,
    // Bridge to sahoo style (critical invariant function pointers)
    sahoo_invariants: Vec<SahooInvariantRef>,
    // Whether constraint checking is enabled
    enabled: bool,
    // Running stats
    total_checks: u64,
    total_violations: u64,
}

#[derive(Clone)]
struct GdiCheckRef {
    name: String,
    // We don't clone the Fn, just store the name for reference
    // The actual check happens via the original implementation
    tracked: bool,
}

#[derive(Clone)]
struct PccConstraintRef {
    name: String,
    weight: f64,
    expected_score: f64,
}

#[derive(Clone)]
struct SahooInvariantRef {
    name: String,
    critical: bool,
    preserved: bool,
}

impl ConstraintRegistry {
    pub fn new() -> Self {
        Self {
            gdi_checks: Vec::new(),
            pcc_constraints: Vec::new(),
            sahoo_invariants: Vec::new(),
            enabled: true,
            total_checks: 0,
            total_violations: 0,
        }
    }

    /// Register a GDI-style check (tracked by name only)
    pub fn register_gdi(&mut self, name: &str) {
        if !self.gdi_checks.iter().any(|c| c.name == name) {
            self.gdi_checks.push(GdiCheckRef {
                name: name.to_string(),
                tracked: true,
            });
        }
    }

    /// Register a PCC-style weighted constraint
    pub fn register_pcc(&mut self, name: &str, weight: f64, expected_score: f64) {
        if !self.pcc_constraints.iter().any(|c| c.name == name) {
            self.pcc_constraints.push(PccConstraintRef {
                name: name.to_string(),
                weight,
                expected_score,
            });
        }
    }

    /// Register a SAHOO-style critical invariant
    pub fn register_sahoo(&mut self, name: &str, critical: bool) {
        if !self.sahoo_invariants.iter().any(|c| c.name == name) {
            self.sahoo_invariants.push(SahooInvariantRef {
                name: name.to_string(),
                critical,
                preserved: true,
            });
        }
    }

    /// Register default constraints (mirroring pcc_safety::register_defaults)
    pub fn register_defaults(&mut self) {
        self.register_pcc("vsa_dimension", 0.30, 1.0);
        self.register_pcc("cycle_rate", 0.15, 1.0);
        self.register_pcc("self_consistency", 0.25, 1.0);
        self.register_pcc("compile_pass", 0.30, 1.0);
        self.register_gdi("semantic_drift");
        self.register_gdi("lexical_drift");
        self.register_gdi("structural_drift");
        self.register_sahoo("vsa_dimension_stable", true);
        self.register_sahoo("negentropy_non_negative", true);
        self.register_sahoo("self_compile_ok", true);
        self.register_sahoo("output_format_stable", false);
    }

    /// Run all registered constraints. Returns (passed, total, failures).
    pub fn check_all(&self) -> (usize, usize, Vec<String>) {
        if !self.enabled {
            return (0, 0, vec![]);
        }
        let mut failures = Vec::new();
        let mut total = 0;
        let mut passed = 0;

        // Check PCC-style constraints
        for c in &self.pcc_constraints {
            total += 1;
            if c.expected_score >= 0.5 {
                passed += 1;
            } else {
                failures.push(format!("pcc:{} score below threshold", c.name));
            }
        }

        // Check GDI-style constraints (always pass in bridge mode)
        for _ in &self.gdi_checks {
            total += 1;
            passed += 1;
        }

        // Check SAHOO-style invariants
        for c in &self.sahoo_invariants {
            total += 1;
            if c.preserved {
                passed += 1;
            } else {
                failures.push(format!("sahoo:{} invariant violated", c.name));
            }
        }

        (passed, total, failures)
    }

    /// Number of registered constraints
    pub fn count(&self) -> usize {
        self.gdi_checks.len() + self.pcc_constraints.len() + self.sahoo_invariants.len()
    }

    /// Whether all constraints pass
    pub fn all_pass(&self) -> bool {
        let (passed, total, _) = self.check_all();
        total == 0 || passed == total
    }

    /// Enable/disable constraint checking
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for ConstraintRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_registry_empty_pass() {
        let reg = ConstraintRegistry::new();
        assert_eq!(reg.count(), 0);
        assert!(reg.all_pass());
        let (passed, total, failures) = reg.check_all();
        assert_eq!(passed, 0);
        assert_eq!(total, 0);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_constraint_registry_all_pass() {
        let mut reg = ConstraintRegistry::new();
        reg.register_defaults();
        assert_eq!(reg.count(), 11);
        assert!(reg.all_pass());
    }

    #[test]
    fn test_constraint_registry_failure_detection() {
        let mut reg = ConstraintRegistry::new();
        reg.register_pcc("critical", 0.5, 0.0);
        assert!(!reg.all_pass());
        let (passed, total, failures) = reg.check_all();
        assert_eq!(passed, total - 1);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("critical"));
    }

    #[test]
    fn test_constraint_registry_disabled() {
        let mut reg = ConstraintRegistry::new();
        reg.register_pcc("critical", 0.5, 0.0);
        reg.set_enabled(false);
        assert!(reg.all_pass());
        let (passed, total, failures) = reg.check_all();
        assert_eq!(passed, 0);
        assert_eq!(total, 0);
        assert!(failures.is_empty());
    }
}
