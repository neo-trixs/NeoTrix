use super::context_artifacts::{Artifact, ArtifactIndexer, ArtifactStore, ArtifactType};
use super::core::BrainMutView;
use super::memory::ReasoningBank;
use super::reasoning_engine::ReasoningEngine;
use super::self_iterating::ReasoningBrain;
use std::path::Path;

#[test]
fn test_engine_with_artifact_indexer_produces_artifact_context() {
    let brain: Box<dyn BrainMutView> = Box::new(ReasoningBrain::new());
    let bank = ReasoningBank::new(100);

    let mut store = ArtifactStore::new();
    store.store(
        Artifact::new(
            "users_db",
            ArtifactType::DatabaseSchema,
            "CREATE TABLE users (id INT)",
        )
        .with_tags(&["database"]),
    );
    store.store(
        Artifact::new("payment_api", ArtifactType::ApiSpec, "POST /payments").with_tags(&["api"]),
    );

    let mut indexer = ArtifactIndexer::new(Path::new("/tmp/dummy.json"));
    *indexer.store_mut() = store;

    let engine = ReasoningEngine::from_env(brain, bank).with_artifact_indexer(indexer);

    let context = engine.build_artifact_context("payment");
    assert!(context.contains("Relevant project artifacts"));
    assert!(context.contains("payment_api"));
}

#[test]
fn test_engine_without_artifact_indexer_works_normally() {
    let brain: Box<dyn BrainMutView> = Box::new(ReasoningBrain::new());
    let bank = ReasoningBank::new(100);
    let engine = ReasoningEngine::from_env(brain, bank);

    let context = engine.build_artifact_context("anything");
    assert!(context.is_empty());
}
