//! Fuzz target: confidence_for_text (pure function, string input)
//! Run: cargo +nightly fuzz run fuzz_confidence

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = neotrix_core::l1_action::nt_crystal_llm_bridge::confidence_for_text(s, 0.5);
        let _ = neotrix_core::l1_action::nt_crystal_llm_bridge::confidence_for_text(s, 0.0);
        let _ = neotrix_core::l1_action::nt_crystal_llm_bridge::confidence_for_text(s, 1.0);
    }
});
