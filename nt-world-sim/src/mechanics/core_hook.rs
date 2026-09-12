use crate::core::{UniversalWorld, Event};
use crate::mechanics::core_pet::{CorePetState, PetStateEnum};

#[derive(Clone, Debug)]
pub enum CoreHookEvent {
    SessionStart { agent: String, session_id: String },
    SessionEnd { agent: String, session_id: String },
    ToolStart { tool: String },
    ToolEnd { tool: String, success: bool },
    PermissionRequest { tool: String, request_id: String },
    PermissionResponse { request_id: String, approved: bool },
}

impl Event for CoreHookEvent {}

pub struct CoreHookManager;

impl CoreHookManager {
    pub fn process_event(world: &mut UniversalWorld, event: CoreHookEvent) {
        match event {
            CoreHookEvent::SessionStart { .. } => {
                let entities = world.entities();
                for entity in entities {
                    if let Some(pet) = world.get_component_mut::<CorePetState>(entity) {
                        pet.state = PetStateEnum::Thinking { duration: 0.0 };
                        pet.state_timer = 0.0;
                    }
                }
            }
            CoreHookEvent::ToolStart { tool } => {
                let entities = world.entities();
                for entity in entities {
                    if let Some(pet) = world.get_component_mut::<CorePetState>(entity) {
                        match tool.as_str() {
                            "edit" | "write" | "bash" => {
                                pet.state = PetStateEnum::Typing { progress: 0.0 };
                            }
                            "read" | "grep" | "glob" => {
                                pet.state = PetStateEnum::Thinking { duration: 0.0 };
                            }
                            _ => {}
                        }
                        pet.state_timer = 0.0;
                    }
                }
            }
            CoreHookEvent::ToolEnd { success, .. } => {
                let entities = world.entities();
                for entity in entities {
                    if let Some(pet) = world.get_component_mut::<CorePetState>(entity) {
                        if success {
                            pet.state = PetStateEnum::Happy;
                        } else {
                            pet.state = PetStateEnum::Error;
                        }
                        pet.state_timer = 0.0;
                    }
                }
            }
            CoreHookEvent::SessionEnd { .. } => {
                let entities = world.entities();
                for entity in entities {
                    if let Some(pet) = world.get_component_mut::<CorePetState>(entity) {
                        pet.state = PetStateEnum::Idle;
                        pet.state_timer = 0.0;
                        pet.idle_timer = 0.0;
                    }
                }
            }
            CoreHookEvent::PermissionRequest { .. } => {
                let entities = world.entities();
                for entity in entities {
                    if let Some(pet) = world.get_component_mut::<CorePetState>(entity) {
                        pet.state = PetStateEnum::Notification;
                        pet.state_timer = 0.0;
                    }
                }
            }
            CoreHookEvent::PermissionResponse { .. } => {
                let entities = world.entities();
                for entity in entities {
                    if let Some(pet) = world.get_component_mut::<CorePetState>(entity) {
                        pet.state = PetStateEnum::Idle;
                        pet.state_timer = 0.0;
                        pet.idle_timer = 0.0;
                    }
                }
            }
        }
    }

    pub fn send_event(world: &mut UniversalWorld, event: CoreHookEvent) {
        world.send_event(event);
    }

    pub fn receive_events(world: &mut UniversalWorld) -> Vec<CoreHookEvent> {
        world.receive_events::<CoreHookEvent>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::UniversalWorld;

    #[test]
    fn test_session_start_sets_thinking() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(entity, CorePetState::new());

        CoreHookManager::process_event(&mut world, CoreHookEvent::SessionStart {
            agent: "test".to_string(),
            session_id: "s1".to_string(),
        });

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Thinking { duration: 0.0 });
    }

    #[test]
    fn test_tool_start_edit_sets_typing() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(entity, CorePetState::new());

        CoreHookManager::process_event(&mut world, CoreHookEvent::ToolStart {
            tool: "edit".to_string(),
        });

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Typing { progress: 0.0 });
    }

    #[test]
    fn test_tool_start_read_sets_thinking() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(entity, CorePetState::new());

        CoreHookManager::process_event(&mut world, CoreHookEvent::ToolStart {
            tool: "read".to_string(),
        });

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Thinking { duration: 0.0 });
    }

    #[test]
    fn test_tool_end_success_sets_happy() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(entity, CorePetState::new());

        CoreHookManager::process_event(&mut world, CoreHookEvent::ToolEnd {
            tool: "bash".to_string(),
            success: true,
        });

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Happy);
    }

    #[test]
    fn test_tool_end_failure_sets_error() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        world.insert_component(entity, CorePetState::new());

        CoreHookManager::process_event(&mut world, CoreHookEvent::ToolEnd {
            tool: "bash".to_string(),
            success: false,
        });

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Error);
    }

    #[test]
    fn test_session_end_sets_idle() {
        let mut world = UniversalWorld::new();
        let entity = world.spawn();
        let mut pet = CorePetState::new();
        pet.state = PetStateEnum::Thinking { duration: 5.0 };
        world.insert_component(entity, pet);

        CoreHookManager::process_event(&mut world, CoreHookEvent::SessionEnd {
            agent: "test".to_string(),
            session_id: "s1".to_string(),
        });

        let pet = world.get_component::<CorePetState>(entity).unwrap();
        assert_eq!(pet.state, PetStateEnum::Idle);
        assert_eq!(pet.idle_timer, 0.0);
    }

    #[test]
    fn test_event_roundtrip() {
        let mut world = UniversalWorld::new();
        let event = CoreHookEvent::SessionStart {
            agent: "test".to_string(),
            session_id: "s1".to_string(),
        };
        CoreHookManager::send_event(&mut world, event);
        let events = CoreHookManager::receive_events(&mut world);
        assert_eq!(events.len(), 1);
    }
}
