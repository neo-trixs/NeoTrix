// ── Re-export hub ──
// self_measure split into self_measure_impl/ for maintainability.
// Sub-modules: subsystem, snapshot, pid, ring, engine, pairwise, display, tests.

mod self_measure_impl;
pub use self_measure_impl::*;
