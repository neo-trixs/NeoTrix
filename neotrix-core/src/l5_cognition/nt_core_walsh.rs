// ── 单点真身收敛（2026-09-30）────────────────────────────────────────
// `WalshMemoryIndex` 的实现真身统一在 `neotrix-types`：
// `crates/neotrix-types/src/core/nt_core_walsh.rs`。
//
// 收敛依据（逐方法比对，非目测）：
// · 两侧 impl 各 **13 个方法**，其中 **11 个归一化后逐字相同**；
// · 仅 `wh_inverse` / `wh_transform` 两处不同，差异是 types 侧多了
//   `.take(WH_DIM)`。⚠️ 该 `.take()` 在当前代码里是**空操作** ——
//   `y`/`x` 都由 `vec![0.0; WH_DIM]` 构造，长度本就是 WH_DIM。
//   ⇒ **不是 bug 修复**，我没有据此报缺陷（差点误报，已核实）。
// · 两侧的 hadamard 矩阵都来自 `hexagram_hadamard()`，而该函数本身
//   已在同一轮收敛为 types 真身 ⇒ 两侧本就是同源。
//
// 为什么本模块整块删除而不是各自保留：
// 实现完全等价却存在两份，其中一份还是「冻结快照」，只会各自漂移。
//
// ⛔ 公开路径**全部保住**（不破坏任何调用方）：
// · `l5_cognition::nt_core_walsh::WalshMemoryIndex`（本模块路径）
// · `nt_action_facade::WalshMemoryIndex`（re-export）
// · `nt_feel_facade::WalshMemoryIndex`（re-export）
// 以上三条都经由本处的 `pub use` 继续可用。
pub use neotrix_types::core::nt_core_walsh::WalshMemoryIndex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let index = WalshMemoryIndex::new();
        let text = "E8 root system with 240 roots and 64 hexagram mapping";
        let emb = index.encode(text);
        let recovered = index.wh_inverse(&emb);
        let re_encoded = index.wh_transform(&recovered);
        let sim = WalshMemoryIndex::wh_dot(&emb, &re_encoded);
        let norm_emb: f64 = emb.iter().map(|x| x * x).sum::<f64>().sqrt();
        let normalized_sim = sim / (norm_emb * norm_emb);
        assert!(
            (normalized_sim - 1.0).abs() < 1e-10,
            "roundtrip failed: {}",
            normalized_sim
        );
    }

    #[test]
    fn test_store_and_search() {
        let mut index = WalshMemoryIndex::new();
        index.store("mem1", "resonance attention mechanism with GWT broadcast");
        index.store("mem2", "E8 root system and hexagram state navigation");
        index.store("mem3", "quantum error correction with surface codes");
        assert_eq!(index.len(), 3);
        let results = index.search("E8 hexagram navigation", 2);
        assert_eq!(results.len(), 2);
        // mem2 should be most relevant
        assert_eq!(results[0].1, "mem2");
    }

    #[test]
    fn test_remove_memory() {
        let mut index = WalshMemoryIndex::new();
        index.store("mem1", "first memory");
        index.store("mem2", "second memory");
        assert!(index.remove("mem1"));
        assert_eq!(index.len(), 1);
        assert!(!index.remove("nonexistent"));
        // search for mem1 content should not find it
        let results = index.search("first", 5);
        assert!(results.iter().all(|(_, id)| id != "mem1"));
    }

    #[test]
    fn test_update_memory() {
        let mut index = WalshMemoryIndex::new();
        index.store("mem1", "old content about resonance");
        let old_results = index.search("resonance", 5);
        assert_eq!(old_results[0].1, "mem1");
        index.store("mem1", "new content about E8 algebra");
        let new_results = index.search("E8 algebra", 5);
        assert_eq!(new_results[0].1, "mem1");
        // old query should no longer match as well
        let old_query_results = index.search("resonance", 5);
        if !old_query_results.is_empty() {
            assert!(old_query_results[0].0 < new_results[0].0 + 0.1);
        }
    }

    #[test]
    fn test_self_retrieval_high_score() {
        let mut index = WalshMemoryIndex::new();
        let text = "E8 reasoning engine with 64-mode state space";
        index.store("self", text);
        let results = index.search(text, 1);
        assert_eq!(results[0].1, "self");
        // Self-similarity should be very high (> 0.9)
        assert!(
            results[0].0 > 0.9,
            "self-similarity too low: {}",
            results[0].0
        );
    }

    #[test]
    fn test_noise_immunity() {
        let index = WalshMemoryIndex::new();
        let text = "Resonance between specialist modules amplifies attention to clusters";
        let original = index.encode(text);
        // Add noise at 20% of signal amplitude
        let _signal_power: f64 = original.iter().map(|x| x * x).sum::<f64>().sqrt();
        let noise: Vec<f64> = original
            .iter()
            .map(|x| x * 0.2 * (if (x * 1e6) as i64 % 2 == 0 { 1.0 } else { -1.0 }))
            .collect();
        let noisy: Vec<f64> = original
            .iter()
            .zip(noise.iter())
            .map(|(s, n)| s + n)
            .collect();
        let denoised = index.denoise(&noisy);
        let ratio = WalshMemoryIndex::recovery_ratio(&original, &denoised);
        assert!(ratio > 0.85, "noise recovery too low: {}", ratio);
    }

    #[test]
    fn test_noise_immune_retrieval_degrades_gracefully() {
        let mut index = WalshMemoryIndex::new();
        let texts = [
            "E8 root system with 240 roots",
            "Walsh-Hadamard orthogonal memory retrieval",
            "GWT broadcast with resonance boosting",
            "Quantum error correction surface code",
        ];
        for (i, text) in texts.iter().enumerate() {
            index.store(&format!("mem{}", i), text);
        }
        let query = "resonance GWT broadcast attention mechanism";
        // Clean retrieval
        let clean_results = index.search(query, 4);
        // mem2 (index 2) should be top
        assert_eq!(clean_results[0].1, "mem2");
    }

    #[test]
    fn test_cross_language_stability() {
        let index = WalshMemoryIndex::new();
        let en = index.encode("resonance attention in global workspace");
        let cn = index.encode("全局工作空间中的共振注意力");
        // Different languages about the same concept should still have some similarity
        let sim = WalshMemoryIndex::wh_dot(&en, &cn);
        let norm = en.iter().map(|x| x * x).sum::<f64>().sqrt()
            * cn.iter().map(|x| x * x).sum::<f64>().sqrt();
        let normalized = sim / norm;
        // Same concept → should be positive (not anti-correlated)
        assert!(
            normalized > -0.1,
            "cross-language anti-correlated: {}",
            normalized
        );
    }

    #[test]
    fn test_empty_index() {
        let index = WalshMemoryIndex::new();
        assert!(index.is_empty());
        let results = index.search("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_orthogonal_embeddings_distinct() {
        let index = WalshMemoryIndex::new();
        let emb1 = index.encode("quantum mechanics wave function collapse");
        let emb2 = index.encode("classical mechanics newtonian physics");
        let emb3 = index.encode("pizza recipe tomato cheese dough");
        let sim12 = WalshMemoryIndex::wh_dot(&emb1, &emb2);
        let sim13 = WalshMemoryIndex::wh_dot(&emb1, &emb3);
        let norm12 = emb1.iter().map(|x| x * x).sum::<f64>().sqrt()
            * emb2.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm13 = emb1.iter().map(|x| x * x).sum::<f64>().sqrt()
            * emb3.iter().map(|x| x * x).sum::<f64>().sqrt();
        // Physics texts should be closer to each other than to pizza
        let n12 = sim12 / norm12;
        let n13 = sim13 / norm13;
        assert!(
            n12 > n13,
            "physics-pizza similarity ({}) should be less than physics-physics ({})",
            n13,
            n12
        );
    }

    #[test]
    fn test_denoise_preserves_identity() {
        let index = WalshMemoryIndex::new();
        let text = "The 64-dim Walsh-Hadamard transform spreads signal across all dimensions";
        let original = index.encode(text);
        // Denoise without noise should preserve original
        let denoised = index.denoise(&original);
        let ratio = WalshMemoryIndex::recovery_ratio(&original, &denoised);
        assert!(ratio > 0.99, "denoise without noise degraded: {}", ratio);
    }

    #[test]
    fn test_multiple_stores_bulk_search() {
        let mut index = WalshMemoryIndex::new();
        let topics = [
            "E8 root system and hexagram state navigation",
            "resonance attention mechanism with GWT broadcast",
            "quantum error correction with surface codes",
            "Walsh-Hadamard orthogonal memory retrieval",
            "vector symbolic architecture hyperdimensional computing",
            "self-iterating reasoning engine with capability vectors",
            "MCP tool integration for Playwright browser automation",
            "SEAL self-improvement loop with external reward",
            "GoalLoop with circuit breaker and rate limiter",
            "nt_world_crawl frontier with Mercator dual-queue strategy",
            "knowledge hypercube with dimension axis projection",
            "consciousness global workspace with specialist modules",
            "E8×64 reasoning state space with 6 binary axes",
            "+1 observer meta-cognitive reflection on trajectories",
            "cross-lingual semantic bridges between Chinese and English",
            "ancient Chinese cosmology Hetu Luoshu Yijing unified field",
            "Dayan number 50 with 49 iteration cycle",
            "Zhang Heng seismoscope resonance amplification mechanism",
            "Shao Yong cosmology with 129600-year cosmic cycle",
            "Mawangdui silk manuscripts astronomy and divination",
        ];
        for (i, text) in topics.iter().enumerate() {
            index.store(&format!("mem{}", i), text);
        }
        assert_eq!(index.len(), 20);
        let results = index.search("E8 hexagram navigation state space", 3);
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|(_, id)| id == "mem0"));
    }
}
