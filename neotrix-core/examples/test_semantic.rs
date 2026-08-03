fn main() {
    use neotrix::neotrix::nt_memory_kb::KnowledgeBase;
    use neotrix::neotrix::nt_memory_kb::nt_memory_embed::EmbeddingConfig;
    let cfg = EmbeddingConfig::default();
    let kb = KnowledgeBase::open(None).expect("open").with_embedding(cfg);
    let r = kb.pq_search("深度学习注意力机制", 5);
    match r {
        Ok(res) => {
            println!("pq_search {} hits:", res.len());
            for x in res.iter().take(5) {
                println!("  [{}] {} score={:.3}", x.node.node_type.as_str(), x.node.title, x.score);
            }
        }
        Err(e) => println!("ERR: {}", e),
    }
    // Compare with semantic (full cosine)
    let r2 = kb.semantic_search("深度学习注意力机制", 5).unwrap();
    println!("\nsemantic (full cosine) {} hits:", r2.len());
    for x in r2.iter().take(5) {
        println!("  [{}] {} score={:.3}", x.node.node_type.as_str(), x.node.title, x.score);
    }
}
