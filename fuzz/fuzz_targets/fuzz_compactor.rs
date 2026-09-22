//! Fuzz target: LosslessCompactor::score_calls (usize input, pure logic)
//! Run: cargo +nightly fuzz run fuzz_compactor

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: u64| {
    let config = neotrix_core::l1_action::nt_harness::compaction::CompactionConfig::default();
    let compactor = neotrix_core::l1_action::nt_harness::compaction::LosslessCompactor::new(config);
    let _ = compactor.score_calls(data as usize);
});
