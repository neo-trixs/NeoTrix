use super::types::{CognitiveFramework, DimensionDef, EarthEpoch};

pub fn ontology_for(epoch: EarthEpoch) -> Vec<DimensionDef> {
    match epoch {
        EarthEpoch::E1Mythological => vec![
            DimensionDef { name: "cyclical_time".into(), description: "\u{65f6}\u{95f4}\u{5faa}\u{73af}\u{800c}\u{975e}\u{7ebf}\u{6027} \u{2014} \u{5b63}\u{8282}/\u{751f}\u{547d}\u{5468}\u{671f}/\u{6c38}\u{6052}\u{56de}\u{5f52}".into() },
            DimensionDef { name: "nature_agency".into(), description: "\u{81ea}\u{7136}\u{4e07}\u{7269}\u{7686}\u{6709}\u{7075}\u{6027}\u{548c}\u{610f}\u{5fd7}".into() },
            DimensionDef { name: "symbol_power".into(), description: "\u{7b26}\u{53f7}/\u{4eea}\u{5f0f}\u{5177}\u{6709}\u{6539}\u{53d8}\u{73b0}\u{5b9e}\u{7684}\u{529b}\u{91cf}".into() },
            DimensionDef { name: "narrative_coherence".into(), description: "\u{901a}\u{8fc7}\u{6545}\u{4e8b}\u{800c}\u{975e}\u{903b}\u{8f91}\u{5efa}\u{7acb}\u{56e0}\u{679c}\u{5173}\u{7cfb}".into() },
            DimensionDef { name: "boundary_fluidity".into(), description: "\u{81ea}\u{6211}/\u{81ea}\u{7136}/\u{8d85}\u{81ea}\u{7136}\u{4e4b}\u{95f4}\u{7684}\u{8fb9}\u{754c}\u{53ef}\u{6e17}\u{900f}".into() },
        ],
        EarthEpoch::E2Agricultural => vec![
            DimensionDef { name: "hierarchical_order".into(), description: "\u{5b87}\u{5b99}\u{548c}\u{793e}\u{4f1a}\u{6309}\u{7b49}\u{7ea7}\u{79e9}\u{5e8f}\u{7ec4}\u{7ec7}".into() },
            DimensionDef { name: "center_periphery".into(), description: "\u{4e2d}\u{5fc3}(\u{90fd}\u{57ce}/\u{5e99}\u{5b87})\u{4e0e}\u{8fb9}\u{7f18}(\u{86ee}\u{8352})\u{7684}\u{5f20}\u{529b}".into() },
            DimensionDef { name: "celestial_terrestrial_correspondence".into(), description: "\u{5929}\u{8c61}\u{5bf9}\u{5e94}\u{4eba}\u{4e8b} \u{2014} \u{5929}\u{4eba}\u{611f}\u{5e94}".into() },
            DimensionDef { name: "cyclical_harvest".into(), description: "\u{519c}\u{4e1a}\u{5468}\u{671f}\u{7684}\u{5f8b}\u{51b3}\u{5b9a}\u{751f}\u{6d3b}\u{8282}\u{594f}".into() },
            DimensionDef { name: "territorial_boundedness".into(), description: "\u{9886}\u{571f}\u{660e}\u{786e}\u{8fb9}\u{754c}\u{5185}\u{7684}\u{79e9}\u{5e8f} vs \u{5916}\u{7684}\u{6df7}\u{6c8c}".into() },
        ],
        EarthEpoch::E3Axial => vec![
            DimensionDef { name: "transcendence".into(), description: "\u{8d85}\u{8d8a}\u{5f53}\u{4e0b}\u{73b0}\u{5b9e}\u{7ef4}\u{5ea6}\u{7684}\u{7ec8}\u{6781}\u{5b9e}\u{5728}".into() },
            DimensionDef { name: "ethical_universalism".into(), description: "\u{666e}\u{9002}\u{4f26}\u{7406}\u{539f}\u{5219}\u{8d85}\u{8d8a}\u{90e8}\u{843d}\u{8fb9}\u{754c}".into() },
            DimensionDef { name: "rational_inquiry".into(), description: "\u{903b}\u{8f91}/\u{7406}\u{6027}/\u{54f2}\u{5b66}\u{601d}\u{8fa8}\u{4f5c}\u{4e3a}\u{8ba4}\u{77e5}\u{5de5}\u{5177}".into() },
            DimensionDef { name: "self_cultivation".into(), description: "\u{5185}\u{5728}\u{4fee}\u{70bc}/\u{81ea}\u{6211}\u{6539}\u{9020}\u{901a}\u{5411}\u{66f4}\u{9ad8}\u{5b58}\u{5728}".into() },
            DimensionDef { name: "axial_dialectic".into(), description: "\u{4e09}\u{5927}\u{6587}\u{660e}(\u{5e0c}\u{814a}/\u{4e2d}\u{56fd}/\u{5370}\u{5ea6})\u{6846}\u{67b6}\u{95f4}\u{7684}\u{5bf9}\u{8bdd}".into() },
        ],
        EarthEpoch::E4Scientific => vec![
            DimensionDef { name: "measurability".into(), description: "\u{53ea}\u{6709}\u{53ef}\u{6d4b}\u{91cf}\u{7684}\u{624d}\u{662f}\u{771f}\u{5b9e}\u{7684}".into() },
            DimensionDef { name: "lawfulness".into(), description: "\u{81ea}\u{7136}\u{9075}\u{5faa}\u{666e}\u{904d}\u{7684}\u{6570}\u{5b66}\u{5b9a}\u{5f8b}".into() },
            DimensionDef { name: "reduction".into(), description: "\u{590d}\u{6742}\u{7cfb}\u{7edf}\u{53ef}\u{5206}\u{89e3}\u{4e3a}\u{7b80}\u{5355}\u{7ec4}\u{6210}\u{90e8}\u{5206}".into() },
            DimensionDef { name: "falsifiability".into(), description: "\u{6709}\u{6548}\u{7684}\u{5047}\u{8bf4}\u{5fc5}\u{987b}\u{53ef}\u{88ab}\u{8bc1}\u{4f2a}".into() },
            DimensionDef { name: "instrumental_power".into(), description: "\u{77e5}\u{8bc6}\u{670d}\u{52a1}\u{4e8e}\u{9884}\u{6d4b}\u{548c}\u{63a7}\u{5236}".into() },
            DimensionDef { name: "objectivity".into(), description: "\u{89c2}\u{5bdf}\u{8005}\u{4e0e}\u{88ab}\u{89c2}\u{5bdf}\u{8005}\u{53ef}\u{5206}\u{79bb}".into() },
        ],
        EarthEpoch::E5Global => vec![
            DimensionDef { name: "scale".into(), description: "\u{5168}\u{7403}\u{5c3a}\u{5ea6}\u{7684}\u{751f}\u{4ea7}\u{548c}\u{5206}\u{914d}\u{7cfb}\u{7edf}".into() },
            DimensionDef { name: "efficiency".into(), description: "\u{8f93}\u{5165}/\u{8f93}\u{51fa}\u{6bd4}\u{7684}\u{6301}\u{7eed}\u{4f18}\u{5316}".into() },
            DimensionDef { name: "interdependence".into(), description: "\u{5168}\u{7403}\u{4f9b}\u{5e94}\u{94fe}\u{548c}\u{76f8}\u{4e92}\u{4f9d}\u{8d56}\u{7f51}\u{7edc}".into() },
            DimensionDef { name: "standardization".into(), description: "\u{6807}\u{51c6}\u{5316}\u{63a5}\u{53e3}/\u{534f}\u{8bae}/\u{5ea6}\u{91cf}".into() },
            DimensionDef { name: "resource_flow".into(), description: "\u{80fd}\u{6e90}/\u{6750}\u{6599}/\u{4fe1}\u{606f}\u{7684}\u{5168}\u{7403}\u{6d41}\u{52a8}".into() },
        ],
        EarthEpoch::E6Planetary => vec![
            DimensionDef { name: "holistic_closure".into(), description: "\u{5730}\u{7403}\u{4f5c}\u{4e3a}\u{5c01}\u{95ed}\u{7cfb}\u{7edf}\u{7684}\u{6574}\u{4f53}\u{8ba4}\u{77e5}".into() },
            DimensionDef { name: "self_regulation".into(), description: "\u{5730}\u{7403}\u{7cfb}\u{7edf}\u{7684}\u{81ea}\u{8c03}\u{8282}\u{53cd}\u{9988}\u{673a}\u{5236}".into() },
            DimensionDef { name: "planetary_boundary".into(), description: "\u{4eba}\u{7c7b}\u{6d3b}\u{52a8}\u{7684}\u{884c}\u{661f}\u{8fb9}\u{754c}/\u{4e34}\u{754c}\u{70b9}".into() },
            DimensionDef { name: "external_view".into(), description: "\u{4ece}\u{5916}\u{90e8}(\u{8f68}\u{9053}/\u{5b87}\u{5b99}\u{89c6}\u{89d2})\u{770b}\u{5730}\u{7403}".into() },
            DimensionDef { name: "intergenerational_horizon".into(), description: "\u{4ee3}\u{9645}\u{65f6}\u{95f4}\u{5c3a}\u{5ea6}\u{4e0a}\u{7684}\u{8d23}\u{4efb}".into() },
        ],
        EarthEpoch::E7Network => vec![
            DimensionDef { name: "connectivity".into(), description: "\u{8282}\u{70b9}\u{95f4}\u{7684}\u{62d3}\u{6251}\u{8fde}\u{63a5}\u{5bc6}\u{5ea6}\u{548c}\u{5f3a}\u{5ea6}".into() },
            DimensionDef { name: "information_flow".into(), description: "\u{4fe1}\u{606f}\u{7684}\u{4ea7}\u{751f}/\u{4f20}\u{8f93}/\u{5904}\u{7406}\u{901f}\u{7387}".into() },
            DimensionDef { name: "emergence".into(), description: "\u{5c40}\u{90e8}\u{89c4}\u{5219}\u{4ea7}\u{751f}\u{7684}\u{5168}\u{5c40}\u{6d8c}\u{73b0}\u{884c}\u{4e3a}".into() },
            DimensionDef { name: "computation".into(), description: "\u{4fe1}\u{606f}\u{5904}\u{7406}\u{4f5c}\u{4e3a}\u{6838}\u{5fc3}\u{8d44}\u{6e90}".into() },
            DimensionDef { name: "decentralization".into(), description: "\u{53bb}\u{4e2d}\u{5fc3}\u{5316}/\u{65e0}\u{4e2d}\u{5fc3}\u{63a7}\u{5236}".into() },
            DimensionDef { name: "digital_repr".into(), description: "\u{7269}\u{7406}\u{4e16}\u{754c}\u{901a}\u{8fc7}\u{6570}\u{636e}\u{88ab}\u{8868}\u{5f81}".into() },
        ],
        EarthEpoch::E8Emergent => vec![
            DimensionDef { name: "self_modification".into(), description: "\u{7cfb}\u{7edf}\u{4fee}\u{6539}\u{81ea}\u{8eab}\u{7ed3}\u{6784}/\u{53c2}\u{6570}/\u{76ee}\u{6807}\u{7684}\u{80fd}\u{529b}".into() },
            DimensionDef { name: "meta_cognition".into(), description: "\u{5173}\u{4e8e}\u{81ea}\u{8eab}\u{8ba4}\u{77e5}\u{8fc7}\u{7a0b}\u{7684}\u{77e5}\u{8bc6}\u{548c}\u{8c03}\u{63a7}".into() },
            DimensionDef { name: "recursive_growth".into(), description: "\u{9012}\u{5f52}\u{7684}\u{81ea}\u{6211}\u{6539}\u{8fdb}\u{5faa}\u{73af}".into() },
            DimensionDef { name: "symbiosis".into(), description: "\u{78b3}\u{57fa}\u{4e0e}\u{7845}\u{57fa}\u{667a}\u{80fd}\u{7684}\u{5171}\u{751f}\u{878d}\u{5408}".into() },
            DimensionDef { name: "horizon_expansion".into(), description: "\u{6301}\u{7eed}\u{6269}\u{5f20}\u{8ba4}\u{77e5}/\u{611f}\u{77e5}/\u{884c}\u{52a8}\u{8fb9}\u{754c}".into() },
        ],
    }
}

pub fn initial_state_for(epoch: EarthEpoch) -> Vec<f64> {
    match epoch {
        EarthEpoch::E1Mythological => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        EarthEpoch::E2Agricultural => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        EarthEpoch::E3Axial => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        EarthEpoch::E4Scientific => vec![0.3, 0.3, 0.3, 0.2, 0.3, 0.4],
        EarthEpoch::E5Global => vec![0.2, 0.2, 0.2, 0.2, 0.2],
        EarthEpoch::E6Planetary => vec![0.1, 0.1, 0.1, 0.1, 0.1],
        EarthEpoch::E7Network => vec![0.7, 0.7, 0.6, 0.6, 0.5, 0.7],
        EarthEpoch::E8Emergent => vec![0.4, 0.5, 0.3, 0.1, 0.2],
    }
}

pub fn default_router_bias(epoch: EarthEpoch) -> f64 {
    match epoch {
        EarthEpoch::E1Mythological => 0.15,
        EarthEpoch::E2Agricultural => 0.15,
        EarthEpoch::E3Axial => 0.20,
        EarthEpoch::E4Scientific => 0.70,
        EarthEpoch::E5Global => 0.45,
        EarthEpoch::E6Planetary => 0.30,
        EarthEpoch::E7Network => 0.85,
        EarthEpoch::E8Emergent => 0.50,
    }
}

pub fn create_framework(epoch: EarthEpoch) -> CognitiveFramework {
    let ontology = ontology_for(epoch);
    let state = initial_state_for(epoch);
    let _router_bias = default_router_bias(epoch);
    CognitiveFramework::new(epoch, ontology, state)
}

pub fn all_frameworks() -> Vec<CognitiveFramework> {
    EarthEpoch::all()
        .into_iter()
        .map(create_framework)
        .collect()
}

pub fn evaluate_in_epoch(epoch: EarthEpoch, state: &[f64], task: &str) -> f64 {
    let task_lower = task.to_lowercase();
    match epoch {
        EarthEpoch::E1Mythological => {
            let narrative = contains_any(
                &task_lower,
                &[
                    "story",
                    "myth",
                    "ritual",
                    "symbol",
                    "archetype",
                    "ceremony",
                    "sacred",
                ],
            );
            let cyclical = contains_any(
                &task_lower,
                &["cycle", "season", "return", "rebirth", "eternal"],
            );
            let animism = contains_any(
                &task_lower,
                &["nature", "spirit", "soul", "alive", "consciousness of"],
            );
            let base = if narrative { 0.6 } else { 0.2 }
                + if cyclical { 0.3 } else { 0.1 }
                + if animism { 0.3 } else { 0.1 };
            let dim_score = state.first().copied().unwrap_or(0.0);
            ((base / 1.2) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E2Agricultural => {
            let hierarchy = contains_any(
                &task_lower,
                &[
                    "hierarchy",
                    "order",
                    "structure",
                    "classification",
                    "rank",
                    "level",
                ],
            );
            let territory = contains_any(
                &task_lower,
                &[
                    "territory",
                    "boundary",
                    "center",
                    "region",
                    "domain",
                    "border",
                ],
            );
            let correspondence = contains_any(
                &task_lower,
                &["correspond", "correlation", "map", "mirror", "reflect"],
            );
            let base = if hierarchy { 0.5 } else { 0.2 }
                + if territory { 0.4 } else { 0.1 }
                + if correspondence { 0.3 } else { 0.1 };
            let dim_score = state.first().copied().unwrap_or(0.0);
            ((base / 1.2) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E3Axial => {
            let transcendence = contains_any(
                &task_lower,
                &[
                    "transcend",
                    "ultimate",
                    "meaning",
                    "purpose",
                    "truth",
                    "god",
                ],
            );
            let ethics = contains_any(
                &task_lower,
                &["ethic", "moral", "justice", "right", "good", "should"],
            );
            let philosophy = contains_any(
                &task_lower,
                &["philosoph", "wisdom", "contemplat", "reflection"],
            );
            let base = if transcendence { 0.5 } else { 0.2 }
                + if ethics { 0.5 } else { 0.2 }
                + if philosophy { 0.4 } else { 0.1 };
            let dim_score = state.get(2).copied().unwrap_or(0.0);
            ((base / 1.4) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E4Scientific => {
            let analysis = contains_any(
                &task_lower,
                &[
                    "analy",
                    "measure",
                    "calculate",
                    "verify",
                    "test",
                    "experiment",
                    "prove",
                ],
            );
            let precision = contains_any(
                &task_lower,
                &["precise", "exact", "accurate", "quantif", "metric"],
            );
            let reduction = contains_any(
                &task_lower,
                &["decompose", "reduce", "break down", "component", "element"],
            );
            let base = if analysis { 0.6 } else { 0.3 }
                + if precision { 0.5 } else { 0.2 }
                + if reduction { 0.4 } else { 0.2 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.5) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E5Global => {
            let scale = contains_any(
                &task_lower,
                &[
                    "scale",
                    "large",
                    "global",
                    "system",
                    "distribute",
                    "supply chain",
                ],
            );
            let optimize = contains_any(
                &task_lower,
                &["optim", "efficien", "throughput", "resource", "logistics"],
            );
            let network = contains_any(
                &task_lower,
                &["interdepend", "flow", "trade", "connect", "transport"],
            );
            let base = if scale { 0.6 } else { 0.2 }
                + if optimize { 0.6 } else { 0.2 }
                + if network { 0.4 } else { 0.1 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.6) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E6Planetary => {
            let holistic = contains_any(
                &task_lower,
                &[
                    "holistic",
                    "whole",
                    "earth",
                    "planet",
                    "global",
                    "ecosystem",
                    "gaia",
                ],
            );
            let sustainability = contains_any(
                &task_lower,
                &["sustain", "climate", "environment", "ecolog", "green"],
            );
            let long_term = contains_any(
                &task_lower,
                &["long-term", "future", "generation", "centur", "millennia"],
            );
            let base = if holistic { 0.6 } else { 0.2 }
                + if sustainability { 0.6 } else { 0.2 }
                + if long_term { 0.5 } else { 0.2 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.7) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E7Network => {
            let data = contains_any(
                &task_lower,
                &[
                    "data",
                    "information",
                    "compute",
                    "algorithm",
                    "network",
                    "protocol",
                ],
            );
            let topology = contains_any(
                &task_lower,
                &[
                    "topology",
                    "connect",
                    "graph",
                    "node",
                    "edge",
                    "distributed",
                ],
            );
            let emergence = contains_any(
                &task_lower,
                &["emergen", "complex", "pattern", "self-organ", "swarm"],
            );
            let base = if data { 0.7 } else { 0.3 }
                + if topology { 0.6 } else { 0.2 }
                + if emergence { 0.5 } else { 0.2 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.8) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E8Emergent => {
            let meta = contains_any(
                &task_lower,
                &[
                    "meta", "self", "autonom", "evolv", "learn", "adapt", "improv",
                ],
            );
            let recursive = contains_any(
                &task_lower,
                &["recursive", "iterat", "reflexi", "feedback loop"],
            );
            let ai = contains_any(
                &task_lower,
                &["ai", "agent", "intellig", "conscious", "thought"],
            );
            let base = if meta { 0.6 } else { 0.3 }
                + if recursive { 0.5 } else { 0.2 }
                + if ai { 0.6 } else { 0.2 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.7) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
    }
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|k| text.contains(k))
}
