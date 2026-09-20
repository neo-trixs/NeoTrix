use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AdNode {
    User {
        name: String,
        sid: String,
        enabled: bool,
    },
    Group {
        name: String,
        sid: String,
        members: Vec<String>,
    },
    Computer {
        name: String,
        sid: String,
        os_version: String,
    },
    Domain {
        name: String,
        sid: String,
    },
}

impl AdNode {
    pub fn sid(&self) -> &str {
        match self {
            AdNode::User { sid, .. } => sid,
            AdNode::Group { sid, .. } => sid,
            AdNode::Computer { sid, .. } => sid,
            AdNode::Domain { sid, .. } => sid,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AdNode::User { name, .. } => name,
            AdNode::Group { name, .. } => name,
            AdNode::Computer { name, .. } => name,
            AdNode::Domain { name, .. } => name,
        }
    }

    pub fn node_type(&self) -> &'static str {
        match self {
            AdNode::User { .. } => "User",
            AdNode::Group { .. } => "Group",
            AdNode::Computer { .. } => "Computer",
            AdNode::Domain { .. } => "Domain",
        }
    }
}

impl fmt::Display for AdNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdNode::User { name, sid, enabled } => {
                write!(f, "User({}, {}, enabled={})", name, sid, enabled)
            }
            AdNode::Group { name, sid, members } => {
                write!(f, "Group({}, {}, members={})", name, sid, members.len())
            }
            AdNode::Computer {
                name,
                sid,
                os_version,
            } => {
                write!(f, "Computer({}, {}, {})", name, sid, os_version)
            }
            AdNode::Domain { name, sid } => {
                write!(f, "Domain({}, {})", name, sid)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AdEdgeType {
    MemberOf,
    HasSession,
    ContainedIn,
    GPOApplies,
}

impl fmt::Display for AdEdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdEdgeType::MemberOf => write!(f, "MemberOf"),
            AdEdgeType::HasSession => write!(f, "HasSession"),
            AdEdgeType::ContainedIn => write!(f, "ContainedIn"),
            AdEdgeType::GPOApplies => write!(f, "GPOApplies"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdEdge {
    pub source: String,
    pub target: String,
    pub edge_type: AdEdgeType,
}

impl AdEdge {
    pub fn new(source: String, target: String, edge_type: AdEdgeType) -> Self {
        Self {
            source,
            target,
            edge_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ad_node_user() {
        let node = AdNode::User {
            name: "admin".to_string(),
            sid: "S-1-5-21-123456".to_string(),
            enabled: true,
        };
        assert_eq!(node.sid(), "S-1-5-21-123456");
        assert_eq!(node.name(), "admin");
        assert_eq!(node.node_type(), "User");
    }

    #[test]
    fn test_ad_node_group() {
        let node = AdNode::Group {
            name: "Domain Admins".to_string(),
            sid: "S-1-5-32-544".to_string(),
            members: vec!["S-1-5-21-123456".to_string()],
        };
        assert_eq!(node.node_type(), "Group");
    }

    #[test]
    fn test_ad_edge_type_display() {
        assert_eq!(AdEdgeType::MemberOf.to_string(), "MemberOf");
        assert_eq!(AdEdgeType::HasSession.to_string(), "HasSession");
    }

    #[test]
    fn test_ad_edge_creation() {
        let edge = AdEdge::new(
            "S-1-5-21-123456".to_string(),
            "S-1-5-32-544".to_string(),
            AdEdgeType::MemberOf,
        );
        assert_eq!(edge.source, "S-1-5-21-123456");
        assert_eq!(edge.target, "S-1-5-32-544");
    }
}
