// Test that the modules can be imported
use neotrix_core::l1_action::nt_io::nt_io_download::{
    DownloadEngine, EngineConfig, DownloadSession, DownloadSource, DownloadProgress,
    DownloadStatus, DownloadMetadata,
};

fn main() {
    // Test 1: Create default config
    let config = EngineConfig::default();
    println!("Config: chunks={}, retries={}, resume={}", 
        config.chunk_count, config.max_retries, config.enable_resume);
    
    // Test 2: Create session
    let path = std::path::PathBuf::from("./models/test.gguf");
    let session = DownloadSession::new(
        "https://example.com/model.gguf".to_string(),
        path,
        "test-user".to_string()
    );
    
    println!("Session ID: {}", session.id);
    println!("Initial status: {:?}", session.status);
    println!("Initial progress: {:?}", session.progress);
    println!("Source: {:?}", session.metadata.source);
    
    // Test 3: Check DownloadSource methods
    let source = DownloadSource::HuggingFace;
    println!("is_hf(): {}", source.is_hf());
    println!("is_github(): {}", source.is_github());
    println!("from_str('hf'): {:?}", DownloadSource::from_str("hf"));
    println!("from_str('github'): {:?}", DownloadSource::from_str("github"));
    println!("from_str('url'): {:?}", DownloadSource::from_str("url"));
    
    // Test 4: Check DownloadSource variants
    println!("HuggingFace variant: {:?}", DownloadSource::HuggingFace);
    println!("GitHub variant: {:?}", DownloadSource::GitHub);
    println!("GenericURL variant: {:?}", DownloadSource::GenericURL);
    println!("ModelRepository variant: {:?}", DownloadSource::ModelRepository);
    
    println!("\n✅ All module import tests passed!");
}
