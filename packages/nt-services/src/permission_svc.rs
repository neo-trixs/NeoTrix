use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use nt_domain::*;
use uuid::Uuid;

#[derive(Clone)]
struct PermissionGrant {
    actor_id: Uuid,
    resource_id: Uuid,
    permission: Permission,
}

pub struct PermissionService {
    resources: Arc<Mutex<HashMap<Uuid, ResourceNode>>>,
    grants: Arc<Mutex<Vec<PermissionGrant>>>,
}

impl PermissionService {
    pub fn new() -> Self {
        Self {
            resources: Arc::new(Mutex::new(HashMap::new())),
            grants: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_resource(&self, parent_id: Option<Uuid>, resource_type: &str, owner_id: Uuid) -> Result<ResourceNode, String> {
        let resource = ResourceNode {
            id: Uuid::new_v4(),
            parent_id,
            resource_type: resource_type.to_string(),
            owner_id,
        };
        self.resources.lock().unwrap().insert(resource.id, resource.clone());
        let mut grants = self.grants.lock().unwrap();
        grants.push(PermissionGrant {
            actor_id: owner_id,
            resource_id: resource.id,
            permission: Permission::Admin,
        });
        Ok(resource)
    }

    pub fn get_resource(&self, id: Uuid) -> Result<ResourceNode, String> {
        self.resources.lock().unwrap().get(&id).cloned().ok_or_else(|| "Resource not found".to_string())
    }

    pub fn grant(&self, actor_id: Uuid, resource_id: Uuid, permission: Permission) -> Result<(), String> {
        if !self.resources.lock().unwrap().contains_key(&resource_id) {
            return Err("Resource not found".to_string());
        }
        self.grants.lock().unwrap().push(PermissionGrant { actor_id, resource_id, permission });
        Ok(())
    }

    pub fn check(&self, actor_id: Uuid, resource_id: Uuid, required: &Permission) -> bool {
        let resources = self.resources.lock().unwrap();
        let grants = self.grants.lock().unwrap();
        self.check_inner(&resources, &grants, actor_id, resource_id, required)
    }

    fn check_inner(
        &self,
        resources: &HashMap<Uuid, ResourceNode>,
        grants: &[PermissionGrant],
        actor_id: Uuid,
        resource_id: Uuid,
        required: &Permission,
    ) -> bool {
        for g in grants.iter() {
            if g.actor_id == actor_id && g.resource_id == resource_id {
                if Self::permission_satisfies(&g.permission, required) {
                    return true;
                }
            }
        }

        if let Some(res) = resources.get(&resource_id) {
            if res.owner_id == actor_id {
                return true;
            }
        }

        if let Some(res) = resources.get(&resource_id) {
            if let Some(parent_id) = res.parent_id {
                return self.check_inner(resources, grants, actor_id, parent_id, required);
            }
        }

        false
    }

    pub fn get_actor_view(&self, actor_id: Uuid, resource_id: Uuid) -> ActorPermissionView {
        let resources = self.resources.lock().unwrap();
        let grants = self.grants.lock().unwrap();
        self.view_inner(&resources, &grants, actor_id, resource_id)
    }

    fn view_inner(
        &self,
        resources: &HashMap<Uuid, ResourceNode>,
        grants: &[PermissionGrant],
        actor_id: Uuid,
        resource_id: Uuid,
    ) -> ActorPermissionView {
        let mut permissions = Vec::new();
        let mut is_inherited = false;

        for g in grants.iter() {
            if g.actor_id == actor_id && g.resource_id == resource_id {
                permissions.push(g.permission.clone());
            }
        }

        if permissions.is_empty() {
            if let Some(res) = resources.get(&resource_id) {
                if let Some(parent_id) = res.parent_id {
                    let parent_view = self.view_inner(resources, grants, actor_id, parent_id);
                    permissions = parent_view.permissions;
                    is_inherited = true;
                }
            }
        }

        if let Some(res) = resources.get(&resource_id) {
            if res.owner_id == actor_id && !permissions.contains(&Permission::Admin) {
                permissions.push(Permission::Admin);
            }
        }

        ActorPermissionView { actor_id, resource_id, permissions, is_inherited }
    }

    fn permission_satisfies(granted: &Permission, required: &Permission) -> bool {
        match (granted, required) {
            (Permission::Admin, _) => true,
            (Permission::Write, Permission::Write) => true,
            (Permission::Write, Permission::Read) => true,
            (Permission::Read, Permission::Read) => true,
            (Permission::Execute, Permission::Execute) => true,
            (Permission::Delegate, Permission::Delegate) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_resource() {
        let svc = PermissionService::new();
        let res = svc.register_resource(None, "workspace", Uuid::new_v4()).unwrap();
        assert_eq!(res.resource_type, "workspace");
    }

    #[test]
    fn test_owner_has_admin() {
        let svc = PermissionService::new();
        let owner = Uuid::new_v4();
        let res = svc.register_resource(None, "workspace", owner).unwrap();
        assert!(svc.check(owner, res.id, &Permission::Admin));
        assert!(svc.check(owner, res.id, &Permission::Read));
    }

    #[test]
    fn test_non_owner_no_access() {
        let svc = PermissionService::new();
        let owner = Uuid::new_v4();
        let stranger = Uuid::new_v4();
        let res = svc.register_resource(None, "workspace", owner).unwrap();
        assert!(!svc.check(stranger, res.id, &Permission::Read));
    }

    #[test]
    fn test_grant_permission() {
        let svc = PermissionService::new();
        let owner = Uuid::new_v4();
        let user = Uuid::new_v4();
        let res = svc.register_resource(None, "workspace", owner).unwrap();
        svc.grant(user, res.id, Permission::Read).unwrap();
        assert!(svc.check(user, res.id, &Permission::Read));
        assert!(!svc.check(user, res.id, &Permission::Write));
    }

    #[test]
    fn test_permission_inheritance() {
        let svc = PermissionService::new();
        let owner = Uuid::new_v4();
        let user = Uuid::new_v4();
        let parent = svc.register_resource(None, "org", owner).unwrap();
        let child = svc.register_resource(Some(parent.id), "workspace", owner).unwrap();
        svc.grant(user, parent.id, Permission::Read).unwrap();
        assert!(svc.check(user, child.id, &Permission::Read));
    }

    #[test]
    fn test_actor_view() {
        let svc = PermissionService::new();
        let owner = Uuid::new_v4();
        let res = svc.register_resource(None, "workspace", owner).unwrap();
        let view = svc.get_actor_view(owner, res.id);
        assert!(view.permissions.contains(&Permission::Admin));
        assert!(!view.is_inherited);
    }

    #[test]
    fn test_permission_hierarchy() {
        let svc = PermissionService::new();
        let owner = Uuid::new_v4();
        let user = Uuid::new_v4();
        let res = svc.register_resource(None, "workspace", owner).unwrap();
        svc.grant(user, res.id, Permission::Write).unwrap();
        assert!(svc.check(user, res.id, &Permission::Read));
        assert!(svc.check(user, res.id, &Permission::Write));
        assert!(!svc.check(user, res.id, &Permission::Admin));
    }

    #[test]
    fn test_grant_invalid_resource() {
        let svc = PermissionService::new();
        assert!(svc.grant(Uuid::new_v4(), Uuid::new_v4(), Permission::Read).is_err());
    }

    #[test]
    fn test_nested_inheritance() {
        let svc = PermissionService::new();
        let owner = Uuid::new_v4();
        let user = Uuid::new_v4();
        let org = svc.register_resource(None, "org", owner).unwrap();
        let team = svc.register_resource(Some(org.id), "team", owner).unwrap();
        let ws = svc.register_resource(Some(team.id), "workspace", owner).unwrap();
        svc.grant(user, org.id, Permission::Read).unwrap();
        assert!(svc.check(user, ws.id, &Permission::Read));
        assert!(!svc.check(user, ws.id, &Permission::Admin));
    }
}
