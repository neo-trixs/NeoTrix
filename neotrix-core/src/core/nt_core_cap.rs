use serde::{Deserialize, Serialize};

macro_rules! define_capability_fields {
    ($($field:ident, $idx_const:ident, $idx_val:expr, $getter:ident, $setter:ident);* $(;)?) => {
        $(pub const $idx_const: usize = $idx_val;)*
        pub const NUM_FIELDS: usize = 23;
        pub const FIELD_NAMES: &'static [&'static str] = &[$(stringify!($field)),*];
    };
}

macro_rules! impl_capability_accessors {
    ($($field:ident, $idx_const:ident, $idx_val:expr, $getter:ident, $setter:ident);* $(;)?) => {
        $(
            pub fn $getter(&self) -> f64 { self.arr[$idx_const] }
            pub fn $setter(&mut self, val: f64) { self.arr[$idx_const] = val; }
        )*
    };
}

define_capability_fields! {
    typography, IDX_TYPOGRAPHY, 0, typography, set_typography;
    grid, IDX_GRID, 1, grid, set_grid;
    color, IDX_COLOR, 2, color, set_color;
    whitespace, IDX_WHITESPACE, 3, whitespace, set_whitespace;
    data_viz, IDX_DATA_VIZ, 4, data_viz, set_data_viz;
    emotion, IDX_EMOTION, 5, emotion, set_emotion;
    minimalism, IDX_MINIMALISM, 6, minimalism, set_minimalism;
    experimental, IDX_EXPERIMENTAL, 7, experimental, set_experimental;
    inference_depth, IDX_INFERENCE_DEPTH, 8, inference_depth, set_inference_depth;
    creativity, IDX_CREATIVITY, 9, creativity, set_creativity;
    analysis, IDX_ANALYSIS, 10, analysis, set_analysis;
    synthesis, IDX_SYNTHESIS, 11, synthesis, set_synthesis;
    domain_specificity, IDX_DOMAIN_SPECIFICITY, 12, domain_specificity, set_domain_specificity;
    accessibility, IDX_ACCESSIBILITY, 13, accessibility, set_accessibility;
    compound_composition, IDX_COMPOUND_COMPOSITION, 14, compound_composition, set_compound_composition;
    tailwind_proficiency, IDX_TAILWIND_PROFICIENCY, 15, tailwind_proficiency, set_tailwind_proficiency;
    react_aria_usage, IDX_REACT_ARIA_USAGE, 16, react_aria_usage, set_react_aria_usage;
    bem_naming, IDX_BEM_NAMING, 17, bem_naming, set_bem_naming;
    figma_integration, IDX_FIGMA_INTEGRATION, 18, figma_integration, set_figma_integration;
    ai_native_states, IDX_AI_NATIVE_STATES, 19, ai_native_states, set_ai_native_states;
    semantic_layer, IDX_SEMANTIC_LAYER, 20, semantic_layer, set_semantic_layer;
    quality_gates, IDX_QUALITY_GATES, 21, quality_gates, set_quality_gates;
    verification, IDX_VERIFICATION, 22, verification, set_verification
}

#[derive(Serialize, Deserialize)]
struct CapabilityVectorHelper {
    typography: f64,
    grid: f64,
    color: f64,
    whitespace: f64,
    data_viz: f64,
    emotion: f64,
    minimalism: f64,
    experimental: f64,
    inference_depth: f64,
    creativity: f64,
    analysis: f64,
    synthesis: f64,
    domain_specificity: f64,
    accessibility: f64,
    compound_composition: f64,
    tailwind_proficiency: f64,
    react_aria_usage: f64,
    bem_naming: f64,
    figma_integration: f64,
    ai_native_states: f64,
    semantic_layer: f64,
    quality_gates: f64,
    verification: f64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    extension: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    extension_named: Option<Vec<(String, f64)>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    provenance: Option<String>,
}

impl From<&CapabilityVector> for CapabilityVectorHelper {
    fn from(cv: &CapabilityVector) -> Self {
        Self {
            typography: cv.arr[IDX_TYPOGRAPHY],
            grid: cv.arr[IDX_GRID],
            color: cv.arr[IDX_COLOR],
            whitespace: cv.arr[IDX_WHITESPACE],
            data_viz: cv.arr[IDX_DATA_VIZ],
            emotion: cv.arr[IDX_EMOTION],
            minimalism: cv.arr[IDX_MINIMALISM],
            experimental: cv.arr[IDX_EXPERIMENTAL],
            inference_depth: cv.arr[IDX_INFERENCE_DEPTH],
            creativity: cv.arr[IDX_CREATIVITY],
            analysis: cv.arr[IDX_ANALYSIS],
            synthesis: cv.arr[IDX_SYNTHESIS],
            domain_specificity: cv.arr[IDX_DOMAIN_SPECIFICITY],
            accessibility: cv.arr[IDX_ACCESSIBILITY],
            compound_composition: cv.arr[IDX_COMPOUND_COMPOSITION],
            tailwind_proficiency: cv.arr[IDX_TAILWIND_PROFICIENCY],
            react_aria_usage: cv.arr[IDX_REACT_ARIA_USAGE],
            bem_naming: cv.arr[IDX_BEM_NAMING],
            figma_integration: cv.arr[IDX_FIGMA_INTEGRATION],
            ai_native_states: cv.arr[IDX_AI_NATIVE_STATES],
            semantic_layer: cv.arr[IDX_SEMANTIC_LAYER],
            quality_gates: cv.arr[IDX_QUALITY_GATES],
            verification: cv.arr[IDX_VERIFICATION],
            extension_named: if cv.extension.is_empty() {
                None
            } else {
                Some(cv.extension.clone())
            },
            extension: None,
            provenance: cv.provenance.clone(),
        }
    }
}

impl From<CapabilityVectorHelper> for CapabilityVector {
    fn from(h: CapabilityVectorHelper) -> Self {
        let mut arr = vec![0.0; NUM_FIELDS];
        arr[IDX_TYPOGRAPHY] = h.typography;
        arr[IDX_GRID] = h.grid;
        arr[IDX_COLOR] = h.color;
        arr[IDX_WHITESPACE] = h.whitespace;
        arr[IDX_DATA_VIZ] = h.data_viz;
        arr[IDX_EMOTION] = h.emotion;
        arr[IDX_MINIMALISM] = h.minimalism;
        arr[IDX_EXPERIMENTAL] = h.experimental;
        arr[IDX_INFERENCE_DEPTH] = h.inference_depth;
        arr[IDX_CREATIVITY] = h.creativity;
        arr[IDX_ANALYSIS] = h.analysis;
        arr[IDX_SYNTHESIS] = h.synthesis;
        arr[IDX_DOMAIN_SPECIFICITY] = h.domain_specificity;
        arr[IDX_ACCESSIBILITY] = h.accessibility;
        arr[IDX_COMPOUND_COMPOSITION] = h.compound_composition;
        arr[IDX_TAILWIND_PROFICIENCY] = h.tailwind_proficiency;
        arr[IDX_REACT_ARIA_USAGE] = h.react_aria_usage;
        arr[IDX_BEM_NAMING] = h.bem_naming;
        arr[IDX_FIGMA_INTEGRATION] = h.figma_integration;
        arr[IDX_AI_NATIVE_STATES] = h.ai_native_states;
        arr[IDX_SEMANTIC_LAYER] = h.semantic_layer;
        arr[IDX_QUALITY_GATES] = h.quality_gates;
        arr[IDX_VERIFICATION] = h.verification;
        let extension = h.extension_named.unwrap_or_else(|| {
            h.extension
                .map(|v| v.into_iter().map(|x| ("unknown".to_string(), x)).collect())
                .unwrap_or_default()
        });
        let provenance = h.provenance;
        Self {
            arr,
            extension,
            provenance,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct CapabilityVector {
    pub arr: Vec<f64>,
    pub extension: Vec<(String, f64)>,
    pub provenance: Option<String>,
}

impl Default for CapabilityVector {
    fn default() -> Self {
        Self {
            arr: vec![0.0; NUM_FIELDS],
            extension: Vec::new(),
            provenance: None,
        }
    }
}

impl Serialize for CapabilityVector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let helper: CapabilityVectorHelper = self.into();
        helper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CapabilityVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = CapabilityVectorHelper::deserialize(deserializer)?;
        Ok(helper.into())
    }
}

impl CapabilityVector {
    pub fn extend_named(&mut self, new_dims: &[(String, f64)]) {
        self.extension.extend_from_slice(new_dims);
    }

    pub fn extension(&self) -> &[(String, f64)] {
        &self.extension
    }

    pub fn extension_mut(&mut self) -> &mut Vec<(String, f64)> {
        &mut self.extension
    }

    pub fn prune_extension(&mut self) {
        self.extension.retain(|(_, x)| *x != 0.0);
    }

    pub fn merge_similar(&mut self, threshold: f64) {
        let mut i = 0;
        while i < self.extension.len() {
            let mut j = i + 1;
            while j < self.extension.len() {
                let name_same = self.extension[i].0 == self.extension[j].0;
                let val_sim = 1.0 - (self.extension[i].1 - self.extension[j].1).abs();
                if name_same || val_sim > threshold {
                    let weight_i = self.extension[i].1.abs();
                    let weight_j = self.extension[j].1.abs();
                    let total = weight_i + weight_j;
                    let merged_val = if total > 0.0 {
                        (self.extension[i].1 * weight_i + self.extension[j].1 * weight_j) / total
                    } else {
                        0.0
                    };
                    self.extension[i].1 = merged_val;
                    self.extension.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    pub fn set_provenance(&mut self, source: String) {
        self.provenance = Some(source);
    }

    pub fn provenance(&self) -> Option<&String> {
        self.provenance.as_ref()
    }

    pub fn total_dim(&self) -> usize {
        NUM_FIELDS + self.extension.len()
    }

    pub fn to_full_vector(&self) -> Vec<f64> {
        let mut full = self.arr.clone();
        full.extend(self.extension.iter().map(|(_, v)| *v));
        full
    }

    pub fn extension_values(&self) -> Vec<f64> {
        self.extension.iter().map(|(_, v)| *v).collect()
    }

    pub fn extension_names(&self) -> Vec<&str> {
        self.extension.iter().map(|(n, _)| n.as_str()).collect()
    }

    pub fn add_extension_dim(&mut self, name: &str, value: f64) {
        for (n, v) in &mut self.extension {
            if n == name {
                *v = value;
                return;
            }
        }
        self.extension.push((name.to_string(), value));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_values(
        typography: f64,
        grid: f64,
        color: f64,
        whitespace: f64,
        data_viz: f64,
        emotion: f64,
        minimalism: f64,
        experimental: f64,
        inference_depth: f64,
        creativity: f64,
        analysis: f64,
        synthesis: f64,
        domain_specificity: f64,
        accessibility: f64,
        compound_composition: f64,
        tailwind_proficiency: f64,
        react_aria_usage: f64,
        bem_naming: f64,
        figma_integration: f64,
        ai_native_states: f64,
        semantic_layer: f64,
        quality_gates: f64,
        verification: f64,
    ) -> Self {
        let arr = vec![
            typography,
            grid,
            color,
            whitespace,
            data_viz,
            emotion,
            minimalism,
            experimental,
            inference_depth,
            creativity,
            analysis,
            synthesis,
            domain_specificity,
            accessibility,
            compound_composition,
            tailwind_proficiency,
            react_aria_usage,
            bem_naming,
            figma_integration,
            ai_native_states,
            semantic_layer,
            quality_gates,
            verification,
        ];
        Self {
            arr,
            extension: Vec::new(),
            provenance: None,
        }
    }

    pub fn extension_similarity(&self, other: &CapabilityVector) -> f64 {
        if self.extension.is_empty() || other.extension.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        for (name, val) in &self.extension {
            norm_a += val * val;
            let other_val = other
                .extension
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            dot += val * other_val;
        }
        for (_, val) in &other.extension {
            norm_b += val * val;
        }
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }

    impl_capability_accessors! {
        typography, IDX_TYPOGRAPHY, 0, typography, set_typography;
        grid, IDX_GRID, 1, grid, set_grid;
        color, IDX_COLOR, 2, color, set_color;
        whitespace, IDX_WHITESPACE, 3, whitespace, set_whitespace;
        data_viz, IDX_DATA_VIZ, 4, data_viz, set_data_viz;
        emotion, IDX_EMOTION, 5, emotion, set_emotion;
        minimalism, IDX_MINIMALISM, 6, minimalism, set_minimalism;
        experimental, IDX_EXPERIMENTAL, 7, experimental, set_experimental;
        inference_depth, IDX_INFERENCE_DEPTH, 8, inference_depth, set_inference_depth;
        creativity, IDX_CREATIVITY, 9, creativity, set_creativity;
        analysis, IDX_ANALYSIS, 10, analysis, set_analysis;
        synthesis, IDX_SYNTHESIS, 11, synthesis, set_synthesis;
        domain_specificity, IDX_DOMAIN_SPECIFICITY, 12, domain_specificity, set_domain_specificity;
        accessibility, IDX_ACCESSIBILITY, 13, accessibility, set_accessibility;
        compound_composition, IDX_COMPOUND_COMPOSITION, 14, compound_composition, set_compound_composition;
        tailwind_proficiency, IDX_TAILWIND_PROFICIENCY, 15, tailwind_proficiency, set_tailwind_proficiency;
        react_aria_usage, IDX_REACT_ARIA_USAGE, 16, react_aria_usage, set_react_aria_usage;
        bem_naming, IDX_BEM_NAMING, 17, bem_naming, set_bem_naming;
        figma_integration, IDX_FIGMA_INTEGRATION, 18, figma_integration, set_figma_integration;
        ai_native_states, IDX_AI_NATIVE_STATES, 19, ai_native_states, set_ai_native_states;
        semantic_layer, IDX_SEMANTIC_LAYER, 20, semantic_layer, set_semantic_layer;
        quality_gates, IDX_QUALITY_GATES, 21, quality_gates, set_quality_gates;
        verification, IDX_VERIFICATION, 22, verification, set_verification
    }

    #[inline]
    pub fn arr(&self) -> &[f64] {
        &self.arr
    }

    #[inline]
    pub fn arr_mut(&mut self) -> &mut [f64] {
        &mut self.arr
    }

    #[inline]
    pub fn to_array(&self) -> Vec<f64> {
        self.arr.clone()
    }

    #[inline]
    pub fn from_array(arr: &[f64]) -> Result<Self, String> {
        if arr.len() != NUM_FIELDS {
            return Err(format!(
                "Expected {} dimensions, got {}",
                NUM_FIELDS,
                arr.len()
            ));
        }
        Ok(Self {
            arr: arr.to_vec(),
            extension: Vec::new(),
            provenance: None,
        })
    }

    pub fn from_full_named(full: &[(String, f64)]) -> Self {
        if full.len() <= NUM_FIELDS {
            let mut arr = vec![0.0; NUM_FIELDS];
            for (i, &(_, val)) in full.iter().enumerate().take(NUM_FIELDS) {
                arr[i] = val;
            }
            Self {
                arr,
                extension: Vec::new(),
                provenance: None,
            }
        } else {
            let mut arr = vec![0.0; NUM_FIELDS];
            for (i, &(_, val)) in full.iter().enumerate().take(NUM_FIELDS) {
                arr[i] = val;
            }
            let extension = full[NUM_FIELDS..].to_vec();
            Self {
                arr,
                extension,
                provenance: None,
            }
        }
    }

    pub fn similarity(&self, other: &CapabilityVector) -> f64 {
        let dot: f64 = self.arr.iter().zip(&other.arr).map(|(a, b)| a * b).sum();
        let norm_a: f64 = self.arr.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = other.arr.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }

    pub fn normalize(&mut self) {
        let max_val = self.arr.iter().cloned().fold(0.0f64, |acc, x| acc.max(x));
        if max_val > 1.0 {
            let scale = 1.0 / max_val;
            self.arr.iter_mut().for_each(|x| *x *= scale);
        }
    }

    pub fn dim(&self) -> usize {
        NUM_FIELDS
    }

    pub const fn const_dim() -> usize {
        NUM_FIELDS
    }

    pub fn to_vector(&self) -> Vec<f64> {
        self.arr.clone()
    }

    pub fn update_from_other(&mut self, source: &CapabilityVector, learning_rate: f64) {
        for i in 0..self.arr.len() {
            let src = source.arr.get(i).copied().unwrap_or(0.0);
            self.arr[i] += learning_rate * (src - self.arr[i]);
        }
    }

    pub fn set_field_by_name(&mut self, name: &str, value: f64) -> bool {
        if let Some(idx) = Self::index_from_name(name) {
            self.arr[idx] = value;
            true
        } else {
            false
        }
    }

    pub fn index_from_name(name: &str) -> Option<usize> {
        FIELD_NAMES.iter().position(|&n| n == name)
    }

    pub fn add_simd(&mut self, other: &[f64], start_idx: usize) {
        for (i, &val) in other.iter().enumerate() {
            let idx = start_idx + i;
            if idx < self.arr.len() {
                self.arr[idx] += val;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_vector_default() {
        let cv = CapabilityVector::default();
        assert_eq!(cv.total_dim(), 23);
    }

    #[test]
    fn test_capability_vector_from_values() {
        let cv = CapabilityVector::from_values(
            0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2,
            0.1, 0.9, 0.8, 0.7, 0.6, 0.5,
        );
        assert_eq!(cv.total_dim(), 23);
    }

    #[test]
    fn test_capability_vector_set_provenance() {
        let mut cv = CapabilityVector::default();
        cv.set_provenance("test_source".into());
        assert_eq!(cv.provenance(), Some(&"test_source".to_string()));
    }

    #[test]
    fn test_capability_vector_extension() {
        let mut cv = CapabilityVector::default();
        cv.add_extension_dim("custom", 0.95);
        assert_eq!(cv.total_dim(), 24);
        assert_eq!(cv.extension_values().len(), 1);
    }

    #[test]
    fn test_capability_vector_to_full_vector() {
        let cv = CapabilityVector::default();
        let full = cv.to_full_vector();
        assert_eq!(full.len(), 23);
    }

    #[test]
    fn test_from_array_wrong_size() {
        let arr_22 = vec![0.0; 22];
        let result = CapabilityVector::from_array(&arr_22);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 23 dimensions"));

        let arr_24 = vec![0.0; 24];
        let result = CapabilityVector::from_array(&arr_24);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 23 dimensions"));

        let arr_23 = vec![0.5; 23];
        let result = CapabilityVector::from_array(&arr_23);
        assert!(result.is_ok());
        let cv = result.expect("from_array with 23 elements should succeed");
        assert_eq!(cv.dim(), 23);
    }

    #[test]
    fn test_similarity() {
        let a = CapabilityVector::from_values(
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        let b = CapabilityVector::from_values(
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        assert!((a.similarity(&b) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize() {
        let mut cv = CapabilityVector::from_values(
            2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        cv.normalize();
        assert!((cv.typography() - 2.0 / 3.0).abs() < 1e-10);
        assert!((cv.grid() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let cv = CapabilityVector::from_values(
            0.5, 0.6, 0.7, 0.8, 0.9, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.1, 0.2, 0.3,
            0.4, 0.5, 0.6, 0.7, 0.8, 0.9,
        );
        let json = serde_json::to_string(&cv).expect("serialize should succeed");
        let deserialized: CapabilityVector =
            serde_json::from_str(&json).expect("deserialize should succeed");
        assert!((cv.similarity(&deserialized) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_set_field_by_name() {
        let mut cv = CapabilityVector::default();
        assert!(cv.set_field_by_name("typography", 0.9));
        assert!((cv.typography() - 0.9).abs() < 1e-10);
        assert!(!cv.set_field_by_name("nonexistent", 0.5));
    }
}
