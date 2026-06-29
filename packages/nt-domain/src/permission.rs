use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Admin,
    Execute,
    Delegate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub resource_type: String,
    pub owner_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActorPermissionView {
    pub actor_id: Uuid,
    pub resource_id: Uuid,
    pub permissions: Vec<Permission>,
    pub is_inherited: bool,
}
