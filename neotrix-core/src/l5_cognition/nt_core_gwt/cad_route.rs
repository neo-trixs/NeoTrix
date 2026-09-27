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
//! generation) rather than extending the enum. The slot registered by
//! `register_default_specialists` is reused (see `register_cad_gwt`); the
//! `"cad_generation"` name is only used as a fallback for a workspace that has
//! no ImageGenerator specialist yet.

use super::module_def::{SpecialistModule, SpecialistType};
use super::workspace::GlobalWorkspace;

/// Name used for the CAD-generation specialist module in the GWT registry.
pub const CAD_GWT_MODULE_NAME: &str = "cad_generation";

/// Register the CAD-generation specialist into the GWT `GlobalWorkspace`.
///
/// Mirrors `GlobalWorkspace::register` (workspace.rs:299): a `SpecialistModule`
/// is constructed from the reused `SpecialistType::ImageGenerator` variant. If
/// the workspace already carries an ImageGenerator specialist (the normal case
/// after `register_default_specialists`), that slot is reused and `true` is
/// returned without inserting a duplicate. Returns `false` only when there is no
/// ImageGenerator slot and the workspace is already at `MODULE_COUNT` capacity.
///
/// CAD generation is image-conditioned generation, so it maps onto the existing
/// `ImageGenerator` `SpecialistType` (no enum extension needed); the fallback
/// registration is keyed by the name `"cad_generation"`.
pub fn register_cad_gwt(ws: &mut GlobalWorkspace) -> bool {
    use SpecialistType::ImageGenerator;
    // Reuse the existing ImageGenerator slot: the GWT holds at most
    // MODULE_COUNT specialists and register_default_specialists already
    // occupies every SpecialistType. Inserting a second ImageGenerator
    // ("cad_generation") would shadow the default one — `specialist_at_index`
    // and `module_index` key off `specialist_type`, so the duplicate is
    // unreachable by resonance routing and just burns a slot.
    if ws.specialist_by_type_mut(&ImageGenerator).is_some() {
        return true;
    }
    let module = SpecialistModule::new(ImageGenerator, CAD_GWT_MODULE_NAME.to_string());
    ws.register(module)
}
