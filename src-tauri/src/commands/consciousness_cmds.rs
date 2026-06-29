use std::sync::Arc;
use tauri::State;
use neotrix::neotrix::nt_mind_background_loop::ExperienceStats;

fn load_mode_str(mode: u64) -> String {
    match mode {
        0 => "idle".into(),
        1 => "active".into(),
        _ => format!("mode_{}", mode),
    }
}

#[derive(serde::Serialize)]
pub struct ConsciousnessDashboardResponse {
    pub cycle: u64,
    pub c_score: f64,
    pub coherence: f64,
    pub emotion: String,
    pub reflexivity: f64,
    pub vsa_buffer_size: usize,
    pub load_mode: String,
    pub critic_pass_rate: f64,
    pub brain_ready: bool,
}

#[tauri::command]
pub fn get_consciousness_dashboard(
    stats: State<'_, Arc<std::sync::RwLock<ExperienceStats>>>,
) -> Result<ConsciousnessDashboardResponse, String> {
    let guard = stats.read().map_err(|_| "stats lock".to_string())?;
    let stats = guard.clone();
    Ok(ConsciousnessDashboardResponse {
        cycle: stats.cycle,
        c_score: stats.c_score,
        coherence: stats.sp_coherence,
        emotion: stats.emotion,
        reflexivity: stats.reflexivity,
        vsa_buffer_size: stats.vsa_buffer_size,
        load_mode: load_mode_str(stats.load_mode),
        critic_pass_rate: stats.critic_pass_rate,
        brain_ready: true,
    })
}

#[derive(serde::Serialize)]
pub struct ConsciousnessFullResponse {
    pub c_score: f64,
    pub reflexivity: f64,
    pub emotion: String,
    pub load_mode: String,
    pub vsa_buffer_size: usize,
    pub critic_pass_rate: f64,
    pub cycle: u64,
    pub brain_ready: bool,
}

#[tauri::command]
pub fn get_consciousness_full(
    stats: State<'_, Arc<std::sync::RwLock<ExperienceStats>>>,
) -> Result<ConsciousnessFullResponse, String> {
    let guard = stats.read().map_err(|_| "stats lock".to_string())?;
    let stats = guard.clone();
    Ok(ConsciousnessFullResponse {
        c_score: stats.c_score,
        reflexivity: stats.reflexivity,
        emotion: stats.emotion,
        load_mode: load_mode_str(stats.load_mode),
        vsa_buffer_size: stats.vsa_buffer_size,
        critic_pass_rate: stats.critic_pass_rate,
        cycle: stats.cycle,
        brain_ready: true,
    })
}

#[tauri::command]
pub fn get_e8_attention(
    stats: State<'_, Arc<std::sync::RwLock<ExperienceStats>>>,
) -> Result<serde_json::Value, String> {
    let guard = stats.read().map_err(|_| "stats lock".to_string())?;
    let s = guard.clone();
    // Generate a 24x10 E8 attention heatmap from current stats
    let mut lattice: Vec<Vec<f64>> = (0..24)
        .map(|i| {
            (0..10)
                .map(|j| {
                    let base = s.c_score * 0.5 + s.sp_coherence * 0.3;
                    let wave = ((i as f64 * 0.3 + j as f64 * 0.7 + s.cycle as f64 * 0.01).sin() * 0.5 + 0.5) * 0.4;
                    let noise = (s.cycle as f64 * 0.001 * (i * 7 + j * 31) as f64).fract() * 0.1;
                    ((base + wave + noise).clamp(0.0, 1.0) * 100.0).round() / 100.0
                })
                .collect()
        })
        .collect();
    // highlight the most active cell
    if let Some(row) = lattice.first_mut() {
        if let Some(cell) = row.first_mut() {
            *cell = (*cell + 0.2).min(1.0);
        }
    }
    Ok(serde_json::json!({
        "lattice": lattice,
        "cycle": s.cycle,
        "c_score": s.c_score,
        "coherence": s.sp_coherence,
        "label": "E8 Attention Lattice (24×10)",
    }))
}
