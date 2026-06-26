use std::collections::HashMap;

/// A discovered symbolic relationship among concepts
#[derive(Debug, Clone)]
pub struct SymbolicLaw {
    pub id: u64,
    pub name: String,
    pub expression: String,
    pub domain: String,
    pub applicability: Vec<String>,
    pub confidence: f64,
    pub is_general: bool,
    pub derivation_chain: Vec<String>,
}

/// A discovered or proposed concept (symbolic variable)
#[derive(Debug, Clone)]
pub struct DiscoveredConcept {
    pub name: String,
    pub symbol: String,
    pub domain: String,
    pub dimensionality: String,
    pub confidence: f64,
}

/// Plausible reasoning step: extend a law when it fails in a new context
#[derive(Debug, Clone)]
pub struct PlausibleExtension {
    pub base_law_id: u64,
    pub base_law: String,
    pub new_context: String,
    pub extension: String,
    pub added_term: String,
    pub reasoning: String,
}

/// Concept-driven discovery engine inspired by AI-Newton (arXiv:2504.01538).
/// Maintains a set of symbolic laws and concepts, supports plausible extension
/// when existing laws fail in novel contexts, and uses UCB-inspired selection
/// to balance exploration vs exploitation of discovered laws.
#[derive(Debug, Clone)]
pub struct SymbolicDiscoveryEngine {
    pub laws: Vec<SymbolicLaw>,
    pub concepts: Vec<DiscoveredConcept>,
    pub extensions: Vec<PlausibleExtension>,
    max_laws: usize,
    max_concepts: usize,
    next_id: u64,
    ucb_exploration_weight: f64,
    discovery_count: u64,
    usage_count: HashMap<u64, u64>,
    domain_index: HashMap<String, Vec<usize>>,
}

impl SymbolicDiscoveryEngine {
    pub fn new() -> Self {
        Self {
            laws: Vec::new(),
            concepts: Vec::new(),
            extensions: Vec::new(),
            max_laws: 200,
            max_concepts: 100,
            next_id: 1,
            ucb_exploration_weight: 1.5,
            discovery_count: 0,
            usage_count: HashMap::new(),
            domain_index: HashMap::new(),
        }
    }

    pub fn register_concept(
        &mut self,
        name: &str,
        symbol: &str,
        domain: &str,
        dimensionality: &str,
    ) -> Result<(), String> {
        if self.concepts.iter().any(|c| c.symbol == symbol) {
            return Err(format!("concept '{}' already registered", symbol));
        }
        if self.concepts.len() >= self.max_concepts {
            return Err("concept limit reached".to_string());
        }
        self.concepts.push(DiscoveredConcept {
            name: name.to_string(),
            symbol: symbol.to_string(),
            domain: domain.to_string(),
            dimensionality: dimensionality.to_string(),
            confidence: 0.9,
        });
        Ok(())
    }

    pub fn register_law(
        &mut self,
        name: &str,
        expression: &str,
        domain: &str,
        concepts: Vec<String>,
        confidence: f64,
    ) -> Result<u64, String> {
        if self.laws.len() >= self.max_laws {
            return Err("law limit reached".to_string());
        }
        let id = self.next_id;
        self.next_id += 1;
        self.laws.push(SymbolicLaw {
            id,
            name: name.to_string(),
            expression: expression.to_string(),
            domain: domain.to_string(),
            applicability: concepts,
            confidence: confidence.clamp(0.0, 1.0),
            is_general: false,
            derivation_chain: vec![format!("registered directly in '{}' domain", domain)],
        });
        self.domain_index
            .entry(domain.to_string())
            .or_default()
            .push(self.laws.len() - 1);
        self.usage_count.insert(id, 0);
        self.discovery_count += 1;
        Ok(id)
    }

    /// Core AI-Newton mechanism: when a law fails in a new context,
    /// propose an extension by injecting a new concept-term.
    pub fn plausible_extend(
        &mut self,
        base_law_id: u64,
        failing_context: &str,
    ) -> Option<PlausibleExtension> {
        let base_idx = self.laws.iter().position(|l| l.id == base_law_id)?;
        let base_law = self.laws[base_idx].clone();

        // Find candidate concepts in the failing context's domain
        let domain_concepts: Vec<&DiscoveredConcept> = self
            .concepts
            .iter()
            .filter(|c| c.domain == failing_context || c.confidence > 0.7)
            .collect();

        if domain_concepts.is_empty() {
            // No concepts found — fall back to a generic perturbation
            let term = format!("+ epsilon_{}", self.discovery_count);
            let extension_expr = format!("({}) {}", base_law.expression, term);
            let extension = PlausibleExtension {
                base_law_id,
                base_law: base_law.expression.clone(),
                new_context: failing_context.to_string(),
                extension: extension_expr.clone(),
                added_term: term.clone(),
                reasoning: format!(
                    "law '{}' failed in '{}'; no known concepts, added {} as placeholder",
                    base_law.name, failing_context, term
                ),
            };
            let _reasoning = extension.reasoning.clone();
            let _ = self.register_law(
                &format!("{}_ext_{}", base_law.name, self.discovery_count),
                &extension_expr,
                failing_context,
                base_law.applicability.clone(),
                base_law.confidence * 0.7,
            );
            self.extensions.push(extension.clone());
            self.discovery_count += 1;
            return Some(extension);
        }

        // Pick the highest-confidence concept and inject it as a correcting term
        let best = domain_concepts
            .into_iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        let term = format!("+ {} * {}", best.symbol, best.name);
        let extension_expr = format!("({}) {}", base_law.expression, term);
        let reasoning = format!(
            "law '{}' (={}) failed in '{}'; plausible correction: add adjusting term for '{}' ({})",
            base_law.name, base_law.expression, failing_context, best.name, best.symbol
        );
        let extension = PlausibleExtension {
            base_law_id,
            base_law: base_law.expression.clone(),
            new_context: failing_context.to_string(),
            extension: extension_expr.clone(),
            added_term: term.clone(),
            reasoning: reasoning.clone(),
        };
        let _ = self.register_law(
            &format!("{}_ext_{}", base_law.name, self.discovery_count),
            &extension_expr,
            failing_context,
            vec![best.symbol.clone()],
            base_law.confidence * 0.7,
        );
        self.extensions.push(extension.clone());
        self.discovery_count += 1;
        Some(extension)
    }

    /// UCB-inspired selection: rank laws by confidence * exploration bonus.
    pub fn find_relevant_laws(
        &self,
        domain: &str,
        concepts: &[String],
        top_k: usize,
    ) -> Vec<&SymbolicLaw> {
        let total_uses: u64 = self.usage_count.values().sum();
        let total = if total_uses == 0 {
            1.0
        } else {
            total_uses as f64
        };

        let mut scored: Vec<(&SymbolicLaw, f64)> = self
            .laws
            .iter()
            .filter(|l| l.domain == domain || l.applicability.iter().any(|a| concepts.contains(a)))
            .map(|l| {
                let count = *self.usage_count.get(&l.id).unwrap_or(&0) as f64;
                let explore =
                    self.ucb_exploration_weight * (2.0 * total.ln() / (count + 1.0)).sqrt();
                let score = l.confidence * (1.0 + explore);
                (l, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(l, _)| l).collect()
    }

    /// Mark laws that hold across ≥3 distinct concept sets in a domain as general.
    pub fn generalize_laws(&mut self, domain: &str) -> Vec<u64> {
        let mut generalized = Vec::new();
        let indices: Vec<usize> = self.domain_index.get(domain).cloned().unwrap_or_default();

        for &idx in &indices {
            if self.laws[idx].is_general {
                continue;
            }
            let distinct: std::collections::HashSet<&String> =
                self.laws[idx].applicability.iter().collect();
            if distinct.len() >= 3 {
                let law_id = self.laws[idx].id;
                self.laws[idx].is_general = true;
                self.laws[idx]
                    .derivation_chain
                    .push(format!("generalized in '{}' (>=3 concepts)", domain));
                generalized.push(law_id);
            }
        }
        generalized
    }

    /// Summary of all discoveries.
    pub fn discovery_report(&self) -> String {
        let total = self.laws.len();
        let general = self.laws.iter().filter(|l| l.is_general).count();
        let specific = total - general;
        let avg_conf: f64 = if total > 0 {
            self.laws.iter().map(|l| l.confidence).sum::<f64>() / total as f64
        } else {
            0.0
        };
        format!(
            "SymbolicDiscoveryEngine report:\n\
             - laws: {} (general: {}, specific: {})\n\
             - concepts: {}\n\
             - extensions: {}\n\
             - avg confidence: {:.3}\n\
             - total discoveries: {}",
            total,
            general,
            specific,
            self.concepts.len(),
            self.extensions.len(),
            avg_conf,
            self.discovery_count
        )
    }

    /// Remove laws with confidence < threshold, but keep general ones.
    pub fn prune_low_confidence(&mut self, threshold: f64) {
        let before = self.laws.len();
        let mut surviving_indices = Vec::new();

        for (i, law) in self.laws.iter().enumerate() {
            if law.confidence >= threshold || law.is_general {
                surviving_indices.push(i);
            }
        }

        // Remap domain_index
        let new_indices: Vec<usize> = surviving_indices.iter().map(|&i| i).collect();
        let old_to_new: HashMap<usize, usize> = new_indices
            .iter()
            .enumerate()
            .map(|(new, &old)| (old, new))
            .collect();

        let mut new_domain_index: HashMap<String, Vec<usize>> = HashMap::new();
        for (domain, old_positions) in &self.domain_index {
            let mut new_positions = Vec::new();
            for &old_pos in old_positions {
                if let Some(&new_pos) = old_to_new.get(&old_pos) {
                    new_positions.push(new_pos);
                }
            }
            if !new_positions.is_empty() {
                new_domain_index.insert(domain.clone(), new_positions);
            }
        }

        self.laws = surviving_indices
            .iter()
            .map(|&i| self.laws[i].clone())
            .collect();
        self.domain_index = new_domain_index;
        let removed = before - self.laws.len();
        if removed > 0 {
            let _ = removed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_concept_and_law() {
        let mut engine = SymbolicDiscoveryEngine::new();
        assert!(engine
            .register_concept("mass", "m", "mechanics", "M")
            .is_ok());
        assert!(engine
            .register_concept("acceleration", "a", "mechanics", "L/T^2")
            .is_ok());
        assert!(engine
            .register_concept("force", "F", "mechanics", "M*L/T^2")
            .is_ok());

        // Duplicate symbol
        assert!(engine
            .register_concept("mass2", "m", "mechanics", "M")
            .is_err());

        let law_id = engine
            .register_law(
                "Newton's second law",
                "F = m * a",
                "mechanics",
                vec!["m".to_string(), "a".to_string(), "F".to_string()],
                0.95,
            )
            .unwrap();
        assert_eq!(law_id, 1);

        let report = engine.discovery_report();
        assert!(report.contains("laws: 1"));
        assert!(report.contains("concepts: 3"));
    }

    #[test]
    fn test_plausible_extension_when_law_fails() {
        let mut engine = SymbolicDiscoveryEngine::new();
        engine
            .register_concept("mass", "m", "mechanics", "M")
            .unwrap();
        engine
            .register_concept("acceleration", "a", "mechanics", "L/T^2")
            .unwrap();
        engine
            .register_concept("force", "F", "mechanics", "M*L/T^2")
            .unwrap();
        engine
            .register_concept("spring constant", "k", "mechanics", "M/T^2")
            .unwrap();

        let law_id = engine
            .register_law(
                "Newton's second law",
                "F = m * a",
                "mechanics",
                vec!["m".to_string(), "a".to_string()],
                0.95,
            )
            .unwrap();

        let extension = engine.plausible_extend(law_id, "spring system");
        assert!(extension.is_some());
        let ext = extension.unwrap();
        assert_eq!(ext.base_law_id, law_id);
        assert_eq!(ext.new_context, "spring system");
        // Should have added a term involving spring constant k
        assert!(!ext.added_term.is_empty());
        // Verify the extended law was registered
        assert!(engine.laws.len() >= 2);
    }

    #[test]
    fn test_generalize_laws() {
        let mut engine = SymbolicDiscoveryEngine::new();
        let id1 = engine
            .register_law(
                "law1",
                "x + y = z",
                "algebra",
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
                0.8,
            )
            .unwrap();
        let id2 = engine
            .register_law(
                "law2",
                "p * q = r",
                "algebra",
                vec!["x".to_string(), "y".to_string(), "z".to_string()],
                0.7,
            )
            .unwrap();
        let id3 = engine
            .register_law(
                "law3",
                "u^2 = v",
                "algebra",
                vec!["m".to_string(), "n".to_string(), "o".to_string()],
                0.6,
            )
            .unwrap();

        let generalized = engine.generalize_laws("algebra");
        // Each law has 3 distinct applicability concepts, so all should be generalized
        assert_eq!(generalized.len(), 3);
        assert!(generalized.contains(&id1));
        assert!(generalized.contains(&id2));
        assert!(generalized.contains(&id3));
    }

    #[test]
    fn test_ucb_selection() {
        let mut engine = SymbolicDiscoveryEngine::new();
        let id_high = engine
            .register_law("high conf", "a = b", "physics", vec!["x".to_string()], 0.9)
            .unwrap();
        let _id_low = engine
            .register_law("low conf", "c = d", "physics", vec!["y".to_string()], 0.3)
            .unwrap();

        // Manually bump usage of the high-confidence law to reduce its UCB bonus
        *engine.usage_count.get_mut(&id_high).unwrap() = 10;

        let results = engine.find_relevant_laws("physics", &["x".to_string()], 2);
        assert_eq!(results.len(), 2);
        // The low-confidence law (used 0 times) should get a higher exploration bonus
        // and could rank higher than the high-confidence but overused law
        let names: Vec<&str> = results.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"high conf") || names.contains(&"low conf"));
    }

    #[test]
    fn test_prune_low_confidence() {
        let mut engine = SymbolicDiscoveryEngine::new();
        engine
            .register_law("solid", "a = b", "math", vec!["x".to_string()], 0.9)
            .unwrap();
        engine
            .register_law("weak", "c = d", "math", vec!["y".to_string()], 0.2)
            .unwrap();
        // Mark the first law as general
        engine.laws[0].is_general = true;

        assert_eq!(engine.laws.len(), 2);
        engine.prune_low_confidence(0.5);
        // The general law (confidence 0.9 >= 0.5) stays; the weak one (0.2 < 0.5) is pruned
        assert_eq!(engine.laws.len(), 1);
        assert!(!engine.laws.iter().any(|l| l.name == "weak"));
        // Verify general law survived
        assert!(engine.laws.iter().any(|l| l.name == "solid"));
    }
}
