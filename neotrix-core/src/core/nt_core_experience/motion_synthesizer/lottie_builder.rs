use std::collections::HashMap;

use super::serializer::{ease_from_sharpness, hex_to_rgba, hex_to_rgba_alpha, value_color_rgba};
use super::types::{
    AnimatedValue, BezierVertex, Keyframe, LottieAnimation, LottieLayer, LottieShape,
    LottieTransform, SlotDef,
};
use super::MotionSynthesizer;
use crate::core::nt_core_experience::identity_generator::VisualSignature;
use crate::core::nt_core_value_system::CoreValue;
use serde_json::json;

// ═══════════════════════════════════════════════════════════════
// SVG Path Parsing (simplified)
// ═══════════════════════════════════════════════════════════════

fn parse_svg_path_to_vertices(path_data: &str, cx: f64, cy: f64) -> Vec<BezierVertex> {
    let data = path_data.trim();
    if data.is_empty() {
        return vec![];
    }

    let mut verts = Vec::new();
    let mut _first_x = 0.0_f64;
    let mut _first_y = 0.0_f64;
    let mut prev_x = 0.0_f64;
    let mut prev_y = 0.0_f64;
    let mut i = 0;
    let bytes = data.as_bytes();

    while i < bytes.len() {
        let cmd = bytes[i] as char;
        if !cmd.is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        i += 1;

        match cmd {
            'M' | 'm' => {
                if let Some((x, y, n)) = parse_pair(&bytes[i..], prev_x, prev_y, cmd == 'm') {
                    let x = x + cx;
                    let y = y + cy;
                    _first_x = x;
                    _first_y = y;
                    prev_x = x;
                    prev_y = y;
                    i += n;
                }
            }
            'L' | 'l' => {
                while let Some((x, y, n)) = parse_pair(&bytes[i..], prev_x, prev_y, cmd == 'l') {
                    let x = x + cx;
                    let y = y + cy;
                    verts.push(BezierVertex {
                        x,
                        y,
                        in_tangent: Some([0.0, 0.0]),
                        out_tangent: Some([0.0, 0.0]),
                        closed: false,
                    });
                    prev_x = x;
                    prev_y = y;
                    i += n;
                }
            }
            'C' | 'c' => {
                let rel = cmd == 'c';
                loop {
                    if let Some((x1, y1, n1)) = parse_pair(&bytes[i..], prev_x, prev_y, rel) {
                        if let Some((x2, y2, n2)) =
                            parse_pair(&bytes[i + n1..], prev_x, prev_y, rel)
                        {
                            if let Some((x, y, n3)) =
                                parse_pair(&bytes[i + n1 + n2..], prev_x, prev_y, rel)
                            {
                                let x = x + cx;
                                let y = y + cy;
                                verts.push(BezierVertex {
                                    x,
                                    y,
                                    in_tangent: Some([-(x2 + cx - x), -(y2 + cy - y)]),
                                    out_tangent: Some([x1 + cx - prev_x, y1 + cy - prev_y]),
                                    closed: false,
                                });
                                prev_x = x;
                                prev_y = y;
                                i += n1 + n2 + n3;
                                continue;
                            }
                        }
                    }
                    break;
                }
            }
            'Z' | 'z' => {
                if let Some(last) = verts.last_mut() {
                    last.closed = true;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    verts
}

fn parse_pair(bytes: &[u8], ref_x: f64, ref_y: f64, relative: bool) -> Option<(f64, f64, usize)> {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() || bytes[i] == b',' {
        i += 1;
    }
    if i >= bytes.len() || bytes[i].is_ascii_alphabetic() {
        return None;
    }
    let (x_str, n1) = parse_number(&bytes[i..]);
    if n1 == 0 {
        return None;
    }
    i += n1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() || bytes[i] == b',' {
        i += 1;
    }
    let (y_str, n2) = parse_number(&bytes[i..]);
    if n2 == 0 {
        return None;
    }
    let x: f64 = x_str.parse().ok()?;
    let y: f64 = y_str.parse().ok()?;
    if relative {
        Some((ref_x + x, ref_y + y, n1 + n2 + (i - (n1))))
    } else {
        Some((x, y, n1 + n2 + (i - (n1))))
    }
}

fn parse_number(bytes: &[u8]) -> (&str, usize) {
    let mut i = 0;
    if i < bytes.len() && bytes[i] == b'-' {
        i += 1;
    }
    while i < bytes.len()
        && (bytes[i].is_ascii_digit()
            || bytes[i] == b'.'
            || bytes[i] == b'e'
            || bytes[i] == b'E'
            || bytes[i] == b'+'
            || bytes[i] == b'-')
    {
        if bytes[i] == b'e' || bytes[i] == b'E' {
            i += 1;
            if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    let s = std::str::from_utf8(&bytes[..i]).unwrap_or("");
    (s, i)
}

// ═══════════════════════════════════════════════════════════════
// Animation Generators
// ═══════════════════════════════════════════════════════════════

impl MotionSynthesizer {
    // ── bouncing_logo ────────────────────────────────────────

    pub fn bouncing_logo(&self, signature: &VisualSignature) -> LottieAnimation {
        let primary = hex_to_rgba(&signature.palette.primary_hex);
        let accent = hex_to_rgba(&signature.palette.accent_hex);
        let _bg = hex_to_rgba_alpha(&signature.palette.bg_hex, 0.0);
        let sharpness = signature.geometry.sharpness;
        let (ease_out, ease_in) = ease_from_sharpness(sharpness);

        let duration = 120;
        let cx = 256.0;
        let cy = 140.0;
        let bounce_height = 180.0;

        let pos_kfs = vec![
            Keyframe {
                frame: 0.0,
                value: [cx, cy, 0.0],
                ease_out: Some(ease_out),
                ease_in: Some([0.42, 0.0]),
            },
            Keyframe {
                frame: 30.0,
                value: [cx, cy + bounce_height, 0.0],
                ease_out: Some([0.58, 1.0]),
                ease_in: Some(ease_in),
            },
            Keyframe {
                frame: 60.0,
                value: [cx, cy, 0.0],
                ease_out: Some(ease_out),
                ease_in: Some([0.42, 0.0]),
            },
            Keyframe {
                frame: 90.0,
                value: [cx, cy + bounce_height, 0.0],
                ease_out: Some([0.58, 1.0]),
                ease_in: Some(ease_in),
            },
            Keyframe {
                frame: 120.0,
                value: [cx, cy, 0.0],
                ease_out: None,
                ease_in: None,
            },
        ];

        let scale_kfs = vec![
            Keyframe {
                frame: 0.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 27.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 30.0,
                value: [120.0, 80.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 33.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 57.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 60.0,
                value: [120.0, 80.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 63.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 87.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 90.0,
                value: [120.0, 80.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 93.0,
                value: [100.0, 100.0, 100.0],
                ease_out: None,
                ease_in: None,
            },
        ];

        let diamond_verts: Vec<BezierVertex> =
            vec![(0.0, -40.0), (30.0, 0.0), (0.0, 40.0), (-30.0, 0.0)]
                .into_iter()
                .map(|(x, y)| BezierVertex {
                    x,
                    y,
                    in_tangent: Some([0.0, 0.0]),
                    out_tangent: Some([0.0, 0.0]),
                    closed: true,
                })
                .collect();

        let layer = LottieLayer {
            name: "Diamond Logo".into(),
            ty: 4,
            index: 1,
            parent: None,
            in_point: 0,
            out_point: duration,
            start_time: 0.0,
            transform: LottieTransform {
                position: AnimatedValue::Keyframed(pos_kfs),
                scale: AnimatedValue::Keyframed(scale_kfs),
                rotation: AnimatedValue::Static(45.0),
                opacity: AnimatedValue::Static(100.0),
                anchor: [0.0, 0.0],
            },
            shapes: vec![
                LottieShape::Path {
                    vertices: diamond_verts,
                },
                LottieShape::Fill {
                    color: primary,
                    opacity: 100.0,
                },
                LottieShape::Stroke {
                    color: accent,
                    width: 3.0,
                    opacity: 100.0,
                },
            ],
            effects: vec![],
            masks: vec![],
        };

        let mut slots = HashMap::new();
        slots.insert(
            "bg-color".into(),
            SlotDef {
                default: json!(signature.palette.bg_hex),
                label: "Background Color".into(),
                min: None,
                max: None,
                step: None,
            },
        );
        slots.insert(
            "primary-color".into(),
            SlotDef {
                default: json!(signature.palette.primary_hex),
                label: "Primary Color".into(),
                min: None,
                max: None,
                step: None,
            },
        );

        LottieAnimation {
            name: format!("Bouncing Logo — {}", signature.palette.primary_hex),
            fps: 60.0,
            duration_frames: duration,
            width: 512,
            height: 512,
            layers: vec![layer],
            slots,
        }
    }

    // ── orbital_rings ────────────────────────────────────────

    pub fn orbital_rings(&self, signature: &VisualSignature) -> LottieAnimation {
        let primary = hex_to_rgba(&signature.palette.primary_hex);
        let accent = hex_to_rgba(&signature.palette.accent_hex);
        let secondary = hex_to_rgba(&signature.palette.secondary_hex);
        let complexity = signature.geometry.complexity.max(0.5).min(3.0) as i32;

        let duration = 180;
        let cx = 256.0;
        let cy = 256.0;
        let ring_radii = [60.0, 100.0, 140.0];

        let mut layers = Vec::new();

        // Core diamond in center
        let core_diamond: Vec<BezierVertex> =
            vec![(0.0, -25.0), (18.0, 0.0), (0.0, 25.0), (-18.0, 0.0)]
                .into_iter()
                .map(|(x, y)| BezierVertex {
                    x,
                    y,
                    in_tangent: Some([0.0, 0.0]),
                    out_tangent: Some([0.0, 0.0]),
                    closed: true,
                })
                .collect();

        layers.push(LottieLayer {
            name: "Core Diamond".into(),
            ty: 4,
            index: 10,
            parent: None,
            in_point: 0,
            out_point: duration,
            start_time: 0.0,
            transform: LottieTransform {
                position: AnimatedValue::Static([cx, cy, 0.0]),
                scale: AnimatedValue::Static([100.0, 100.0, 100.0]),
                rotation: AnimatedValue::Keyframed(vec![
                    Keyframe {
                        frame: 0.0,
                        value: 0.0,
                        ease_out: None,
                        ease_in: None,
                    },
                    Keyframe {
                        frame: duration as f64,
                        value: 360.0,
                        ease_out: None,
                        ease_in: None,
                    },
                ]),
                opacity: AnimatedValue::Static(100.0),
                anchor: [0.0, 0.0],
            },
            shapes: vec![
                LottieShape::Path {
                    vertices: core_diamond,
                },
                LottieShape::Fill {
                    color: primary,
                    opacity: 100.0,
                },
            ],
            effects: vec![],
            masks: vec![],
        });

        // Orbital rings
        let ring_colors = [accent, secondary, primary];
        for (i, &radius) in ring_radii.iter().enumerate() {
            let speed_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            let rotation_kfs = vec![
                Keyframe {
                    frame: 0.0,
                    value: 0.0,
                    ease_out: None,
                    ease_in: None,
                },
                Keyframe {
                    frame: duration as f64,
                    value: 360.0 * speed_sign,
                    ease_out: None,
                    ease_in: None,
                },
            ];

            layers.push(LottieLayer {
                name: format!("Ring {}", i + 1),
                ty: 4,
                index: 9 - i as i32,
                parent: None,
                in_point: 0,
                out_point: duration,
                start_time: 0.0,
                transform: LottieTransform {
                    position: AnimatedValue::Static([cx, cy, 0.0]),
                    scale: AnimatedValue::Static([100.0, 100.0, 100.0]),
                    rotation: AnimatedValue::Keyframed(rotation_kfs),
                    opacity: AnimatedValue::Static(60.0 + (i as f64 * 15.0)),
                    anchor: [0.0, 0.0],
                },
                shapes: vec![
                    LottieShape::Ellipse {
                        position: [0.0, 0.0],
                        size: [radius * 2.0, radius * 2.0],
                    },
                    LottieShape::Stroke {
                        color: ring_colors[i],
                        width: 2.0,
                        opacity: 80.0,
                    },
                ],
                effects: vec![],
                masks: vec![],
            });

            // Orbital nodes on this ring
            let num_nodes = (2 + (complexity as usize)).min(5);
            for n in 0..num_nodes {
                let angle = (n as f64 / num_nodes as f64) * std::f64::consts::TAU;
                let nx = angle.cos() * radius;
                let ny = angle.sin() * radius;
                let node_size = 6.0 + (i as f64 * 2.0);

                let node_pos_kfs = vec![
                    Keyframe {
                        frame: 0.0,
                        value: [cx + nx, cy + ny, 0.0],
                        ease_out: None,
                        ease_in: None,
                    },
                    Keyframe {
                        frame: duration as f64,
                        value: [
                            cx + (angle + std::f64::consts::TAU * speed_sign).cos() * radius,
                            cy + (angle + std::f64::consts::TAU * speed_sign).sin() * radius,
                            0.0,
                        ],
                        ease_out: None,
                        ease_in: None,
                    },
                ];

                // Pulsing opacity for nodes
                let pulse_offset = n as f64 * (duration as f64 / num_nodes as f64);
                let opacity_kfs = vec![
                    Keyframe {
                        frame: pulse_offset % duration as f64,
                        value: 100.0,
                        ease_out: None,
                        ease_in: None,
                    },
                    Keyframe {
                        frame: (pulse_offset + 30.0) % duration as f64,
                        value: 30.0,
                        ease_out: None,
                        ease_in: None,
                    },
                    Keyframe {
                        frame: (pulse_offset + 60.0) % duration as f64,
                        value: 100.0,
                        ease_out: None,
                        ease_in: None,
                    },
                ];

                layers.push(LottieLayer {
                    name: format!("Node {}-{}", i + 1, n + 1),
                    ty: 4,
                    index: 5 - (i * 2 + n) as i32,
                    parent: None,
                    in_point: 0,
                    out_point: duration,
                    start_time: 0.0,
                    transform: LottieTransform {
                        position: AnimatedValue::Keyframed(node_pos_kfs),
                        scale: AnimatedValue::Static([100.0, 100.0, 100.0]),
                        rotation: AnimatedValue::Static(0.0),
                        opacity: AnimatedValue::Keyframed(opacity_kfs),
                        anchor: [0.0, 0.0],
                    },
                    shapes: vec![
                        LottieShape::Ellipse {
                            position: [0.0, 0.0],
                            size: [node_size, node_size],
                        },
                        LottieShape::Fill {
                            color: ring_colors[i],
                            opacity: 100.0,
                        },
                    ],
                    effects: vec![],
                    masks: vec![],
                });
            }
        }

        let mut slots = HashMap::new();
        slots.insert(
            "bg-color".into(),
            SlotDef {
                default: json!(signature.palette.bg_hex),
                label: "Background Color".into(),
                min: None,
                max: None,
                step: None,
            },
        );

        LottieAnimation {
            name: format!("Orbital Rings — {}", signature.palette.primary_hex),
            fps: 60.0,
            duration_frames: duration,
            width: 512,
            height: 512,
            layers,
            slots,
        }
    }

    // ── pulse_heartbeat ──────────────────────────────────────

    pub fn pulse_heartbeat(&self, signature: &VisualSignature) -> LottieAnimation {
        let primary = hex_to_rgba(&signature.palette.primary_hex);
        let accent = hex_to_rgba(&signature.palette.accent_hex);
        let warmth = signature.palette.warmth;

        let duration = 120;
        let cx = 256.0;
        let cy = 256.0;

        // Glow layer — larger, fades in and out
        let glow_opacity_kfs = vec![
            Keyframe {
                frame: 0.0,
                value: 20.0,
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 20.0,
                value: 50.0,
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 40.0,
                value: 20.0,
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 60.0,
                value: 15.0,
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 80.0,
                value: 45.0,
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 100.0,
                value: 20.0,
                ease_out: None,
                ease_in: None,
            },
        ];

        let glow_scale_kfs = vec![
            Keyframe {
                frame: 0.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 20.0,
                value: [130.0, 130.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 40.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 60.0,
                value: [95.0, 95.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 80.0,
                value: [125.0, 125.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 100.0,
                value: [100.0, 100.0, 100.0],
                ease_out: None,
                ease_in: None,
            },
        ];

        // Main circle — core beat
        let main_scale_kfs = vec![
            Keyframe {
                frame: 0.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 15.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 20.0,
                value: [115.0, 115.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 25.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 55.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 60.0,
                value: [100.0, 100.0, 100.0],
                ease_out: Some([0.25, 0.0]),
                ease_in: Some([0.75, 1.0]),
            },
            Keyframe {
                frame: 65.0,
                value: [112.0, 112.0, 100.0],
                ease_out: Some([0.42, 0.0]),
                ease_in: Some([0.58, 1.0]),
            },
            Keyframe {
                frame: 70.0,
                value: [100.0, 100.0, 100.0],
                ease_out: None,
                ease_in: None,
            },
        ];

        let glow_color = if warmth > 0.5 {
            hex_to_rgba_alpha(&signature.palette.accent_hex, 0.3)
        } else {
            hex_to_rgba_alpha(&signature.palette.primary_hex, 0.3)
        };

        let layers = vec![
            // Glow ring
            LottieLayer {
                name: "Glow".into(),
                ty: 4,
                index: 2,
                parent: None,
                in_point: 0,
                out_point: duration,
                start_time: 0.0,
                transform: LottieTransform {
                    position: AnimatedValue::Static([cx, cy, 0.0]),
                    scale: AnimatedValue::Keyframed(glow_scale_kfs),
                    rotation: AnimatedValue::Static(0.0),
                    opacity: AnimatedValue::Keyframed(glow_opacity_kfs),
                    anchor: [0.0, 0.0],
                },
                shapes: vec![
                    LottieShape::Ellipse {
                        position: [0.0, 0.0],
                        size: [160.0, 160.0],
                    },
                    LottieShape::Fill {
                        color: glow_color,
                        opacity: 100.0,
                    },
                ],
                effects: vec![],
                masks: vec![],
            },
            // Main circle
            LottieLayer {
                name: "Core".into(),
                ty: 4,
                index: 1,
                parent: None,
                in_point: 0,
                out_point: duration,
                start_time: 0.0,
                transform: LottieTransform {
                    position: AnimatedValue::Static([cx, cy, 0.0]),
                    scale: AnimatedValue::Keyframed(main_scale_kfs),
                    rotation: AnimatedValue::Static(0.0),
                    opacity: AnimatedValue::Static(100.0),
                    anchor: [0.0, 0.0],
                },
                shapes: vec![
                    LottieShape::Ellipse {
                        position: [0.0, 0.0],
                        size: [100.0, 100.0],
                    },
                    LottieShape::Fill {
                        color: primary,
                        opacity: 100.0,
                    },
                    LottieShape::Stroke {
                        color: accent,
                        width: 3.0,
                        opacity: 100.0,
                    },
                ],
                effects: vec![],
                masks: vec![],
            },
        ];

        let mut slots = HashMap::new();
        slots.insert(
            "bg-color".into(),
            SlotDef {
                default: json!(signature.palette.bg_hex),
                label: "Background Color".into(),
                min: None,
                max: None,
                step: None,
            },
        );

        LottieAnimation {
            name: format!("Pulse Heartbeat — {}", signature.palette.primary_hex),
            fps: 60.0,
            duration_frames: duration,
            width: 512,
            height: 512,
            layers,
            slots,
        }
    }

    // ── path_reveal ──────────────────────────────────────────

    pub fn path_reveal(&self, svg_path_data: &str, signature: &VisualSignature) -> LottieAnimation {
        let accent = hex_to_rgba(&signature.palette.accent_hex);
        let sharpness = signature.geometry.sharpness;

        let duration = 90;
        let cx = 256.0;
        let cy = 256.0;

        let (ease_out, ease_in) = ease_from_sharpness(sharpness);

        // Parse SVG path data into bezier vertices (simplified: treat as straight line segments)
        let vertices = parse_svg_path_to_vertices(svg_path_data, cx, cy);

        #[allow(unused_variables)]
        let _trim_start_kfs = if vertices.is_empty() {
            vec![]
        } else {
            vec![
                Keyframe {
                    frame: 0.0,
                    value: 100.0,
                    ease_out: Some(ease_out),
                    ease_in: Some(ease_in),
                },
                Keyframe {
                    frame: duration as f64,
                    value: 0.0,
                    ease_out: None,
                    ease_in: None,
                },
            ]
        };

        #[allow(unused_variables)]
        let _trim_end_kfs = vec![
            Keyframe {
                frame: 0.0,
                value: 0.0,
                ease_out: Some(ease_out),
                ease_in: Some(ease_in),
            },
            Keyframe {
                frame: duration as f64,
                value: 100.0,
                ease_out: None,
                ease_in: None,
            },
        ];

        let layer = LottieLayer {
            name: "Revealed Path".into(),
            ty: 4,
            index: 1,
            parent: None,
            in_point: 0,
            out_point: duration,
            start_time: 0.0,
            transform: LottieTransform {
                position: AnimatedValue::Static([0.0, 0.0, 0.0]),
                scale: AnimatedValue::Static([100.0, 100.0, 100.0]),
                rotation: AnimatedValue::Static(0.0),
                opacity: AnimatedValue::Static(100.0),
                anchor: [0.0, 0.0],
            },
            shapes: vec![
                LottieShape::Path {
                    vertices: vertices.clone(),
                },
                LottieShape::Stroke {
                    color: accent,
                    width: 4.0,
                    opacity: 100.0,
                },
                LottieShape::Trim {
                    start: 0.0,
                    end: 100.0,
                    offset: 0.0,
                },
            ],
            effects: vec![],
            masks: vec![],
        };

        let mut slots = HashMap::new();
        slots.insert(
            "stroke-color".into(),
            SlotDef {
                default: json!(signature.palette.accent_hex),
                label: "Stroke Color".into(),
                min: None,
                max: None,
                step: None,
            },
        );

        LottieAnimation {
            name: format!("Path Reveal — {}", &signature.palette.accent_hex),
            fps: 60.0,
            duration_frames: duration,
            width: 512,
            height: 512,
            layers: vec![layer],
            slots,
        }
    }

    // ── value_bloom ──────────────────────────────────────────

    pub fn value_bloom(&self, signature: &VisualSignature) -> LottieAnimation {
        let all_values = [
            CoreValue::Curiosity,
            CoreValue::KnowledgeGrowth,
            CoreValue::Coherence,
            CoreValue::Autonomy,
            CoreValue::Helpfulness,
            CoreValue::Truthfulness,
            CoreValue::Efficiency,
        ];

        let dominant = signature.dominant_value;
        let complexity = signature.geometry.complexity;

        let duration = 150;
        let cx = 256.0;
        let cy = 256.0;

        let num_active = (2.0 + complexity * 5.0).round().min(7.0) as usize;

        let mut layers = Vec::new();
        let mut index = 10i32;

        // Central core — dominant value
        if let Some(dom) = dominant {
            let dom_color = value_color_rgba(dom, 1.0);

            let core_scale_kfs = vec![
                Keyframe {
                    frame: 0.0,
                    value: [0.0, 0.0, 100.0],
                    ease_out: Some([0.0, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 30.0,
                    value: [100.0, 100.0, 100.0],
                    ease_out: Some([0.42, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 75.0,
                    value: [110.0, 110.0, 100.0],
                    ease_out: Some([0.42, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 150.0,
                    value: [120.0, 120.0, 100.0],
                    ease_out: None,
                    ease_in: None,
                },
            ];

            let core_rot_kfs = vec![
                Keyframe {
                    frame: 0.0,
                    value: 0.0,
                    ease_out: None,
                    ease_in: None,
                },
                Keyframe {
                    frame: duration as f64,
                    value: 360.0,
                    ease_out: None,
                    ease_in: None,
                },
            ];

            layers.push(LottieLayer {
                name: format!("Core {:?}", dom),
                ty: 4,
                index: index,
                parent: None,
                in_point: 0,
                out_point: duration,
                start_time: 0.0,
                transform: LottieTransform {
                    position: AnimatedValue::Static([cx, cy, 0.0]),
                    scale: AnimatedValue::Keyframed(core_scale_kfs),
                    rotation: AnimatedValue::Keyframed(core_rot_kfs),
                    opacity: AnimatedValue::Static(100.0),
                    anchor: [0.0, 0.0],
                },
                shapes: vec![
                    LottieShape::Path {
                        vertices: vec![(0.0, -30.0), (22.0, 0.0), (0.0, 30.0), (-22.0, 0.0)]
                            .into_iter()
                            .map(|(x, y)| BezierVertex {
                                x,
                                y,
                                in_tangent: Some([0.0, 0.0]),
                                out_tangent: Some([0.0, 0.0]),
                                closed: true,
                            })
                            .collect(),
                    },
                    LottieShape::Fill {
                        color: dom_color,
                        opacity: 100.0,
                    },
                ],
                effects: vec![],
                masks: vec![],
            });
            index -= 1;
        }

        // Bloom particles for each active value
        for (i, value) in all_values.iter().enumerate().take(num_active) {
            if Some(*value) == dominant {
                continue;
            }

            let angle = (i as f64 / num_active as f64) * std::f64::consts::TAU;
            let bloom_radius = 60.0 + (i as f64 * 20.0);
            let target_x = angle.cos() * bloom_radius;
            let target_y = angle.sin() * bloom_radius;

            let color = value_color_rgba(*value, 1.0);
            let is_dominant = Some(*value) == dominant;
            let particle_size = if is_dominant {
                28.0
            } else {
                14.0 + i as f64 * 3.0
            };

            // Position: from center outward
            let pos_kfs = vec![
                Keyframe {
                    frame: 0.0,
                    value: [cx, cy, 0.0],
                    ease_out: Some([0.42, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 45.0,
                    value: [cx + target_x * 0.5, cy + target_y * 0.5, 0.0],
                    ease_out: Some([0.42, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 90.0,
                    value: [cx + target_x, cy + target_y, 0.0],
                    ease_out: None,
                    ease_in: None,
                },
            ];

            // Scale: grow from 0 to target
            let scale_kfs = vec![
                Keyframe {
                    frame: 0.0,
                    value: [0.0, 0.0, 100.0],
                    ease_out: Some([0.0, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 30.0 + (i as f64 * 5.0),
                    value: [100.0, 100.0, 100.0],
                    ease_out: None,
                    ease_in: None,
                },
            ];

            // Opacity: fade in, then pulse
            let opacity_kfs = vec![
                Keyframe {
                    frame: 0.0,
                    value: 0.0,
                    ease_out: Some([0.0, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 20.0,
                    value: 100.0,
                    ease_out: Some([0.42, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 75.0,
                    value: 80.0,
                    ease_out: Some([0.42, 0.0]),
                    ease_in: Some([0.58, 1.0]),
                },
                Keyframe {
                    frame: 100.0,
                    value: 100.0,
                    ease_out: None,
                    ease_in: None,
                },
            ];

            layers.push(LottieLayer {
                name: format!("Bloom {:?}", value),
                ty: 4,
                index: index,
                parent: None,
                in_point: 0,
                out_point: duration,
                start_time: 0.0,
                transform: LottieTransform {
                    position: AnimatedValue::Keyframed(pos_kfs),
                    scale: AnimatedValue::Keyframed(scale_kfs),
                    rotation: AnimatedValue::Static(0.0),
                    opacity: AnimatedValue::Keyframed(opacity_kfs),
                    anchor: [0.0, 0.0],
                },
                shapes: vec![
                    LottieShape::Ellipse {
                        position: [0.0, 0.0],
                        size: [particle_size, particle_size],
                    },
                    LottieShape::Fill {
                        color,
                        opacity: 100.0,
                    },
                ],
                effects: vec![],
                masks: vec![],
            });
            index -= 1;
        }

        let mut slots = HashMap::new();
        slots.insert(
            "bg-color".into(),
            SlotDef {
                default: json!(signature.palette.bg_hex),
                label: "Background Color".into(),
                min: None,
                max: None,
                step: None,
            },
        );

        LottieAnimation {
            name: format!(
                "Value Bloom — {:?}",
                dominant.unwrap_or(CoreValue::Coherence)
            ),
            fps: 60.0,
            duration_frames: duration,
            width: 512,
            height: 512,
            layers,
            slots,
        }
    }
}
