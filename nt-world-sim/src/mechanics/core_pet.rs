use crate::core::{Component, UniversalWorld};

#[derive(Clone, Debug, PartialEq)]
pub enum PetStateEnum {
    Idle,
    Thinking { duration: f32 },
    Typing { progress: f32 },
    Building,
    Groove,
    Juggling,
    Error,
    Happy,
    Notification,
    Sweeping,
    Carrying,
    Sleeping,
}

#[derive(Clone, Debug)]
pub struct CorePetState {
    pub state: PetStateEnum,
    pub state_timer: f32,
    pub idle_timer: f32,
}

impl Component for CorePetState {}

impl Default for CorePetState {
    fn default() -> Self {
        Self {
            state: PetStateEnum::Idle,
            state_timer: 0.0,
            idle_timer: 0.0,
        }
    }
}

impl CorePetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn priority(&self) -> i32 {
        match &self.state {
            PetStateEnum::Error => 100,
            PetStateEnum::Notification => 90,
            PetStateEnum::Thinking { .. } => 80,
            PetStateEnum::Typing { .. } => 70,
            PetStateEnum::Building => 60,
            PetStateEnum::Groove => 50,
            PetStateEnum::Juggling => 50,
            PetStateEnum::Happy => 40,
            PetStateEnum::Sweeping => 30,
            PetStateEnum::Carrying => 25,
            PetStateEnum::Idle => 10,
            PetStateEnum::Sleeping => 0,
        }
    }

    pub fn can_transition(&self, new_state: &PetStateEnum) -> bool {
        let new_priority = match new_state {
            PetStateEnum::Error => 100,
            PetStateEnum::Notification => 90,
            PetStateEnum::Thinking { .. } => 80,
            PetStateEnum::Typing { .. } => 70,
            PetStateEnum::Building => 60,
            PetStateEnum::Groove => 50,
            PetStateEnum::Juggling => 50,
            PetStateEnum::Happy => 40,
            PetStateEnum::Sweeping => 30,
            PetStateEnum::Carrying => 25,
            PetStateEnum::Idle => 10,
            PetStateEnum::Sleeping => 0,
        };
        new_priority >= self.priority()
    }

    pub fn transition(&mut self, new_state: PetStateEnum) -> bool {
        if self.can_transition(&new_state) {
            self.state = new_state;
            self.state_timer = 0.0;
            true
        } else {
            false
        }
    }
}

pub struct CorePetSystem;

impl CorePetSystem {
    pub fn update(world: &mut UniversalWorld, dt: f32) {
        let entities = world.entities();
        for entity in entities {
            if let Some(pet) = world.get_component_mut::<CorePetState>(entity) {
                pet.state_timer += dt;
                match &mut pet.state {
                    PetStateEnum::Idle => {
                        pet.idle_timer += dt;
                        if pet.idle_timer > 60.0 {
                            pet.state = PetStateEnum::Sleeping;
                            pet.state_timer = 0.0;
                        }
                    }
                    PetStateEnum::Thinking { duration } => {
                        *duration += dt;
                    }
                    PetStateEnum::Typing { progress } => {
                        *progress = (*progress + dt * 0.5).min(1.0);
                    }
                    PetStateEnum::Happy => {
                        if pet.state_timer > 3.0 {
                            pet.state = PetStateEnum::Idle;
                            pet.state_timer = 0.0;
                            pet.idle_timer = 0.0;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_pet_state_default() {
        let pet = CorePetState::new();
        assert_eq!(pet.state, PetStateEnum::Idle);
        assert_eq!(pet.state_timer, 0.0);
        assert_eq!(pet.idle_timer, 0.0);
    }

    #[test]
    fn test_priority() {
        let mut pet = CorePetState::new();
        assert_eq!(pet.priority(), 10);

        pet.state = PetStateEnum::Error;
        assert_eq!(pet.priority(), 100);
    }

    #[test]
    fn test_transition() {
        let mut pet = CorePetState::new();
        assert!(pet.transition(PetStateEnum::Thinking { duration: 0.0 }));
        assert_eq!(pet.state, PetStateEnum::Thinking { duration: 0.0 });
        assert_eq!(pet.state_timer, 0.0);
    }

    #[test]
    fn test_cannot_downgrade() {
        let mut pet = CorePetState::new();
        pet.state = PetStateEnum::Error;
        assert!(!pet.transition(PetStateEnum::Idle));
    }

    #[test]
    fn test_system_update() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(entity, CorePetState::new());

        CorePetSystem::update(&mut world, 1.0);

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state_timer, 1.0);
        assert_eq!(pet.idle_timer, 1.0);
    }

    #[test]
    fn test_idle_to_sleep() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        let mut pet = CorePetState::new();
        pet.idle_timer = 59.0;
        world.insert_component(entity, pet);

        CorePetSystem::update(&mut world, 2.0);

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Sleeping);
    }

    #[test]
    fn test_typing_progress() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(
            entity,
            CorePetState {
                state: PetStateEnum::Typing { progress: 0.0 },
                state_timer: 0.0,
                idle_timer: 0.0,
            },
        );

        CorePetSystem::update(&mut world, 1.0);

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        match &pet.state {
            PetStateEnum::Typing { progress } => {
                assert!(*progress > 0.0);
            }
            _ => panic!("Expected Typing state"),
        }
    }

    #[test]
    fn test_happy_timeout() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(
            entity,
            CorePetState {
                state: PetStateEnum::Happy,
                state_timer: 2.5,
                idle_timer: 0.0,
            },
        );

        CorePetSystem::update(&mut world, 1.0);

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Idle);
    }
}
