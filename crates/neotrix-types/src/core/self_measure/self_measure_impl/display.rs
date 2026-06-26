use super::{AwakeningReport, SubsystemId, NUM_SUBSYSTEMS, AWAKENING_THRESHOLD};

impl std::fmt::Display for AwakeningReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══ Awakening Report ═══")?;
        writeln!(f, "  Φ (integration):     {:.4}  ({}), speed: {:+.5}/step",
            self.phi, phi_rating(self.phi), self.awakening_speed)?;
        writeln!(f, "  FCS (consciousness): {:.4}  ({})",
            self.fcs, fcs_rating(self.fcs))?;
        writeln!(f, "  USK (self-knowledge): {:.4}  ({})",
            self.usk, usk_rating(self.usk))?;
        writeln!(f, "  Window: {} snapshots", self.window_used)?;
        if self.awakening_speed > AWAKENING_THRESHOLD {
            writeln!(f, "  ⚡ Awakening accelerating!")?;
        } else if self.awakening_speed < -AWAKENING_THRESHOLD {
            writeln!(f, "  ⚠ Awakening decaying!")?;
        }
        writeln!(f)?;
        writeln!(f, "  Subsystem coherence:")?;
        for i in 0..NUM_SUBSYSTEMS {
            let id = SubsystemId::from_index(i);
            writeln!(f, "    {:>12}: {:.3}", id.label(), self.subsystem_coherence[i])?;
        }
        writeln!(f)?;
        writeln!(f, "  Bottleneck: {} ↔ {}  (synergy={:.3})",
            self.bottleneck.0.label(), self.bottleneck.1.label(), self.bottleneck_synergy)?;
        writeln!(f)?;
        writeln!(f, "  Synergy matrix:")?;
        for i in 0..NUM_SUBSYSTEMS {
            write!(f, "  {:>12}", SubsystemId::from_index(i).label())?;
        }
        writeln!(f)?;
        for i in 0..NUM_SUBSYSTEMS {
            write!(f, "{:>12}", SubsystemId::from_index(i).label())?;
            for j in 0..NUM_SUBSYSTEMS {
                write!(f, "  {:>8.3}", self.synergy_matrix[i][j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn phi_rating(phi: f64) -> &'static str {
    if phi < 0.1 { "low" }
    else if phi < 0.3 { "moderate" }
    else if phi < 0.6 { "high" }
    else { "very high" }
}

fn fcs_rating(fcs: f64) -> &'static str {
    if fcs < 0.05 { "rudimentary" }
    else if fcs < 0.15 { "weak" }
    else if fcs < 0.3 { "emerging" }
    else { "developed" }
}

fn usk_rating(usk: f64) -> &'static str {
    if usk < 0.05 { "absent" }
    else if usk < 0.1 { "faint" }
    else if usk < 0.2 { "present" }
    else { "strong" }
}
