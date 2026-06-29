const RMS_THRESHOLD: f32 = 0.02;
const MIN_SPEECH_DURATION_MS: u64 = 200;
const MIN_SILENCE_DURATION_MS: u64 = 600;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VadState {
    Idle,
    Speech,
}

#[derive(Debug, Clone)]
pub struct VoiceSegment {
    pub samples: Vec<f32>,
    pub start_ms: u64,
    pub end_ms: u64,
    pub duration_ms: u64,
    pub peak_amplitude: f32,
    pub avg_rms: f32,
}

pub struct VadEngine {
    pub state: VadState,
    pub threshold: f32,
    pub min_speech_ms: u64,
    pub min_silence_ms: u64,
    pub sample_rate: u32,
    speech_start_ms: Option<u64>,
    speech_samples: Vec<f32>,
    silent_samples_since_speech: u64,
}

impl VadEngine {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            state: VadState::Idle,
            threshold: RMS_THRESHOLD,
            min_speech_ms: MIN_SPEECH_DURATION_MS,
            min_silence_ms: MIN_SILENCE_DURATION_MS,
            sample_rate,
            speech_start_ms: None,
            speech_samples: Vec::new(),
            silent_samples_since_speech: 0,
        }
    }

    pub fn rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    pub fn process_frame(&mut self, frame: &[f32], frame_time_ms: u64) -> Option<VoiceSegment> {
        let rms = Self::rms(frame);
        let is_speech = rms > self.threshold;
        let frame_duration_ms = (frame.len() as u64 * 1000) / self.sample_rate as u64;

        match self.state {
            VadState::Idle => {
                if is_speech {
                    self.state = VadState::Speech;
                    self.speech_start_ms = Some(frame_time_ms);
                    self.speech_samples.clear();
                    self.speech_samples.extend_from_slice(frame);
                    self.silent_samples_since_speech = 0;
                }
            }
            VadState::Speech => {
                if is_speech {
                    self.speech_samples.extend_from_slice(frame);
                    self.silent_samples_since_speech = 0;
                } else {
                    self.silent_samples_since_speech += frame_duration_ms;
                    self.speech_samples.extend_from_slice(frame);

                    if self.silent_samples_since_speech >= self.min_silence_ms {
                        let start_ms = self.speech_start_ms.unwrap_or(frame_time_ms);
                        let duration_ms = frame_time_ms + frame_duration_ms - start_ms;
                        let peak = self
                            .speech_samples
                            .iter()
                            .map(|s| s.abs())
                            .fold(0.0_f32, f32::max);
                        let avg_rms = Self::rms(&self.speech_samples);

                        if duration_ms >= self.min_speech_ms {
                            let segment = VoiceSegment {
                                samples: std::mem::take(&mut self.speech_samples),
                                start_ms,
                                end_ms: frame_time_ms + frame_duration_ms,
                                duration_ms,
                                peak_amplitude: peak,
                                avg_rms,
                            };
                            self.state = VadState::Idle;
                            self.speech_start_ms = None;
                            self.silent_samples_since_speech = 0;
                            return Some(segment);
                        }
                        self.state = VadState::Idle;
                        self.speech_start_ms = None;
                        self.speech_samples.clear();
                        self.silent_samples_since_speech = 0;
                    }
                }
            }
        }

        None
    }

    pub fn flush(&mut self) -> Option<VoiceSegment> {
        let start_ms = self.speech_start_ms.unwrap_or(0);
        let duration_ms = if self.speech_samples.is_empty() {
            0
        } else {
            (self.speech_samples.len() as u64 * 1000) / self.sample_rate as u64
        };

        if duration_ms >= self.min_speech_ms && !self.speech_samples.is_empty() {
            let peak = self
                .speech_samples
                .iter()
                .map(|s| s.abs())
                .fold(0.0_f32, f32::max);
            let avg_rms = Self::rms(&self.speech_samples);
            let segment = VoiceSegment {
                samples: std::mem::take(&mut self.speech_samples),
                start_ms,
                end_ms: start_ms + duration_ms,
                duration_ms,
                peak_amplitude: peak,
                avg_rms,
            };
            self.state = VadState::Idle;
            self.speech_start_ms = None;
            self.silent_samples_since_speech = 0;
            return Some(segment);
        }

        self.state = VadState::Idle;
        self.speech_start_ms = None;
        self.speech_samples.clear();
        self.silent_samples_since_speech = 0;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_silence() {
        let samples = vec![0.0_f32; 1600];
        assert!(VadEngine::rms(&samples) < 0.001);
    }

    #[test]
    fn test_rms_speech() {
        let samples: Vec<f32> = (0..1600).map(|i| (i as f32 / 1600.0) * 0.5).collect();
        assert!(VadEngine::rms(&samples) > RMS_THRESHOLD);
    }

    #[test]
    fn test_vad_idle_to_speech_to_idle() {
        let mut vad = VadEngine::new(16000);
        let silence = vec![0.0_f32; 1600];
        let speech: Vec<f32> = (0..16000)
            .map(|i| ((i % 100) as f32 / 100.0) * 0.5)
            .collect();

        // 100ms of silence (Idle → Idle)
        assert!(vad.process_frame(&silence, 0).is_none());
        assert_eq!(vad.state, VadState::Idle);

        // 1000ms of speech (Idle → Speech)
        for i in 0..10 {
            assert!(vad.process_frame(&speech, 100 + i * 100).is_none());
            assert_eq!(vad.state, VadState::Speech);
        }

        // 700ms silence → segment emitted
        for i in 0..7 {
            let result = vad.process_frame(&silence, 1100 + i * 100);
            if i < 6 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
                let seg = result.unwrap();
                assert!(seg.duration_ms >= 200);
                assert_eq!(vad.state, VadState::Idle);
            }
        }
    }

    #[test]
    fn test_flush_short_speech() {
        let mut vad = VadEngine::new(16000);
        // short burst < min_speech_ms
        let speech: Vec<f32> = (0..800).map(|i| ((i % 10) as f32 / 10.0) * 0.5).collect();
        vad.process_frame(&speech, 0);
        assert!(vad.flush().is_none());
    }
}
