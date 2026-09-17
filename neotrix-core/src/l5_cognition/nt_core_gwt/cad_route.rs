//! CAD generation routing registration for the GWT (Global Workspace Theory) system.
//!
//! Wires image→CAD generation (GenCAD: CSR→CCIP→CDP→Decoder) as a routed
//! specialist/capability inside the consciousness core. This is the T3 production
//! wiring counterpart to the CAD SelfTests in `l2_world_impl/cad_selftest.rs`.
//!
//! Pattern mirrored from `nt_core_gwt/workspace.rs::register_default_specialists`
//! (workspace.rs:914): build a `SpecialistModule` and hand it to
//! `GlobalWorkspace::register`.
//!
//! NOTE: `SpecialistType` is exhaustively matched elsewhere (cognitive_type.rs:178),
//! so we REUSE the `ImageGenerator` variant (image→CAD is image-conditioned
//! generation) rather than extending the enum. The module is given the distinct
//! name `"cad_generation"`, so it is addressable independently of the generic
//! `ImageGenerator` module.

use super::module_def::{SpecialistModule, SpecialistType};
use super::workspace::GlobalWorkspace;

/// Name used for the CAD-generation specialist module in the GWT registry.
pub const CAD_GWT_MODULE_NAME: &str = "cad_generation";

/// Register the CAD-generation specialist into the GWT `GlobalWorkspace`.
///
/// Mirrors `GlobalWorkspace::register` (workspace.rs:299): a `SpecialistModule`
/// is constructed from the reused `SpecialistType::ImageGenerator` variant and
/// inserted into the workspace's `specialists` map. Returns `true` if the module
/// was registered, `false` if the workspace is already at `MODULE_COUNT` capacity.
///
/// CAD generation is image-conditioned generation, so it maps onto the existing
/// `ImageGenerator` `SpecialistType` (no enum extension needed). It is keyed by
/// the distinct name `"cad_generation"` so resonance/attention routing can target
/// it specifically when an image→CAD request enters the global workspace.
pub fn register_cad_gwt(ws: &mut GlobalWorkspace) -> bool {
    use SpecialistType::ImageGenerator;
    let module = SpecialistModule::new(ImageGenerator, CAD_GWT_MODULE_NAME.to_string());
    ws.register(module)
}
