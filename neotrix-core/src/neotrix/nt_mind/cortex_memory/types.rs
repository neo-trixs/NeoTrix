use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 维度视角 — 每条知识的"坐标轴"
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DimensionTag {
    // 时间链
    TimelineGeology,      // 地质年代
    TimelineLife,         // 生命进化
    TimelineHuman,        // 人类进化
    TimelineCivilization, // 文明史
    TimelineFuture,       // 未来预测
    // 文明链
    CivilizationRise,   // 文明兴起
    CivilizationFall,   // 文明衰落
    CivilizationTheory, // 文明理论
    // 科技链
    TechAgriculture, // 农业革命
    TechIndustrial,  // 工业革命
    TechInformation, // 信息革命
    TechSpace,       // 太空技术
    TechAI,          // 人工智能
    // 物种链
    SpeciesEvolution,  // 物种进化
    SpeciesExtinction, // 物种灭绝
    SpeciesHuman,      // 人类物种
    // 地理链
    GeoGeology,   // 地质
    GeoClimate,   // 气候
    GeoEcosystem, // 生态
    // 宇宙链
    CosmoSpacetime,  // 时空
    CosmoMultiverse, // 多元宇宙
    CosmoDimension,  // 维度
    // 知识链
    KnowledgePhilosophy, // 哲学
    KnowledgeScience,    // 科学
    KnowledgeCulture,    // 文化
    // 通用
    General,
}

impl DimensionTag {
    pub fn all() -> Vec<DimensionTag> {
        use DimensionTag::*;
        vec![
            TimelineGeology,
            TimelineLife,
            TimelineHuman,
            TimelineCivilization,
            TimelineFuture,
            CivilizationRise,
            CivilizationFall,
            CivilizationTheory,
            TechAgriculture,
            TechIndustrial,
            TechInformation,
            TechSpace,
            TechAI,
            SpeciesEvolution,
            SpeciesExtinction,
            SpeciesHuman,
            GeoGeology,
            GeoClimate,
            GeoEcosystem,
            CosmoSpacetime,
            CosmoMultiverse,
            CosmoDimension,
            KnowledgePhilosophy,
            KnowledgeScience,
            KnowledgeCulture,
        ]
    }

    pub fn category(&self) -> &'static str {
        use DimensionTag::*;
        match self {
            TimelineGeology | TimelineLife | TimelineHuman | TimelineCivilization
            | TimelineFuture => "时间链",
            CivilizationRise | CivilizationFall | CivilizationTheory => "文明链",
            TechAgriculture | TechIndustrial | TechInformation | TechSpace | TechAI => "科技链",
            SpeciesEvolution | SpeciesExtinction | SpeciesHuman => "物种链",
            GeoGeology | GeoClimate | GeoEcosystem => "地理链",
            CosmoSpacetime | CosmoMultiverse | CosmoDimension => "宇宙链",
            KnowledgePhilosophy | KnowledgeScience | KnowledgeCulture => "知识链",
            General => "通用",
        }
    }

    pub fn detect(title: &str, text: &str) -> Vec<DimensionTag> {
        let mut dims = Vec::new();
        let t = title.to_lowercase();
        let b = text.to_lowercase();

        if b.contains("billion years")
            || b.contains("million years")
            || b.contains("geologic")
            || b.contains("era")
        {
            dims.push(DimensionTag::TimelineGeology);
        }
        if b.contains("evolution")
            || b.contains("natural selection")
            || b.contains("common descent")
            || b.contains("life")
        {
            dims.push(DimensionTag::TimelineLife);
        }
        if b.contains("human evolution")
            || b.contains("hominin")
            || b.contains("australopithecus")
            || b.contains("homo")
        {
            dims.push(DimensionTag::TimelineHuman);
        }
        if b.contains("civilization")
            || b.contains("empire")
            || b.contains("dynasty")
            || b.contains("kingdom")
        {
            dims.push(DimensionTag::TimelineCivilization);
        }
        if b.contains("future")
            || b.contains("prediction")
            || b.contains("scenario")
            || b.contains("2050")
        {
            dims.push(DimensionTag::TimelineFuture);
        }
        if b.contains("rise and fall") || b.contains("collapse") {
            dims.push(DimensionTag::CivilizationRise);
            dims.push(DimensionTag::CivilizationFall);
        }
        if t.contains("spengler") || t.contains("toynbee") || t.contains("axial") {
            dims.push(DimensionTag::CivilizationTheory);
        }
        if b.contains("agriculture") || b.contains("neolithic") || b.contains("domestication") {
            dims.push(DimensionTag::TechAgriculture);
        }
        if b.contains("industrial")
            || b.contains("steam")
            || b.contains("factory")
            || b.contains("manufacturing")
        {
            dims.push(DimensionTag::TechIndustrial);
        }
        if b.contains("information")
            || b.contains("computer")
            || b.contains("digital")
            || b.contains("internet")
        {
            dims.push(DimensionTag::TechInformation);
        }
        if b.contains("space")
            || b.contains("rocket")
            || b.contains("satellite")
            || b.contains("nasa")
        {
            dims.push(DimensionTag::TechSpace);
        }
        if b.contains("artificial intelligence")
            || b.contains("machine learning")
            || b.contains("neural")
        {
            dims.push(DimensionTag::TechAI);
        }
        if b.contains("species") || b.contains("extinction") || b.contains("biodiversity") {
            dims.push(DimensionTag::SpeciesExtinction);
        }
        if b.contains("plate")
            || b.contains("tectonic")
            || b.contains("continent")
            || b.contains("ocean")
        {
            dims.push(DimensionTag::GeoGeology);
        }
        if b.contains("climate") || b.contains("temperature") || b.contains("global warming") {
            dims.push(DimensionTag::GeoClimate);
        }
        if b.contains("ecosystem")
            || b.contains("habitat")
            || b.contains("organism")
            || b.contains("species")
        {
            dims.push(DimensionTag::GeoEcosystem);
        }
        if b.contains("spacetime") || b.contains("relativity") || b.contains("time") {
            dims.push(DimensionTag::CosmoSpacetime);
        }
        if b.contains("multiverse") || b.contains("parallel universe") {
            dims.push(DimensionTag::CosmoMultiverse);
        }
        if b.contains("dimension") || b.contains("string theory") || b.contains("brane") {
            dims.push(DimensionTag::CosmoDimension);
        }
        if b.contains("philosophy") || b.contains("ethics") || b.contains("consciousness") {
            dims.push(DimensionTag::KnowledgePhilosophy);
        }
        if b.contains("science")
            || b.contains("physics")
            || b.contains("biology")
            || b.contains("chemistry")
        {
            dims.push(DimensionTag::KnowledgeScience);
        }
        if b.contains("culture")
            || b.contains("religion")
            || b.contains("art")
            || b.contains("language")
        {
            dims.push(DimensionTag::KnowledgeCulture);
        }

        if dims.is_empty() {
            dims.push(DimensionTag::General);
        }
        dims
    }
}

// ============================================================
// 模态 — 信息载体类型
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modality {
    Text,
    Image {
        format: String,
        caption: String,
    },
    Video {
        format: String,
        duration_secs: Option<f64>,
    },
    Audio {
        format: String,
        duration_secs: Option<f64>,
    },
    WebPage,
    StructuredData,
    KnowledgeSource,
    ReasoningTrace,
}

impl Modality {
    pub fn name(&self) -> &'static str {
        match self {
            Modality::Text => "text",
            Modality::Image { .. } => "image",
            Modality::Video { .. } => "video",
            Modality::Audio { .. } => "audio",
            Modality::WebPage => "webpage",
            Modality::StructuredData => "structured",
            Modality::KnowledgeSource => "knowledge",
            Modality::ReasoningTrace => "reasoning",
        }
    }
}

// ============================================================
// 核心：记忆痕迹
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrace {
    pub id: String,
    pub timestamp: i64,
    pub source: String,
    pub source_type: String,
    pub title: String,
    pub summary: String,
    pub modality: Modality,
    pub dimensions: Vec<DimensionTag>,
    pub embedding: Option<Vec<f64>>,
    pub tags: Vec<String>,
    pub importance: f64,
    pub associations: Vec<String>,
    pub content_path: Option<String>,
    pub content_length: usize,
    pub metadata: HashMap<String, String>,
}

impl MemoryTrace {
    pub fn new(
        title: &str,
        source: &str,
        summary: &str,
        modality: Modality,
        dimensions: Vec<DimensionTag>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp(),
            source: source.to_string(),
            source_type: Self::detect_source_type(source),
            title: title.to_string(),
            summary: summary.to_string(),
            modality,
            dimensions,
            embedding: None,
            tags: Vec::new(),
            importance: 0.5,
            associations: Vec::new(),
            content_path: None,
            content_length: summary.len(),
            metadata: HashMap::new(),
        }
    }

    pub(crate) fn detect_source_type(source: &str) -> String {
        let s = source.to_lowercase();
        if s.contains("wikipedia") {
            "wikipedia".to_string()
        } else if s.contains("arxiv") {
            "arxiv".to_string()
        } else if s.contains("github") {
            "github".to_string()
        } else if s.contains(".pdf") || s.contains("paper") {
            "paper".to_string()
        } else if s.contains("youtube") || s.contains("video") {
            "video".to_string()
        } else if s.contains("image") || s.contains("photo") {
            "image".to_string()
        } else {
            "web".to_string()
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f64>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    const MAX_ASSOCIATIONS: usize = 10000;

    pub fn with_association(mut self, trace_id: &str) -> Self {
        self.associations.push(trace_id.to_string());
        if self.associations.len() > Self::MAX_ASSOCIATIONS {
            self.associations.drain(0..Self::MAX_ASSOCIATIONS / 5);
        }
        self
    }
}

// ============================================================
// HyperMem 层级 — 超图记忆三层层级 (EverOS HyperMem)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLayer {
    Sensory,
    Topic,
    Event,
    Fact,
}

impl MemoryLayer {
    pub fn all() -> Vec<MemoryLayer> {
        vec![
            MemoryLayer::Sensory,
            MemoryLayer::Topic,
            MemoryLayer::Event,
            MemoryLayer::Fact,
        ]
    }
    pub fn next(&self) -> Option<MemoryLayer> {
        match self {
            MemoryLayer::Sensory => Some(MemoryLayer::Topic),
            MemoryLayer::Topic => Some(MemoryLayer::Event),
            MemoryLayer::Event => Some(MemoryLayer::Fact),
            MemoryLayer::Fact => None,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            MemoryLayer::Sensory => "nt_world_sense",
            MemoryLayer::Topic => "topic",
            MemoryLayer::Event => "event",
            MemoryLayer::Fact => "fact",
        }
    }

    pub fn update_frequency(&self) -> usize {
        match self {
            MemoryLayer::Sensory => 1,
            MemoryLayer::Topic => 1,
            MemoryLayer::Event => 3,
            MemoryLayer::Fact => 7,
        }
    }

    pub fn promote_threshold(&self) -> f64 {
        match self {
            MemoryLayer::Sensory => 0.4,
            MemoryLayer::Topic => 0.6,
            MemoryLayer::Event => 0.8,
            MemoryLayer::Fact => 0.95,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CmsConfig {
    pub topic_frequency: usize,
    pub event_frequency: usize,
    pub fact_frequency: usize,
    pub topic_threshold: f64,
    pub event_threshold: f64,
    pub fact_threshold: f64,
}

impl Default for CmsConfig {
    fn default() -> Self {
        Self {
            topic_frequency: MemoryLayer::Topic.update_frequency(),
            event_frequency: MemoryLayer::Event.update_frequency(),
            fact_frequency: MemoryLayer::Fact.update_frequency(),
            topic_threshold: MemoryLayer::Topic.promote_threshold(),
            event_threshold: MemoryLayer::Event.promote_threshold(),
            fact_threshold: MemoryLayer::Fact.promote_threshold(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CmsResult {
    pub nt_world_sense_to_topic: usize,
    pub topic_to_event: usize,
    pub event_to_fact: usize,
    pub topic_layer_size: usize,
    pub event_layer_size: usize,
    pub fact_layer_size: usize,
}

impl std::ops::Add for CmsResult {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        CmsResult {
            nt_world_sense_to_topic: self.nt_world_sense_to_topic + other.nt_world_sense_to_topic,
            topic_to_event: self.topic_to_event + other.topic_to_event,
            event_to_fact: self.event_to_fact + other.event_to_fact,
            topic_layer_size: other.topic_layer_size,
            event_layer_size: other.event_layer_size,
            fact_layer_size: other.fact_layer_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CortexStats {
    pub nt_world_sense_count: usize,
    pub long_term_count: usize,
    pub total_traces: usize,
    pub per_dimension: HashMap<String, usize>,
    pub per_modality: HashMap<String, usize>,
}
