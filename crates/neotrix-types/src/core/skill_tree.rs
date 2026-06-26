use std::collections::HashMap;
use crate::core::skill::Skill;

#[derive(Clone, Debug)]
pub struct SkillTreeNode {
    pub name: String,
    pub description: String,
    pub children: Vec<SkillTreeNode>,
    pub skills: Vec<Skill>,
    pub depth: u32,
}

impl SkillTreeNode {
    pub fn new(name: &str, description: &str, depth: u32) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            children: Vec::new(),
            skills: Vec::new(),
            depth,
        }
    }

    pub fn add_child(&mut self, child: SkillTreeNode) {
        self.children.push(child);
    }

    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.push(skill);
    }

    pub fn total_skills(&self) -> usize {
        let mut count = self.skills.len();
        for child in &self.children {
            count += child.total_skills();
        }
        count
    }

    pub fn total_nodes(&self) -> usize {
        let mut count = 1;
        for child in &self.children {
            count += child.total_nodes();
        }
        count
    }
}

#[derive(Clone, Debug)]
pub struct ConvergenceTracker {
    tracking: HashMap<String, (Vec<usize>, Option<usize>)>,
    pub convergence_threshold: f64,
}

impl ConvergenceTracker {
    pub fn new(threshold: f64) -> Self {
        Self {
            tracking: HashMap::new(),
            convergence_threshold: threshold,
        }
    }

    pub fn record_call(&mut self, task_name: &str, call_count: usize) {
        let entry = self.tracking.entry(task_name.to_string()).or_insert_with(|| (Vec::new(), None));
        entry.0.push(call_count);
        if entry.0.len() >= 2 {
            let initial = entry.0[0];
            let latest = call_count;
            if (latest as f64) <= self.convergence_threshold * (initial as f64) {
                entry.1 = Some(latest);
            }
        }
    }

    pub fn is_converged(&self, task_name: &str) -> bool {
        self.tracking.get(task_name).and_then(|e| e.1).is_some()
    }

    pub fn convergence_ratio(&self, task_name: &str) -> Option<f64> {
        self.tracking.get(task_name).map(|(counts, _)| {
            if counts.is_empty() {
                return 1.0;
            }
            let initial = counts[0] as f64;
            if initial == 0.0 {
                return 1.0;
            }
            let latest = counts.last().unwrap_or(&0);
            *latest as f64 / initial
        })
    }
}

#[derive(Clone, Debug)]
pub struct SkillTree {
    pub root: SkillTreeNode,
    pub convergence: ConvergenceTracker,
}

impl SkillTree {
    pub fn new(root_name: &str, root_desc: &str) -> Self {
        Self {
            root: SkillTreeNode::new(root_name, root_desc, 0),
            convergence: ConvergenceTracker::new(0.2),
        }
    }

    pub fn insert_skill(&mut self, domain_path: &[&str], skill: Skill) {
        let mut path = Vec::new();
        let mut node = &self.root;
        for seg in domain_path {
            if let Some(i) = node.children.iter().position(|c| c.name == *seg) {
                path.push(i);
                node = &node.children[i];
            } else {
                break;
            }
        }
        let mut node = &mut self.root;
        for &i in &path {
            node = &mut node.children[i];
        }
        for seg in domain_path.iter().skip(path.len()) {
            let new = SkillTreeNode::new(seg, "", node.depth + 1);
            node.children.push(new);
            node = node.children.last_mut().expect("just pushed a new child");
        }
        node.add_skill(skill);
    }

    pub fn find_by_name(&self, name: &str) -> Option<&SkillTreeNode> {
        find_in_node(&self.root, name)
    }

    pub fn record_convergence(&mut self, task_name: &str, call_count: usize) {
        self.convergence.record_call(task_name, call_count);
    }

    pub fn is_converged(&self, task_name: &str) -> bool {
        self.convergence.is_converged(task_name)
    }
}

fn find_in_node<'a>(node: &'a SkillTreeNode, name: &str) -> Option<&'a SkillTreeNode> {
    if node.name == name {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_in_node(child, name) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::skill::Skill;

    fn make_skill(name: &str) -> Skill {
        Skill::new(
            name.to_string(),
            format!("{} description", name),
            vec![],
            vec![],
            vec![],
        )
    }

    #[test]
    fn test_tree_creation_with_root() {
        let tree = SkillTree::new("root", "root node");
        assert_eq!(tree.root.name, "root");
        assert_eq!(tree.root.description, "root node");
        assert_eq!(tree.root.depth, 0);
        assert_eq!(tree.root.total_nodes(), 1);
        assert_eq!(tree.root.total_skills(), 0);
    }

    #[test]
    fn test_insert_skill_at_domain_path() {
        let mut tree = SkillTree::new("skills", "all skills");
        let s = make_skill("read_file");
        tree.insert_skill(&["io", "file"], s);

        let node = tree.find_by_name("file").expect("file node should exist");
        assert_eq!(node.name, "file");
        assert_eq!(node.depth, 2);
        assert_eq!(node.skills.len(), 1);
        assert_eq!(node.skills[0].name, "read_file");
    }

    #[test]
    fn test_convergence_tracked() {
        let mut tree = SkillTree::new("root", "");
        tree.convergence.convergence_threshold = 0.2;

        tree.record_convergence("read_wechat", 32);
        assert!(!tree.is_converged("read_wechat"));

        tree.record_convergence("read_wechat", 5);
        assert!(tree.is_converged("read_wechat"));

        let ratio = tree.convergence.convergence_ratio("read_wechat");
        assert!(ratio.is_some());
        assert!((ratio.expect("convergence_ratio should be Some") - 5.0 / 32.0).abs() < 1e-10);
    }

    #[test]
    fn test_convergence_not_yet_reached() {
        let mut tree = SkillTree::new("root", "");
        tree.convergence.convergence_threshold = 0.2;

        tree.record_convergence("write_file", 20);
        tree.record_convergence("write_file", 15);
        assert!(!tree.is_converged("write_file"));
    }

    #[test]
    fn test_multiple_skills_same_node() {
        let mut tree = SkillTree::new("skills", "");
        tree.insert_skill(&["net", "http"], make_skill("get"));
        tree.insert_skill(&["net", "http"], make_skill("post"));

        let node = tree.find_by_name("http").expect("http node should exist");
        assert_eq!(node.skills.len(), 2);
    }

    #[test]
    fn test_tree_stats() {
        let mut tree = SkillTree::new("root", "");
        tree.insert_skill(&["net", "http"], make_skill("get"));
        tree.insert_skill(&["net", "http"], make_skill("post"));
        tree.insert_skill(&["io", "file"], make_skill("read"));

        assert_eq!(tree.root.total_nodes(), 5);
        assert_eq!(tree.root.total_skills(), 3);
    }

    #[test]
    fn test_convergence_ratio_nonexistent() {
        let tree = SkillTree::new("root", "");
        assert!(tree.convergence.convergence_ratio("nope").is_none());
    }
}
