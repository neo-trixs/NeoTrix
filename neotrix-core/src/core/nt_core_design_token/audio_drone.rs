use std::f64::consts::PI;

const SAMPLE_RATE: u32 = 44100;
const BYTES_PER_SAMPLE: u16 = 2;
const NUM_CHANNELS: u16 = 1;
const BITS_PER_SAMPLE: u16 = 16;
const DETUNE: f64 = 1.005;

pub struct WavWriter {
    data: Vec<i16>,
}

impl WavWriter {
    pub fn new() -> Self {
        WavWriter { data: Vec::new() }
    }

    pub fn sample_duration(duration_secs: f64) -> usize {
        (SAMPLE_RATE as f64 * duration_secs) as usize
    }

    pub fn push_tone(&mut self, freq: f64, amplitude: f64, num_samples: usize) {
        for i in 0..num_samples {
            let sample = sin_wave(i, freq, SAMPLE_RATE) * amplitude;
            self.data.push((sample * 32767.0).round() as i16);
        }
    }

    pub fn push_silence(&mut self, num_samples: usize) {
        self.data.extend(std::iter::repeat(0i16).take(num_samples));
    }

    pub fn write_wav(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        let data_len = self.data.len() as u32 * BYTES_PER_SAMPLE as u32;
        let riff_size = 36 + data_len;

        f.write_all(b"RIFF")?;
        f.write_all(&riff_size.to_le_bytes())?;
        f.write_all(b"WAVE")?;

        f.write_all(b"fmt ")?;
        f.write_all(&16u32.to_le_bytes())?;
        f.write_all(&1u16.to_le_bytes())?;
        f.write_all(&NUM_CHANNELS.to_le_bytes())?;
        f.write_all(&SAMPLE_RATE.to_le_bytes())?;
        f.write_all(&(SAMPLE_RATE as u32 * BYTES_PER_SAMPLE as u32).to_le_bytes())?;
        f.write_all(&BYTES_PER_SAMPLE.to_le_bytes())?;
        f.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;

        f.write_all(b"data")?;
        f.write_all(&data_len.to_le_bytes())?;

        for &sample in &self.data {
            f.write_all(&sample.to_le_bytes())?;
        }

        Ok(())
    }
}

pub fn sin_wave(sample_index: usize, freq: f64, sample_rate: u32) -> f64 {
    let phase = 2.0 * PI * freq * sample_index as f64 / sample_rate as f64;
    phase.sin()
}

pub fn saw_wave(sample_index: usize, freq: f64, sample_rate: u32) -> f64 {
    let phase = (sample_index as f64 * freq / sample_rate as f64) % 1.0;
    let max_harmonics = (sample_rate as f64 / (2.0 * freq)).floor() as usize;
    let n = max_harmonics.min(127);
    let mut sum = 0.0;
    for k in 1..=n {
        let h = k as f64;
        sum += (2.0 * PI * h * phase).sin() / h;
    }
    sum * (2.0 / PI)
}

pub fn generate_ambient_drone_5layer(duration_secs: f64, seed: u64) -> Vec<i16> {
    let num_samples = (SAMPLE_RATE as f64 * duration_secs) as usize;
    let seed_mod = (seed % 12) as f64;

    let f1 = 48.0 + seed_mod * 2.0;
    let f2 = 110.0 + seed_mod * 5.0;
    let f3 = (220.0 + seed_mod * 5.0) * DETUNE;
    let f5 = 440.0;

    let mut mix = vec![0.0f64; num_samples];

    for i in 0..num_samples {
        let t = i as f64 / SAMPLE_RATE as f64;

        let mut sample = 0.0;

        sample += sin_wave(i, f1, SAMPLE_RATE) * 0.4;
        sample += sin_wave(i, f2, SAMPLE_RATE) * 0.25;
        sample += saw_wave(i, f3, SAMPLE_RATE) * 0.15;

        sample += sin_wave(i, 4400.0, SAMPLE_RATE) * 0.008;
        sample += sin_wave(i, 5500.0, SAMPLE_RATE) * 0.008;
        sample += sin_wave(i, 6600.0, SAMPLE_RATE) * 0.008;

        let lfo = 0.5 + 0.5 * (t * 0.2).sin();
        sample += sin_wave(i, f5, SAMPLE_RATE) * lfo * 0.3;

        mix[i] = sample;
    }

    let fade_in_n = (0.5 * SAMPLE_RATE as f64).min(num_samples as f64) as usize;
    for i in 0..fade_in_n {
        let gain = i as f64 / fade_in_n as f64;
        mix[i] *= gain;
    }

    let fade_out_n = (2.0 * SAMPLE_RATE as f64).min(num_samples as f64) as usize;
    let start = num_samples.saturating_sub(fade_out_n);
    for i in start..num_samples {
        let gain = (num_samples - 1 - i) as f64 / fade_out_n as f64;
        mix[i] *= gain;
    }

    let max_abs = mix.iter().map(|&s| s.abs()).fold(0.0f64, f64::max);
    let scale = if max_abs > 0.0 { 32767.0 / max_abs } else { 1.0 };

    mix.iter().map(|&s| (s * scale).round() as i16).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_header_size() {
        let mut w = WavWriter::new();
        let n = WavWriter::sample_duration(1.0);
        w.push_tone(440.0, 0.5, n);

        let tmp = std::env::temp_dir().join("test_drone.wav");
        w.write_wav(tmp.to_str().unwrap()).unwrap();

        let bytes = std::fs::read(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");

        let file_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(file_size, bytes.len() as u32 - 8);

        assert_eq!(&bytes[12..16], b"fmt ");
        let fmt_size = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        assert_eq!(fmt_size, 16);
        let audio_format = u16::from_le_bytes([bytes[20], bytes[21]]);
        assert_eq!(audio_format, 1);
        let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
        assert_eq!(channels, 1);
        let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        assert_eq!(sample_rate, 44100);
        let bits_per_sample = u16::from_le_bytes([bytes[34], bytes[35]]);
        assert_eq!(bits_per_sample, 16);

        assert_eq!(&bytes[36..40], b"data");
        let data_len = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
        assert_eq!(data_len, n as u32 * 2);

        let data_start = 44;
        let data_bytes = &bytes[data_start..];
        assert_eq!(data_bytes.len() as u32, data_len);
        assert_eq!(data_bytes.len(), n * 2);

        assert_eq!(bytes.len(), 44 + data_bytes.len());
    }
}
