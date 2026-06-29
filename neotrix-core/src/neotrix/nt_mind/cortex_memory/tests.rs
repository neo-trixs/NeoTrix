use super::*;

#[test]
fn test_cortex_new() {
    let cx = CortexMemory::new(10, 100);
    assert_eq!(cx.nt_world_sense_buffer.len(), 0);
    assert_eq!(cx.long_term.len(), 0);
}

#[test]
fn test_store_trace() {
    let mut cx = CortexMemory::new(10, 100);
    let trace = MemoryTrace::new(
        "Test",
        "wiki",
        "summary text here",
        Modality::Text,
        vec![DimensionTag::General],
    );
    let id = cx.store(trace);
    assert!(!id.is_empty());
    assert_eq!(cx.nt_world_sense_buffer.len(), 1);
}

#[test]
fn test_dimension_detection() {
    let dims = DimensionTag::detect(
        "History of Earth",
        "billion years ago geologic era evolution of life",
    );
    assert!(dims.contains(&DimensionTag::TimelineGeology));
    assert!(dims.contains(&DimensionTag::TimelineLife));
}

#[test]
fn test_query_by_dimension() {
    let mut cx = CortexMemory::new(10, 100);
    let t1 = MemoryTrace::new(
        "Geo",
        "wiki",
        "plate tectonics continental drift",
        Modality::Text,
        vec![DimensionTag::GeoGeology],
    );
    let t2 = MemoryTrace::new(
        "Life",
        "wiki",
        "evolution natural selection",
        Modality::Text,
        vec![DimensionTag::TimelineLife],
    );
    let t3 = MemoryTrace::new(
        "Space",
        "wiki",
        "spacetime relativity",
        Modality::Text,
        vec![DimensionTag::CosmoSpacetime],
    );
    cx.store(t1);
    cx.store(t2);
    cx.store(t3);

    let geo = cx.query_by_dimension(DimensionTag::GeoGeology, 10);
    assert_eq!(geo.len(), 1);
    assert!(geo[0].title.contains("Geo"));

    let life = cx.query_by_dimension(DimensionTag::TimelineLife, 10);
    assert_eq!(life.len(), 1);
}

#[test]
fn test_dimension_chain() {
    let mut cx = CortexMemory::new(10, 100);
    let t1 = MemoryTrace::new(
        "Geo",
        "wiki",
        "plate tectonics",
        Modality::Text,
        vec![DimensionTag::GeoGeology],
    );
    let t2 = MemoryTrace::new(
        "Climate",
        "wiki",
        "climate change",
        Modality::Text,
        vec![DimensionTag::GeoClimate],
    );
    cx.store(t1);
    cx.store(t2);

    let chain = cx.dimension_chain("地理链", 10);
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_consolidate() {
    let mut cx = CortexMemory::new(10, 5);
    cx.store(
        MemoryTrace::new(
            "A",
            "src",
            "summary a",
            Modality::Text,
            vec![DimensionTag::General],
        )
        .with_importance(0.8),
    );
    cx.store(
        MemoryTrace::new(
            "B",
            "src",
            "summary b",
            Modality::Text,
            vec![DimensionTag::General],
        )
        .with_importance(0.3),
    );
    cx.store(
        MemoryTrace::new(
            "C",
            "src",
            "summary c",
            Modality::Text,
            vec![DimensionTag::General],
        )
        .with_importance(0.9),
    );

    let consolidated = cx.consolidate(0.5);
    assert!(consolidated >= 2);
    assert_eq!(cx.nt_world_sense_buffer.len(), 1);
}

#[test]
fn test_recall() {
    let mut cx = CortexMemory::new(10, 100);
    cx.store(
        MemoryTrace::new(
            "Dinosaur extinction",
            "wiki",
            "asteroid impact killed dinosaurs",
            Modality::Text,
            vec![DimensionTag::SpeciesExtinction],
        )
        .with_importance(0.9),
    );
    cx.store(
        MemoryTrace::new(
            "Modern art",
            "wiki",
            "abstract expressionism painting",
            Modality::Text,
            vec![DimensionTag::KnowledgeCulture],
        )
        .with_importance(0.5),
    );

    let results = cx.recall("dinosaur asteroid extinction", 5);
    assert!(!results.is_empty());
    assert!(results[0].0.title.contains("Dinosaur"));
}

#[test]
fn test_query_by_modality() {
    let mut cx = CortexMemory::new(10, 100);
    cx.store(MemoryTrace::new(
        "Text article",
        "wiki",
        "text content",
        Modality::Text,
        vec![DimensionTag::General],
    ));
    let results = cx.query_by_modality("text", 10);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_detect_source_type() {
    assert_eq!(
        MemoryTrace::detect_source_type("https://en.wikipedia.org/wiki/Earth"),
        "wikipedia"
    );
    assert_eq!(
        MemoryTrace::detect_source_type("https://arxiv.org/abs/2501.00001"),
        "arxiv"
    );
    assert_eq!(
        MemoryTrace::detect_source_type("https://github.com/rust-lang/rust"),
        "github"
    );
}

#[test]
fn test_dimension_category() {
    assert_eq!(DimensionTag::TimelineGeology.category(), "时间链");
    assert_eq!(DimensionTag::CivilizationTheory.category(), "文明链");
    assert_eq!(DimensionTag::TechAI.category(), "科技链");
    assert_eq!(DimensionTag::CosmoSpacetime.category(), "宇宙链");
}

#[test]
fn test_export_json() {
    let mut cx = CortexMemory::new(5, 10);
    cx.store(MemoryTrace::new(
        "Test",
        "wiki",
        "test summary",
        Modality::Text,
        vec![DimensionTag::General],
    ));
    let json = cx.export_json();
    assert_eq!(json["total_traces"].as_i64().expect("result"), 1);
}

#[test]
fn test_stats() {
    let mut cx = CortexMemory::new(5, 10);
    cx.store(MemoryTrace::new(
        "T1",
        "s1",
        "text",
        Modality::Text,
        vec![DimensionTag::General],
    ));
    let stats = cx.stats();
    assert_eq!(stats.total_traces, 1);
}
