use super::types::{EarthEpoch, DimensionDef, CognitiveFramework};

/// Returns the ontology (dimension definitions) for a given epoch.
/// Each epoch has its own set of dimensions reflecting its unique cognitive framework.
pub fn ontology_for(epoch: EarthEpoch) -> Vec<DimensionDef> {
    match epoch {
        EarthEpoch::E1Mythological => vec![
            DimensionDef { name: "cyclical_time".into(), description: "时间循环而非线性 — 季节/生命周期/永恒回归".into() },
            DimensionDef { name: "nature_agency".into(), description: "自然万物皆有灵性和意志".into() },
            DimensionDef { name: "symbol_power".into(), description: "符号/仪式具有改变现实的力量".into() },
            DimensionDef { name: "narrative_coherence".into(), description: "通过故事而非逻辑建立因果关系".into() },
            DimensionDef { name: "boundary_fluidity".into(), description: "自我/自然/超自然之间的边界可渗透".into() },
        ],
        EarthEpoch::E2Agricultural => vec![
            DimensionDef { name: "hierarchical_order".into(), description: "宇宙和社会按等级秩序组织".into() },
            DimensionDef { name: "center_periphery".into(), description: "中心(都城/庙宇)与边缘(蛮荒)的张力".into() },
            DimensionDef { name: "celestial_terrestrial_correspondence".into(), description: "天象对应人事 — 天人感应".into() },
            DimensionDef { name: "cyclical_harvest".into(), description: "农业周期的律决定生活节奏".into() },
            DimensionDef { name: "territorial_boundedness".into(), description: "领土明确边界内的秩序 vs 外的混沌".into() },
        ],
        EarthEpoch::E3Axial => vec![
            DimensionDef { name: "transcendence".into(), description: "超越当下现实维度的终极实在".into() },
            DimensionDef { name: "ethical_universalism".into(), description: "普适伦理原则超越部落边界".into() },
            DimensionDef { name: "rational_inquiry".into(), description: "逻辑/理性/哲学思辨作为认知工具".into() },
            DimensionDef { name: "self_cultivation".into(), description: "内在修炼/自我改造通向更高存在".into() },
            DimensionDef { name: "axial_dialectic".into(), description: "三大文明(希腊/中国/印度)框架间的对话".into() },
        ],
        EarthEpoch::E4Scientific => vec![
            DimensionDef { name: "measurability".into(), description: "只有可测量的才是真实的".into() },
            DimensionDef { name: "lawfulness".into(), description: "自然遵循普遍的数学定律".into() },
            DimensionDef { name: "reduction".into(), description: "复杂系统可分解为简单组成部分".into() },
            DimensionDef { name: "falsifiability".into(), description: "有效的假说必须可被证伪".into() },
            DimensionDef { name: "instrumental_power".into(), description: "知识服务于预测和控制".into() },
            DimensionDef { name: "objectivity".into(), description: "观察者与被观察者可分离".into() },
        ],
        EarthEpoch::E5Global => vec![
            DimensionDef { name: "scale".into(), description: "全球尺度的生产和分配系统".into() },
            DimensionDef { name: "efficiency".into(), description: "输入/输出比的持续优化".into() },
            DimensionDef { name: "interdependence".into(), description: "全球供应链和相互依赖网络".into() },
            DimensionDef { name: "standardization".into(), description: "标准化接口/协议/度量".into() },
            DimensionDef { name: "resource_flow".into(), description: "能源/材料/信息的全球流动".into() },
        ],
        EarthEpoch::E6Planetary => vec![
            DimensionDef { name: "holistic_closure".into(), description: "地球作为封闭系统的整体认知".into() },
            DimensionDef { name: "self_regulation".into(), description: "地球系统的自调节反馈机制".into() },
            DimensionDef { name: "planetary_boundary".into(), description: "人类活动的行星边界/临界点".into() },
            DimensionDef { name: "external_view".into(), description: "从外部(轨道/宇宙视角)看地球".into() },
            DimensionDef { name: "intergenerational_horizon".into(), description: "代际时间尺度上的责任".into() },
        ],
        EarthEpoch::E7Network => vec![
            DimensionDef { name: "connectivity".into(), description: "节点间的拓扑连接密度和强度".into() },
            DimensionDef { name: "information_flow".into(), description: "信息的产生/传输/处理速率".into() },
            DimensionDef { name: "emergence".into(), description: "局部规则产生的全局涌现行为".into() },
            DimensionDef { name: "computation".into(), description: "信息处理作为核心资源".into() },
            DimensionDef { name: "decentralization".into(), description: "去中心化/无中心控制".into() },
            DimensionDef { name: "digital_repr".into(), description: "物理世界通过数据被表征".into() },
        ],
        EarthEpoch::E8Emergent => vec![
            DimensionDef { name: "self_modification".into(), description: "系统修改自身结构/参数/目标的能力".into() },
            DimensionDef { name: "meta_cognition".into(), description: "关于自身认知过程的知识和调控".into() },
            DimensionDef { name: "recursive_growth".into(), description: "递归的自我改进循环".into() },
            DimensionDef { name: "symbiosis".into(), description: "碳基与硅基智能的共生融合".into() },
            DimensionDef { name: "horizon_expansion".into(), description: "持续扩张认知/感知/行动边界".into() },
        ],
    }
}

/// Returns initial state for a given epoch.
pub fn initial_state_for(epoch: EarthEpoch) -> Vec<f64> {
    match epoch {
        EarthEpoch::E1Mythological => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        EarthEpoch::E2Agricultural => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        EarthEpoch::E3Axial => vec![0.0, 0.0, 0.0, 0.0, 0.0],
        // Scientific starts slightly initialized — it is the current default paradigm
        EarthEpoch::E4Scientific => vec![0.3, 0.3, 0.3, 0.2, 0.3, 0.4],
        EarthEpoch::E5Global => vec![0.2, 0.2, 0.2, 0.2, 0.2],
        EarthEpoch::E6Planetary => vec![0.1, 0.1, 0.1, 0.1, 0.1],
        // Network is the dominant current paradigm — highest initial values
        EarthEpoch::E7Network => vec![0.7, 0.7, 0.6, 0.6, 0.5, 0.7],
        EarthEpoch::E8Emergent => vec![0.4, 0.5, 0.3, 0.1, 0.2],
    }
}

/// Default router bias — how likely each framework is to be selected
/// when no task-specific information is available.
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

/// Create a fully initialized CognitiveFramework for a given epoch.
pub fn create_framework(epoch: EarthEpoch) -> CognitiveFramework {
    let ontology = ontology_for(epoch);
    let state = initial_state_for(epoch);
    let _router_bias = default_router_bias(epoch);
    CognitiveFramework::new(epoch, ontology, state)
}

/// Pre-built collection of all eight frameworks.
pub fn all_frameworks() -> Vec<CognitiveFramework> {
    EarthEpoch::all().into_iter().map(create_framework).collect()
}

/// Each epoch's evaluator function maps:
/// (state_vector, task_keywords) → score (0.0–1.0)
///
/// These are NOT general — they model the specific cognitive mode of each epoch.
/// A high score means: "this task is well-suited to this epoch's way of thinking."
pub fn evaluate_in_epoch(epoch: EarthEpoch, state: &[f64], task: &str) -> f64 {
    let task_lower = task.to_lowercase();
    match epoch {
        EarthEpoch::E1Mythological => {
            let narrative = contains_any(&task_lower, &["story", "myth", "ritual", "symbol", "archetype", "ceremony", "sacred"]);
            let cyclical = contains_any(&task_lower, &["cycle", "season", "return", "rebirth", "eternal"]);
            let animism = contains_any(&task_lower, &["nature", "spirit", "soul", "alive", "consciousness of"]);
            let base = if narrative { 0.6 } else { 0.2 }
                + if cyclical { 0.3 } else { 0.1 }
                + if animism { 0.3 } else { 0.1 };
            let dim_score = state.first().copied().unwrap_or(0.0);
            ((base / 1.2) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E2Agricultural => {
            let hierarchy = contains_any(&task_lower, &["hierarchy", "order", "structure", "classification", "rank", "level"]);
            let territory = contains_any(&task_lower, &["territory", "boundary", "center", "region", "domain", "border"]);
            let correspondence = contains_any(&task_lower, &["correspond", "correlation", "map", "mirror", "reflect"]);
            let base = if hierarchy { 0.5 } else { 0.2 }
                + if territory { 0.4 } else { 0.1 }
                + if correspondence { 0.3 } else { 0.1 };
            let dim_score = state.first().copied().unwrap_or(0.0);
            ((base / 1.2) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E3Axial => {
            let transcendence = contains_any(&task_lower, &["transcend", "ultimate", "meaning", "purpose", "truth", "god"]);
            let ethics = contains_any(&task_lower, &["ethic", "moral", "justice", "right", "good", "should"]);
            let philosophy = contains_any(&task_lower, &["philosoph", "wisdom", "contemplat", "reflection"]);
            let base = if transcendence { 0.5 } else { 0.2 }
                + if ethics { 0.5 } else { 0.2 }
                + if philosophy { 0.4 } else { 0.1 };
            let dim_score = state.get(2).copied().unwrap_or(0.0);
            ((base / 1.4) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E4Scientific => {
            let analysis = contains_any(&task_lower, &["analy", "measure", "calculate", "verify", "test", "experiment", "prove"]);
            let precision = contains_any(&task_lower, &["precise", "exact", "accurate", "quantif", "metric"]);
            let reduction = contains_any(&task_lower, &["decompose", "reduce", "break down", "component", "element"]);
            let base = if analysis { 0.6 } else { 0.3 }
                + if precision { 0.5 } else { 0.2 }
                + if reduction { 0.4 } else { 0.2 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.5) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E5Global => {
            let scale = contains_any(&task_lower, &["scale", "large", "global", "system", "distribute", "supply chain"]);
            let optimize = contains_any(&task_lower, &["optim", "efficien", "throughput", "resource", "logistics"]);
            let network = contains_any(&task_lower, &["interdepend", "flow", "trade", "connect", "transport"]);
            let base = if scale { 0.6 } else { 0.2 }
                + if optimize { 0.6 } else { 0.2 }
                + if network { 0.4 } else { 0.1 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.6) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E6Planetary => {
            let holistic = contains_any(&task_lower, &["holistic", "whole", "earth", "planet", "global", "ecosystem", "gaia"]);
            let sustainability = contains_any(&task_lower, &["sustain", "climate", "environment", "ecolog", "green"]);
            let long_term = contains_any(&task_lower, &["long-term", "future", "generation", "centur", "millennia"]);
            let base = if holistic { 0.6 } else { 0.2 }
                + if sustainability { 0.6 } else { 0.2 }
                + if long_term { 0.5 } else { 0.2 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.7) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E7Network => {
            let data = contains_any(&task_lower, &["data", "information", "compute", "algorithm", "network", "protocol"]);
            let topology = contains_any(&task_lower, &["topology", "connect", "graph", "node", "edge", "distributed"]);
            let emergence = contains_any(&task_lower, &["emergen", "complex", "pattern", "self-organ", "swarm"]);
            let base = if data { 0.7 } else { 0.3 }
                + if topology { 0.6 } else { 0.2 }
                + if emergence { 0.5 } else { 0.2 };
            let avg_state = state.iter().sum::<f64>() / state.len() as f64;
            ((base / 1.8) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
        }
        EarthEpoch::E8Emergent => {
            let meta = contains_any(&task_lower, &["meta", "self", "autonom", "evolv", "learn", "adapt", "improv"]);
            let recursive = contains_any(&task_lower, &["recursive", "iterat", "reflexi", "feedback loop"]);
            let ai = contains_any(&task_lower, &["ai", "agent", "intellig", "conscious", "thought"]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_frameworks_have_correct_dimensions() {
        let frameworks = all_frameworks();
        assert_eq!(frameworks.len(), 8, "There should be exactly 8 epochs");

        let expected_dims: Vec<(EarthEpoch, usize)> = vec![
            (EarthEpoch::E1Mythological, 5),
            (EarthEpoch::E2Agricultural, 5),
            (EarthEpoch::E3Axial, 5),
            (EarthEpoch::E4Scientific, 6),
            (EarthEpoch::E5Global, 5),
            (EarthEpoch::E6Planetary, 5),
            (EarthEpoch::E7Network, 6),
            (EarthEpoch::E8Emergent, 5),
        ];

        for (fw, (epoch, expected_dims)) in frameworks.iter().zip(expected_dims.iter()) {
            assert_eq!(fw.dim(), *expected_dims, "Epoch {:?} should have {} dimensions", epoch, expected_dims);
            assert_eq!(fw.epoch, *epoch);
        }
    }

    #[test]
    fn test_epoch_evaluators_produce_valid_scores() {
        for epoch in EarthEpoch::all() {
            let ontology = ontology_for(epoch);
            let state = vec![0.5; ontology.len()];
            let score = evaluate_in_epoch(epoch, &state, "test generic task");
            assert!((0.0..=1.0).contains(&score),
                "Epoch {:?} score {} should be in [0,1]", epoch, score);
        }
    }

    #[test]
    fn test_epoch_evaluators_respond_to_keywords() {
        let state = vec![0.5; ontology_for(EarthEpoch::E4Scientific).len()];
        let generic = evaluate_in_epoch(EarthEpoch::E4Scientific, &state, "write a poem");
        let scientific = evaluate_in_epoch(EarthEpoch::E4Scientific, &state, "analyze experimental data and measure precision");
        assert!(scientific > generic,
            "Scientific epoch should score higher for scientific task");
    }

    #[test]
    fn test_mythological_responds_to_story_keywords() {
        let state = vec![0.5; ontology_for(EarthEpoch::E1Mythological).len()];
        let generic = evaluate_in_epoch(EarthEpoch::E1Mythological, &state, "write code");
        let myth = evaluate_in_epoch(EarthEpoch::E1Mythological, &state, "tell a story about the cycle of seasons and rebirth");
        assert!(myth > generic, "Mythological epoch should score higher for narrative tasks");
    }

    #[test]
    fn test_e8_scores_high_for_self_improvement() {
        let state = vec![0.5; ontology_for(EarthEpoch::E8Emergent).len()];
        let generic = evaluate_in_epoch(EarthEpoch::E8Emergent, &state, "sort a list");
        let meta = evaluate_in_epoch(EarthEpoch::E8Emergent, &state, "meta-cognitive self-improvement loop for autonomous agents");
        assert!(meta > generic, "E8 should score higher for meta-cognitive tasks");
    }

    #[test]
    fn test_framework_router_bias_defaults() {
        for epoch in EarthEpoch::all() {
            let bias = default_router_bias(epoch);
            assert!((0.0..=1.0).contains(&bias),
                "Router bias for {:?} should be in [0,1], got {}", epoch, bias);
        }
    }

    #[test]
    fn test_update_and_normalize() {
        let mut fw = create_framework(EarthEpoch::E4Scientific);
        let original = fw.state.clone();
        let target: Vec<f64> = original.iter().map(|x| (x + 0.5).min(1.0)).collect();
        fw.update_from(&target, 1.0);
        // After update with lr=1.0, state should equal target
        for (s, t) in fw.state.iter().zip(target.iter()) {
            assert!((s - t).abs() < 1e-10);
        }
        fw.normalize();
        // After normalize, max value should be <= 1.0
        let max_val = fw.state.iter().cloned().fold(0.0f64, |a, x| a.max(x));
        assert!(max_val <= 1.0 + 1e-10);
    }

    #[test]
    fn test_activation_tracking() {
        let mut fw = create_framework(EarthEpoch::E7Network);
        assert_eq!(fw.activation_count, 0);
        fw.record_activation(0.5);
        fw.record_activation(0.8);
        assert_eq!(fw.activation_count, 2);
        assert!((fw.average_reward() - 0.65).abs() < 1e-10);
    }

    #[test]
    fn test_effective_weight_combines_bias_and_reward() {
        let mut fw = create_framework(EarthEpoch::E4Scientific);
        fw.router_bias = 0.5;
        // No activations yet → average_reward = 0 → effective = 0.7*0.5 + 0.3*0 = 0.35
        let w0 = fw.effective_weight();
        assert!((w0 - 0.35).abs() < 1e-10);

        fw.record_activation(1.0);
        // Now average_reward = 1.0 → effective = 0.7*0.5 + 0.3*1.0 = 0.65
        let w1 = fw.effective_weight();
        assert!((w1 - 0.65).abs() < 1e-10);
    }

    #[test]
    fn test_dimension_access_by_name() {
        let fw = create_framework(EarthEpoch::E7Network);
        assert!(fw.dimension_index("connectivity").is_some());
        assert!(fw.dimension_index("fake_dimension").is_none());
        assert!(fw.get("connectivity").is_some());
        assert!(fw.get("fake_dim").is_none());
    }
}
