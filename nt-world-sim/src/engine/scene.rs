use std::collections::HashMap;
use crate::core::{EntityId, UniversalEntity, Transform, Vec2, Rect};
use crate::engine::renderer::Camera;

/// Query predicate for `find_entity`.
pub enum EntityQuery {
    /// Match by z_index range.
    ZIndex { min: i32, max: i32 },
    /// Match by visibility.
    Visible(bool),
    /// Match by name tag (stored as the entity's archetype generation as a u32 key).
    /// For a real engine you'd store a name component; here we use a simple tag map.
    Tag(String),
}

/// A node in the scene graph hierarchy.
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub entity: UniversalEntity,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    pub local_transform: Transform,
    pub world_transform: Transform,
    pub visible: bool,
    pub z_index: i32,
    pub dirty: bool,
    pub tag: Option<String>,
}

impl SceneNode {
    pub fn new(entity: UniversalEntity) -> Self {
        Self {
            entity, parent: None, children: Vec::new(),
            local_transform: Transform::default(),
            world_transform: Transform::default(),
            visible: true, z_index: 0, dirty: true, tag: None,
        }
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self { self.local_transform.position = Vec2 { x, y }; self }
    pub fn with_parent(mut self, parent: EntityId) -> Self { self.parent = Some(parent); self }
    pub fn with_z_index(mut self, z: i32) -> Self { self.z_index = z; self }
    pub fn with_tag(mut self, tag: &str) -> Self { self.tag = Some(tag.to_string()); self }
}

/// Scene graph managing hierarchical transforms, dirty flags, queries.
pub struct SceneGraph {
    nodes: HashMap<EntityId, SceneNode>,
    root: EntityId,
    tag_index: HashMap<String, EntityId>,
}

impl SceneGraph {
    pub fn new(root_entity: UniversalEntity) -> Self {
        let root = root_entity.id;
        let mut nodes = HashMap::new();
        nodes.insert(root, SceneNode::new(root_entity));
        Self { nodes, root, tag_index: HashMap::new() }
    }

    // -- basic ops --

    pub fn add_node(&mut self, node: SceneNode) {
        let eid = node.entity.id;
        if let Some(ref tag) = node.tag {
            self.tag_index.insert(tag.clone(), eid);
        }
        if let Some(parent_id) = node.parent {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.children.push(eid);
                parent.dirty = true;
            }
        }
        self.nodes.insert(eid, node);
    }

    pub fn remove_node(&mut self, entity_id: EntityId) -> Option<SceneNode> {
        if let Some(node) = self.nodes.remove(&entity_id) {
            if let Some(ref tag) = node.tag {
                self.tag_index.remove(tag);
            }
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|&c| c != entity_id);
                    parent.dirty = true;
                }
            }
            for child_id in &node.children {
                self.remove_node(*child_id);
            }
            Some(node)
        } else {
            None
        }
    }

    pub fn get_node(&self, entity_id: EntityId) -> Option<&SceneNode> { self.nodes.get(&entity_id) }
    pub fn get_node_mut(&mut self, entity_id: EntityId) -> Option<&mut SceneNode> { self.nodes.get_mut(&entity_id) }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn root(&self) -> EntityId { self.root }

    // -- hierarchy queries --

    pub fn children(&self, entity_id: EntityId) -> Vec<&SceneNode> {
        self.nodes.get(&entity_id)
            .map(|n| n.children.iter().filter_map(|cid| self.nodes.get(cid)).collect())
            .unwrap_or_default()
    }

    pub fn children_ids(&self, entity_id: EntityId) -> Vec<EntityId> {
        self.nodes.get(&entity_id).map_or_else(Vec::new, |n| n.children.clone())
    }

    pub fn get_parent(&self, entity_id: EntityId) -> Option<EntityId> {
        self.nodes.get(&entity_id).and_then(|n| n.parent)
    }

    pub fn descendants(&self, entity_id: EntityId) -> Vec<&SceneNode> {
        let mut result = Vec::new();
        let mut stack = vec![entity_id];
        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(&current) {
                for &cid in &node.children {
                    if let Some(child) = self.nodes.get(&cid) {
                        result.push(child);
                        stack.push(cid);
                    }
                }
            }
        }
        result
    }

    pub fn all_descendants_ids(&self, entity_id: EntityId) -> Vec<EntityId> {
        let mut result = Vec::new();
        let mut stack = vec![entity_id];
        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(&current) {
                for &cid in &node.children {
                    result.push(cid);
                    stack.push(cid);
                }
            }
        }
        result
    }

    // -- reparenting --

    pub fn reparent(&mut self, entity_id: EntityId, new_parent: EntityId) -> bool {
        if entity_id == self.root { return false; }
        if !self.nodes.contains_key(&new_parent) { return false; }

        // Detach from old parent
        let old_parent = self.nodes.get(&entity_id).and_then(|n| n.parent);
        if let Some(old_pid) = old_parent {
            if let Some(old_parent) = self.nodes.get_mut(&old_pid) {
                old_parent.children.retain(|&c| c != entity_id);
                old_parent.dirty = true;
            }
        }

        // Attach to new parent
        if let Some(parent) = self.nodes.get_mut(&new_parent) {
            parent.children.push(entity_id);
            parent.dirty = true;
        }
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.parent = Some(new_parent);
            node.dirty = true;
        }

        // Mark descendants dirty
        let descendants = self.all_descendants_ids(entity_id);
        for did in descendants {
            if let Some(n) = self.nodes.get_mut(&did) {
                n.dirty = true;
            }
        }

        true
    }

    // -- tag queries --

    pub fn find_by_tag(&self, tag: &str) -> Option<&SceneNode> {
        self.tag_index.get(tag).and_then(|eid| self.nodes.get(eid))
    }

    pub fn find_by_tag_mut(&mut self, tag: &str) -> Option<&mut SceneNode> {
        if let Some(eid) = self.tag_index.get(tag).copied() {
            self.nodes.get_mut(&eid)
        } else {
            None
        }
    }

    // -- general queries --

    pub fn find_entity(&self, query: &EntityQuery) -> Vec<&SceneNode> {
        self.nodes.values()
            .filter(|n| match query {
                EntityQuery::ZIndex { min, max } => n.z_index >= *min && n.z_index <= *max,
                EntityQuery::Visible(v) => n.visible == *v,
                EntityQuery::Tag(tag) => n.tag.as_deref() == Some(tag.as_str()),
            })
            .collect()
    }

    pub fn find_entities_at(&self, world_pos: Vec2, tolerance: f32) -> Vec<&SceneNode> {
        self.nodes.values()
            .filter(|n| {
                let dx = n.world_transform.position.x - world_pos.x;
                let dy = n.world_transform.position.y - world_pos.y;
                dx * dx + dy * dy <= tolerance * tolerance
            })
            .collect()
    }

    pub fn find_entities_in_rect(&self, rect: &Rect) -> Vec<&SceneNode> {
        self.nodes.values()
            .filter(|n| {
                let p = n.world_transform.position;
                rect.contains(&p)
            })
            .collect()
    }

    // -- visibility & z-sorting --

    pub fn visible_nodes(&self) -> Vec<&SceneNode> {
        let mut visible: Vec<&SceneNode> = self.nodes.values().filter(|n| n.visible).collect();
        visible.sort_by_key(|n| n.z_index);
        visible
    }

    pub fn visible_nodes_in_camera(&self, camera: &Camera) -> Vec<&SceneNode> {
        let cam_rect = camera.visible_world_rect();
        let mut visible: Vec<&SceneNode> = self.nodes.values()
            .filter(|n| {
                if !n.visible { return false; }
                // Include nodes whose position is in view, or have children in view
                let p = n.world_transform.position;
                let half_w = 64.0; // generous margin for sprites
                let node_rect = Rect::new(p.x - half_w, p.y - half_w, half_w * 2.0, half_w * 2.0);
                cam_rect.intersects(&node_rect)
            })
            .collect();
        visible.sort_by_key(|n| n.z_index);
        visible
    }

    pub fn set_z_index(&mut self, entity_id: EntityId, z: i32) {
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.z_index = z;
        }
    }

    pub fn sort_children_by_z(&mut self, entity_id: EntityId) {
        if let Some(node) = self.nodes.get(&entity_id) {
            let children = node.children.clone();
            let mut z_values: Vec<(EntityId, i32)> = children
                .iter()
                .map(|cid| (*cid, self.nodes.get(cid).map_or(0, |n| n.z_index)))
                .collect();
            z_values.sort_by_key(|(_, z)| *z);
            if let Some(node) = self.nodes.get_mut(&entity_id) {
                node.children = z_values.into_iter().map(|(eid, _)| eid).collect();
            }
        }
    }

    // -- dirty flag propagation --

    pub fn mark_dirty(&mut self, entity_id: EntityId) {
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.dirty = true;
        }
        let descendants = self.all_descendants_ids(entity_id);
        for did in descendants {
            if let Some(n) = self.nodes.get_mut(&did) {
                n.dirty = true;
            }
        }
    }

    pub fn has_dirty(&self) -> bool {
        self.nodes.values().any(|n| n.dirty)
    }

    pub fn dirty_count(&self) -> usize {
        self.nodes.values().filter(|n| n.dirty).count()
    }

    // -- world transform --

    pub fn get_world_transform(&self, entity_id: EntityId) -> Option<Transform> {
        self.nodes.get(&entity_id).map(|n| n.world_transform)
    }

    pub fn get_world_position(&self, entity_id: EntityId) -> Option<Vec2> {
        self.get_world_transform(entity_id).map(|t| t.position)
    }

    // -- transform update (full or dirty-only) --

    pub fn update_transforms(&mut self) {
        let root = self.root;
        self.update_transform_recursive(root, Transform::default());
    }

    pub fn update_dirty_transforms(&mut self) {
        let root = self.root;
        self.update_dirty_recursive(root, Transform::default());
    }

    fn update_transform_recursive(&mut self, entity_id: EntityId, parent_world: Transform) {
        let (local, children) = match self.nodes.get(&entity_id) {
            Some(n) => (n.local_transform, n.children.clone()),
            None => return,
        };
        let world = Transform {
            position: Vec2 { x: parent_world.position.x + local.position.x, y: parent_world.position.y + local.position.y },
            rotation: parent_world.rotation + local.rotation,
            scale: Vec2 { x: parent_world.scale.x * local.scale.x, y: parent_world.scale.y * local.scale.y },
        };
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.world_transform = world;
            node.dirty = false;
        }
        for cid in children {
            self.update_transform_recursive(cid, world);
        }
    }

    fn update_dirty_recursive(&mut self, entity_id: EntityId, parent_world: Transform) {
        let (is_dirty, local, children) = match self.nodes.get(&entity_id) {
            Some(n) => (n.dirty, n.local_transform, n.children.clone()),
            None => return,
        };
        if !is_dirty { return; }
        let world = Transform {
            position: Vec2 { x: parent_world.position.x + local.position.x, y: parent_world.position.y + local.position.y },
            rotation: parent_world.rotation + local.rotation,
            scale: Vec2 { x: parent_world.scale.x * local.scale.x, y: parent_world.scale.y * local.scale.y },
        };
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.world_transform = world;
            node.dirty = false;
        }
        for cid in children {
            self.update_dirty_recursive(cid, world);
        }
    }

    // -- local transform setters (mark dirty) --

    pub fn set_position(&mut self, entity_id: EntityId, x: f32, y: f32) {
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.local_transform.position = Vec2 { x, y };
            node.dirty = true;
        }
        self.mark_dirty(entity_id);
    }

    pub fn set_scale(&mut self, entity_id: EntityId, sx: f32, sy: f32) {
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.local_transform.scale = Vec2 { x: sx, y: sy };
            node.dirty = true;
        }
        self.mark_dirty(entity_id);
    }

    pub fn set_rotation(&mut self, entity_id: EntityId, rotation: f32) {
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.local_transform.rotation = rotation;
            node.dirty = true;
        }
        self.mark_dirty(entity_id);
    }

    pub fn set_visible(&mut self, entity_id: EntityId, visible: bool) {
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.visible = visible;
        }
    }

    // -- statistics --

    pub fn visible_count(&self) -> usize { self.nodes.values().filter(|n| n.visible).count() }
    pub fn total_descendant_count(&self, entity_id: EntityId) -> usize {
        self.all_descendants_ids(entity_id).len()
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        let root_entity = UniversalEntity::new(EntityId(0), 0);
        Self::new(root_entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::UniversalWorld;

    fn make_entity(world: &mut UniversalWorld) -> UniversalEntity { world.spawn() }

    #[test]
    fn test_add_remove_node() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let child = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(child).with_parent(root.id).with_position(10.0, 20.0));
        assert_eq!(graph.node_count(), 2);
        assert!(graph.remove_node(child.id).is_some());
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_hierarchy_transforms() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let child = make_entity(&mut world);
        let gc = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(child).with_parent(root.id).with_position(10.0, 0.0));
        graph.add_node(SceneNode::new(gc).with_parent(child.id).with_position(5.0, 0.0));
        graph.update_transforms();
        let gc_node = graph.get_node(gc.id).unwrap();
        assert!((gc_node.world_transform.position.x - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_dirty_flag() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let child = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(child).with_parent(root.id));
        graph.update_transforms();
        assert!(!graph.has_dirty());
        graph.mark_dirty(child.id);
        assert!(graph.has_dirty());
        assert_eq!(graph.dirty_count(), 1); // child marked dirty (no descendants)
    }

    #[test]
    fn test_reparent() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let a = make_entity(&mut world);
        let b = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(a).with_parent(root.id));
        graph.add_node(SceneNode::new(b).with_parent(a.id));
        assert_eq!(graph.children(a.id).len(), 1);
        assert!(graph.reparent(b.id, root.id));
        assert_eq!(graph.children(a.id).len(), 0);
        assert_eq!(graph.children(root.id).len(), 2);
    }

    #[test]
    fn test_find_by_tag() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let player = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(player).with_parent(root.id).with_tag("player"));
        let found = graph.find_by_tag("player");
        assert!(found.is_some());
        assert!(graph.find_by_tag("enemy").is_none());
    }

    #[test]
    fn test_find_at_position() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let e1 = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(e1).with_parent(root.id).with_position(10.0, 20.0));
        graph.update_transforms();
        let found = graph.find_entities_at(Vec2::new(10.0, 20.0), 1.0);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_sort_children_by_z() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let a = make_entity(&mut world);
        let b = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(a).with_parent(root.id).with_z_index(10));
        graph.add_node(SceneNode::new(b).with_parent(root.id).with_z_index(2));
        graph.sort_children_by_z(root.id);
        let kids = graph.children_ids(root.id);
        assert_eq!(kids[0], b.id);
        assert_eq!(kids[1], a.id);
    }

    #[test]
    fn test_visible_in_camera() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let e1 = make_entity(&mut world);
        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(e1).with_parent(root.id).with_position(0.0, 0.0));
        graph.update_transforms();
        let cam = Camera::new(100.0, 100.0);
        let visible = graph.visible_nodes_in_camera(&cam);
        assert_eq!(visible.len(), 2); // root + e1
    }
}
