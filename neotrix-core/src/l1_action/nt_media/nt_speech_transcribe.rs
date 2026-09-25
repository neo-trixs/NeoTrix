//! 自研语音转写 — Whisper 兼容 log-Mel 前端 + ort 推理（sherpa 编解码配对）。
//!
//! 前端参数与 openai/whisper 一致：16kHz，n_fft=400，hop=160，Hann 窗，
//! 80 维 Slaney Mel 滤波器组，`log10(max(mel,1e-10))` 后 `(x+4)/4` 归一。
//! 推理：encoder（30s 窗）→ decoder 贪心自回归 → 时间戳切分。
//! 权重：sherpa-onnx whisper-tiny.en int8（encoder/decoder 配对），
//! BPE 表由其 `tiny.en-tokens.txt`（base64 行）提供，无需 tiktoken。

/// 16kHz 单声道 → (80, n_frames) log-Mel（行主序展平）。
pub fn log_mel_spectrogram(pcm_16k_mono: &[f32]) -> Vec<f32> {
    const SR: usize = 16000;
    const N_FFT: usize = 400;
    const HOP: usize = 160;
    const N_MELS: usize = 80;
    const FMIN: f64 = 0.0;
    const FMAX: f64 = 8000.0;
    let _ = SR;
    let hann: Vec<f64> = (0..N_FFT)
        .map(|n| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * n as f64 / N_FFT as f64).cos())
        .collect();
    let filters = mel_filterbank(N_MELS, N_FFT, FMIN, FMAX);
    let n_frames = pcm_16k_mono.len() / HOP;
    // 先存线性 Mel 能量（f64），再按 whisper 原生做动态归一：
    // log10(max(e,1e-10)) → 上限 max-8 → (x+4)/4。
    let mut energy = vec![0f64; N_MELS * n_frames];
    let mut frame = vec![0f64; N_FFT];
    for t in 0..n_frames {
        for n in 0..N_FFT {
            let idx = t * HOP + n;
            let s = if idx < pcm_16k_mono.len() {
                pcm_16k_mono[idx] as f64
            } else {
                0.0
            };
            frame[n] = s * hann[n];
        }
        let power = dft_power(&frame);
        for m in 0..N_MELS {
            let mut e = 0f64;
            let row = &filters[m * (N_FFT / 2 + 1)..(m + 1) * (N_FFT / 2 + 1)];
            for (k, w) in row.iter().enumerate() {
                e += power[k] * w;
            }
            energy[m * n_frames + t] = e;
        }
    }
    let max_log = energy
        .iter()
        .map(|&e| e.max(1e-10).log10())
        .fold(f64::NEG_INFINITY, f64::max);
    energy
        .iter()
        .map(|&e| {
            let v = e.max(1e-10).log10().max(max_log - 8.0);
            ((v + 4.0) / 4.0) as f32
        })
        .collect()
}

/// 实信号 DFT 功率谱（朴素 O(n²)，spike 用；生产切 rustfft）。
fn dft_power(frame: &[f64]) -> Vec<f64> {
    let n = frame.len();
    let mut out = vec![0f64; n / 2 + 1];
    for k in 0..=n / 2 {
        let (mut re, mut im) = (0f64, 0f64);
        for (idx, &s) in frame.iter().enumerate() {
            let ang = 2.0 * std::f64::consts::PI * k as f64 * idx as f64 / n as f64;
            re += s * ang.cos();
            im -= s * ang.sin();
        }
        out[k] = re * re + im * im;
    }
    out
}

fn hz_to_mel_slaney(hz: f64) -> f64 {
    const F_SP: f64 = 200.0 / 3.0;
    const MIN_LOG_HZ: f64 = 1000.0;
    if hz >= MIN_LOG_HZ {
        slaney_mel(hz)
    } else {
        hz / F_SP
    }
}

fn slaney_mel(hz: f64) -> f64 {
    const F_SP: f64 = 200.0 / 3.0;
    const MIN_LOG_HZ: f64 = 1000.0;
    const MIN_LOG_MEL: f64 = MIN_LOG_HZ / F_SP;
    const LOGSTEP: f64 = 0.06875177742094912; // ln(6.4)/27 (Slaney)
    if hz >= MIN_LOG_HZ {
        MIN_LOG_MEL + ((hz / MIN_LOG_HZ).ln() / LOGSTEP)
    } else {
        hz / F_SP
    }
}

fn mel_to_hz_slaney(mel: f64) -> f64 {
    const F_SP: f64 = 200.0 / 3.0;
    const MIN_LOG_HZ: f64 = 1000.0;
    const MIN_LOG_MEL: f64 = MIN_LOG_HZ / F_SP;
    const LOGSTEP: f64 = 0.06875177742094912; // ln(6.4)/27 (Slaney)
    if mel >= MIN_LOG_MEL {
        MIN_LOG_HZ * ((mel - MIN_LOG_MEL) * LOGSTEP).exp()
    } else {
        mel * F_SP
    }
}

/// Slaney Mel 三角滤波器组（行主序：mel × (n_fft/2+1)）。
fn mel_filterbank(n_mels: usize, n_fft: usize, fmin: f64, fmax: f64) -> Vec<f64> {
    let n_bins = n_fft / 2 + 1;
    let mel_min = slaney_mel(fmin);
    let mel_max = slaney_mel(fmax);
    let points: Vec<f64> = (0..n_mels + 2)
        .map(|i| mel_min + (mel_max - mel_min) * i as f64 / (n_mels + 1) as f64)
        .collect();
    let freqs: Vec<f64> = (0..n_bins)
        .map(|k| k as f64 * fmax * 2.0 / n_fft as f64)
        .collect();
    let mut fb = vec![0f64; n_mels * n_bins];
    for m in 0..n_mels {
        let (f0, f1, f2) = (
            mel_to_hz_slaney(points[m]),
            mel_to_hz_slaney(points[m + 1]),
            mel_to_hz_slaney(points[m + 2]),
        );
        for (k, &f) in freqs.iter().enumerate() {
            let w = if f >= f0 && f <= f1 {
                (f - f0) / (f1 - f0).max(1e-12)
            } else if f > f1 && f <= f2 {
                (f2 - f) / (f2 - f1).max(1e-12)
            } else {
                0.0
            };
            fb[m * n_bins + k] = w;
        }
    }
    fb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mel_silence_floor() {
        let pcm = vec![0f32; 16000];
        let mel = log_mel_spectrogram(&pcm);
        assert_eq!(mel.len(), 80 * 100);
        // 全静音：能量全 0 → log=-10，高于动态下限(-18)，输出 (-10+4)/4 = -1.5
        assert!(mel.iter().all(|&v| (v - (-1.5)).abs() < 1e-4));
    }

    #[test]
    fn test_mel_sine_peak() {
        let pcm: Vec<f32> = (0..16000)
            .map(|n| (2.0 * std::f64::consts::PI * 440.0 * n as f64 / 16000.0).sin() as f32)
            .collect();
        let mel = log_mel_spectrogram(&pcm);
        assert_eq!(mel.len(), 80 * 100);
        assert!(mel.iter().all(|v| v.is_finite()));
        let peak = mel.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(peak > -1.0, "sine must lift energy above floor");
        let mid_frame = 50;
        let mut argmax = 0;
        let mut best = f32::NEG_INFINITY;
        for m in 0..80 {
            let v = mel[m * 100 + mid_frame];
            if v > best {
                best = v;
                argmax = m;
            }
        }
        assert!((10..30).contains(&argmax), "440Hz peak mel bin={argmax}");
    }

    #[test]
    fn test_mel_deterministic() {
        let pcm: Vec<f32> = (0..8000).map(|n| (n as f32 * 0.01).sin()).collect();
        assert_eq!(log_mel_spectrogram(&pcm), log_mel_spectrogram(&pcm));
    }

    /// 对标 numpy 参考实现（需 NT_MEL_REF_PCM=/tmp/pcm16.raw, NT_MEL_REF_MEL=/tmp/mel_ref.raw）。
    #[test]
    fn test_mel_against_numpy_reference() {
        let (pcm_path, mel_path) = match (
            std::env::var("NT_MEL_REF_PCM"),
            std::env::var("NT_MEL_REF_MEL"),
        ) {
            (Ok(a), Ok(b)) => (a, b),
            _ => return,
        };
        let raw = std::fs::read(&pcm_path).expect("read pcm");
        let pcm: Vec<f32> = raw
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        let raw = std::fs::read(&mel_path).expect("read mel");
        let reference: Vec<f32> = raw
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        let mine = log_mel_spectrogram(&pcm);
        assert_eq!(mine.len(), reference.len());
        let max_diff = mine
            .iter()
            .zip(reference.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0f32, f32::max);
        eprintln!("mel max-abs-diff vs numpy: {max_diff}");
        assert!(max_diff < 5e-3, "mel mismatch {max_diff}");
    }
}

// ═══════════════════════════════════════════════════════════════════
// 推理：sherpa whisper-tiny.en 编解码配对（onnx feature 门控）
// ═══════════════════════════════════════════════════════════════════

/// 转写片段（含时间戳）。
#[derive(Debug, Clone, PartialEq)]
pub struct SpeechSegment {
    pub start_s: f64,
    pub end_s: f64,
    pub text: String,
}

/// 解析 sherpa `tokens.txt` 行：`<base64> <id>` → (id, 文本)。
/// 非 base64 行返回 None（调用方跳过）。
pub fn parse_token_line(line: &str) -> Option<(i64, String)> {
    let line = line.trim_end();
    let sp = line.rfind(' ')?;
    let id: i64 = line[sp + 1..].parse().ok()?;
    let b64 = &line[..sp];
    let bytes = base64_decode(b64)?;
    Some((id, String::from_utf8_lossy(&bytes).into_owned()))
}

/// 最小 base64 解码（标准表 + `=` 填充；非法字符整行拒绝）。
fn base64_decode(s: &str) -> Option<Vec<u8>> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

/// whisper 时间戳 token → 秒（每 token 0.02s）。
pub fn timestamp_to_secs(token_id: i64, timestamp_begin: i64) -> f64 {
    (token_id - timestamp_begin).max(0) as f64 * 0.02
}

/// token 序列 → 切分片段（纯函数，可单测）。
///
/// 规则：时间戳 token 开新段并结算上一段；文本 token（< eot 且非时间戳）
/// 追加当前段；eot 结束。悬空文本以 `end_hint_s` 收尾。
pub fn assemble_segments(
    ids: &[i64],
    token_text: &[String],
    eot: i64,
    timestamp_begin: i64,
    end_hint_s: f64,
) -> Vec<SpeechSegment> {
    let mut segs = Vec::new();
    let mut seg_start: Option<f64> = None;
    let mut buf = String::new();
    let flush = |segs: &mut Vec<SpeechSegment>, seg_start: &mut Option<f64>, buf: &mut String, end: f64| {
        if !buf.is_empty() {
            segs.push(SpeechSegment {
                start_s: seg_start.unwrap_or(0.0),
                end_s: end,
                text: std::mem::take(buf),
            });
        }
        *seg_start = None;
    };
    for &id in ids {
        if id == eot {
            let end = seg_start.map(|_| end_hint_s).unwrap_or(end_hint_s);
            flush(&mut segs, &mut seg_start, &mut buf, end);
            break;
        }
        if id >= timestamp_begin {
            let ts = timestamp_to_secs(id, timestamp_begin);
            if !buf.is_empty() {
                flush(&mut segs, &mut seg_start, &mut buf, ts);
            }
            seg_start = Some(ts);
            continue;
        }
        if id >= 0 {
            if let Some(t) = token_text.get(id as usize) {
                buf.push_str(t);
            }
        }
    }
    if !buf.is_empty() {
        flush(&mut segs, &mut seg_start, &mut buf, end_hint_s);
    }
    segs
}

/// 30s 窗切分：返回每窗样点数（最后一窗不足补齐由调用方 pad）。
pub fn window_ranges(total_samples: usize, window_samples: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    while start < total_samples {
        out.push((start, (start + window_samples).min(total_samples)));
        start += window_samples;
    }
    out
}

#[cfg(feature = "onnx")]
pub mod inference {
    use super::{assemble_segments, log_mel_spectrogram, parse_token_line, window_ranges};
    use super::SpeechSegment;
    use ort::session::Session;
    use ort::value::Tensor;

    /// tiny.en 特殊 token（读自 encoder 会话 metadata，与 openai 编号不同；
    /// sherpa 导出：eot=50256, sot=50257, transcribe=50358,
    /// no_timestamps=50362, timestamp_begin=50363, n_vocab=51864）。
    /// prompt 约定（sherpa greedy，timestamp 模式）：[sot]；
    /// offset = 当前输入在全序列中的起始位置；单步只喂新 token。
    pub const SOT: i64 = 50257;
    pub const EOT: i64 = 50256;
    pub const TRANSCRIBE: i64 = 50358;
    pub const NO_TIMESTAMPS: i64 = 50362;
    pub const TIMESTAMP_BEGIN: i64 = 50363;
    const N_MELS: usize = 80;
    const N_FRAMES: usize = 3000;
    const N_CACHE: usize = 448;
    const N_LAYER: usize = 4;
    const HIDDEN: usize = 384;
    const MAX_TOKENS: usize = 448;

    pub struct SpeechModel {
        enc: Session,
        dec: Session,
        token_text: Vec<String>,
    }

    impl SpeechModel {
        /// 从模型目录加载（encoder/decoder int8 onnx + tokens.txt）。
        pub fn load(model_dir: &std::path::Path) -> Result<Self, String> {
            let enc_path = model_dir.join("tiny.en-encoder.int8.onnx");
            let dec_path = model_dir.join("tiny.en-decoder.int8.onnx");
            let tok_path = model_dir.join("tiny.en-tokens.txt");
            let enc = Session::builder()
                .and_then(|mut b| b.commit_from_file(&enc_path))
                .map_err(|e| format!("encoder load: {e}"))?;
            let dec = Session::builder()
                .and_then(|mut b| b.commit_from_file(&dec_path))
                .map_err(|e| format!("decoder load: {e}"))?;
            let raw = std::fs::read_to_string(&tok_path)
                .map_err(|e| format!("tokens read: {e}"))?;
            let mut token_text = vec![String::new(); 50256];
            for line in raw.lines() {
                if let Some((id, text)) = parse_token_line(line) {
                    if id >= 0 && (id as usize) < token_text.len() {
                        token_text[id as usize] = text;
                    }
                }
            }
            Ok(Self { enc, dec, token_text })
        }

        fn argmax_last(logits: &[f32], vocab: usize, frames: usize) -> i64 {
            let base = (frames - 1) * vocab;
            let mut best = 0;
            let mut best_v = f32::NEG_INFINITY;
            for i in 0..vocab {
                let v = logits[base + i];
                if v > best_v {
                    best_v = v;
                    best = i;
                }
            }
            best as i64
        }

        /// 单步 decoder：增量输入 + offset=起始位置（sherpa 语义）。
        /// 返回 (logits展平, 帧数, 词表)。
        fn run_decoder(
            &mut self,
            tokens: &[i64],
            offset: i64,
            cross_k: Vec<f32>,
            cross_v: Vec<f32>,
            kv_cache: &mut (Vec<f32>, Vec<f32>),
        ) -> Result<(Vec<f32>, usize, usize), String> {
            let t = Tensor::from_array((
                vec![1usize, tokens.len()],
                tokens.to_vec(),
            ))
            .map_err(|e| format!("tokens tensor: {e}"))?;
            let zk = Tensor::from_array((
                vec![N_LAYER, 1, N_CACHE, HIDDEN],
                kv_cache.0.clone(),
            ))
            .map_err(|e| format!("kv tensor: {e}"))?;
            let zv = Tensor::from_array((
                vec![N_LAYER, 1, N_CACHE, HIDDEN],
                kv_cache.1.clone(),
            ))
            .map_err(|e| format!("kv tensor: {e}"))?;
            let ck = Tensor::from_array((vec![N_LAYER, 1, 1500, HIDDEN], cross_k))
                .map_err(|e| format!("cross tensor: {e}"))?;
            let cv = Tensor::from_array((vec![N_LAYER, 1, 1500, HIDDEN], cross_v))
                .map_err(|e| format!("cross tensor: {e}"))?;
            let off = Tensor::from_array((vec![1usize], vec![offset]))
                .map_err(|e| format!("offset tensor: {e}"))?;
            let out = self
                .dec
                .run(ort::inputs![t, zk, zv, ck, cv, off])
                .map_err(|e| format!("decoder run: {e}"))?;
            let mut logits: Vec<f32> = Vec::new();
            let mut vocab = 0usize;
            let mut frames = 0usize;
            for (name, value) in out.iter() {
                if name == "logits" {
                    let (shape, data) = value
                        .try_extract_tensor::<f32>()
                        .map_err(|e| format!("logits extract: {e}"))?;
                    frames = shape[1] as usize;
                    vocab = shape[2] as usize;
                    logits = data.to_vec();
                } else if name == "out_n_layer_self_k_cache" {
                    let (_, data) = value
                        .try_extract_tensor::<f32>()
                        .map_err(|e| format!("kv extract: {e}"))?;
                    kv_cache.0 = data.to_vec();
                } else if name == "out_n_layer_self_v_cache" {
                    let (_, data) = value
                        .try_extract_tensor::<f32>()
                        .map_err(|e| format!("kv extract: {e}"))?;
                    kv_cache.1 = data.to_vec();
                }
            }
            if logits.is_empty() || vocab == 0 || frames == 0 {
                return Err("decoder returned no logits".into());
            }
            Ok((logits, frames, vocab))
        }

        /// 单 30s 窗解码 → token id 序列（含时间戳）。
        fn decode_window(&mut self, mel: &[f32]) -> Result<Vec<i64>, String> {
            if mel.len() != N_MELS * N_FRAMES {
                return Err(format!("mel shape: got {}, want {}", mel.len(), N_MELS * N_FRAMES));
            }
            let mt = Tensor::from_array((vec![1usize, N_MELS, N_FRAMES], mel.to_vec()))
                .map_err(|e| format!("mel tensor: {e}"))?;
            let (cross_k, cross_v) = {
                let enc_out = self
                    .enc
                    .run(ort::inputs![mt])
                    .map_err(|e| format!("encoder run: {e}"))?;
                let mut cross_k = Vec::new();
                let mut cross_v = Vec::new();
                for (name, value) in enc_out.iter() {
                    if name.contains("cross_k") {
                        let (_, data) = value
                            .try_extract_tensor::<f32>()
                            .map_err(|e| format!("cross extract: {e}"))?;
                        cross_k = data.to_vec();
                    } else if name.contains("cross_v") {
                        let (_, data) = value
                            .try_extract_tensor::<f32>()
                            .map_err(|e| format!("cross extract: {e}"))?;
                        cross_v = data.to_vec();
                    }
                }
                (cross_k, cross_v)
            };
            if cross_k.is_empty() || cross_v.is_empty() {
                return Err("encoder returned no cross kv".into());
            }
            let mut kv_cache = (
                vec![0f32; N_LAYER * N_CACHE * HIDDEN],
                vec![0f32; N_LAYER * N_CACHE * HIDDEN],
            );
            let mut tokens = vec![SOT];
            let mut offset: i64 = 0;
            let mut predicted: Vec<i64> = Vec::new();
            for _ in 0..MAX_TOKENS {
                let (logits, frames, vocab) = self.run_decoder(
                    &tokens,
                    offset,
                    cross_k.clone(),
                    cross_v.clone(),
                    &mut kv_cache,
                )?;
                let id = Self::argmax_last(&logits, vocab, frames);
                offset += tokens.len() as i64;
                tokens = vec![id];
                if id == EOT {
                    break;
                }
                predicted.push(id);
            }
            Ok(predicted)
        }

        /// 16kHz 单声道全长转写（30s 分窗，时间累加）。
        pub fn transcribe_pcm_16k(&mut self, pcm: &[f32]) -> Result<Vec<SpeechSegment>, String> {
            const WINDOW: usize = 16000 * 30;
            let mut segs = Vec::new();
            for (s, e) in window_ranges(pcm.len(), WINDOW) {
                let mut win = vec![0f32; WINDOW];
                win[..e - s].copy_from_slice(&pcm[s..e]);
                let mel = log_mel_spectrogram(&win);
                let ids = self.decode_window(&mel)?;
                let base = s as f64 / 16000.0;
                for mut seg in assemble_segments(
                    &ids,
                    &self.token_text,
                    EOT,
                    TIMESTAMP_BEGIN,
                    30.0,
                ) {
                    seg.start_s += base;
                    seg.end_s += base;
                    segs.push(seg);
                }
            }
            Ok(segs)
        }
    }
}

#[cfg(test)]
mod inference_tests {
    use super::*;

    #[test]
    fn test_token_line() {
        assert_eq!(
            parse_token_line("IA== 220"),
            Some((220, " ".into()))
        );
        assert_eq!(parse_token_line("garbage"), None);
        assert_eq!(parse_token_line("!!! 5"), None);
    }

    #[test]
    fn test_timestamp_secs() {
        assert!((timestamp_to_secs(50364, 50364) - 0.0).abs() < 1e-9);
        assert!((timestamp_to_secs(50464, 50364) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_assemble_segments() {
        let vocab = vec!["<s>".to_string(), " Hello".to_string(), " world".to_string()];
        // [ts0, hello, world, ts100(=2s), hello, eot]
        let ids = vec![50364, 1, 2, 50464, 1, 50257];
        let segs = assemble_segments(&ids, &vocab, 50257, 50364, 30.0);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, " Hello world");
        assert!((segs[0].end_s - 2.0).abs() < 1e-9);
        assert_eq!(segs[1].text, " Hello");
        assert!((segs[1].start_s - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_window_ranges() {
        assert_eq!(window_ranges(0, 10), vec![]);
        assert_eq!(window_ranges(5, 10), vec![(0, 5)]);
        assert_eq!(window_ranges(25, 10), vec![(0, 10), (10, 20), (20, 25)]);
    }
}
