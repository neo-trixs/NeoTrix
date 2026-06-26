use serde_json::Value;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// Core Lottie Types
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct LottieAnimation {
    pub name: String,
    pub fps: f64,
    pub duration_frames: i32,
    pub width: i32,
    pub height: i32,
    pub layers: Vec<LottieLayer>,
    pub slots: HashMap<String, SlotDef>,
}

#[derive(Debug, Clone)]
pub struct LottieLayer {
    pub name: String,
    pub ty: i32,
    pub index: i32,
    pub parent: Option<i32>,
    pub in_point: i32,
    pub out_point: i32,
    pub start_time: f64,
    pub transform: LottieTransform,
    pub shapes: Vec<LottieShape>,
    pub effects: Vec<LottieEffect>,
    pub masks: Vec<LottieMask>,
}

#[derive(Debug, Clone)]
pub struct LottieTransform {
    pub position: AnimatedValue<[f64; 3]>,
    pub scale: AnimatedValue<[f64; 3]>,
    pub rotation: AnimatedValue<f64>,
    pub opacity: AnimatedValue<f64>,
    pub anchor: [f64; 2],
}

impl Default for LottieTransform {
    fn default() -> Self {
        Self {
            position: AnimatedValue::Static([0.0, 0.0, 0.0]),
            scale: AnimatedValue::Static([100.0, 100.0, 100.0]),
            rotation: AnimatedValue::Static(0.0),
            opacity: AnimatedValue::Static(100.0),
            anchor: [0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AnimatedValue<T: Clone> {
    Static(T),
    Keyframed(Vec<Keyframe<T>>),
}

#[derive(Debug, Clone)]
pub struct Keyframe<T: Clone> {
    pub frame: f64,
    pub value: T,
    pub ease_in: Option<[f64; 2]>,
    pub ease_out: Option<[f64; 2]>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LottieShape {
    Group {
        name: String,
        items: Vec<LottieShape>,
        transform: LottieTransform,
    },
    Rectangle {
        position: [f64; 2],
        size: [f64; 2],
        radius: f64,
    },
    Ellipse {
        position: [f64; 2],
        size: [f64; 2],
    },
    Path {
        vertices: Vec<BezierVertex>,
    },
    Fill {
        color: [f64; 4],
        opacity: f64,
    },
    Stroke {
        color: [f64; 4],
        width: f64,
        opacity: f64,
    },
    Trim {
        start: f64,
        end: f64,
        offset: f64,
    },
    Repeater {
        copies: i32,
        offset: f64,
        transform: LottieTransform,
    },
    Merge {
        mode: MergeMode,
    },
}

#[derive(Debug, Clone)]
pub struct BezierVertex {
    pub x: f64,
    pub y: f64,
    pub in_tangent: Option<[f64; 2]>,
    pub out_tangent: Option<[f64; 2]>,
    pub closed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum MergeMode {
    Merge,
    Add,
    Subtract,
    Intersect,
    Exclude,
}

#[derive(Debug, Clone)]
pub struct SlotDef {
    pub default: Value,
    pub label: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct LottieEffect {
    pub name: String,
    pub ty: i32,
}

#[derive(Debug, Clone)]
pub struct LottieMask {
    pub inverted: bool,
    pub shape: LottieShape,
}

// ═══════════════════════════════════════════════════════════════
// AnimatedValue helpers
// ═══════════════════════════════════════════════════════════════

impl<T: Clone> AnimatedValue<T> {
    pub fn is_animated(&self) -> bool {
        matches!(self, AnimatedValue::Keyframed(_))
    }
}
