//! Fuzz target: format_dispatch_plan (struct construction from raw bytes)
//! Run: cargo +nightly fuzz run fuzz_dispatch_plan
//!
//! Strategy: parse first 8 bytes as task count + seed, rest as task names.
//! Exercises string formatting paths without hitting real LLM providers.

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    let n = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let seed = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let rest = &data[8..];

    use neotrix_core::l5_cognition::nt_core::nt_crt::{CrtPlan, CrtTimeScale};

    let scale = match seed % 3 {
        0 => CrtTimeScale::Gaitian,
        1 => CrtTimeScale::Huntian,
        _ => CrtTimeScale::Xuanye,
    };

    let sub_tasks: Vec<neotrix_core::l1_action::nt_core_task_dispatcher::SubTask> = (0..n.min(16))
        .map(|i| {
            let name = String::from_utf8_lossy(
                &rest.get(i * 4..i * 4 + 4).unwrap_or(&[b'.'; 4]),
            )
            .into_owned();
            neotrix_core::l1_action::nt_core_task_dispatcher::SubTask {
                id: format!("fuzz-{i}"),
                title: name.clone(),
                description: name,
                prompt: String::new(),
                context: std::collections::HashMap::new(),
                priority: 5,
                estimated_complexity: 0.5,
                required_capabilities: vec![],
                dependencies: vec![],
                crt_scale: CrtTimeScale::Gaitian,
                hexagram_bias: None,
            }
        })
        .collect();

    let decomposition = neotrix_core::l1_action::nt_core_task_dispatcher::DecompositionResult {
        original_task: "fuzz-root".into(),
        sub_tasks,
        execution_order: vec![],
        crt_plan: CrtPlan::new(scale, 60.0),
        estimated_total_time: 60.0,
        confidence: 0.5,
    };

    let _ = neotrix_core::l1_action::nt_core_task_dispatcher::format_dispatch_plan(&decomposition);
});
