use nt_domain::*;

pub trait PermissionRepo: Send + Sync {
    fn create_resource(&self, resource: &ResourceNode) -> Result<ResourceNode, String>;
    fn get_resource(&self, id: uuid::Uuid) -> Option<ResourceNode>;
    fn set_permission(&self, actor_id: uuid::Uuid, resource_id: uuid::Uuid, permission: Permission) -> Result<(), String>;
    fn check_permission(&self, actor_id: uuid::Uuid, resource_id: uuid::Uuid, permission: &Permission) -> bool;
    fn get_actor_permissions(&self, actor_id: uuid::Uuid, resource_id: uuid::Uuid) -> ActorPermissionView;
}
