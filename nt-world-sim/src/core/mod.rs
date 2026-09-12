pub mod entity;
pub mod world;
pub mod change_detection;
pub mod scheduler;
pub mod math;

pub use entity::{UniversalEntity, EntityId, ArchetypeId, Archetype, Chunk, ComponentStorage, SoAStorage};
pub use world::{UniversalWorld, Component, Resource, Event, ComponentTuple};
pub use change_detection::{Changed, ChangeTick, ChangeDetector, ChangeTracker};
pub use scheduler::{UniversalSystem, SystemDependency, ParallelScheduler};
pub use math::{Vec2, Rect, Color, Transform};
