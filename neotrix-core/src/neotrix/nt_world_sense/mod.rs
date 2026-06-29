//! Sensory perception + world consciousness integration layer.
//! Bridges pure core nt_world_sense types with the runtime system.

pub mod auditory_cortex;
pub mod nt_world_sense_hub;
pub mod omniscient_view;
pub mod real_sensors;
pub mod visual_cortex;
pub mod world_consciousness;

pub use auditory_cortex::*;
pub use nt_world_sense_hub::*;
pub use omniscient_view::*;
pub use visual_cortex::*;
pub use world_consciousness::*;

#[cfg(test)]
mod tests;
