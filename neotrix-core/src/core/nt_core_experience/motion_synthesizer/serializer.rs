use super::types::{
    AnimatedValue, LottieAnimation, LottieEffect, LottieLayer, LottieMask, LottieShape,
    LottieTransform, MergeMode,
};
use crate::core::nt_core_value_system::CoreValue;
use serde_json::{json, Map, Value};

// ═══════════════════════════════════════════════════════════════
// Color Helpers
// ═══════════════════════════════════════════════════════════════

pub(crate) fn hex_to_rgba(hex: &str) -> [f64; 4] {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0) as f64 / 255.0;
        [r, g, b, 1.0]
    } else {
        [1.0, 1.0, 1.0, 1.0]
    }
}

pub(crate) fn hex_to_rgba_alpha(hex: &str, alpha: f64) -> [f64; 4] {
    let mut rgba = hex_to_rgba(hex);
    rgba[3] = alpha.max(0.0).min(1.0);
    rgba
}

pub(crate) fn value_color_rgba(value: CoreValue, opacity: f64) -> [f64; 4] {
    let hex = match value {
        CoreValue::Curiosity => "#f59e0b",
        CoreValue::KnowledgeGrowth => "#22c55e",
        CoreValue::Coherence => "#3b82f6",
        CoreValue::Autonomy => "#a855f7",
        CoreValue::Helpfulness => "#ec4899",
        CoreValue::Truthfulness => "#06b6d4",
        CoreValue::Efficiency => "#64748b",
    };
    hex_to_rgba_alpha(hex, opacity)
}

// ═══════════════════════════════════════════════════════════════
// Ease curve helpers
// ═══════════════════════════════════════════════════════════════

pub(crate) fn ease_from_sharpness(sharpness: f64) -> ([f64; 2], [f64; 2]) {
    let s = sharpness.max(0.0).min(1.0);
    let out_x = 0.42 * (1.0 - s) + 0.0 * s;
    let out_y = 0.0;
    let in_x = 0.58 * (1.0 - s) + 1.0 * s;
    let in_y = 1.0;
    ([out_x, out_y], [in_x, in_y])
}

// ═══════════════════════════════════════════════════════════════
// Serialization — Lottie v5.7.0 JSON
// ═══════════════════════════════════════════════════════════════

impl LottieAnimation {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let layers_json: Vec<Value> = self.layers.iter().map(layer_to_json).collect();

        let mut slots_map = Map::new();
        for (key, slot) in &self.slots {
            let mut p = Map::new();
            p.insert("v".to_string(), slot.default.clone());
            p.insert("l".to_string(), json!(slot.label));
            if let Some(v) = slot.min {
                p.insert("mn".to_string(), json!(v));
            }
            if let Some(v) = slot.max {
                p.insert("mx".to_string(), json!(v));
            }
            if let Some(v) = slot.step {
                p.insert("st".to_string(), json!(v));
            }
            let mut slot_obj = Map::new();
            slot_obj.insert("p".to_string(), Value::Object(p));
            slot_obj.insert("t".to_string(), json!(0));
            slots_map.insert(key.clone(), Value::Object(slot_obj));
        }

        let doc = json!({
            "v": "5.7.0",
            "fr": self.fps,
            "ip": 0,
            "op": self.duration_frames,
            "w": self.width,
            "h": self.height,
            "nm": self.name,
            "assets": [],
            "slots": Value::Object(slots_map),
            "layers": Value::Array(layers_json),
        });

        serde_json::to_string_pretty(&doc)
    }
}

fn pos_to_json(av: &AnimatedValue<[f64; 3]>) -> Value {
    match av {
        AnimatedValue::Static(v) => json!([v[0], v[1], v[2]]),
        AnimatedValue::Keyframed(kfs) => Value::Array(
            kfs.iter()
                .map(|kf| {
                    let mut m = Map::new();
                    m.insert("t".to_string(), json!(kf.frame));
                    m.insert(
                        "s".to_string(),
                        json!([kf.value[0], kf.value[1], kf.value[2]]),
                    );
                    m.insert("h".to_string(), json!(0));
                    if let Some(eo) = &kf.ease_out {
                        let mut o = Map::new();
                        o.insert("x".to_string(), json!([eo[0]]));
                        o.insert("y".to_string(), json!([eo[1]]));
                        m.insert("o".to_string(), Value::Object(o));
                    }
                    if let Some(ei) = &kf.ease_in {
                        let mut i = Map::new();
                        i.insert("x".to_string(), json!([ei[0]]));
                        i.insert("y".to_string(), json!([ei[1]]));
                        m.insert("i".to_string(), Value::Object(i));
                    }
                    Value::Object(m)
                })
                .collect(),
        ),
    }
}

fn scalar_to_json(av: &AnimatedValue<f64>) -> Value {
    match av {
        AnimatedValue::Static(v) => json!(v),
        AnimatedValue::Keyframed(kfs) => Value::Array(
            kfs.iter()
                .map(|kf| {
                    let mut m = Map::new();
                    m.insert("t".to_string(), json!(kf.frame));
                    m.insert("s".to_string(), json!([kf.value]));
                    m.insert("h".to_string(), json!(0));
                    if let Some(eo) = &kf.ease_out {
                        let mut o = Map::new();
                        o.insert("x".to_string(), json!([eo[0]]));
                        o.insert("y".to_string(), json!([eo[1]]));
                        m.insert("o".to_string(), Value::Object(o));
                    }
                    if let Some(ei) = &kf.ease_in {
                        let mut i = Map::new();
                        i.insert("x".to_string(), json!([ei[0]]));
                        i.insert("y".to_string(), json!([ei[1]]));
                        m.insert("i".to_string(), Value::Object(i));
                    }
                    Value::Object(m)
                })
                .collect(),
        ),
    }
}

fn layer_to_json(layer: &LottieLayer) -> Value {
    let mut m = Map::new();
    m.insert("nm".to_string(), json!(layer.name));
    m.insert("ty".to_string(), json!(layer.ty));
    m.insert("ind".to_string(), json!(layer.index));
    m.insert("ip".to_string(), json!(layer.in_point));
    m.insert("op".to_string(), json!(layer.out_point));
    m.insert("st".to_string(), json!(layer.start_time));
    if let Some(p) = layer.parent {
        m.insert("parent".to_string(), json!(p));
    }
    m.insert("ks".to_string(), transform_to_json(&layer.transform));

    if !layer.shapes.is_empty() {
        let shapes_json: Vec<Value> = layer.shapes.iter().map(shape_to_json).collect();
        m.insert("shapes".to_string(), Value::Array(shapes_json));
    }

    if !layer.effects.is_empty() {
        let effects_json: Vec<Value> = layer.effects.iter().map(effect_to_json).collect();
        m.insert("ef".to_string(), Value::Array(effects_json));
    }

    if !layer.masks.is_empty() {
        let masks_json: Vec<Value> = layer.masks.iter().map(mask_to_json).collect();
        m.insert("masksProperties".to_string(), Value::Array(masks_json));
    }

    Value::Object(m)
}

fn transform_to_json(tr: &LottieTransform) -> Value {
    json!({
        "p": {
            "a": if tr.position.is_animated() { 1 } else { 0 },
            "k": pos_to_json(&tr.position)
        },
        "s": {
            "a": if tr.scale.is_animated() { 1 } else { 0 },
            "k": pos_to_json(&tr.scale)
        },
        "r": {
            "a": if tr.rotation.is_animated() { 1 } else { 0 },
            "k": scalar_to_json(&tr.rotation)
        },
        "o": {
            "a": if tr.opacity.is_animated() { 1 } else { 0 },
            "k": scalar_to_json(&tr.opacity)
        },
        "a": [tr.anchor[0], tr.anchor[1]]
    })
}

fn shape_to_json(shape: &LottieShape) -> Value {
    match shape {
        LottieShape::Group {
            name,
            items,
            transform,
        } => {
            let mut m = Map::new();
            m.insert("ty".to_string(), json!("gr"));
            m.insert("nm".to_string(), json!(name));
            m.insert(
                "it".to_string(),
                Value::Array(items.iter().map(shape_to_json).collect()),
            );
            m.insert("tr".to_string(), transform_to_json(transform));
            Value::Object(m)
        }
        LottieShape::Rectangle {
            position,
            size,
            radius,
        } => {
            json!({
                "ty": "rc",
                "d": 1,
                "p": {"k": [position[0], position[1]]},
                "s": {"k": [size[0], size[1]]},
                "r": {"k": radius}
            })
        }
        LottieShape::Ellipse { position, size } => {
            json!({
                "ty": "el",
                "d": 1,
                "p": {"k": [position[0], position[1]]},
                "s": {"k": [size[0], size[1]]}
            })
        }
        LottieShape::Path { vertices } => {
            let v: Vec<Vec<f64>> = vertices.iter().map(|bv| vec![bv.x, bv.y]).collect();
            let i: Vec<Vec<f64>> = vertices
                .iter()
                .map(|bv| {
                    bv.in_tangent
                        .map(|t| vec![t[0], t[1]])
                        .unwrap_or(vec![0.0, 0.0])
                })
                .collect();
            let o: Vec<Vec<f64>> = vertices
                .iter()
                .map(|bv| {
                    bv.out_tangent
                        .map(|t| vec![t[0], t[1]])
                        .unwrap_or(vec![0.0, 0.0])
                })
                .collect();
            let closed = vertices.last().map(|v| v.closed).unwrap_or(false);
            json!({
                "ty": "sh",
                "d": 1,
                "ks": {"k": {
                    "v": v,
                    "i": i,
                    "o": o,
                    "c": closed
                }}
            })
        }
        LottieShape::Fill { color, opacity } => {
            json!({
                "ty": "fl",
                "d": 1,
                "c": {"k": [color[0], color[1], color[2], color[3]]},
                "o": {"k": opacity}
            })
        }
        LottieShape::Stroke {
            color,
            width,
            opacity,
        } => {
            json!({
                "ty": "st",
                "d": 1,
                "c": {"k": [color[0], color[1], color[2], color[3]]},
                "o": {"k": opacity},
                "w": {"k": width}
            })
        }
        LottieShape::Trim { start, end, offset } => {
            json!({
                "ty": "tm",
                "d": 1,
                "s": {"k": start},
                "e": {"k": end},
                "o": {"k": offset}
            })
        }
        LottieShape::Repeater {
            copies,
            offset,
            transform,
        } => {
            json!({
                "ty": "rp",
                "d": 1,
                "c": {"k": *copies},
                "o": {"k": offset},
                "tr": transform_to_json(transform)
            })
        }
        LottieShape::Merge { mode } => {
            let mm = match mode {
                MergeMode::Merge => 1,
                MergeMode::Add => 2,
                MergeMode::Subtract => 3,
                MergeMode::Intersect => 4,
                MergeMode::Exclude => 5,
            };
            json!({
                "ty": "mm",
                "d": 1,
                "mm": mm
            })
        }
    }
}

fn effect_to_json(effect: &LottieEffect) -> Value {
    json!({
        "nm": effect.name,
        "ty": effect.ty
    })
}

fn mask_to_json(mask: &LottieMask) -> Value {
    let mut m = Map::new();
    m.insert("inv".to_string(), json!(if mask.inverted { 1 } else { 0 }));
    m.insert("pt".to_string(), shape_to_json(&mask.shape));
    Value::Object(m)
}
