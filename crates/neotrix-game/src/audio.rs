// 音效系统 — 吸收 Godot AudioStreamPlayer.
// 真后端：启动时代码合成 WAV（零资产）→ macroquad 加载 → play() 实发声；
// 加载失败回退日志（旧桩行为），不断链。

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect { Hit, Hurt, Select, Confirm, LevelUp, QuestComplete, Dialogue }

/// 合成采样率（电话级够用，体积小）
pub const SAMPLE_RATE: u32 = 22050;

/// f32 单声道采样 → 16-bit PCM WAV 字节
pub fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + n * 2).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(n * 2).to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn secs(s: f32) -> usize {
    (s * SAMPLE_RATE as f32) as usize
}

/// 正弦扫频 + 指数衰减（cells：Select/Confirm/LevelUp/Dialogue）
fn tone(freq0: f32, freq1: f32, dur: f32) -> Vec<f32> {
    let n = secs(dur).max(1);
    let mut phase = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let f = freq0 + (freq1 - freq0) * t;
            // 先采样后推进：首采样恒 0（无 click），尾包络归 0
            let s = (phase * std::f32::consts::TAU).sin() * (1.0 - t).powi(2);
            phase += f / SAMPLE_RATE as f32;
            s
        })
        .collect()
}

/// 方波扫频（Hurt 下坠感更糙）
fn square(freq0: f32, freq1: f32, dur: f32) -> Vec<f32> {
    tone(freq0, freq1, dur)
        .into_iter()
        .map(|s| if s >= 0.0 { 0.7 } else { -0.7 } * s.abs().sqrt())
        .collect()
}

/// 噪声爆点（Hit 打击感；确定性 LCG，单测可复现）
fn noise_hit(dur: f32) -> Vec<f32> {
    let n = secs(dur).max(1);
    let mut st: u64 = 0x12345678;
    (0..n)
        .map(|i| {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r = ((st >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0;
            let t = i as f32 / n as f32;
            r * (1.0 - t).powi(3)
        })
        .collect()
}

/// 琶音（LevelUp 三音上行；每音自带衰减包络，自然断句）
fn arp(freqs: &[f32], note_dur: f32) -> Vec<f32> {
    let mut out = Vec::new();
    for f in freqs {
        out.extend(tone(*f, *f, note_dur));
    }
    out
}

/// 五声音阶宫调式 BGM（Karplus-Strong 弹拨合成，无资产古筝循环）
const PENTA: [f32; 8] = [261.63, 293.66, 329.63, 392.0, 440.0, 523.25, 587.33, 659.25];
/// 16 步乐句（None 休止，宫音收尾 + 尾隙呼吸，循环无 click 感由首尾淡入淡出保证）
const PHRASE: [Option<usize>; 16] = [
    Some(0), Some(1), Some(2), Some(4), Some(5), Some(4), Some(2), Some(1),
    Some(0), Some(1), Some(2), None, Some(1), Some(0), None, None,
];
const NOTE_STEP: f32 = 0.42;
const NOTE_LEN: f32 = 0.9;

/// Karplus-Strong 弹拨弦（确定性种子，可复现）
fn karplus(freq: f32, dur: f32, seed: u64) -> Vec<f32> {
    let n = secs(dur).max(2);
    let period = (SAMPLE_RATE as f32 / freq).max(2.0) as usize;
    let mut buf = vec![0.0f32; period];
    let mut st = seed;
    for b in buf.iter_mut() {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *b = ((st >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0;
    }
    let mut out = Vec::with_capacity(n);
    let mut idx = 0;
    for _ in 0..n {
        let cur = buf[idx];
        let nxt = buf[(idx + 1) % period];
        let v = 0.996 * 0.5 * (cur + nxt);
        buf[idx] = v;
        idx = (idx + 1) % period;
        out.push(v);
    }
    out
}

/// BGM 循环体 → WAV 字节（叠加+归一+首尾淡入淡出，纯函数可测）
pub fn bgm_loop() -> Vec<u8> {
    let total = ((PHRASE.len() as f32 * NOTE_STEP + 0.4) * SAMPLE_RATE as f32) as usize;
    let mut mix = vec![0.0f32; total];
    for (i, deg) in PHRASE.iter().enumerate() {
        if let Some(d) = deg {
            let note = karplus(PENTA[*d % PENTA.len()], NOTE_LEN, 0x9E3779B9 + i as u64);
            let start = (i as f32 * NOTE_STEP * SAMPLE_RATE as f32) as usize;
            for (j, s) in note.iter().enumerate() {
                if start + j < mix.len() {
                    mix[start + j] += s * 0.5;
                }
            }
        }
    }
    let peak = mix.iter().map(|s| s.abs()).fold(0.0f32, f32::max).max(1e-6);
    for s in mix.iter_mut() {
        *s = (*s / peak * 0.8).clamp(-1.0, 1.0);
    }
    // 首尾各 10ms 淡入淡出（循环接缝无 click）
    let fade = (0.01 * SAMPLE_RATE as f32) as usize;
    for i in 0..fade.min(mix.len() / 2) {
        let k = i as f32 / fade as f32;
        mix[i] *= k;
        let j = mix.len() - 1 - i;
        mix[j] *= k;
    }
    wav_bytes(&mix)
}

/// 单音效合成 → WAV 字节（纯函数，可测）
pub fn sfx_bytes(sfx: SoundEffect) -> Vec<u8> {
    let samples = match sfx {
        SoundEffect::Hit => noise_hit(0.12),
        SoundEffect::Hurt => square(300.0, 120.0, 0.20),
        SoundEffect::Select => tone(880.0, 880.0, 0.06),
        SoundEffect::Confirm => {
            let mut a = tone(660.0, 660.0, 0.07);
            a.extend(tone(990.0, 990.0, 0.09));
            a
        }
        SoundEffect::LevelUp => arp(&[523.0, 659.0, 784.0], 0.11),
        SoundEffect::QuestComplete => {
            let mut a = tone(784.0, 784.0, 0.10);
            a.extend(tone(1046.0, 1046.0, 0.16));
            a
        }
        SoundEffect::Dialogue => tone(1200.0, 900.0, 0.04),
    };
    wav_bytes(&samples)
}

pub struct AudioSystem {
    pub enabled: bool,
    pub volume: f32,
    sounds: HashMap<SoundEffect, macroquad::audio::Sound>,
    bgm: Option<macroquad::audio::Sound>,
}

impl AudioSystem {
    pub fn new() -> Self {
        Self { enabled: true, volume: 0.7, sounds: HashMap::new(), bgm: None }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
    }

    /// 启动时调用一次（async）：全量合成+加载；单项失败记日志继续
    pub async fn init(&mut self) {
        use macroquad::audio::load_sound_from_bytes;
        const ALL: [SoundEffect; 7] = [
            SoundEffect::Hit,
            SoundEffect::Hurt,
            SoundEffect::Select,
            SoundEffect::Confirm,
            SoundEffect::LevelUp,
            SoundEffect::QuestComplete,
            SoundEffect::Dialogue,
        ];
        for sfx in ALL {
            match load_sound_from_bytes(&sfx_bytes(sfx)).await {
                Ok(s) => {
                    self.sounds.insert(sfx, s);
                }
                Err(e) => log::debug!("[Audio] 加载 {:?} 失败，回退静默: {}", sfx, e),
            }
        }
    }

    pub fn play(&self, sfx: SoundEffect) {
        if !self.enabled || self.volume < 0.01 {
            return;
        }
        match self.sounds.get(&sfx) {
            Some(s) => macroquad::audio::play_sound(
                s,
                macroquad::audio::PlaySoundParams { looped: false, volume: self.volume },
            ),
            None => log::debug!("[Audio] {:?} 未加载，回退静默", sfx),
        }
    }

    /// 已加载数（启动自检/测试探针）
    pub fn loaded(&self) -> usize {
        self.sounds.len()
    }

    /// BGM 循环启动（幂等；音量取 0.6 系数衬底，不抢 SFX）
    pub async fn start_bgm(&mut self) {
        if self.bgm.is_some() {
            return;
        }
        match macroquad::audio::load_sound_from_bytes(&bgm_loop()).await {
            Ok(s) => {
                macroquad::audio::play_sound(
                    &s,
                    macroquad::audio::PlaySoundParams {
                        looped: true,
                        volume: self.volume * 0.6,
                    },
                );
                self.bgm = Some(s);
            }
            Err(e) => log::debug!("[Audio] BGM 加载失败: {}", e),
        }
    }

    pub fn bgm_on(&self) -> bool {
        self.bgm.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_ok(bytes: &[u8], expect_samples: usize) {
        assert!(bytes.starts_with(b"RIFF"));
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(bytes.len(), 44 + expect_samples * 2);
    }

    #[test]
    fn wav_header_and_clipping() {
        let b = wav_bytes(&[0.0, 1.0, -1.0, 2.0, -2.0]);
        wav_ok(&b, 5);
        // 钳制：±2.0 → ±32767（对称映射，避开 -32768 不对称）
        assert_eq!(&b[44 + 3 * 2..44 + 4 * 2], &32767i16.to_le_bytes());
        assert_eq!(&b[44 + 4 * 2..44 + 5 * 2], &(-32767i16).to_le_bytes());
    }

    #[test]
    fn all_recipes_valid_wavs() {
        use SoundEffect::*;
        for sfx in [Hit, Hurt, Select, Confirm, LevelUp, QuestComplete, Dialogue] {
            let b = sfx_bytes(sfx);
            assert!(b.len() > 44, "{:?} 空", sfx);
            assert!(b.starts_with(b"RIFF"), "{:?} 头坏", sfx);
        }
        // 时长分级：Hit 短促 < LevelUp 琶音
        assert!(sfx_bytes(SoundEffect::Hit).len() < sfx_bytes(SoundEffect::LevelUp).len());
    }

    #[test]
    fn tone_decays_and_noise_bounded() {
        let t = tone(440.0, 440.0, 0.1);
        assert!(t[0].abs() < 0.05); // 起振接近 0
        assert!(t[t.len() - 1].abs() < 0.05); // 尾衰减完
        assert!(t.iter().all(|s| s.abs() <= 1.0));
        let n = noise_hit(0.1);
        assert!(n.iter().all(|s| s.abs() <= 1.0));
        // 确定性：同种子同波形
        assert_eq!(noise_hit(0.05), noise_hit(0.05));
    }

    #[test]
    fn bgm_loop_shape() {
        let b = bgm_loop();
        assert!(b.starts_with(b"RIFF"));
        // 时长 ≈ 16 步×0.42 + 0.4s 尾（±1 采样容差）
        let expect = ((16.0 * NOTE_STEP + 0.4) * SAMPLE_RATE as f32) as usize;
        assert!((b.len() as i64 - (44 + expect * 2) as i64).abs() <= 2);
        // 首尾淡入淡出（循环无 click）
        assert_eq!(&b[44..46], &0i16.to_le_bytes());
        // 确定性：种子固定，两次一致
        assert_eq!(bgm_loop(), b);
    }

    #[test]
    fn karplus_plucks_and_decays() {
        let k = karplus(440.0, 0.2, 7);
        assert!(!k.is_empty());
        assert!(k.iter().all(|s| s.abs() <= 1.0));
        // 拨弦衰减：后半能量小于前半
        let h = k.len() / 2;
        let e0: f32 = k[..h].iter().map(|s| s * s).sum();
        let e1: f32 = k[h..].iter().map(|s| s * s).sum();
        assert!(e1 < e0);
    }
}
