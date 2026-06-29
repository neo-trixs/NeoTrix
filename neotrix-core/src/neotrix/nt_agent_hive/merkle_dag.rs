use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// SHA-256 content identifier — the CID for a Merkle DAG node.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ContentId([u8; 32]);

impl ContentId {
    pub fn compute(data: &[u8]) -> Self {
        let hash = Sha256::digest(data);
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        ContentId(id)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut id = [0u8; 32];
        let len = bytes.len().min(32);
        id[..len].copy_from_slice(&bytes[..len]);
        ContentId(id)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0[..8] {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// A single node in the Merkle DAG.
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub id: ContentId,
    pub data: Vec<u8>,
    pub parents: Vec<ContentId>,
    pub timestamp_ns: u64,
    pub node_type: String,
}

/// Content-addressed Merkle DAG storage.
///
/// Nodes are addressed by their SHA-256 content hash.
/// Lineage forms a Git-style DAG — each node records parent CIDs,
/// enabling full provenance audit from any node back to origin.
///
/// Compatible with IPFS/helia CID semantics.
pub struct MerkleDagStore {
    nodes: HashMap<ContentId, MerkleNode>,
    by_type: HashMap<String, Vec<ContentId>>,
    max_nodes: usize,
}

impl MerkleDagStore {
    pub fn new(max_nodes: usize) -> Self {
        MerkleDagStore {
            nodes: HashMap::with_capacity(max_nodes.min(64)),
            by_type: HashMap::new(),
            max_nodes,
        }
    }

    /// Insert data with parent CIDs. Returns the ContentId (SHA-256 of data).
    /// Deduplicates: if the same data already exists, returns existing CID.
    pub fn insert(&mut self, data: Vec<u8>, parents: Vec<ContentId>, node_type: &str) -> ContentId {
        let id = ContentId::compute(&data);
        if self.nodes.contains_key(&id) {
            return id;
        }

        if self.nodes.len() >= self.max_nodes {
            self.evict_oldest();
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.nodes.insert(
            id.clone(),
            MerkleNode {
                id: id.clone(),
                data,
                parents,
                timestamp_ns: now,
                node_type: node_type.to_string(),
            },
        );

        self.by_type
            .entry(node_type.to_string())
            .or_default()
            .push(id.clone());

        id
    }

    /// Insert a KnowledgePacket into the DAG, preserving its provenance as parent CIDs.
    pub fn insert_packet(&mut self, packet: &super::KnowledgePacket, node_type: &str) -> ContentId {
        let serialized = serde_json::to_vec(packet).unwrap_or_else(|_| {
            let mut buf = Vec::new();
            buf.extend_from_slice(packet.domain.as_bytes());
            buf.extend_from_slice(packet.text_summary.as_bytes());
            buf.extend_from_slice(&packet.local_negentropy_gain.to_le_bytes());
            buf
        });

        let parent_ids: Vec<ContentId> = packet
            .provenance
            .iter()
            .map(|pid| ContentId::from_bytes(pid.as_bytes()))
            .collect();

        self.insert(serialized, parent_ids, node_type)
    }

    pub fn get(&self, id: &ContentId) -> Option<&MerkleNode> {
        self.nodes.get(id)
    }

    /// Get all children of a node (nodes that list this CID as a parent).
    pub fn get_children(&self, id: &ContentId) -> Vec<&MerkleNode> {
        self.nodes
            .values()
            .filter(|n| n.parents.contains(id))
            .collect()
    }

    /// Full lineage from this node back to root (breadth-first parent traversal).
    pub fn get_lineage(&self, id: &ContentId) -> Vec<ContentId> {
        let mut lineage = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(id.clone());

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            lineage.push(current.clone());
            if let Some(node) = self.nodes.get(&current) {
                for parent in &node.parents {
                    queue.push_back(parent.clone());
                }
            }
        }
        lineage
    }

    /// Verify that every parent CID in the DAG exists and that the chain is acyclic.
    pub fn verify_chain(&self, id: &ContentId) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![id.clone()];

        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                return false;
            }
            if let Some(node) = self.nodes.get(&current) {
                for parent in &node.parents {
                    if !self.nodes.contains_key(parent) {
                        return false;
                    }
                    stack.push(parent.clone());
                }
            }
        }
        true
    }

    /// Compute a HashSeq-style ordered hash chain over a list of ContentIds.
    /// Each entry: H(prev_hash || entry_cid), starting from zero hash.
    /// Compatible with IPFS HashSeq semantics but using SHA-256.
    pub fn hashseq(ids: &[ContentId]) -> Vec<u8> {
        let mut chain = vec![0u8; 32];
        for id in ids {
            let mut hasher = Sha256::new();
            hasher.update(&chain);
            hasher.update(id.as_bytes());
            let result = hasher.finalize();
            chain.copy_from_slice(&result);
        }
        chain.to_vec()
    }

    pub fn contains(&self, id: &ContentId) -> bool {
        self.nodes.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn nodes_by_type(&self, node_type: &str) -> Vec<&MerkleNode> {
        self.by_type
            .get(node_type)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest) = self
            .nodes
            .values()
            .min_by_key(|n| n.timestamp_ns)
            .map(|n| n.id.clone())
        {
            if let Some(node) = self.nodes.remove(&oldest) {
                self.by_type
                    .entry(node.node_type)
                    .or_default()
                    .retain(|id| id != &oldest);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> MerkleDagStore {
        MerkleDagStore::new(100)
    }

    #[test]
    fn test_insert_and_get() {
        let mut s = store();
        let id = s.insert(b"hello world".to_vec(), vec![], "test");
        assert!(s.contains(&id));
        let node = s.get(&id).unwrap();
        assert_eq!(node.data, b"hello world");
    }

    #[test]
    fn test_dedup_same_data() {
        let mut s = store();
        let id1 = s.insert(b"dedup".to_vec(), vec![], "test");
        let id2 = s.insert(b"dedup".to_vec(), vec![], "test");
        assert_eq!(id1, id2);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_dag_lineage() {
        let mut s = store();
        let root = s.insert(b"root".to_vec(), vec![], "test");
        let child = s.insert(b"child".to_vec(), vec![root.clone()], "test");
        let grandchild = s.insert(b"grandchild".to_vec(), vec![child.clone()], "test");

        let lineage = s.get_lineage(&grandchild);
        assert!(lineage.contains(&root));
        assert!(lineage.contains(&child));
        assert!(lineage.contains(&grandchild));
    }

    #[test]
    fn test_verify_chain_valid() {
        let mut s = store();
        let root = s.insert(b"root".to_vec(), vec![], "test");
        let child = s.insert(b"child".to_vec(), vec![root.clone()], "test");
        assert!(s.verify_chain(&child));
    }

    #[test]
    fn test_verify_chain_missing_parent() {
        let missing = ContentId::compute(b"ghost");
        let mut s = store();
        let child = s.insert(b"orphan".to_vec(), vec![missing], "test");
        assert!(!s.verify_chain(&child));
    }

    #[test]
    fn test_get_children() {
        let mut s = store();
        let root = s.insert(b"root".to_vec(), vec![], "test");
        let _c1 = s.insert(b"c1".to_vec(), vec![root.clone()], "test");
        let _c2 = s.insert(b"c2".to_vec(), vec![root.clone()], "test");

        let children = s.get_children(&root);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_hashseq_ordered() {
        let a = ContentId::compute(b"a");
        let b = ContentId::compute(b"b");
        let h1 = MerkleDagStore::hashseq(&[a.clone(), b.clone()]);
        let h2 = MerkleDagStore::hashseq(&[a.clone(), b.clone()]);
        assert_eq!(h1, h2);

        let h3 = MerkleDagStore::hashseq(&[b, a]);
        assert_ne!(h1, h3, "order must matter");
    }

    #[test]
    fn test_nodes_by_type() {
        let mut s = store();
        s.insert(b"a".to_vec(), vec![], "alpha");
        s.insert(b"b".to_vec(), vec![], "beta");
        s.insert(b"c".to_vec(), vec![], "alpha");
        assert_eq!(s.nodes_by_type("alpha").len(), 2);
        assert_eq!(s.nodes_by_type("beta").len(), 1);
        assert_eq!(s.nodes_by_type("gamma").len(), 0);
    }

    #[test]
    fn test_evict_oldest_when_full() {
        let mut s = MerkleDagStore::new(3);
        let a = s.insert(b"a".to_vec(), vec![], "t");
        s.insert(b"b".to_vec(), vec![], "t");
        s.insert(b"c".to_vec(), vec![], "t");
        assert_eq!(s.len(), 3);
        s.insert(b"d".to_vec(), vec![], "t");
        assert_eq!(s.len(), 3);
        assert!(!s.contains(&a));
    }
}
