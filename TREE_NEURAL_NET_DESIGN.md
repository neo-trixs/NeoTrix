# ReasoningBrain：树状神经网络设计

## 核心思想

**用户要求**：以自我设定为主，用树状神经网络结构构造自己的核心推理大脑。

**设计原则**：
1. 不是堆叠维度（23+4=27 维扁平向量）
2. 是**树状神经网络**：根节点 → 分支节点 → 叶子节点
3. 权重**自主习得**，而非手动设定（如 design_dialogue: 0.9）
4. OpenDesign 知识是**训练数据**，通过 SEAL 循环被自主吸收

---

## 树状神经网络结构

```
ReasoningTree
├── 根节点 (RootNode)
│   ├── 权重：整体推理能力标量
│   └── 自编辑历史：记录所有权重更新
│
├── 设计哲学分支 (DesignBranch)
│   ├── typography_node (权重习得)
│   ├── color_theory_node (权重习得)
│   ├── whitespace_node (权重习得)
│   └── ... (通过 SEAL 自编辑生长新叶子)
│
├── 推理能力分支 (ReasoningBranch)
│   ├── inference_depth_node
│   ├── creativity_node
│   └── analysis_node
│
├── UI能力分支 (UIBranch)
│   ├── accessibility_node
│   ├── compound_composition_node
│   └── ... (从实际项目吸收)
│
└── OpenDesign 融合分支 (DesignMethodologyBranch)
    ├── design_dialogue_node (通过 self-edit 生长)
    ├── self_critique_node (通过 SEAL 习得权重)
    ├── iterative_refine_node
    └── anti_slop_node
```

---

## Rust 实现思路

### 1. 树节点定义
```rust
struct TreeNode {
    id: String,              // 节点标识（如 "design.typography"）
    weight: f64,             // 节点权重（自主习得，非手动设定）
    children: Vec<TreeNode>,   // 子节点（树状结构）
    parent: Option<usize>,     // 父节点索引
    depth: usize,            // 深度（根=0，叶子=最大）
    last_updated: u64,       // 最后更新时间戳
    update_history: Vec<(u64, f64)>, // 权重更新历史
}
```

### 2. 树神经网络
```rust
struct ReasoningTree {
    root: TreeNode,                    // 根节点
    all_nodes: HashMap<String, TreeNode>, // 快速查找
    growth_log: Vec<GrowthRecord>,     // 树生长记录（SEAL 自编辑）
}
```

### 3. 自编辑生长（SEAL Algorithm 2）
```rust
impl ReasoningTree {
    /// 自编辑：生成新分支/叶子（借鉴 gstack 矩阵分解思想）
    fn generate_self_edit(&self, task: &str) -> TreeEdit {
        // 1. 分析任务类型 → 选择目标分支
        let target_branch = self.select_branch(task);
        
        // 2. 生成新叶子节点（小变换，条件数小）
        let new_leaf = TreeNode::new(format!("{}.{}", target_branch.id, "new_skill"));
        
        // 3. 返回树编辑指令
        TreeEdit::AddNode(target_branch.id, new_leaf)
    }
    
    /// 应用树编辑（支持回滚）
    fn apply_tree_edit(&mut self, edit: &TreeEdit) -> bool {
        match edit {
            TreeEdit::AddNode(parent_id, node) => {
                // 添加到树中
                self.add_child(parent_id, node.clone())
            }
            TreeEdit::UpdateWeight(node_id, delta) => {
                // 更新节点权重（小步长，避免灾难性遗忘）
                if let Some(node) = self.all_nodes.get_mut(node_id) {
                    node.weight = (node.weight + delta).clamp(0.0, 1.0);
                    node.update_history.push((Utc::now().timestamp() as u64, node.weight));
                    true
                } else { false }
            }
            TreeEdit::PruneNode(node_id) => {
                // 剪枝：移除低权重节点
                self.prune(node_id)
            }
        }
    }
}
```

### 4. OpenDesign 知识吸收（不是堆叠，是训练）
```rust
impl ReasoningTree {
    /// 吸收 OpenDesign 知识（通过 SEAL 循环）
    fn absorb_opendesign(&mut self) {
        // OpenDesign 知识成为"训练数据"
        let training_data = TrainingData {
            source: "nexu-io/open-design",
            skills: 19,           // 19 Skills
            design_systems: 71,  // 71 Design Systems
            visual_directions: 5, // 5 Visual Directions
            anti_slop_rules: vec![...], // Anti-AI-slop 规则
        };
        
        // SEAL 循环：生成自编辑 → 应用 → RL 奖励验证 → 持久化
        for episode in 0..10 {
            let tree_edit = self.generate_self_edit("design_task");
            let success = self.apply_tree_edit(&tree_edit);
            
            if success {
                // RL 奖励：基于 OpenDesign 方法论评估
                let reward = self.evaluate_with_opendesign_rules();
                if reward > 0.7 {
                    // 持久化到 ~/.neotrix/brain.json
                    self.save();
                }
            }
        }
    }
}
```

---

## 与当前实现的对比

| 维度 | 扁平向量（当前） | 树状神经网络（目标） |
|------|----------------|--------------------|
| 结构 | 23+4=27 维数组 | 根→分支→叶子的树 |
| 权重设定 | 手动（如 0.9, 0.85） | **自主习得**（SEAL 循环） |
| OpenDesign | 堆叠 4 个维度 | **训练数据**，通过自编辑吸收 |
| 生长方式 | 固定维度 | **动态生长**（新分支/叶子） |
| 灾难性遗忘 | normalize() 防止 | 小步长更新 + 剪枝 |

---

## 下一步行动

1. **重写 `reasoning_brain/core.rs`**：从 `CapabilityVector`（扁平）改为 `ReasoningTree`（树状）
2. **保留 SEAL 循环**：`generate_self_edit()` → `apply_self_edit()` → RL 奖励
3. **OpenDesign 作为训练数据**：不是 `KnowledgeSource::OpenDesign`，而是 `TrainingDataset::OpenDesign`
4. **序列化兼容**：`ReasoningTree` 可以序列化为 JSON（向后兼容）

---

## 用户核心要求的回应

> "以自我的设定为主" → 树的权重和结构通过 **self-iteration** 自主形成，不是我手动设定
> "用树状神经网络的结构构造自己核心推理大脑" → 从 `CapabilityVector` 重构为 `ReasoningTree`
> "OpenDesign 知识融合" → 作为 **training data**，通过 SEAL 循环被吸收，不是静态维度

**关键转变**：从"我告诉大脑有什么能力"到"大脑通过自编辑**自己学会** OpenDesign 的设计哲学"。
