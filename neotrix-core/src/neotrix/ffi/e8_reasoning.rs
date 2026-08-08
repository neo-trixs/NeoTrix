// E8 Hexagram Reasoning Implementation
// 64-hexagram yijing-style reasoning engine

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;
use std::time::Instant;

struct E8ReasoningInner {
    library: Vec<HexagramInfo>,
    current: HexagramState,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct E8ReasoningImpl {
    inner: Arc<RwLock<E8ReasoningInner>>,
}

#[uniffi::export]
impl E8ReasoningImpl {
    #[uniffi::constructor]
    pub fn init(_config: NeoTrixConfig) -> Result<Self, NeoTrixError> {
        let library = build_hexagram_library();
        Ok(Self {
            inner: Arc::new(RwLock::new(E8ReasoningInner {
                library,
                current: HexagramState {
                    lines: 1,
                    interpretation: "Initial state — The Creative".into(),
                    confidence: 1.0,
                    timestamp: now_ms(),
                },
            })),
        })
    }

    pub fn reason(&self, request: ReasoningRequest) -> Result<ReasoningResponse, NeoTrixError> {
        if request.query.is_empty() {
            return Err(NeoTrixError::InvalidInput);
        }
        let start = Instant::now();
        let hash = hash_string(&format!("{}{}", request.query, request.context));
        let hexagram_index = (hash % 64) as u8;
        let lines = (hash >> 56 & 0x3F) as u8;
        
        let inner = self.inner.read().unwrap();
        let info = &inner.library[hexagram_index as usize];

        let chain = vec![
            format!("Stage 1 — Query parsed: {} tokens", request.query.split_whitespace().count()),
            format!("Stage 2 — Hexagram {} ({}) selected", info.name, info.chinese_name),
            format!("Stage 3 — Judgment: {}", info.judgment),
        ];

        let conclusion = if request.use_consciousness {
            format!("{} — with consciousness-aware resonance adjustment", info.judgment)
        } else {
            info.judgment.clone()
        };

        Ok(ReasoningResponse {
            hexagram: HexagramState {
                lines,
                interpretation: info.judgment.clone(),
                confidence: confidence_from_hash(hash),
                timestamp: now_ms(),
            },
            reasoning_chain: chain,
            conclusion,
            confidence: confidence_from_hash(hash),
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    pub fn get_current_hexagram(&self) -> HexagramState {
        self.inner.read().unwrap().current.clone()
    }

    pub fn evolve_hexagram(&self, feedback: String, outcome: bool) -> HexagramState {
        let mut inner = self.inner.write().unwrap();
        let base = inner.current.lines;
        let shift = if outcome { 1 } else { 2 };
        let new_lines = base.rotate_left(shift) ^ hash_string(&feedback) as u8;
        let new_state = HexagramState {
            lines: new_lines & 0x3F,
            interpretation: inner.library[(new_lines & 0x3F) as usize].judgment.clone(),
            confidence: if outcome { 0.9 } else { 0.7 },
            timestamp: now_ms(),
        };
        inner.current = new_state.clone();
        new_state
    }

    pub fn get_hexagram_library(&self) -> HexagramLibrary {
        HexagramLibrary { hexagrams: self.inner.read().unwrap().library.clone() }
    }
}

fn hash_string(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn confidence_from_hash(h: u64) -> f32 {
    0.5 + ((h & 0xFF) as f32 / 255.0) * 0.5
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn build_hexagram_library() -> Vec<HexagramInfo> {
    let names = [
        ("Qian", "乾", "The Creative", "Heaven above, Heaven below"),
        ("Kun", "坤", "The Receptive", "Earth above, Earth below"),
        ("Zhun", "屯", "Difficulty at the Beginning", "Thunder above, Water below"),
        ("Meng", "蒙", "Youthful Folly", "Mountain above, Water below"),
        ("Xu", "需", "Waiting", "Water above, Heaven below"),
        ("Song", "讼", "Conflict", "Heaven above, Water below"),
        ("Shi", "师", "The Army", "Earth above, Water below"),
        ("Bi", "比", "Holding Together", "Water above, Earth below"),
        ("Xiao Chu", "小畜", "Small Taming", "Wind above, Heaven below"),
        ("Lu", "履", "Treading", "Heaven above, Lake below"),
        ("Tai", "泰", "Peace", "Earth above, Heaven below"),
        ("Pi", "否", "Standstill", "Heaven above, Earth below"),
        ("Tong Ren", "同人", "Fellowship", "Heaven above, Fire below"),
        ("Da You", "大有", "Great Possession", "Fire above, Heaven below"),
        ("Qian", "谦", "Modesty", "Earth above, Mountain below"),
        ("Yu", "豫", "Enthusiasm", "Thunder above, Earth below"),
        ("Sui", "随", "Following", "Lake above, Thunder below"),
        ("Gu", "蛊", "Work on the Decayed", "Mountain above, Wind below"),
        ("Lin", "临", "Approach", "Earth above, Lake below"),
        ("Guan", "观", "Contemplation", "Wind above, Earth below"),
        ("Shi He", "噬嗑", "Biting Through", "Fire above, Thunder below"),
        ("Bi", "贲", "Grace", "Mountain above, Fire below"),
        ("Bo", "剥", "Splitting Apart", "Mountain above, Earth below"),
        ("Fu", "复", "Return", "Earth above, Thunder below"),
        ("Wu Wang", "无妄", "Innocence", "Heaven above, Thunder below"),
        ("Da Xu", "大畜", "Great Taming", "Mountain above, Heaven below"),
        ("Yi", "颐", "Nourishment", "Mountain above, Thunder below"),
        ("Da Guo", "大过", "Great Exceeding", "Lake above, Wind below"),
        ("Kan", "坎", "The Abysmal", "Water above, Water below"),
        ("Li", "离", "The Clinging", "Fire above, Fire below"),
        ("Xian", "咸", "Influence", "Lake above, Mountain below"),
        ("Heng", "恒", "Duration", "Thunder above, Wind below"),
        ("Dun", "遁", "Retreat", "Heaven above, Mountain below"),
        ("Da Zhuang", "大壮", "Great Power", "Thunder above, Heaven below"),
        ("Jin", "晋", "Progress", "Fire above, Earth below"),
        ("Ming Yi", "明夷", "Darkening of the Light", "Earth above, Fire below"),
        ("Jia Ren", "家人", "The Family", "Wind above, Fire below"),
        ("Kui", "睽", "Opposition", "Fire above, Lake below"),
        ("Jian", "蹇", "Obstruction", "Water above, Mountain below"),
        ("Jie", "解", "Deliverance", "Thunder above, Water below"),
        ("Sun", "损", "Decrease", "Mountain above, Lake below"),
        ("Yi", "益", "Increase", "Wind above, Thunder below"),
        ("Guai", "夬", "Breakthrough", "Lake above, Heaven below"),
        ("Gou", "姤", "Coming to Meet", "Heaven above, Wind below"),
        ("Cui", "萃", "Gathering Together", "Lake above, Earth below"),
        ("Sheng", "升", "Pushing Upward", "Earth above, Wind below"),
        ("Kun", "困", "Oppression", "Lake above, Water below"),
        ("Jing", "井", "The Well", "Water above, Wind below"),
        ("Ge", "革", "Revolution", "Lake above, Fire below"),
        ("Ding", "鼎", "The Cauldron", "Fire above, Wind below"),
        ("Zhen", "震", "The Arousing", "Thunder above, Thunder below"),
        ("Gen", "艮", "Keeping Still", "Mountain above, Mountain below"),
        ("Jian", "渐", "Development", "Wind above, Mountain below"),
        ("Gui Mei", "归妹", "The Marrying Maiden", "Thunder above, Lake below"),
        ("Feng", "丰", "Abundance", "Thunder above, Fire below"),
        ("Lv", "旅", "The Wanderer", "Fire above, Mountain below"),
        ("Xun", "巽", "The Gentle", "Wind above, Wind below"),
        ("Dui", "兑", "The Joyous", "Lake above, Lake below"),
        ("Huan", "涣", "Dispersion", "Wind above, Water below"),
        ("Jie", "节", "Limitation", "Water above, Lake below"),
        ("Zhong Fu", "中孚", "Inner Truth", "Wind above, Lake below"),
        ("Xiao Guo", "小过", "Small Exceeding", "Thunder above, Mountain below"),
        ("Ji Ji", "既济", "After Completion", "Water above, Fire below"),
        ("Wei Ji", "未济", "Before Completion", "Fire above, Water below"),
    ];

    names
        .iter()
        .enumerate()
        .map(|(i, (name, cn, judgment, image))| HexagramInfo {
            index: i as u8,
            name: name.to_string(),
            chinese_name: cn.to_string(),
            judgment: judgment.to_string(),
            image: image.to_string(),
            lines: (1..=6)
                .map(|pos| LineInfo {
                    position: pos,
                    yin_yang: (i >> (pos - 1)) & 1 == 1,
                    text: format!("Line {} — {}", pos, judgment),
                })
                .collect(),
        })
        .collect()
}