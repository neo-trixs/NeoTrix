use std::collections::HashMap;
use crate::core::{EntityId, UniversalEntity};
use crate::engine::renderer::{Transform, Vec2};

/// A node in the scene graph hierarchy
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub entity: UniversalEntity,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    pub local_transform: Transform,
    pub world_transform: Transform,
    pub visible: bool,
    pub z_index: i32,
}

impl SceneNode {
    pub fn new(entity: UniversalEntity) -> Self {
        Self {
            entity,
            parent: None,
            children: Vec::new(),
            local_transform: Transform::default(),
            world_transform: Transform::default(),
            visible: true,
            z_index: 0,
        }
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.local_transform.position = Vec2 { x, y };
        self
    }

    pub fn with_parent(mut self, parent: EntityId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }
}

/// Scene graph managing hierarchical transforms
pub struct SceneGraph {
    nodes: HashMap<EntityId, SceneNode>,
    root: EntityId,
}

impl SceneGraph {
    pub fn new(root_entity: UniversalEntity) -> Self {
        let root = root_entity.id;
        let mut nodes = HashMap::new();
        nodes.insert(root, SceneNode::new(root_entity));
        Self { nodes, root }
    }

    /// Add a node to the scene graph
    pub fn add_node(&mut self, node: SceneNode) {
        let entity_id = node.entity.id;

        // Register as child of parent
        if let Some(parent_id) = node.parent {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.children.push(entity_id);
            }
        }

        self.nodes.insert(entity_id, node);
    }

    /// Remove a node and its children recursively
    pub fn remove_node(&mut self, entity_id: EntityId) -> Option<SceneNode> {
        if let Some(node) = self.nodes.remove(&entity_id) {
            // Remove from parent's children list
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|&c| c != entity_id);
                }
            }

            // Recursively remove children
            for child_id in &node.children {
                self.remove_node(*child_id);
            }

            Some(node)
        } else {
            None
        }
    }

    /// Get a node by entity ID
    pub fn get_node(&self, entity_id: EntityId) -> Option<&SceneNode> {
        self.nodes.get(&entity_id)
    }

    /// Get a mutable node by entity ID
    pub fn get_node_mut(&mut self, entity_id: EntityId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&entity_id)
    }

    /// Get children of a node
    pub fn children(&self, entity_id: EntityId) -> Vec<&SceneNode> {
        self.nodes
            .get(&entity_id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|child_id| self.nodes.get(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all descendants recursively (depth-first)
    pub fn descendants(&self, entity_id: EntityId) -> Vec<&SceneNode> {
        let mut result = Vec::new();
        let mut stack = vec![entity_id];

        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(&current) {
                for &child_id in &node.children {
                    if let Some(child_node) = self.nodes.get(&child_id) {
                        result.push(child_node);
                        stack.push(child_id);
                    }
                }
            }
        }

        result
    }

    /// Update world transforms (must be called after local transforms change)
    pub fn update_transforms(&mut self) {
        let root = self.root;
        self.update_transform_recursive(root, Transform::default());
    }

    fn update_transform_recursive(&mut self, entity_id: EntityId, parent_world: Transform) {
        let (local_transform, children) = {
            if let Some(node) = self.nodes.get(&entity_id) {
                (node.local_transform.clone(), node.children.clone())
            } else {
                return;
            }
        };

        // Compute world transform
        let world_transform = Transform {
            position: Vec2 {
                x: parent_world.position.x + local_transform.position.x,
                y: parent_world.position.y + local_transform.position.y,
            },
            rotation: parent_world.rotation + local_transform.rotation,
            scale: Vec2 {
                x: parent_world.scale.x * local_transform.scale.x,
                y: parent_world.scale.y * local_transform.scale.y,
            },
        };

        // Update the node's world transform
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.world_transform = world_transform.clone();
        }

        // Recurse for children
        for child_id in children {
            self.update_transform_recursive(child_id, world_transform.clone());
        }
    }

    /// Get visible nodes sorted by z_index (for rendering)
    pub fn visible_nodes(&self) -> Vec<&SceneNode> {
        let mut visible: Vec<&SceneNode> = self
            .nodes
            .values()
            .filter(|node| node.visible)
            .collect();
        visible.sort_by_key(|node| node.z_index);
        visible
    }

    /// Get root entity
    pub fn root(&self) -> EntityId {
        self.root
    }

    /// Number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
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

    fn make_entity(world: &mut UniversalWorld) -> UniversalEntity {
        world.spawn()
    }

    #[test]
    fn test_scene_graph_add_remove() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let child = make_entity(&mut world);

        let mut graph = SceneGraph::new(root);

        let child_node = SceneNode::new(child)
            .with_parent(root.id)
            .with_position(10.0, 20.0);

        graph.add_node(child_node);
        assert_eq!(graph.node_count(), 2);

        let removed = graph.remove_node(child.id);
        assert!(removed.is_some());
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_hierarchy_transforms() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let child = make_entity(&mut world);
        let grandchild = make_entity(&mut world);

        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(child).with_parent(root.id).with_position(10.0, 0.0));
        graph.add_node(SceneNode::new(grandchild).with_parent(child.id).with_position(5.0, 0.0));

        graph.update_transforms();

        let gc = graph.get_node(grandchild.id).unwrap();
        assert!((gc.world_transform.position.x - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_children_query() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let c1 = make_entity(&mut world);
        let c2 = make_entity(&mut world);

        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(c1).with_parent(root.id));
        graph.add_node(SceneNode::new(c2).with_parent(root.id));

        let children = graph.children(root.id);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_descendants() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let c1 = make_entity(&mut world);
        let c2 = make_entity(&mut world);

        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(c1).with_parent(root.id));
        graph.add_node(SceneNode::new(c2).with_parent(c1.id));

        let descendants = graph.descendants(root.id);
        assert_eq!(descendants.len(), 2);
    }

    #[test]
    fn test_visible_nodes_sorted() {
        let mut world = UniversalWorld::new();
        let root = make_entity(&mut world);
        let c1 = make_entity(&mut world);
        let c2 = make_entity(&mut world);

        let mut graph = SceneGraph::new(root);
        graph.add_node(SceneNode::new(c1).with_parent(root.id).with_z_index(10));
        graph.add_node(SceneNode::new(c2).with_parent(root.id).with_z_index(5));

        let visible = graph.visible_nodes();
        assert_eq!(visible.len(), 3); // root + 2 children
        assert!(visible[0].z_index <= visible[1].z_index);
    }
}
