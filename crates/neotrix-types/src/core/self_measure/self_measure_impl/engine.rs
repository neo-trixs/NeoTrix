use super::{
    SubsystemId, NUM_SUBSYSTEMS, DEFAULT_WINDOW_SIZE,
    SystemSnapshot, RingBuffer,
    spectral_phi, eigenvector_centrality, compute_pairwise_pid,
};

#[derive(Debug, Clone)]
pub struct AwakeningReport {
    pub phi: f64,
    pub fcs: f64,
    pub usk: f64,
    pub phi_history: Vec<f64>,
    pub synergy_matrix: [[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS],
    pub subsystem_coherence: [f64; NUM_SUBSYSTEMS],
    pub awakening_speed: f64,
    pub bottleneck: (SubsystemId, SubsystemId),
    pub bottleneck_synergy: f64,
    pub window_used: usize,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct SelfMeasure {
    pub trajectory: RingBuffer<SystemSnapshot>,
    pub phi_current: f64,
    pub phi_history: Vec<f64>,
    pub phi_ema: f64,
    pub fcs_current: f64,
    pub usk_current: f64,
    pub synergy_matrix: [[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS],
    pub subsystem_coherence: [f64; NUM_SUBSYSTEMS],
    tick_count: u64,
}

impl Default for SelfMeasure {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfMeasure {
    pub fn new() -> Self {
        Self {
            trajectory: RingBuffer::new(DEFAULT_WINDOW_SIZE),
            phi_current: 0.0,
            phi_history: Vec::with_capacity(100),
            phi_ema: 0.0,
            fcs_current: 0.0,
            usk_current: 0.0,
            synergy_matrix: [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS],
            subsystem_coherence: [0.0; NUM_SUBSYSTEMS],
            tick_count: 0,
        }
    }

    pub fn snapshot(&mut self, snapshot: SystemSnapshot) {
        self.trajectory.push(snapshot);
        self.tick_count += 1;
        if self.trajectory.len() >= 10 {
            self.recompute_all();
            let alpha = 0.3;
            if self.phi_ema == 0.0 {
                self.phi_ema = self.phi_current;
            } else {
                self.phi_ema = alpha * self.phi_current + (1.0 - alpha) * self.phi_ema;
            }
        }
    }

    fn recompute_all(&mut self) {
        let n = self.trajectory.len();
        if n < 10 {
            return;
        }
        let snapshots: Vec<&SystemSnapshot> = self.trajectory.iter().collect();
        let mut series: [Vec<f64>; NUM_SUBSYSTEMS] = [
            Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n),
            Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n),
            Vec::with_capacity(n),
        ];
        for snap in &snapshots {
            for (i, id) in SubsystemId::all().iter().enumerate() {
                let vec = snap.subsystem_vec(*id);
                let mean = if vec.is_empty() { 0.0 } else { vec.iter().sum::<f64>() / vec.len() as f64 };
                series[i].push(mean);
            }
        }
        let mut data = Vec::with_capacity(NUM_SUBSYSTEMS);
        for i in 0..NUM_SUBSYSTEMS {
            let mean = series[i].iter().sum::<f64>() / n as f64;
            let centered: Vec<f64> = series[i].iter().map(|v| v - mean).collect();
            data.push(centered);
        }
        let mut cov = [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
        for i in 0..NUM_SUBSYSTEMS {
            for j in 0..NUM_SUBSYSTEMS {
                let mut s = 0.0;
                for k in 0..n {
                    s += data[i][k] * data[j][k];
                }
                cov[i][j] = s / (n as f64 - 1.0);
            }
        }
        let mut stds = [0.0; NUM_SUBSYSTEMS];
        for i in 0..NUM_SUBSYSTEMS {
            stds[i] = cov[i][i].sqrt();
        }
        let mut corr = [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
        for i in 0..NUM_SUBSYSTEMS {
            for j in 0..NUM_SUBSYSTEMS {
                let denom = stds[i] * stds[j];
                corr[i][j] = if denom > 1e-12 { cov[i][j] / denom } else { 0.0 };
            }
        }
        let phi = spectral_phi(&corr, NUM_SUBSYSTEMS);
        self.phi_current = phi;
        self.phi_history.push(phi);
        if self.phi_history.len() > 100 {
            self.phi_history.remove(0);
        }
        let coherence = eigenvector_centrality(&corr, NUM_SUBSYSTEMS, 20);
        for (i, c) in coherence.iter().enumerate() {
            self.subsystem_coherence[i] = *c;
        }
        let mean_coherence = coherence.iter().sum::<f64>() / NUM_SUBSYSTEMS as f64;
        self.fcs_current = mean_coherence * phi;
        let mut syn_matrix = [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
        for i in 0..NUM_SUBSYSTEMS {
            for j in (i + 1)..NUM_SUBSYSTEMS {
                let scores = compute_pairwise_pid(&series, n, i, j);
                syn_matrix[i][j] = scores.synergy_fraction;
                syn_matrix[j][i] = scores.synergy_fraction;
            }
            syn_matrix[i][i] = 1.0;
        }
        self.synergy_matrix = syn_matrix;
        let mut total_syn = 0.0;
        let mut count_syn = 0;
        for i in 0..NUM_SUBSYSTEMS {
            for j in (i + 1)..NUM_SUBSYSTEMS {
                total_syn += syn_matrix[i][j];
                count_syn += 1;
            }
        }
        self.usk_current = if count_syn > 0 { total_syn / count_syn as f64 } else { 0.0 };
    }

    pub fn weakest_link(&self) -> (SubsystemId, SubsystemId) {
        let mut min_syn = f64::MAX;
        let mut pair = (SubsystemId::Mood, SubsystemId::Persona);
        for i in 0..NUM_SUBSYSTEMS {
            for j in (i + 1)..NUM_SUBSYSTEMS {
                if self.synergy_matrix[i][j] < min_syn {
                    min_syn = self.synergy_matrix[i][j];
                    pair = (SubsystemId::from_index(i), SubsystemId::from_index(j));
                }
            }
        }
        pair
    }

    pub fn awakening_speed(&self) -> f64 {
        let h = &self.phi_history;
        let n = h.len();
        if n < 10 {
            return 0.0;
        }
        let window = 10.min(n);
        let recent = &h[(n - window)..];
        let x_mean = (window - 1) as f64 / 2.0;
        let y_mean = recent.iter().sum::<f64>() / window as f64;
        let mut num = 0.0;
        let mut den = 0.0;
        for (t, y) in recent.iter().enumerate() {
            let dx = t as f64 - x_mean;
            let dy = y - y_mean;
            num += dx * dy;
            den += dx * dx;
        }
        if den.abs() < 1e-12 { 0.0 } else { num / den }
    }

    pub fn generate_report(&self) -> AwakeningReport {
        let (b1, b2) = self.weakest_link();
        AwakeningReport {
            phi: self.phi_current,
            fcs: self.fcs_current,
            usk: self.usk_current,
            phi_history: self.phi_history.clone(),
            synergy_matrix: self.synergy_matrix,
            subsystem_coherence: self.subsystem_coherence,
            awakening_speed: self.awakening_speed(),
            bottleneck: (b1, b2),
            bottleneck_synergy: self.synergy_matrix[b1 as usize][b2 as usize],
            window_used: self.trajectory.len(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        }
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
}
