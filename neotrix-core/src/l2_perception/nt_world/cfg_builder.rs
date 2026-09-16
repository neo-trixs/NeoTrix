//! CFGBuilder — Build Control Flow Graphs from binary code

use std::collections::HashMap;

/// A basic block in the control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub start_address: u64,
    pub end_address: u64,
    pub instructions: Vec<String>,
    pub is_entry: bool,
    pub is_exit: bool,
}

/// An edge between basic blocks
#[derive(Debug, Clone)]
pub struct CFGEdge {
    pub from: usize,
    pub to: usize,
    pub edge_type: EdgeType,
}

/// Type of control flow edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Unconditional,
    ConditionalTrue,
    ConditionalFalse,
    Call,
    Return,
    Exception,
}

/// A complete Control Flow Graph
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub basic_blocks: Vec<BasicBlock>,
    pub edges: Vec<CFGEdge>,
    pub entry_point: usize,
    pub exit_points: Vec<usize>,
}

/// Builds Control Flow Graphs from binary code
pub struct CFGBuilder {
    cfg: Option<ControlFlowGraph>,
    block_map: HashMap<u64, usize>,
}

impl CFGBuilder {
    /// Create a new CFGBuilder
    pub fn new() -> Self {
        Self {
            cfg: None,
            block_map: HashMap::new(),
        }
    }

    /// Build a CFG from the given binary code bytes
    pub fn build(&mut self, code: &[u8], entry_point: u64) -> &ControlFlowGraph {
        self.cfg = None;
        self.block_map.clear();

        let blocks = self.identify_basic_blocks(code, entry_point);
        let edges = self.identify_edges(&blocks);

        let entry_idx = self.block_map.get(&entry_point).copied().unwrap_or(0);
        let exit_points = blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| b.is_exit)
            .map(|(i, _)| i)
            .collect();

        self.cfg = Some(ControlFlowGraph {
            basic_blocks: blocks,
            edges,
            entry_point: entry_idx,
            exit_points,
        });

        self.cfg.as_ref().unwrap()
    }

    /// Get the basic blocks of the built CFG
    pub fn get_basic_blocks(&self) -> &[BasicBlock] {
        match &self.cfg {
            Some(cfg) => &cfg.basic_blocks,
            None => &[],
        }
    }

    /// Get the edges of the built CFG
    pub fn get_edges(&self) -> &[CFGEdge] {
        match &self.cfg {
            Some(cfg) => &cfg.edges,
            None => &[],
        }
    }

    /// Get the entry point index of the CFG
    pub fn get_entry_point(&self) -> usize {
        match &self.cfg {
            Some(cfg) => cfg.entry_point,
            None => 0,
        }
    }

    fn identify_basic_blocks(&mut self, code: &[u8], entry_point: u64) -> Vec<BasicBlock> {
        let mut blocks = Vec::new();
        let chunk_size = code.len().max(1).min(16);
        let mut addr = entry_point;
        let mut id = 0;

        for chunk in code.chunks(chunk_size) {
            let is_entry = addr == entry_point;
            let is_exit = chunk.len() < chunk_size || id == (code.len() / chunk_size) - 1;

            let instructions = chunk
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect();

            let block = BasicBlock {
                id,
                start_address: addr,
                end_address: addr + chunk.len() as u64,
                instructions,
                is_entry,
                is_exit,
            };

            self.block_map.insert(addr, id);
            blocks.push(block);
            id += 1;
            addr += chunk.len() as u64;
        }

        blocks
    }

    fn identify_edges(&self, blocks: &[BasicBlock]) -> Vec<CFGEdge> {
        let mut edges = Vec::new();

        for (i, block) in blocks.iter().enumerate() {
            if block.is_exit {
                continue;
            }

            if i + 1 < blocks.len() {
                edges.push(CFGEdge {
                    from: i,
                    to: i + 1,
                    edge_type: EdgeType::Unconditional,
                });
            }

            if block.instructions.len() > 1 && i + 2 < blocks.len() {
                edges.push(CFGEdge {
                    from: i,
                    to: i + 2,
                    edge_type: EdgeType::ConditionalTrue,
                });
            }
        }

        edges
    }
}

impl Default for CFGBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let mut builder = CFGBuilder::new();
        let code = vec![0u8; 64];
        let cfg = builder.build(&code, 0x0);
        assert!(!cfg.basic_blocks.is_empty());
    }

    #[test]
    fn test_get_basic_blocks() {
        let mut builder = CFGBuilder::new();
        let code = vec![0u8; 64];
        builder.build(&code, 0x0);
        let blocks = builder.get_basic_blocks();
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_get_edges() {
        let mut builder = CFGBuilder::new();
        let code = vec![0u8; 64];
        builder.build(&code, 0x0);
        let edges = builder.get_edges();
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_get_entry_point() {
        let mut builder = CFGBuilder::new();
        let code = vec![0u8; 64];
        builder.build(&code, 0x1000);
        let entry = builder.get_entry_point();
        assert_eq!(entry, 0);
    }
}
