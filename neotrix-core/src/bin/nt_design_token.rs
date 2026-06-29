//! nt_design_token — CLI for design-token introspection and ffmpeg filter generation.
//!
//! Commands:
//!   list          — list all tokens with type + semantic path
//!   resolve <n>   — resolve a token name, print its value
//!   filter <s>    — generate an ffmpeg filter chain for a scene type
//!   diagnostic    — full DesignTokenIntegrator diagnostic

use clap::Parser;
use log;
use neotrix::core::nt_core_design_token::{DesignTokenIntegrator, TokenRegistry, TokenValue};

#[derive(Parser, Debug)]
#[command(
    name = "nt_design_token",
    about = "Design token → ffmpeg filter chain generator"
)]
enum Cli {
    List,
    Resolve {
        name: String,
        #[arg(short, long, help = "Output raw value only (no units, no metadata)")]
        raw: bool,
    },
    Filter {
        scene_type: String,
    },
    Diagnostic,
}

fn format_token_value(value: &TokenValue) -> String {
    match value {
        TokenValue::Color { r, g, b, a } => {
            format!(
                "rgba({:.0},{:.0},{:.0},{:.2})",
                r * 255.0,
                g * 255.0,
                b * 255.0,
                a
            )
        }
        TokenValue::Spacing(v) => format!("{}px", v),
        TokenValue::Easing { x1, y1, x2, y2 } => {
            format!("cubic-bezier({},{},{},{})", x1, y1, x2, y2)
        }
        TokenValue::Shadow {
            offset_x,
            offset_y,
            blur,
            spread,
            r,
            g,
            b,
            a,
        } => {
            format!(
                "inset? {}px {}px {}px {}px rgba({:.0},{:.0},{:.0},{:.2})",
                offset_x,
                offset_y,
                blur,
                spread,
                r * 255.0,
                g * 255.0,
                b * 255.0,
                a
            )
        }
        TokenValue::Motion {
            duration_ms,
            stiffness,
            damping,
        } => {
            format!(
                "{}ms spring(stiffness={},damping={})",
                duration_ms, stiffness, damping
            )
        }
        TokenValue::Font {
            family,
            size,
            weight,
        } => {
            format!("{} {}px weight={}", family, size, weight)
        }
        TokenValue::Radius(v) => format!("{}px", v),
        TokenValue::Opacity(v) => format!("{:.2}", v),
    }
}

fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::List => {
            let reg = TokenRegistry::default();
            for token in &reg.tokens {
                let path = token.semantic_path.join("/");
                log::info!(
                    "{}  [{}]  path={}",
                    token.name,
                    token.token_type.name(),
                    path
                );
            }
        }
        Cli::Resolve { name, raw } => {
            let reg = TokenRegistry::default();
            match reg.resolve(&name) {
                Some(token) => {
                    let val = format_token_value(&token.value);
                    if raw {
                        let raw_val = match &token.value {
                            TokenValue::Spacing(v) => format!("{}", v),
                            TokenValue::Radius(v) => format!("{}", v),
                            TokenValue::Opacity(v) => format!("{:.3}", v),
                            TokenValue::Color { r, g, b, a } => format!(
                                "{:.0},{:.0},{:.0},{:.2}",
                                r * 255.0,
                                g * 255.0,
                                b * 255.0,
                                a
                            ),
                            _ => val,
                        };
                        log::info!("{}", raw_val);
                    } else {
                        log::info!(
                            "{} = {}  [{}]  {}",
                            token.name,
                            val,
                            token.token_type.name(),
                            token.description
                        );
                    }
                }
                None => {
                    log::error!("error: token '{}' not found", name);
                    std::process::exit(1);
                }
            }
        }
        Cli::Filter { scene_type } => {
            let integrator = DesignTokenIntegrator::new();
            let chain = integrator.generate_filter_chain(&scene_type);
            log::info!("{}", chain);
        }
        Cli::Diagnostic => {
            let integrator = DesignTokenIntegrator::new();
            log::info!("{}", integrator.diagnostic());
        }
    }
}
