use std::any::TypeId;
use std::collections::HashMap;

use super::world::{Component, UniversalWorld};
use super::entity::{EntityId, UniversalEntity};

pub struct Changed<T: Component> {
    pub value: T,
    pub tick: u64,
    pub changed: bool,
}

impl<T: Component> Changed<T> {
    pub fn new(value: T, tick: u64) -> Self {
        Self {
            value,
            tick,
            changed: false,
        }
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub fn was_changed(&self) -> bool {
        self.changed
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn inner(&self) -> &T {
        &self.value
    }

    pub fn inner_mut(&mut self) -> &mut T {
        self.changed = true;
        &mut self.value
    }
}

impl<T: Component> std::ops::Deref for Changed<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Component> std::ops::DerefMut for Changed<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.changed = true;
        &mut self.value
    }
}

pub struct ChangeTick {
    pub current: u64,
    pub last_read: u64,
}

impl ChangeTick {
    pub fn new() -> Self {
        Self {
            current: 0,
            last_read: 0,
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;
    }

    pub fn is_changed(&self, tick: u64) -> bool {
        tick > self.last_read
    }

    pub fn mark_read(&mut self) {
        self.last_read = self.current;
    }

    pub fn ticks_since(&self, tick: u64) -> u64 {
        self.current.saturating_sub(tick)
    }
}

impl Default for ChangeTick {
    fn default() -> Self {
        Self::new()
    }
}

struct ChangeRecord {
    tick: u64,
    changed: bool,
}

pub struct ChangeDetector {
    records: HashMap<(EntityId, TypeId), ChangeRecord>,
    change_ticks: HashMap<TypeId, ChangeTick>,
}

impl ChangeDetector {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            change_ticks: HashMap::new(),
        }
    }

    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.change_ticks.entry(type_id).or_insert_with(ChangeTick::new);
    }

    pub fn mark_changed<T: Component>(&mut self, entity_id: EntityId, tick: u64) {
        let type_id = TypeId::of::<T>();
        self.records.insert(
            (entity_id, type_id),
            ChangeRecord { tick, changed: true },
        );
        if let Some(ct) = self.change_ticks.get_mut(&type_id) {
            if tick > ct.current {
                ct.current = tick;
            }
        }
    }

    pub fn was_changed<T: Component>(&self, entity_id: EntityId, since_tick: u64) -> bool {
        let type_id = TypeId::of::<T>();
        self.records
            .get(&(entity_id, type_id))
            .map_or(false, |r| r.changed && r.tick > since_tick)
    }

    pub fn was_any_changed<T: Component>(&self, since_tick: u64) -> Vec<EntityId> {
        let type_id = TypeId::of::<T>();
        self.records
            .iter()
            .filter(|((_, tid), r)| *tid == type_id && r.changed && r.tick > since_tick)
            .map(|((eid, _), _)| *eid)
            .collect()
    }

    pub fn clear_changed<T: Component>(&mut self, entity_id: EntityId) {
        let type_id = TypeId::of::<T>();
        if let Some(record) = self.records.get_mut(&(entity_id, type_id)) {
            record.changed = false;
        }
    }

    pub fn get_change_tick<T: Component>(&self) -> Option<&ChangeTick> {
        self.change_ticks.get(&TypeId::of::<T>())
    }

    pub fn get_change_tick_mut<T: Component>(&mut self) -> Option<&mut ChangeTick> {
        self.change_ticks.get_mut(&TypeId::of::<T>())
    }

    pub fn clear_all(&mut self) {
        self.records.clear();
        for ct in self.change_ticks.values_mut() {
            ct.mark_read();
        }
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ChangeTracker {
    tick: u64,
    detector: ChangeDetector,
    world_ticks: HashMap<EntityId, HashMap<TypeId, u64>>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            tick: 0,
            detector: ChangeDetector::new(),
            world_ticks: HashMap::new(),
        }
    }

    pub fn advance_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    pub fn detect_component_change<T: Component>(
        &mut self,
        entity_id: EntityId,
        value: &T,
    ) -> Changed<T> {
        let entity_ticks = self.world_ticks.entry(entity_id).or_insert_with(HashMap::new);
        let type_id = TypeId::of::<T>();
        let last_tick = entity_ticks.get(&type_id).copied().unwrap_or(0);
        let is_changed = self.tick > last_tick;

        if is_changed {
            entity_ticks.insert(type_id, self.tick);
            self.detector.mark_changed::<T>(entity_id, self.tick);
        }

        Changed::new(value.clone(), self.tick)
    }

    pub fn was_changed<T: Component>(&self, entity_id: EntityId) -> bool {
        self.detector.was_changed::<T>(entity_id, self.tick.saturating_sub(1))
    }

    pub fn was_any_changed<T: Component>(&self) -> Vec<EntityId> {
        self.detector.was_any_changed::<T>(self.tick.saturating_sub(1))
    }

    pub fn get_detector(&self) -> &ChangeDetector {
        &self.detector
    }

    pub fn get_detector_mut(&mut self) -> &mut ChangeDetector {
        &mut self.detector
    }

    pub fn entities_changed_since<T: Component>(&self, since_tick: u64) -> Vec<EntityId> {
        self.detector.was_any_changed::<T>(since_tick)
    }

    pub fn clear(&mut self) {
        self.detector.clear_all();
    }

    pub fn insert_changed<T: Component>(&mut self, world: &mut UniversalWorld, entity: UniversalEntity, value: T) {
        let changed = Changed::new(value.clone(), self.tick);
        self.detector.mark_changed::<T>(entity.id, self.tick);
        world.insert_component(entity, changed);
    }
}

impl Default for ChangeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {}

    #[derive(Clone, Debug, PartialEq)]
    struct Health(i32);

    impl Component for Health {}

    #[test]
    fn test_changed_new() {
        let pos = Position { x: 1.0, y: 2.0 };
        let changed = Changed::new(pos.clone(), 5);
        assert_eq!(changed.value, pos);
        assert_eq!(changed.tick, 5);
        assert!(!changed.changed);
    }

    #[test]
    fn test_changed_mark_changed() {
        let pos = Position { x: 0.0, y: 0.0 };
        let mut changed = Changed::new(pos, 1);
        assert!(!changed.was_changed());
        changed.mark_changed();
        assert!(changed.was_changed());
    }

    #[test]
    fn test_changed_deref() {
        let pos = Position { x: 3.0, y: 4.0 };
        let mut changed = Changed::new(pos, 0);
        assert_eq!(changed.x, 3.0);
        assert_eq!(changed.y, 4.0);
        assert!(!changed.changed);
        changed.x = 10.0;
        assert!(changed.changed);
    }

    #[test]
    fn test_changed_into_inner() {
        let pos = Position { x: 1.0, y: 2.0 };
        let changed = Changed::new(pos.clone(), 0);
        let inner = changed.into_inner();
        assert_eq!(inner, pos);
    }

    #[test]
    fn test_change_tick() {
        let mut ct = ChangeTick::new();
        assert_eq!(ct.current, 0);
        assert_eq!(ct.last_read, 0);

        ct.increment();
        assert_eq!(ct.current, 1);
        assert!(ct.is_changed(0));
        assert!(!ct.is_changed(1));

        ct.mark_read();
        assert_eq!(ct.last_read, 1);
        assert!(!ct.is_changed(1));
    }

    #[test]
    fn test_change_tick_ticks_since() {
        let ct = ChangeTick { current: 10, last_read: 3 };
        assert_eq!(ct.ticks_since(5), 5);
        assert_eq!(ct.ticks_since(10), 0);
        assert_eq!(ct.ticks_since(15), 0);
    }

    #[test]
    fn test_change_detector_mark_and_query() {
        let mut det = ChangeDetector::new();
        let eid = EntityId(1);

        det.mark_changed::<Position>(eid, 5);
        assert!(det.was_changed::<Position>(eid, 3));
        assert!(!det.was_changed::<Position>(eid, 5));

        det.clear_changed::<Position>(eid);
        assert!(!det.was_changed::<Position>(eid, 0));
    }

    #[test]
    fn test_change_detector_any_changed() {
        let mut det = ChangeDetector::new();
        det.mark_changed::<Position>(EntityId(0), 2);
        det.mark_changed::<Position>(EntityId(1), 4);
        det.mark_changed::<Health>(EntityId(2), 3);

        let changed = det.was_any_changed::<Position>(2);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&EntityId(1)));
    }

    #[test]
    fn test_change_tracker_detect() {
        let mut tracker = ChangeTracker::new();
        let pos = Position { x: 0.0, y: 0.0 };

        tracker.advance_tick();
        let changed = tracker.detect_component_change(EntityId(0), &pos);
        assert!(changed.was_changed());
        assert_eq!(changed.tick, 1);

        let changed2 = tracker.detect_component_change(EntityId(0), &pos);
        assert!(!changed2.was_changed());
    }

    #[test]
    fn test_change_tracker_entities_changed_since() {
        let mut tracker = ChangeTracker::new();
        let pos = Position { x: 0.0, y: 0.0 };

        tracker.advance_tick();
        tracker.detect_component_change(EntityId(0), &pos);

        tracker.advance_tick();
        tracker.detect_component_change(EntityId(1), &pos);

        let changed = tracker.entities_changed_since::<Position>(1);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&EntityId(1)));
    }

    #[test]
    fn test_change_tracker_insert_changed() {
        let mut tracker = ChangeTracker::new();
        let mut world = UniversalWorld::new();
        let entity = world.spawn();

        tracker.advance_tick();
        tracker.insert_changed(&mut world, entity, Position { x: 1.0, y: 2.0 });

        let changed = world.get_component::<Changed<Position>>(entity).unwrap();
        assert!(changed.was_changed());
        assert_eq!(changed.value.x, 1.0);
    }
}
