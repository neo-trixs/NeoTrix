// ── Re-export hub ──
// self_measure split into self_measure_impl/ for maintainability.
// Sub-modules: subsystem, snapshot, pid, ring, engine, pairwise, display, tests.

mod calib_weights;

mod self_measure_impl;
pub use calib_weights::{CALIB_MAX_PENALTY, CALIB_W_ECE, CALIB_W_HCE};
pub use self_measure_impl::*;
