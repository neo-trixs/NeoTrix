use super::nt_builders::ReasoningEngine;
use super::nt_finalize_broadcast::{_split_response_into_steps, hydrate_ewhr_hypotheses};
use super::nt_prediction_fusion::{
    aggregate_video_features, referenced_image_path, referenced_video_path, sample_video_frames,
};
use super::nt_prepare_call::_detect_refusal_response;
use crate::l0_substrate::nt_core_hex::{FullReasoningState, ReasoningHexagram};

/// Compatibility shim: single representative frame (first scene frame, or the
/// uniform fallback's first frame) as PNG bytes.
#[cfg(test)]
fn sample_video_frame(path: &std::path::Path) -> Option<Vec<u8>> {
    let frames = sample_video_frames(path, 6);
    frames.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::{
        _detect_refusal_response, _split_response_into_steps, aggregate_video_features,
        hydrate_ewhr_hypotheses, referenced_image_path, referenced_video_path, sample_video_frame,
        FullReasoningState, ReasoningEngine, ReasoningHexagram,
    };

    #[test]
    fn test_detect_refusal_response_empty() {
        assert!(_detect_refusal_response(""));
        assert!(_detect_refusal_response("   "));
        assert!(_detect_refusal_response("no"));
    }

    #[test]
    fn test_detect_refusal_response_explicit() {
        assert!(_detect_refusal_response("I cannot fulfill that request."));
        assert!(_detect_refusal_response(
            "Sorry, but I cannot help with this."
        ));
        assert!(_detect_refusal_response(
            "I'm sorry, I cannot provide that information."
        ));
        assert!(_detect_refusal_response(
            "As an AI language model, I cannot do that."
        ));
        assert!(_detect_refusal_response(
            "I'm not able to assist with this request."
        ));
    }

    #[test]
    fn test_detect_refusal_response_normal() {
        assert!(!_detect_refusal_response(
            "Here is a detailed analysis of your code..."
        ));
        assert!(!_detect_refusal_response(
            "The answer to your question is..."
        ));
        assert!(!_detect_refusal_response("Let me help you with that."));
        assert!(!_detect_refusal_response("Here is the implementation:"));
    }

    #[test]
    fn test_hydrate_ewhr_hypotheses_adds_nodes() {
        use crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork;
        let mut net = HypothesisNetwork::new();
        let proposed = vec![
            "agent should adopt Direct strategy when context is high".to_string(),
            "error recovery benefits from rollback-first policy".to_string(),
        ];
        let added = hydrate_ewhr_hypotheses(&mut net, &proposed, 42);
        assert_eq!(added, 2);
        assert_eq!(net.hypotheses.len(), 2);
        assert!(net.get_hypothesis("ewhr_42_0").is_some());
        assert!(net.get_hypothesis("ewhr_42_1").is_some());
    }

    #[test]
    fn test_hydrate_ewhr_hypotheses_idempotent() {
        use crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork;
        let mut net = HypothesisNetwork::new();
        let proposed = vec!["same proposal repeated".to_string()];
        // 同一 tick 两次调用 → 第二次不重复落点
        let added1 = hydrate_ewhr_hypotheses(&mut net, &proposed, 7);
        let added2 = hydrate_ewhr_hypotheses(&mut net, &proposed, 7);
        assert_eq!(added1, 1);
        assert_eq!(added2, 0, "same tick must not duplicate hypotheses");
        assert_eq!(net.hypotheses.len(), 1);
        // 不同 tick → 允许新增 (新一轮轨迹)
        let added3 = hydrate_ewhr_hypotheses(&mut net, &proposed, 8);
        assert_eq!(added3, 1);
        assert_eq!(net.hypotheses.len(), 2);
    }

    #[test]
    fn test_hydrate_ewhr_hypotheses_long_title_truncated() {
        use crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisNetwork;
        let mut net = HypothesisNetwork::new();
        let long = "a very long hypothesis description that definitely exceeds the thirty two character title limit for display purposes".to_string();
        let added = hydrate_ewhr_hypotheses(&mut net, &[long.clone()], 1);
        assert_eq!(added, 1);
        let h = net.get_hypothesis("ewhr_1_0").expect("node exists");
        assert!(
            h.title.len() <= 33,
            "title must be truncated to 32+ellipsis, got len {}",
            h.title.len()
        );
    }

    #[test]
    fn test_split_response_into_steps_by_newline() {
        let steps = _split_response_into_steps("First step\nSecond step\nThird step");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].text, "First step");
        assert_eq!(steps[1].step_idx, 1);
        assert!(steps[0].token_count > 0);
    }

    #[test]
    fn test_split_response_into_steps_by_sentence() {
        let steps = _split_response_into_steps("No newlines. Only sentences here.");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].text, "No newlines");
    }

    #[test]
    fn test_split_response_into_steps_empty() {
        assert!(_split_response_into_steps("").is_empty());
        assert!(_split_response_into_steps("   \n  ").is_empty());
    }

    #[test]
    fn test_referenced_image_path_finds_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let png = tmp.path().join("shot.png");
        std::fs::write(&png, b"\x89PNG\r\n\x1a\nnot-valid-but-exists").unwrap();
        let task = format!("analyze this screenshot at {}", png.display());
        let found = referenced_image_path(&task);
        assert_eq!(found, Some(png));
        // Missing file → None, not a false positive.
        assert!(referenced_image_path("look at /nonexistent/foo.png").is_none());
        assert!(referenced_image_path("no media here").is_none());
    }

    #[test]
    fn test_referenced_video_path_finds_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let vid = tmp.path().join("clip.mp4");
        std::fs::write(&vid, b"not-valid-but-exists").unwrap();
        let task = format!("summarize the video {}", vid.display());
        assert_eq!(referenced_video_path(&task), Some(vid));
        assert!(referenced_video_path("analyze the /missing/thing.mp4").is_none());
        assert!(referenced_video_path("no media here").is_none());
    }

    #[test]
    fn test_sample_video_frame_gracefully_degrades() {
        // A fake video file must not panic; ffmpeg fails → None.
        let tmp = tempfile::tempdir().unwrap();
        let fake = tmp.path().join("fake.mp4");
        std::fs::write(&fake, b"not a real video").unwrap();
        let _ = sample_video_frame(&fake); // must not panic
                                           // A real video (if ffmpeg present) yields decodable frame bytes.
        let real = std::path::Path::new("/tmp/neotrix_test_src.mp4");
        if real.exists() {
            if let Some(bytes) = sample_video_frame(real) {
                assert!(!bytes.is_empty());
            }
        }
    }

    #[test]
    fn test_aggregate_video_features_dedups_blank_and_near_dup() {
        use crate::l2_perception::nt_core_e8::nt_multimodal::VisionBridge;
        // Two distinct synthetic frames: a white-with-black-text "document"
        // pattern and a solid black frame (blank → dropped).
        fn doc_frame() -> Vec<u8> {
            let mut img = image::RgbImage::new(64, 64);
            for (px, _py, p) in img.enumerate_pixels_mut() {
                let band = px / 8;
                let stripe = (band % 2) == 0;
                *p = if stripe {
                    image::Rgb([20, 20, 20])
                } else {
                    image::Rgb([240, 240, 240])
                };
            }
            let mut buf = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgb8(img)
                .write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode");
            buf.into_inner()
        }
        fn blank_frame() -> Vec<u8> {
            let mut buf = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgb8(image::RgbImage::new(64, 64))
                .write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode");
            buf.into_inner()
        }
        // dup of doc_frame (phash distance ~0)
        let mut dup = doc_frame();
        dup.push(0); // trailing bytes still decode; same pixels.

        let frames = vec![doc_frame(), blank_frame(), dup];
        let (feat, summary) = aggregate_video_features(&frames).expect("aggregate");
        assert_eq!(summary.frames, 3, "all sampled frames counted");
        assert_eq!(summary.key_frames, 1, "blank + dup dropped, one kept");
        assert!(
            !summary.classifications.contains("blank"),
            "got {}",
            summary.classifications
        );
        assert_eq!(
            feat.len(),
            crate::l2_perception::nt_core_e8::nt_multimodal::IMAGE_FEATURE_DIM
        );
        let norm: f64 = feat.iter().map(|x| x * x).sum();
        assert!((norm - 1.0).abs() < 1e-6, "pooled feature normalized");
        // Blank-only input aggregates to None.
        assert!(aggregate_video_features(&[blank_frame()]).is_none());
        let _ = VisionBridge::phash_distance(0, 0);
    }

    #[test]
    fn test_distill_trace_wires_control_distiller() {
        let mut engine = ReasoningEngine::from_env();
        let response = "First we compute the sum.\nWait, rethink.\nThen we verify.";
        engine.learn_from_trace("math", response);
        assert_eq!(
            engine.distilled_sequences.len(),
            1,
            "distillation must run via learn_from_trace (R-P36 grounding)"
        );
        let seq = &engine.distilled_sequences[0];
        assert!(
            !seq.segments.is_empty(),
            "alternating sequence must contain segments"
        );
    }

    #[test]
    fn test_train_from_distilled_closes_loop() {
        use crate::l5_cognition::nt_core_policy::E8Policy;
        use crate::l5_cognition::nt_core_prm::ProcessRewardLearner;
        let mut engine = ReasoningEngine::from_env();
        let prm = ProcessRewardLearner::new(
            E8Policy::default(),
            Box::new(crate::l5_cognition::nt_core_prm::HeuristicCoach::new("test")),
        );
        engine = engine.with_prm(prm);

        // Accumulate CONTROL_TRAIN_BATCH distilled sequences with takeover markers
        let responses = [
            "Compute the integral.\nWait, reconsider the bounds.\nThen verify the result.",
            "Solve the equation.\nActually, switch to substitution.\nCheck the algebra.",
            "Derive the formula.\nHmm, backtrack to the derivative.\nValidate step by step.",
            "Factor the polynomial.\nAlternatively use grouping.\nConfirm each factor.",
            "Simplify the fraction.\nOn second thought, use common denominator.\nVerify the simplification.",
            "Evaluate the limit.\nWait, apply L'Hopital.\nThen check continuity.",
            "Prove the theorem.\nLet me rethink the induction base.\nValidate the inductive step.",
            "Compute the determinant.\nActually, expand along the first row.\nVerify the arithmetic.",
        ];
        for r in responses {
            engine.learn_from_trace("math", r);
        }

        // Batch threshold reached → training must have run and drained sequences
        assert_eq!(
            engine.train_batch, 0,
            "train_batch must reset after training"
        );
        assert!(
            engine.distilled_sequences.len() < responses.len(),
            "training must consume distilled sequences"
        );
        if let Some(ref prm) = engine.prm {
            assert!(
                prm.learning_count >= 1,
                "PRM learning_count must advance after training"
            );
        } else {
            panic!("PRM must be configured");
        }
    }

    #[test]
    fn test_e8_state_json_roundtrip() {
        let mut engine = ReasoningEngine::from_env();
        engine.current_state = FullReasoningState::new(
            ReasoningHexagram::new(42),
            crate::l5_cognition::nt_core_hex::MetaState::new(2),
        );
        engine.state_trajectory.push(FullReasoningState::new(
            ReasoningHexagram::new(9),
            crate::l5_cognition::nt_core_hex::MetaState::new(1),
        ));
        let json = engine.e8_state_json().expect("serialize ok");
        assert!(!json.is_empty());

        let mut reloaded = ReasoningEngine::from_env();
        reloaded.load_e8_state_json(&json).expect("deserialize ok");
        assert_eq!(reloaded.current_state.mode.0, 42);
        assert_eq!(reloaded.current_state.meta.0, 2);
        assert_eq!(reloaded.state_trajectory.len(), 1);
        assert_eq!(reloaded.state_trajectory[0].mode.0, 9);
    }

    #[test]
    fn test_call_llm_wires_model_router_route() {
        // cumora 借鉴接线回归 (T3): call_llm 必须经 ModelRouter.route() 选模型,
        // 而非恒用 default_model。捕获 LlmRequest.model 断言 route 决策生效。
        use crate::l1_action::nt_core_llm::{FinishReason, LlmProvider, LlmRequest, LlmResponse, Usage};
        use crate::l0_substrate::nt_core_span::CostTracker;

        struct CapturingProvider {
            seen_model: std::sync::Mutex<Option<String>>,
            seen_max_tokens: std::sync::Mutex<Option<u32>>,
        }
        #[async_trait::async_trait]
        impl LlmProvider for CapturingProvider {
    fn set_proxy(&mut self, _proxy_url: &str) {}
            fn data_trust(&self) -> crate::l1_action::nt_core_llm::DataTrust {
                crate::l1_action::nt_core_llm::DataTrust::Trusted
            }

            async fn complete_raw(
                &self,
                request: &LlmRequest,
            ) -> Result<LlmResponse, crate::l1_action::nt_core_llm::LlmError> {
                *self.seen_model.lock().unwrap() = Some(request.model.clone());
                *self.seen_max_tokens.lock().unwrap() = Some(request.max_tokens);
                Ok(LlmResponse {
                    content: "routed response".to_string(),
                    model: request.model.clone(),
                    usage: Usage {
                        prompt_tokens: 5,
                        completion_tokens: 5,
                        total_tokens: 10,
                    },
                    finish_reason: FinishReason::Stop,
                    tool_calls: None,
                    reasoning: None,
                })
            }
            async fn stream_complete_raw(
                &self,
                _request: &LlmRequest,
            ) -> Result<
                tokio::sync::mpsc::Receiver<
                    Result<LlmResponse, crate::l1_action::nt_core_llm::LlmError>,
                >,
                crate::l1_action::nt_core_llm::LlmError,
            > {
                unimplemented!("not used in this test")
            }
        }

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio rt");
        rt.block_on(async {
            let provider = std::sync::Arc::new(CapturingProvider {
                seen_model: std::sync::Mutex::new(None),
                seen_max_tokens: std::sync::Mutex::new(None),
            });
            let provider_clone = provider.clone();
            let mut engine = ReasoningEngine::from_env().with_gateway(provider);
            engine.cost_tracker = Some(CostTracker::default());

            // 简单问候 → T0 → pinned 模型名 (cumora 模型 pin)
            let res = engine.call_llm("Hello, how are you?");
            assert!(res.is_ok(), "call_llm must succeed with routed model");
            assert_eq!(res.unwrap(), "routed response");
            let model = provider_clone.seen_model.lock().unwrap().clone();
            let max_tokens = provider_clone.seen_max_tokens.lock().unwrap().clone();
            assert_eq!(
                model.as_deref(),
                Some("codestral-latest"),
                "T0 greeting must route to pinned keyless mini model, got {:?}",
                model
            );
            assert_eq!(
                max_tokens,
                Some(256),
                "route max_tokens must flow into request"
            );
        });
    }
}
