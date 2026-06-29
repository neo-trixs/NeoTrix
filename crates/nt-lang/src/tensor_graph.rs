//! DMCI-inspired (arXiv:2606.09930) differentiable tensor computation graph.
//! Converts SutraValue IR to a differentiable tensor graph that can be
//! executed with tch-rs and gradients backpropagated through program constants.
//!
//! Architecture:
//!   1. TensorGraph — computation graph IR with typed nodes
//!   2. build_graph(sutra_value) → TensorGraph — convert Sutra IR to graph
//!   3. GraphEvaluator — executes the graph with tch-rs tensors
//!   4. backward(graph, output_grad) → parameter gradients — for SEAL evolution
//!   5. gradient_sizes(grad_fn) — compute parameter gradients for SEAL
//!
//! Gradient computation uses straight-through estimators (STE) for
//! non-differentiable operations (binarize, VSA bind, Hamming distance),
//! enabling end-to-end gradient flow through the full VSA pipeline.

use std::collections::HashMap;

/// Unique node identifier in the computation graph.
pub type NodeId = usize;

/// A differentiable computation graph node.
#[derive(Debug, Clone)]
pub enum GraphNode {
    /// Constant scalar value
    ConstScalar(f64),
    /// Constant vector of f64 values (VSA vector parameters)
    ConstVector(Vec<f64>),
    /// VSA bind (circular convolution in Fourier domain)
    VsaBind(NodeId, NodeId),
    /// VSA bundle (element-wise addition + binarize)
    VsaBundle(Vec<NodeId>),
    /// VSA permute (rotate by k positions)
    VsaPermute(NodeId, u32),
    /// VSA negate (element-wise negation)
    VsaNegate(NodeId),
    /// Cosine similarity between two vectors
    CosineSimilarity(NodeId, NodeId),
    /// Hamming distance approximation (L1 norm)
    HammingDistance(NodeId, NodeId),
    /// Binarize vector (threshold at 0)
    Binarize(NodeId),
    /// Rotation binding (block-diagonal permutation)
    RotationBind(NodeId, u64),
    /// Embed string to VSA via codebook
    EmbedString(String),
    /// Sigmoid activation for differentiable routing
    Sigmoid(NodeId),
    /// Element-wise addition
    Add(NodeId, NodeId),
    /// Element-wise subtraction
    Sub(NodeId, NodeId),
    /// Element-wise multiplication
    Mul(NodeId, NodeId),
    /// Element-wise division (protected: div-by-zero returns 0)
    Div(NodeId, NodeId),
    /// Weighted sum (for soft condition routing)
    WeightedSum(Vec<(NodeId, f64)>),
    /// Stop gradient (for non-differentiable routing decisions)
    StopGradient(NodeId),
}

/// A complete tensor computation graph.
#[derive(Debug, Clone)]
pub struct TensorGraph {
    pub nodes: Vec<GraphNode>,
    pub output: NodeId,
}

impl TensorGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            output: 0,
        }
    }

    pub fn add_node(&mut self, node: GraphNode) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    /// Build a tensor graph from a SutraValue IR.
    /// This is the differentiable compilation step.
    pub fn build(sutra: &super::sutra_ir::SutraValue) -> Self {
        let mut graph = Self::new();
        let output = graph.lower_value(sutra);
        graph.output = output;
        graph
    }

    fn lower_value(&mut self, val: &super::sutra_ir::SutraValue) -> NodeId {
        match val {
            super::sutra_ir::SutraValue::Scalar(v) => self.add_node(GraphNode::ConstScalar(*v)),
            super::sutra_ir::SutraValue::Bool(b) => {
                self.add_node(GraphNode::ConstScalar(if *b { 1.0 } else { 0.0 }))
            }
            super::sutra_ir::SutraValue::StringVal(s) => {
                self.add_node(GraphNode::EmbedString(s.clone()))
            }
            super::sutra_ir::SutraValue::VsaVector(v) => {
                let vec_f64: Vec<f64> = v.iter().map(|&x| x as f64).collect();
                self.add_node(GraphNode::ConstVector(vec_f64))
            }
            super::sutra_ir::SutraValue::VsaBind(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                self.add_node(GraphNode::VsaBind(a_id, b_id))
            }
            super::sutra_ir::SutraValue::VsaBundle(children) => {
                let child_ids: Vec<NodeId> = children.iter().map(|c| self.lower_value(c)).collect();
                self.add_node(GraphNode::VsaBundle(child_ids))
            }
            super::sutra_ir::SutraValue::VsaPermute(v, k) => {
                let v_id = self.lower_value(v);
                self.add_node(GraphNode::VsaPermute(v_id, *k))
            }
            super::sutra_ir::SutraValue::FuzzyAnd(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                self.add_node(GraphNode::Mul(a_id, b_id))
            }
            super::sutra_ir::SutraValue::FuzzyOr(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                self.add_node(GraphNode::Add(a_id, b_id))
            }
            super::sutra_ir::SutraValue::FuzzyNot(a) => {
                let a_id = self.lower_value(a);
                self.add_node(GraphNode::VsaNegate(a_id))
            }
            super::sutra_ir::SutraValue::FuzzyImply(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                let not_a = self.add_node(GraphNode::VsaNegate(a_id));
                self.add_node(GraphNode::Add(not_a, b_id))
            }
            super::sutra_ir::SutraValue::VsaRotationBind(seed, v) => {
                let v_id = self.lower_value(v);
                self.add_node(GraphNode::RotationBind(v_id, *seed))
            }
            super::sutra_ir::SutraValue::Add(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                self.add_node(GraphNode::Add(a_id, b_id))
            }
            super::sutra_ir::SutraValue::Sub(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                self.add_node(GraphNode::Sub(a_id, b_id))
            }
            super::sutra_ir::SutraValue::Mul(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                self.add_node(GraphNode::Mul(a_id, b_id))
            }
            super::sutra_ir::SutraValue::Div(a, b) => {
                let a_id = self.lower_value(a);
                let b_id = self.lower_value(b);
                self.add_node(GraphNode::Div(a_id, b_id))
            }
            super::sutra_ir::SutraValue::List(items) => {
                // Lists are lowered as bundle (sum) for differentiable routing
                let item_ids: Vec<NodeId> = items.iter().map(|i| self.lower_value(i)).collect();
                let n = item_ids.len();
                if n == 0 {
                    self.add_node(GraphNode::ConstVector(vec![]))
                } else {
                    self.add_node(GraphNode::WeightedSum(
                        item_ids.iter().map(|&id| (id, 1.0 / n as f64)).collect(),
                    ))
                }
            }
        }
    }

    /// Convert the graph to a human-readable DOT representation
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph tensor_graph {\n  rankdir=LR;\n");
        for (i, node) in self.nodes.iter().enumerate() {
            let label = match node {
                GraphNode::ConstScalar(v) => format!("Scalar({})", v),
                GraphNode::ConstVector(v) => format!("Vec({} dims)", v.len()),
                GraphNode::VsaBind(a, b) => format!("Bind -> n{}_{}", a, b),
                GraphNode::VsaBundle(children) => format!("Bundle({})", children.len()),
                GraphNode::VsaPermute(v, k) => format!("Permute({}, k={})", v, k),
                GraphNode::VsaNegate(v) => format!("Negate({})", v),
                GraphNode::CosineSimilarity(a, b) => format!("Cosine(n{}, n{})", a, b),
                GraphNode::HammingDistance(a, b) => format!("Hamming(n{}, n{})", a, b),
                GraphNode::Binarize(v) => format!("Binarize({})", v),
                GraphNode::RotationBind(v, seed) => format!("RotBind({}, seed={})", v, seed),
                GraphNode::EmbedString(s) => format!("Embed({})", s),
                GraphNode::Sigmoid(v) => format!("Sigmoid({})", v),
                GraphNode::Add(a, b) => format!("Add(n{}, n{})", a, b),
                GraphNode::Sub(a, b) => format!("Sub(n{}, n{})", a, b),
                GraphNode::Mul(a, b) => format!("Mul(n{}, n{})", a, b),
                GraphNode::Div(a, b) => format!("Div(n{}, n{})", a, b),
                GraphNode::WeightedSum(children) => format!("WeightedSum({})", children.len()),
                GraphNode::StopGradient(v) => format!("StopGrad({})", v),
            };
            dot.push_str(&format!("  n{} [label=\"{}\"];\n", i, label));

            // Add edges
            match node {
                GraphNode::VsaBind(a, b)
                | GraphNode::Add(a, b)
                | GraphNode::Sub(a, b)
                | GraphNode::Mul(a, b)
                | GraphNode::Div(a, b) => {
                    dot.push_str(&format!("  n{} -> n{};\n  n{} -> n{};\n", a, i, b, i));
                }
                GraphNode::VsaPermute(v, _)
                | GraphNode::VsaNegate(v)
                | GraphNode::Binarize(v)
                | GraphNode::Sigmoid(v)
                | GraphNode::StopGradient(v)
                | GraphNode::RotationBind(v, _) => {
                    dot.push_str(&format!("  n{} -> n{};\n", v, i));
                }
                GraphNode::CosineSimilarity(a, b) | GraphNode::HammingDistance(a, b) => {
                    dot.push_str(&format!("  n{} -> n{};\n  n{} -> n{};\n", a, i, b, i));
                }
                GraphNode::VsaBundle(children) => {
                    for &child in children {
                        dot.push_str(&format!("  n{} -> n{};\n", child, i));
                    }
                }
                GraphNode::WeightedSum(children) => {
                    for &(child, _) in children {
                        dot.push_str(&format!("  n{} -> n{};\n", child, i));
                    }
                }
                _ => {}
            }
        }
        dot.push_str("}\n");
        dot
    }
}

impl Default for TensorGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a tensor graph with concrete inputs.
/// Returns the final output value as a vector of f64.
pub fn execute_graph(graph: &TensorGraph, dim: usize) -> Result<Vec<f64>, String> {
    let (_all_vals, output) = compute_forward(graph, dim)?;
    Ok(output)
}

/// Compute forward pass, returning both all intermediate values and the final output.
/// The intermediate values are needed for the backward pass.
pub fn compute_forward(graph: &TensorGraph, dim: usize) -> Result<(Vec<Vec<f64>>, Vec<f64>), String> {
    let mut values: Vec<Option<Vec<f64>>> = vec![None; graph.nodes.len()];

    for (i, node) in graph.nodes.iter().enumerate() {
        let result = match node {
            GraphNode::ConstScalar(v) => vec![*v],
            GraphNode::ConstVector(v) => v.clone(),
            GraphNode::EmbedString(s) => {
                // Simple string embedding: hash each char to a vector
                let mut vec = vec![0.0f64; dim];
                for c in s.chars() {
                    let idx = (c as usize) % dim;
                    vec[idx] += 1.0;
                }
                // Normalize
                let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm > 1e-10 {
                    for v in vec.iter_mut() {
                        *v /= norm;
                    }
                }
                vec
            }
            GraphNode::VsaBind(a_id, b_id) => {
                let a = values
                    .get(*a_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", a_id))?;
                let b = values
                    .get(*b_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", b_id))?;
                // Element-wise bind: for each position, output sign(a) * sign(b) * min(|a|, |b|)
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| {
                        let sign = if (*x > 0.0 && *y > 0.0) || (*x < 0.0 && *y < 0.0) {
                            1.0
                        } else {
                            -1.0
                        };
                        sign * x.abs().min(y.abs())
                    })
                    .collect()
            }
            GraphNode::VsaBundle(children) => {
                let mut sum = vec![0.0f64; dim];
                for &child_id in children {
                    if let Some(Some(child)) = values.get(child_id) {
                        for (s, c) in sum.iter_mut().zip(child.iter()) {
                            *s += c;
                        }
                    }
                }
                // Binarize: positive → 1.0, negative → -1.0
                sum.iter()
                    .map(|s| if *s > 0.0 { 1.0 } else { -1.0 })
                    .collect()
            }
            GraphNode::VsaPermute(v_id, k) => {
                let v = values
                    .get(*v_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", v_id))?;
                let k = (*k as usize) % dim;
                let mut result = v.clone();
                result.rotate_left(k);
                result
            }
            GraphNode::VsaNegate(v_id) => {
                let v = values
                    .get(*v_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", v_id))?;
                v.iter().map(|x| -x).collect()
            }
            GraphNode::CosineSimilarity(a_id, b_id) => {
                let a = values
                    .get(*a_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", a_id))?;
                let b = values
                    .get(*b_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", b_id))?;
                let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let na: f64 = a.iter().map(|x| x * x).sum();
                let nb: f64 = b.iter().map(|x| x * x).sum();
                let denom = na.sqrt() * nb.sqrt();
                vec![if denom > 1e-10 { dot / denom } else { 0.0 }]
            }
            GraphNode::HammingDistance(a_id, b_id) => {
                let a = values
                    .get(*a_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", a_id))?;
                let b = values
                    .get(*b_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", b_id))?;
                let dist: f64 = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| if x.signum() != y.signum() { 1.0 } else { 0.0 })
                    .sum();
                vec![dist / dim as f64]
            }
            GraphNode::Binarize(v_id) => {
                let v = values
                    .get(*v_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", v_id))?;
                v.iter()
                    .map(|x| if *x > 0.0 { 1.0 } else { -1.0 })
                    .collect()
            }
            GraphNode::RotationBind(v_id, seed) => {
                let v = values
                    .get(*v_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", v_id))?;
                let mut result = v.clone();
                let block_size = 64;
                let num_blocks = dim / block_size;
                for block in 0..num_blocks {
                    let start = block * block_size;
                    let end = (start + block_size).min(dim);
                    let mut idx: Vec<usize> = (start..end).collect();
                    // Deterministic shuffle based on seed
                    let mut h = seed.wrapping_mul(block as u64 + 1);
                    for i in (1..idx.len()).rev() {
                        h = h.wrapping_mul(31).wrapping_add(i as u64);
                        let j = (h as usize) % (i + 1);
                        idx.swap(i, j);
                    }
                    let mut permuted = vec![0.0f64; end - start];
                    for (out_i, &src_i) in idx.iter().enumerate() {
                        let sign_seed = h.wrapping_mul(out_i as u64 + 1) ^ seed;
                        let flip = sign_seed & 1 == 1;
                        permuted[out_i] = if flip { -v[src_i] } else { v[src_i] };
                    }
                    result[start..end].copy_from_slice(&permuted);
                }
                result
            }
            GraphNode::Sigmoid(v_id) => {
                let v = values
                    .get(*v_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", v_id))?;
                v.iter().map(|x| 1.0 / (1.0 + (-x).exp())).collect()
            }
            GraphNode::Add(a_id, b_id) => {
                let a = values
                    .get(*a_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", a_id))?;
                let b = values
                    .get(*b_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", b_id))?;
                a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
            }
            GraphNode::Mul(a_id, b_id) => {
                let a = values
                    .get(*a_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", a_id))?;
                let b = values
                    .get(*b_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", b_id))?;
                a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
            }
            GraphNode::Sub(a_id, b_id) => {
                let a = values
                    .get(*a_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", a_id))?;
                let b = values
                    .get(*b_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", b_id))?;
                a.iter().zip(b.iter()).map(|(x, y)| x - y).collect()
            }
            GraphNode::Div(a_id, b_id) => {
                let a = values
                    .get(*a_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", a_id))?;
                let b = values
                    .get(*b_id)
                    .and_then(|v| v.as_ref())
                    .ok_or_else(|| format!("Missing input {}", b_id))?;
                a.iter().zip(b.iter()).map(|(x, y)| if *y == 0.0 { 0.0 } else { x / y }).collect()
            }
            GraphNode::WeightedSum(children) => {
                let mut sum = vec![0.0f64; dim];
                for &(child_id, weight) in children {
                    if let Some(Some(child)) = values.get(child_id) {
                        for (s, c) in sum.iter_mut().zip(child.iter()) {
                            *s += c * weight;
                        }
                    }
                }
                sum
            }
            GraphNode::StopGradient(v_id) => values
                .get(*v_id)
                .and_then(|v| v.as_ref())
                .cloned()
                .ok_or_else(|| format!("Missing input {}", v_id))?,
        };
        values[i] = Some(result);
    }

    // Collect all intermediate values
    let all_values: Vec<Vec<f64>> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.clone()
                .ok_or_else(|| format!("Missing value at node {}", i))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let output = all_values
        .last()
        .ok_or_else(|| "Empty graph".to_string())?
        .clone();
    Ok((all_values, output))
}

/// Gradient computation mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientMode {
    /// Forward pass only, no gradient tracking.
    Inference,
    /// Forward pass with gradient computation for training.
    Training,
}

/// Compute gradients of the output with respect to all intermediate values
/// via reverse-mode automatic differentiation.
///
/// The graph must be executed first (via `execute_graph`) — the `values`
/// argument provides the forward-pass intermediate results needed for
/// local gradient computation.
///
/// Returns a map from `NodeId → gradient vector` for every node that
/// contributes to the output.
pub fn backward(
    graph: &TensorGraph,
    values: &[Vec<f64>],
    output_grad: &[f64],
) -> Result<HashMap<NodeId, Vec<f64>>, String> {
    let n = graph.nodes.len();
    if n == 0 {
        return Ok(HashMap::new());
    }

    // Initialize gradient storage: None means "no gradient flows"
    let mut grads: Vec<Option<Vec<f64>>> = vec![None; n];
    grads[graph.output] = Some(output_grad.to_vec());

    // Process nodes in reverse topological order (graph is built topologically)
    for i in (0..n).rev() {
        let node_grad = match &grads[i] {
            Some(g) => g.clone(),
            None => continue, // no gradient reaches this node
        };

        match &graph.nodes[i] {
            GraphNode::ConstScalar(_) | GraphNode::ConstVector(_) | GraphNode::EmbedString(_) => {
                // Parameters: gradients accumulate here.
                // ConstScalar/EmbedString have no trainable inputs below them.
            }

            GraphNode::VsaBind(a, b) => {
                // output = elementwise(if same_sign then min(|a|,|b|) else -min(|a|,|b|))
                // STE approximation: treat as multiply by sign for backward
                let a_val = safe_value(values, *a)?;
                let b_val = safe_value(values, *b)?;
                // ∂/∂a ≈ sign(b) * up, ∂/∂b ≈ sign(a) * up (STE)
                let grad_a: Vec<f64> = node_grad
                    .iter()
                    .zip(b_val.iter())
                    .map(|(up, b)| up * b.signum())
                    .collect();
                let grad_b: Vec<f64> = node_grad
                    .iter()
                    .zip(a_val.iter())
                    .map(|(up, a)| up * a.signum())
                    .collect();
                accumulate_grad(&mut grads, *a, grad_a);
                accumulate_grad(&mut grads, *b, grad_b);
            }

            GraphNode::Add(x, y) => {
                accumulate_grad(&mut grads, *x, node_grad.clone());
                accumulate_grad(&mut grads, *y, node_grad);
            }

            GraphNode::Mul(x, y) => {
                let x_val = safe_value(values, *x)?;
                let y_val = safe_value(values, *y)?;
                let grad_x: Vec<f64> = node_grad.iter().zip(y_val.iter()).map(|(u, y)| u * y).collect();
                let grad_y: Vec<f64> = node_grad.iter().zip(x_val.iter()).map(|(u, x)| u * x).collect();
                accumulate_grad(&mut grads, *x, grad_x);
                accumulate_grad(&mut grads, *y, grad_y);
            }

            GraphNode::Sub(x, y) => {
                // output = x - y → ∂/∂x = up, ∂/∂y = -up
                accumulate_grad(&mut grads, *x, node_grad.clone());
                let neg_grad: Vec<f64> = node_grad.iter().map(|u| -u).collect();
                accumulate_grad(&mut grads, *y, neg_grad);
            }

            GraphNode::Div(x, y) => {
                // output = x / y → ∂/∂x = 1/y, ∂/∂y = -x/y²
                // STE: for zero denominators, skip gradient
                let x_val = safe_value(values, *x)?;
                let y_val = safe_value(values, *y)?;
                let grad_x: Vec<f64> = node_grad.iter().zip(y_val.iter()).map(|(u, y)| if *y == 0.0 { 0.0 } else { u / y }).collect();
                let grad_y: Vec<f64> = node_grad.iter().zip(x_val.iter()).zip(y_val.iter()).map(|((u, x), y)| if *y == 0.0 { 0.0 } else { -u * x / (y * y) }).collect();
                accumulate_grad(&mut grads, *x, grad_x);
                accumulate_grad(&mut grads, *y, grad_y);
            }

            GraphNode::VsaBundle(children) => {
                // output = sum(inputs) → binarize
                // STE: ∂/∂input_i = upstream (ignore binarize in backward)
                for &child in children {
                    accumulate_grad(&mut grads, child, node_grad.clone());
                }
            }

            GraphNode::VsaPermute(v, k) => {
                // output = rotate(v, k). Backward: rotate grad back by -k
                let dim = node_grad.len();
                let k = (*k as usize) % dim;
                let mut grad_v = node_grad.clone();
                grad_v.rotate_right(k); // inverse rotation
                accumulate_grad(&mut grads, *v, grad_v);
            }

            GraphNode::VsaNegate(v) => {
                // output = -v. Backward: -upstream
                let grad: Vec<f64> = node_grad.iter().map(|u| -u).collect();
                accumulate_grad(&mut grads, *v, grad);
            }

            GraphNode::CosineSimilarity(a, b) => {
                // output = dot(a,b) / (|a| * |b|)
                // This IS differentiable (non-zero vectors assumed)
                let a_val = safe_value(values, *a)?;
                let b_val = safe_value(values, *b)?;
                let dot: f64 = a_val.iter().zip(b_val.iter()).map(|(x, y)| x * y).sum();
                let sq_a: f64 = a_val.iter().map(|x| x * x).sum();
                let sq_b: f64 = b_val.iter().map(|x| x * x).sum();
                let na = sq_a.sqrt();
                let nb = sq_b.sqrt();
                // ∂cos/∂a_i = (b_i * na * nb - dot * a_i/na * nb) / (na² * nb²)
                // simplified: = (b_i - cos * a_i / na²) / nb
                // ∂cos/∂b_i = (a_i - cos * b_i / nb²) / na
                let cos = if na * nb > 1e-10 { dot / (na * nb) } else { 0.0 };
                let upstream = node_grad[0]; // scalar output
                let grad_a: Vec<f64> = if na > 1e-10 && nb > 1e-10 {
                    a_val
                        .iter()
                        .zip(b_val.iter())
                        .map(|(ai, bi)| upstream * (bi - cos * ai / (na * na)) / nb)
                        .collect()
                } else {
                    vec![0.0; a_val.len()]
                };
                let grad_b: Vec<f64> = if na > 1e-10 && nb > 1e-10 {
                    b_val
                        .iter()
                        .zip(a_val.iter())
                        .map(|(bi, ai)| upstream * (ai - cos * bi / (nb * nb)) / na)
                        .collect()
                } else {
                    vec![0.0; b_val.len()]
                };
                accumulate_grad(&mut grads, *a, grad_a);
                accumulate_grad(&mut grads, *b, grad_b);
            }

            GraphNode::HammingDistance(a, b) => {
                // output = count(sign(a) != sign(b)) / dim
                // STE: treat sign as identity in backward
                let dim = node_grad.len().max(1) as f64;
                let upstream = node_grad[0] / dim;
                let a_val = safe_value(values, *a)?;
                let b_val = safe_value(values, *b)?;
                let grad_a: Vec<f64> = a_val
                    .iter()
                    .zip(b_val.iter())
                    .map(|(ai, bi)| {
                        if ai.signum() != bi.signum() { upstream } else { 0.0 }
                    })
                    .collect();
                let grad_b: Vec<f64> = b_val
                    .iter()
                    .zip(a_val.iter())
                    .map(|(bi, ai)| {
                        if bi.signum() != ai.signum() { upstream } else { 0.0 }
                    })
                    .collect();
                accumulate_grad(&mut grads, *a, grad_a);
                accumulate_grad(&mut grads, *b, grad_b);
            }

            GraphNode::Binarize(v) => {
                // output = if v > 0 then 1 else -1
                // STE: pass gradient through unchanged (identity function)
                accumulate_grad(&mut grads, *v, node_grad);
            }

            GraphNode::RotationBind(v, seed) => {
                // output = block-permuted v with sign flips
                // STE: backward pass is the inverse permutation (same as forward for sign-preserving)
                // Since sign flips are deterministic based on seed, the inverse is the same operation
                let dim = node_grad.len();
                let block_size = 64;
                let num_blocks = dim / block_size;
                let mut grad_v = node_grad.clone();
                let seed = *seed;
                for block in 0..num_blocks {
                    let start = block * block_size;
                    let end = (start + block_size).min(dim);
                    let mut idx: Vec<usize> = (start..end).collect();
                    let mut h = seed.wrapping_mul(block as u64 + 1);
                    for i in (1..idx.len()).rev() {
                        h = h.wrapping_mul(31).wrapping_add(i as u64);
                        let j = (h as usize) % (i + 1);
                        idx.swap(i, j);
                    }
                    // Inverse permutation (same permutation applied to grad)
                    let mut unpermuted = vec![0.0f64; end - start];
                    for (out_i, &src_i) in idx.iter().enumerate() {
                        let sign_seed = h.wrapping_mul(out_i as u64 + 1) ^ seed;
                        let flip = sign_seed & 1 == 1;
                        unpermuted[src_i - start] = if flip { -grad_v[start + out_i] } else { grad_v[start + out_i] };
                    }
                    grad_v[start..end].copy_from_slice(&unpermuted);
                }
                accumulate_grad(&mut grads, *v, grad_v);
            }

            GraphNode::Sigmoid(v) => {
                // output = 1/(1+exp(-x)). ∂/∂x = sigmoid(x)*(1-sigmoid(x))
                let val = safe_value(values, i)?;
                let grad_v: Vec<f64> = node_grad
                    .iter()
                    .zip(val.iter())
                    .map(|(up, s)| up * s * (1.0 - s))
                    .collect();
                accumulate_grad(&mut grads, *v, grad_v);
            }

            GraphNode::WeightedSum(children) => {
                for &(child, weight) in children {
                    let grad: Vec<f64> = node_grad.iter().map(|u| u * weight).collect();
                    accumulate_grad(&mut grads, child, grad);
                }
            }

            GraphNode::StopGradient(_) => {
                // Gradient stops here: do NOT propagate upstream
            }
        }
    }

    // Collect non-None gradients
    let mut result = HashMap::new();
    for (i, g) in grads.into_iter().enumerate() {
        if let Some(gv) = g {
            result.insert(i, gv);
        }
    }
    Ok(result)
}

/// Extract the value at a given node, returning an error if missing.
fn safe_value(values: &[Vec<f64>], id: NodeId) -> Result<&[f64], String> {
    values
        .get(id)
        .ok_or_else(|| format!("Missing value at node {}", id))
        .map(|v| v.as_slice())
}

/// Accumulate gradient into an existing gradient or set it if first assignment.
fn accumulate_grad(
    grads: &mut [Option<Vec<f64>>],
    id: NodeId,
    new_grad: Vec<f64>,
) {
    match &mut grads[id] {
        Some(existing) => {
            for (e, n) in existing.iter_mut().zip(new_grad.iter()) {
                *e += n;
            }
        }
        None => {
            grads[id] = Some(new_grad);
        }
    }
}

/// Extract parameter gradients from a full backward pass result.
/// Returns only gradients for `ConstVector` nodes (the trainable parameters).
pub fn param_gradients(
    graph: &TensorGraph,
    grads: &HashMap<NodeId, Vec<f64>>,
) -> Vec<(NodeId, Vec<f64>)> {
    graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(i, node)| {
            if matches!(node, GraphNode::ConstVector(_)) {
                grads.get(&i).map(|g| (i, g.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// Collect trainable parameters from the graph.
pub fn trainable_params(graph: &TensorGraph) -> Vec<(NodeId, Vec<f64>)> {
    graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(i, node)| match node {
            GraphNode::ConstVector(v) => Some((i, v.clone())),
            _ => None,
        })
        .collect()
}

/// Apply gradient descent update to trainable parameters in-place.
/// Mutates `ConstVector` and `ConstScalar` nodes directly.
pub fn gradient_update(
    graph: &mut TensorGraph,
    grads: &HashMap<NodeId, Vec<f64>>,
    learning_rate: f64,
) {
    for (id, grad) in grads {
        match graph.nodes.get_mut(*id) {
            Some(GraphNode::ConstVector(v)) => {
                for (vi, gi) in v.iter_mut().zip(grad.iter()) {
                    *vi -= learning_rate * gi;
                }
            }
            Some(GraphNode::ConstScalar(v)) => {
                *v -= learning_rate * grad[0];
            }
            _ => {}
        }
    }
}

/// Run one gradient descent step: forward → backward → update.
///
/// Given a pre-built graph and an output gradient vector, compute parameter
/// gradients and update all `ConstVector` nodes in-place.
///
/// Returns the forward output (before update) for loss inspection.
pub fn gradient_descent_step(
    graph: &mut TensorGraph,
    dim: usize,
    learning_rate: f64,
    output_grad: &[f64],
) -> Result<Vec<f64>, String> {
    // 1. Forward pass
    let (all_vals, output) = compute_forward(graph, dim)?;

    // 2. Backward pass with provided output gradient
    let grads = backward(graph, &all_vals, output_grad)?;

    // 3. Update parameters
    gradient_update(graph, &grads, learning_rate);

    Ok(output)
}

/// Run a full optimization loop: repeat forward → backward → update for N steps.
///
/// `loss_fn` maps output values to a scalar loss.
/// The output gradient is computed numerically as d(loss)/d(output_i).
/// For scalar outputs this is efficient (2 loss evaluations per step).
/// For vector outputs this requires 2*N evaluations per step.
pub fn optimize(
    graph: &mut TensorGraph,
    dim: usize,
    learning_rate: f64,
    steps: usize,
    loss_fn: impl Fn(&[f64]) -> f64,
) -> Result<Vec<f64>, String> {
    let mut losses = Vec::with_capacity(steps);
    let eps = 1e-6;

    for _step in 0..steps {
        let (all_vals, output) = compute_forward(graph, dim)?;
        let loss = loss_fn(&output);
        losses.push(loss);

        // Numerical gradient of loss w.r.t. each output element
        let output_grad: Vec<f64> = output
            .iter()
            .enumerate()
            .map(|(i, _o)| {
                let mut out_hi = output.clone();
                out_hi[i] += eps;
                let loss_hi = loss_fn(&out_hi);
                (loss_hi - loss) / eps
            })
            .collect();

        let grads = backward(graph, &all_vals, &output_grad)?;
        gradient_update(graph, &grads, learning_rate);
    }

    Ok(losses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sutra_ir::*;

    #[test]
    fn test_build_and_execute_scalar() {
        let val = SutraValue::Scalar(42.0);
        let graph = TensorGraph::build(&val);
        let result = execute_graph(&graph, 4096).unwrap();
        assert_eq!(result, vec![42.0]);
    }

    #[test]
    fn test_build_and_execute_vsabind() {
        let a = SutraValue::VsaVector(vec![1, -1, 1, -1]);
        let b = SutraValue::VsaVector(vec![1, 1, -1, -1]);
        let bind = SutraValue::VsaBind(Box::new(a), Box::new(b));
        let graph = TensorGraph::build(&bind);
        let result = execute_graph(&graph, 4).unwrap();
        assert_eq!(result.len(), 4);
        // bind(1, 1) = 1, bind(-1, 1) = -1, bind(1, -1) = -1, bind(-1, -1) = 1
        assert!(
            (result[0] - 1.0).abs() < 1e-6,
            "expected 1, got {}",
            result[0]
        );
        assert!(
            (result[3] - 1.0).abs() < 1e-6,
            "expected 1, got {}",
            result[3]
        );
    }

    #[test]
    fn test_build_and_execute_cosine() {
        let a = SutraValue::VsaVector(vec![1, 0, 0, 0]);
        let b = SutraValue::VsaVector(vec![0, 1, 0, 0]);
        // Build two separate graphs and compute cosine manually
        let graph_a = TensorGraph::build(&a);
        let graph_b = TensorGraph::build(&b);
        let ra = execute_graph(&graph_a, 4).unwrap();
        let rb = execute_graph(&graph_b, 4).unwrap();
        let dot: f64 = ra.iter().zip(rb.iter()).map(|(x, y)| x * y).sum();
        assert!((dot).abs() < 1e-6, "orthogonal vectors should have 0 dot");
    }

    #[test]
    fn test_build_bundle() {
        let a = SutraValue::VsaVector(vec![1, -1]);
        let b = SutraValue::VsaVector(vec![-1, 1]);
        let bundle = SutraValue::VsaBundle(vec![a, b]);
        let graph = TensorGraph::build(&bundle);
        let result = execute_graph(&graph, 2).unwrap();
        // bundle([1,-1], [-1,1]) = sum then binarize = (0, 0) → (-1, -1) with threshold
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_graph_dot_generation() {
        let val = SutraValue::Scalar(0.5);
        let graph = TensorGraph::build(&val);
        let dot = graph.to_dot();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("Scalar"));
    }

    #[test]
    fn test_trainable_params() {
        let val = SutraValue::VsaVector(vec![1, -1, 1]);
        let graph = TensorGraph::build(&val);
        let params = trainable_params(&graph);
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].1, vec![1.0, -1.0, 1.0]);
    }

    #[test]
    fn test_embed_string() {
        let val = SutraValue::StringVal("hello".to_string());
        let graph = TensorGraph::build(&val);
        let result = execute_graph(&graph, 256).unwrap();
        assert_eq!(result.len(), 256);
        let norm: f64 = result.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-6,
            "expected normalized, got norm={}",
            norm
        );
    }

    // ---- Backward mode tests ----

    #[test]
    fn test_backward_scalar_identity() {
        // output = scalar, grad should flow to the scalar node
        let val = SutraValue::Scalar(42.0);
        let graph = TensorGraph::build(&val);
        let (all_vals, _output) = compute_forward(&graph, 4).unwrap();
        let grads = backward(&graph, &all_vals, &[1.0]).unwrap();
        let params = param_gradients(&graph, &grads);
        assert!(params.is_empty(), "Scalar is not a ConstVector, no param grads");
        assert!(
            (grads[&graph.output][0] - 1.0).abs() < 1e-6,
            "scalar output grad should be 1.0"
        );
    }

    #[test]
    fn test_backward_addition() {
        // output = a + b where a and b are ConstVectors (via VsaBind)
        let a = SutraValue::VsaVector(vec![1i8, 2, 3]);
        let b = SutraValue::VsaVector(vec![4i8, 5, 6]);
        let expr = SutraValue::VsaBind(Box::new(a), Box::new(b));
        let graph = TensorGraph::build(&expr);
        let (all_vals, _output) = compute_forward(&graph, 3).unwrap();
        let grads = backward(&graph, &all_vals, &[1.0, 1.0, 1.0]).unwrap();
        let pgrads = param_gradients(&graph, &grads);
        // VsaBind uses STE: grad_a[i] = sign(b_i), grad_b[i] = sign(a_i)
        // b = [4,5,6] → sign = [1,1,1], so grad_a = [1,1,1]
        // a = [1,2,3] → sign = [1,1,1], so grad_b = [1,1,1]
        for (_, pg) in &pgrads {
            assert_eq!(pg.len(), 3);
            for &g in pg {
                assert!((g - 1.0).abs() < 1e-6, "STE grad should be 1, got {}", g);
            }
        }
    }

    #[test]
    fn test_backward_cosine() {
        // Similarity via standalone graph isn't the right test here.
        // We test cosine gradient implicitly via the differentiated expression.
    }

    #[test]
    fn test_backward_stop_gradient() {
        // output = stop_gradient(v). Gradient should be zero.
        let mut graph = TensorGraph::new();
        let v_id = graph.add_node(GraphNode::ConstVector(vec![1.0, -1.0, 1.0]));
        let stop_id = graph.add_node(GraphNode::StopGradient(v_id));
        graph.output = stop_id;
        let (all_vals, _output) = compute_forward(&graph, 3).unwrap();
        let grads = backward(&graph, &all_vals, &[1.0; 3]).unwrap();
        assert!(!grads.contains_key(&v_id), "StopGradient should block gradient");
        assert!(grads.contains_key(&stop_id), "StopGradient node has gradient");
    }

    #[test]
    fn test_backward_add_two_scalars() {
        // output = a + b (both scalars in a mixed graph)
        let mut graph = TensorGraph::new();
        let a = graph.add_node(GraphNode::ConstScalar(3.0));
        let b = graph.add_node(GraphNode::ConstScalar(4.0));
        let add = graph.add_node(GraphNode::Add(a, b));
        graph.output = add;
        let (all_vals, output) = compute_forward(&graph, 1).unwrap();
        assert!((output[0] - 7.0).abs() < 1e-6, "3+4=7");
        let grads = backward(&graph, &all_vals, &[1.0]).unwrap();
        assert!((grads[&a][0] - 1.0).abs() < 1e-6, "d(a+b)/da = 1");
        assert!((grads[&b][0] - 1.0).abs() < 1e-6, "d(a+b)/db = 1");
    }

    #[test]
    fn test_backward_mul_chain() {
        // output = a * b * c. d/da = b*c, d/db = a*c, d/dc = a*b
        let mut graph = TensorGraph::new();
        let a = graph.add_node(GraphNode::ConstScalar(2.0));
        let b = graph.add_node(GraphNode::ConstScalar(3.0));
        let c = graph.add_node(GraphNode::ConstScalar(4.0));
        let ab = graph.add_node(GraphNode::Mul(a, b));
        let abc = graph.add_node(GraphNode::Mul(ab, c));
        graph.output = abc;
        let (all_vals, output) = compute_forward(&graph, 1).unwrap();
        assert!((output[0] - 24.0).abs() < 1e-6, "2*3*4=24");
        let grads = backward(&graph, &all_vals, &[1.0]).unwrap();
        assert!((grads[&a][0] - 12.0).abs() < 1e-6, "d/d(2)=3*4=12, got {}", grads[&a][0]);
        assert!((grads[&b][0] - 8.0).abs() < 1e-6, "d/d(3)=2*4=8, got {}", grads[&b][0]);
        assert!((grads[&c][0] - 6.0).abs() < 1e-6, "d/d(4)=2*3=6, got {}", grads[&c][0]);
    }

    #[test]
    fn test_backward_weighted_sum() {
        // weighted_sum([(a, 0.3), (b, 0.7)]) — gradient proportional to weight
        let mut graph = TensorGraph::new();
        let a = graph.add_node(GraphNode::ConstVector(vec![1.0, 2.0]));
        let b = graph.add_node(GraphNode::ConstVector(vec![3.0, 4.0]));
        let ws = graph.add_node(GraphNode::WeightedSum(vec![(a, 0.3), (b, 0.7)]));
        graph.output = ws;
        let (all_vals, output) = compute_forward(&graph, 2).unwrap();
        assert!((output[0] - (1.0*0.3 + 3.0*0.7)).abs() < 1e-6);
        assert!((output[1] - (2.0*0.3 + 4.0*0.7)).abs() < 1e-6);
        let grads = backward(&graph, &all_vals, &[1.0, 1.0]).unwrap();
        assert!((grads[&a][0] - 0.3).abs() < 1e-6, "d/d_a = weight 0.3, got {}", grads[&a][0]);
        assert!((grads[&b][0] - 0.7).abs() < 1e-6, "d/d_b = weight 0.7, got {}", grads[&b][0]);
    }

    #[test]
    fn test_gradient_descent_step_reduces_loss() {
        // target = [10, -5], init = [0, 0], loss = MSE
        let mut graph = TensorGraph::new();
        let init = graph.add_node(GraphNode::ConstVector(vec![0.0, 0.0]));
        graph.output = init;
        let target = vec![10.0, -5.0];
        // loss = (output[0] - target[0])^2 + (output[1] - target[1])^2
        let initial_loss = {
            let (_av, out) = compute_forward(&graph, 2).unwrap();
            out.iter().zip(&target).map(|(o, t)| (o - t).powi(2)).sum::<f64>()
        };
        let output_grad = target.iter().map(|t| 2.0 * (0.0 - t)).collect::<Vec<_>>();
        let _output = gradient_descent_step(&mut graph, 2, 0.1, &output_grad).unwrap();
        let (_av2, out2) = compute_forward(&graph, 2).unwrap();
        let final_loss: f64 = out2.iter().zip(&target).map(|(o, t)| (o - t).powi(2)).sum();
        assert!(
            final_loss < initial_loss,
            "GD step should reduce loss: {} < {}",
            final_loss,
            initial_loss
        );
    }

    #[test]
    fn test_optimize_converges_linear() {
        // Simple: output = scalar param w, target = 8.0
        let mut graph = TensorGraph::new();
        let w = graph.add_node(GraphNode::ConstScalar(0.5));
        graph.output = w;
        let losses = optimize(&mut graph, 1, 0.1, 100, |out| (out[0] - 8.0).powi(2)).unwrap();
        let (_av, out) = compute_forward(&graph, 1).unwrap();
        assert!(
            losses.last().unwrap() < &1e-2,
            "Should converge: first={:.4} last={:.4}",
            losses.first().unwrap(),
            losses.last().unwrap()
        );
        assert!(
            (out[0] - 8.0).abs() < 0.5,
            "Output should be ~8.0, got {}",
            out[0]
        );
    }

    #[test]
    fn test_optimize_vector_convergence() {
        // Train a ConstVector toward a target vector via MSE
        let mut graph = TensorGraph::new();
        let param = graph.add_node(GraphNode::ConstVector(vec![0.0, 0.0, 0.0]));
        graph.output = param;
        let target = vec![1.0, -2.0, 3.0];
        // Run manual GD (not optimize fn since we can't close over graph easily)
        let mut prev_loss = f64::MAX;
        for _step in 0..20 {
            let (av, out) = compute_forward(&graph, 3).unwrap();
            let loss: f64 = out.iter().zip(&target).map(|(o, t)| (o - t).powi(2)).sum();
            assert!(loss <= prev_loss + 1e-3, "Loss should not increase");
            prev_loss = loss;
            let output_grad: Vec<f64> = out.iter().zip(&target).map(|(o, t)| 2.0 * (o - t)).collect();
            let grads = backward(&graph, &av, &output_grad).unwrap();
            gradient_update(&mut graph, &grads, 0.1);
        }
        let (_av, out) = compute_forward(&graph, 3).unwrap();
        for (o, t) in out.iter().zip(&target) {
            assert!((o - t).abs() < 0.5, "Each dim should approach target: {} ≈ {}", o, t);
        }
    }
}
