//! NTX 知识图谱段 — 追加只读邻接表
//!
//! 解决痛点 #6: 便携性 — 图谱嵌入单文件
//! O(1) 邻居查询, 顺序扫描全图

use std::io::{Read, Seek, Write};
use serde::{Serialize, Deserialize};

/// 边方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EdgeDirection {
    Outgoing = 0,
    Incoming = 1,
    Bidirectional = 2,
}

/// 图谱边
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: [u8; 36],
    pub target: [u8; 36],
    pub edge_type: u16,
    pub weight: f32,
    pub direction: EdgeDirection,
}

/// 图谱节点
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub node_id: [u8; 36],
    pub degree: u32,
    pub neighbors: Vec<GraphNeighbor>,
}

/// 邻居
#[derive(Debug, Clone)]
pub(crate) struct GraphNeighbor {
    pub node_id: [u8; 36],
    pub edge_type: u16,
    pub weight: f32,
    pub direction: EdgeDirection,
}

use std::collections::HashMap;

/// 图谱段 (O(1) 邻居查询 via HashMap 索引)
pub struct GraphSegment {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    node_index: HashMap<[u8; 36], usize>,  // node_id → nodes 索引
}

impl GraphSegment {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), node_index: HashMap::new() }
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: GraphEdge) {
        let edge_clone = edge.clone();
        self.edges.push(edge);
        self.update_adjacency(&edge_clone);
    }

    /// 更新邻接表 (O(1) via HashMap 索引)
    fn update_adjacency(&mut self, edge: &GraphEdge) {
        // 源节点
        if let Some(&idx) = self.node_index.get(&edge.source) {
            let node = &mut self.nodes[idx];
            node.neighbors.push(GraphNeighbor {
                node_id: edge.target,
                edge_type: edge.edge_type,
                weight: edge.weight,
                direction: edge.direction,
            });
            node.degree += 1;
        } else {
            let idx = self.nodes.len();
            self.nodes.push(GraphNode {
                node_id: edge.source,
                degree: 1,
                neighbors: vec![GraphNeighbor {
                    node_id: edge.target,
                    edge_type: edge.edge_type,
                    weight: edge.weight,
                    direction: edge.direction,
                }],
            });
            self.node_index.insert(edge.source, idx);
        }

        // 目标节点 (双向或入边)
        if edge.direction == EdgeDirection::Bidirectional || edge.direction == EdgeDirection::Incoming {
            if let Some(&idx) = self.node_index.get(&edge.target) {
                let node = &mut self.nodes[idx];
                node.neighbors.push(GraphNeighbor {
                    node_id: edge.source,
                    edge_type: edge.edge_type,
                    weight: edge.weight,
                    direction: EdgeDirection::Outgoing,
                });
                node.degree += 1;
            } else {
                let idx = self.nodes.len();
                self.nodes.push(GraphNode {
                    node_id: edge.target,
                    degree: 1,
                    neighbors: vec![GraphNeighbor {
                        node_id: edge.source,
                        edge_type: edge.edge_type,
                        weight: edge.weight,
                        direction: EdgeDirection::Outgoing,
                    }],
                });
                self.node_index.insert(edge.target, idx);
            }
        }
    }

    /// 获取节点邻居 (O(1))
    pub fn neighbors(&self, node_id: &[u8; 36]) -> Option<&[GraphNeighbor]> {
        self.node_index.get(node_id)
            .map(|&idx| self.nodes[idx].neighbors.as_slice())
    }

    /// 获取度数 (O(1))
    pub fn degree(&self, node_id: &[u8; 36]) -> Option<u32> {
        self.node_index.get(node_id).map(|&idx| self.nodes[idx].degree)
    }

    /// BFS 遍历
    pub fn bfs(&self, start: &[u8; 36], max_depth: usize) -> Vec<([u8; 36], usize)> {
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((*start, 0));
        visited.insert(*start);

        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_depth { continue; }
            result.push((current, depth));

            if let Some(node) = self.nodes.iter().find(|n| n.node_id == current) {
                for neighbor in &node.neighbors {
                    if visited.insert(neighbor.node_id) {
                        queue.push_back((neighbor.node_id, depth + 1));
                    }
                }
            }
        }

        result
    }

    /// 获取所有节点
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }

    /// 获取所有边
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// 节点数
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 边数
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 写入段 (magic + data)
    pub fn write_to(&self, writer: &mut (impl Write + Seek)) -> std::io::Result<u64> {
        let start = writer.stream_position()?;

        // Magic
        writer.write_all(b"NTGR")?;

        writer.write_all(&(self.nodes.len() as u64).to_le_bytes())?;
        writer.write_all(&(self.edges.len() as u64).to_le_bytes())?;

        for node in &self.nodes {
            writer.write_all(&node.node_id)?;
            writer.write_all(&node.degree.to_le_bytes())?;
            writer.write_all(&(node.neighbors.len() as u32).to_le_bytes())?;
            for neighbor in &node.neighbors {
                writer.write_all(&neighbor.node_id)?;
                writer.write_all(&neighbor.edge_type.to_le_bytes())?;
                writer.write_all(&neighbor.weight.to_le_bytes())?;
                writer.write_all(&[neighbor.direction as u8])?;
            }
        }

        for edge in &self.edges {
            writer.write_all(&edge.source)?;
            writer.write_all(&edge.target)?;
            writer.write_all(&edge.edge_type.to_le_bytes())?;
            writer.write_all(&edge.weight.to_le_bytes())?;
            writer.write_all(&[edge.direction as u8])?;
        }

        Ok(start)
    }

    /// 读取段
    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        // Magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != *b"NTGR" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid Graph magic: expected NTGR, got {:?}", magic),
            ));
        }

        let mut node_count_buf = [0u8; 8];
        reader.read_exact(&mut node_count_buf)?;
        let node_count = u64::from_le_bytes(node_count_buf) as usize;

        let mut edge_count_buf = [0u8; 8];
        reader.read_exact(&mut edge_count_buf)?;
        let edge_count = u64::from_le_bytes(edge_count_buf) as usize;

        let mut nodes = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            let mut node_id = [0u8; 36];
            reader.read_exact(&mut node_id)?;

            let mut degree_buf = [0u8; 4];
            reader.read_exact(&mut degree_buf)?;
            let degree = u32::from_le_bytes(degree_buf);

            let mut neighbor_count_buf = [0u8; 4];
            reader.read_exact(&mut neighbor_count_buf)?;
            let neighbor_count = u32::from_le_bytes(neighbor_count_buf) as usize;

            let mut neighbors = Vec::with_capacity(neighbor_count);
            for _ in 0..neighbor_count {
                let mut nid = [0u8; 36];
                reader.read_exact(&mut nid)?;
                let mut et_buf = [0u8; 2];
                reader.read_exact(&mut et_buf)?;
                let mut w_buf = [0u8; 4];
                reader.read_exact(&mut w_buf)?;
                let mut dir_buf = [0u8; 1];
                reader.read_exact(&mut dir_buf)?;
                neighbors.push(GraphNeighbor {
                    node_id: nid,
                    edge_type: u16::from_le_bytes(et_buf),
                    weight: f32::from_le_bytes(w_buf),
                    direction: match dir_buf[0] {
                        0 => EdgeDirection::Outgoing,
                        1 => EdgeDirection::Incoming,
                        _ => EdgeDirection::Bidirectional,
                    },
                });
            }

            nodes.push(GraphNode { node_id, degree, neighbors });
        }

        let mut edges = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
            let mut source = [0u8; 36];
            reader.read_exact(&mut source)?;
            let mut target = [0u8; 36];
            reader.read_exact(&mut target)?;
            let mut et_buf = [0u8; 2];
            reader.read_exact(&mut et_buf)?;
            let mut w_buf = [0u8; 4];
            reader.read_exact(&mut w_buf)?;
            let mut dir_buf = [0u8; 1];
            reader.read_exact(&mut dir_buf)?;
            edges.push(GraphEdge {
                source,
                target,
                edge_type: u16::from_le_bytes(et_buf),
                weight: f32::from_le_bytes(w_buf),
                direction: match dir_buf[0] {
                    0 => EdgeDirection::Outgoing,
                    1 => EdgeDirection::Incoming,
                    _ => EdgeDirection::Bidirectional,
                },
            });
        }

        Ok(Self { nodes, edges, node_index: std::collections::HashMap::new() })
    }
}

impl Default for GraphSegment {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nid(n: u8) -> [u8; 36] { let mut id = [0u8; 36]; id[0] = n; id }

    #[test]
    fn test_graph_roundtrip() {
        let mut g = GraphSegment::new();
        g.add_edge(GraphEdge {
            source: nid(1), target: nid(2), edge_type: 0, weight: 1.0,
            direction: EdgeDirection::Bidirectional,
        });
        g.add_edge(GraphEdge {
            source: nid(2), target: nid(3), edge_type: 1, weight: 0.5,
            direction: EdgeDirection::Outgoing,
        });

        let mut buf = std::io::Cursor::new(Vec::new());
        g.write_to(&mut buf).unwrap();
        buf.set_position(0);
        let restored = GraphSegment::read_from(&mut buf).unwrap();

        assert_eq!(restored.node_count(), 3);
        assert_eq!(restored.edge_count(), 2);
    }

    #[test]
    fn test_bfs() {
        let mut g = GraphSegment::new();
        g.add_edge(GraphEdge { source: nid(1), target: nid(2), edge_type: 0, weight: 1.0, direction: EdgeDirection::Bidirectional });
        g.add_edge(GraphEdge { source: nid(2), target: nid(3), edge_type: 0, weight: 1.0, direction: EdgeDirection::Bidirectional });

        let visited = g.bfs(&nid(1), 2);
        assert_eq!(visited.len(), 3);
    }

    #[test]
    fn test_neighbors() {
        let mut g = GraphSegment::new();
        g.add_edge(GraphEdge { source: nid(1), target: nid(2), edge_type: 0, weight: 1.0, direction: EdgeDirection::Outgoing });
        let neighbors = g.neighbors(&nid(1)).unwrap();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].node_id, nid(2));
    }
}
