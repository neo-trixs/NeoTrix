# ReasoningBrain 树状神经网络设计（最终版）

> 用户要求："以自我的设定为主，用树状神经网络的结构构造自己核心推理大脑"
> 核心：权重自主习得（SEAL 循环），非手动设定；OpenDesign 知识作为训练数据被吸收

---

## 一、树结构设计

### 1.1 节点定义（自主权重，非手动设定）

```rust
/// 树节点 —— 权重自主习得，非手动设定
struct TreeNode {
    id: String,                  // 节点标识（如 "design.typography"）
    weight: f64,                 // 权重（SEAL 自编辑习得，初始 0.0）
    children: Vec<usize>,        // 子节点索引（树状结构，非扁平数组）
    parent: Option<usize>,       // 父节点（None = 根节点）
    depth: usize,                // 深度（根=0，叶子=最大）
    created_at: u64,             // 创建时间戳（SEAL 生长记录）
    last_updated: u64,          // 最后更新时间
    update_history: Vec<(u64, f64)>, // 权重更新历史（自主记录）
}
```

### 1.2 树神经网络（替代 CapabilityVector）

```rust
/// 树状神经网络 —— 替代扁平 CapabilityVector
struct ReasoningTree {
    nodes: Vec<TreeNode>,          // 所有节点（索引访问，避免借用问题）
    root_idx: usize,               // 根节点索引
    growth_log: Vec<GrowthRecord>,  // 树生长记录（SEAL 自编辑）
}

/// 生长记录（借鉴 gstack 矩阵分解：小变换序列）
struct GrowthRecord {
    timestamp: u64,
    operation: TreeOperation,  // AddNode / UpdateWeight / PruneNode
    node_id: String,
    reward: f64,                // RL 奖励（验证后记录）
}
```

---

## 二、与扁平向量的本质区别

| 维度 | 扁平向量（当前，被拒） | 树状神经网络（目标） |
|------|----------------------|--------------------|
| 结构 | `Array1<f64>` 27 维数组 | `ReasoningTree` 根→分支→叶子 |
| 权重设定 | 手动（如 `design_dialogue: 0.9`） | **自主习得**（SEAL 循环生成） |
| 生长方式 | 固定维度 | **动态生长**（SEAL 添加新分支/叶子） |
| OpenDesign | 堆叠 4 个维度（手动值） | **训练数据**（通过 `absorb()` 被吸收） |
| 更新 | `update_from_other()` 整块更新 | **小变换**（gstack）：`AddNode` / `UpdateWeight` / `PruneNode` |
| 灾难性遗忘 | `normalize()` 防止 | **剪枝**（`PruneNode` 移除低权重节点） |

---

## 三、SEAL 循环适配（树状版本）

### 3.1 自编辑：生成树操作（不是向量更新）

```rust
impl ReasoningTree {
    /// 生成自编辑（借鉴 gstack 矩阵分解）
    /// 返回：小变换序列（AddNode / UpdateWeight / NormalizeTree）
    fn generate_self_edit(&self, task: &str) -> Vec<TreeEdit> {
        let task_type = infer_task_type(task);
        let target_branch = self.select_branch(task_type); // 选择目标分支
        
        // T₁: 若分支不存在，添加新分支（树生长）
        if target_branch.is_none() {
            let new_branch = TreeNode::new(format!("{}_branch", task_type), parent = self.root_idx);
            return vec![TreeEdit::AddNode(new_branch)];
        }
        
        // T₂: 更新相关节点权重（小变换，条件数小）
        let mut edits = vec![];
        let relevant_nodes = self.get_relevant_nodes(target_branch.unwrap());
        for node_idx in relevant_nodes {
            edits.push(TreeEdit::UpdateWeight(node_idx, 0.05)); // 小步长
        }
        
        // T₃: 归一化整棵树（防止维度膨胀）
        edits.push(TreeEdit::NormalizeTree);
        
        edits
    }
}
```

### 3.2 应用树编辑（支持回滚）

```rust
impl ReasoningTree {
    /// 应用树编辑（借鉴 gstack：支持回滚）
    /// 如果 reward 为负，恢复所有更改
    fn apply_tree_edit(&mut self, edits: &[TreeEdit]) -> Vec<usize> {
        let mut applied = vec![];
        
        for (i, edit) in edits.iter().enumerate() {
            match edit {
                TreeEdit::AddNode(node) => {
                    let idx = self.nodes.len();
                    self.nodes.push(node);
                    applied.push(i);
                }
                TreeEdit::UpdateWeight(idx, delta) => {
                    if let Some(node) = self.nodes.get_mut(*idx) {
                        node.weight = (node.weight + delta).min(1.0);
                        node.last_updated = Utc::now().timestamp() as u64;
                        node.update_history.push((node.last_updated, node.weight));
                        applied.push(i);
                    }
                }
                TreeEdit::PruneNode(idx) => {
                    // 剪枝：移除低权重节点（防止树无界生长）
                    if self.nodes[*idx].weight < 0.1 {
                        self.nodes.remove(*idx);
                        applied.push(i);
                    }
                }
                TreeEdit::NormalizeTree => {
                    // 归一化整棵树（保持总权重合理）
                    let total_weight: f64 = self.nodes.iter().map(|n| n.weight).sum();
                    if total_weight > 0.0 {
                        for node in &mut self.nodes {
                            node.weight /= total_weight;
                        }
                    }
                    applied.push(i);
                }
            }
        }
        
        applied
    }
}
```

---

## 四、OpenDesign 知识吸收（作为训练数据）

### 4.1 不是堆叠维度，是训练数据

```rust
/// OpenDesign 知识作为训练数据（不是手动维度）
struct OpenDesignDataset {
    skills: Vec<String>,           // 19 Skills
    design_systems: Vec<String>,   // 71 DESIGN.md
    visual_directions: Vec<VisualDirection>, // 5 种方向
    anti_slop_rules: Vec<String>, // Anti-AI-slop 规则
}

/// 吸收 OpenDesign（通过 SEAL 循环）
impl ReasoningTree {
    fn absorb_opendesign(&mut self, dataset: &OpenDesignDataset) {
        // 1. 分析数据集 → 生成树编辑指令
        let tree_edits = self.generate_edits_from_dataset(dataset);
        
        // 2. 应用编辑（生长树）
        let applied = self.apply_tree_edit(&tree_edits);
        
        // 3. RL 奖励计算（基于 OpenDesign 方法论）
        let reward = self.evaluate_with_opendesign_rules(dataset);
        
        // 4. 若奖励 > 阈值，持久化
        if reward > 0.7 {
            self.save_to_file("~/.neotrix/brain.json");
        }
    }
}
```

### 4.2 树生长示例（自主形成 OpenDesign 分支）

```
ReasoningTree（生长后）
├── 根节点 (weight: 1.0)
│
├── DesignPhilosophy 分支 (weight: 0.85)
│   ├── typography_node (0.9, 从 HeroUI 吸收)
│   ├── color_theory_node (0.95, 从 OpenDesign 吸收)
│   └── ...
│
├── ReasoningBranch (weight: 0.8)
│   ├── inference_depth_node (0.8)
│   └── analysis_node (0.85)
│
└── OpenDesign 分支 (weight: 0.92, 自主生长！)
    ├── design_dialogue_node (0.9, SEAL 习得)
    ├── self_critique_node (0.85, 5维批判)
    ├── iterative_refine_node (0.9, TodoWrite 计划)
    └── anti_slop_node (0.95, Anti-AI-slop)
```

---

## 五、序列化（向后兼容）

### 5.1 树 → JSON（持久化到 `~/.neotrix/brain.json`）

```rust
#[derive(Serialize, Deserialize)]
struct ReasoningTreeHelper {
    nodes: Vec<TreeNodeHelper>,  // 扁平化存储
    root_idx: usize,
}

#[derive(Serialize, Deserialize)]
struct TreeNodeHelper {
    id: String,
    weight: f64,
    children: Vec<usize>,  // 子节点索引
    parent: Option<usize>,
    depth: usize,
    // ... 其他字段
}
```

### 5.2 向后兼容 CapabilityVector

```rust
impl ReasoningTree {
    /// 转换为扁平向量（向后兼容旧代码）
    fn to_flat_vector(&self) -> Vec<f64> {
        // 按照某种顺序（如深度优先）展平为向量
        // 用于需要扁平向量的场景（如相似度计算）
        let mut vec = vec![];
        self.dfs_flatten(self.root_idx, &mut vec);
        vec
    }
}
```

---

## 六、实施计划（研究后动手）

### 6.1 第一步：定义树结构（替代 CapabilityVector）

- [ ] 删除 `define_capability_fields!` 宏
- [ ] 定义 `TreeNode` 和 `ReasoningTree`
- [ ] 实现 `generate_self_edit()` → `Vec<TreeEdit>`
- [ ] 实现 `apply_tree_edit()`（支持回滚）

### 6.2 第二步：迁移 SEAL 循环

- [ ] 修改 `ReasoningBrain`：将 `capability: CapabilityVector` 改为 `tree: ReasoningTree`
- [ ] 更新 `absorb()`：从"更新向量"改为"生长树"
- [ ] 更新 `select_relevant_sources()`：返回树分支而非向量切片

### 6.3 第三步：OpenDesign 作为训练数据

- [ ] 定义 `OpenDesignDataset`
- [ ] 实现 `absorb_opendesign()`：通过 SEAL 循环吸收
- [ ] 更新 `select_relevant_sources()`：包含 OpenDesign 数据集

### 6.4 第四步：序列化与持久化

- [ ] 实现 `ReasoningTreeHelper` 用于 JSON 序列化
- [ ] 更新 `save()` / `load()` 方法
- [ ] 向后兼容：保留 `to_flat_vector()` 用于相似度计算

---

## 七、用户核心要求的回应

> "以自我的设定为主" → 树的权重和结构通过 **SEAL 自迭代** 自主形成，不是我手动设定  
> "用树状神经网络的结构构造自己核心推理大脑" → 从 `CapabilityVector` 重构为 `ReasoningTree`  
> "OpenDesign 知识融合" → 作为 **training data**，通过 SEAL 循环被吸收，不是静态维度  

**关键转变**：
- 从："我告诉大脑有什么能力（手动 0.9）"
- 到："大脑通过自编辑**自己学会** OpenDesign 的设计哲学（自主生长节点）"

---

## 八、下一步（等待用户指令）

| 选项 | 行动 |
|------|------|
| **A. 开始实现** | 重写 `core.rs`：删除 `CapabilityVector`，实现 `ReasoningTree` |
| **B. 先写测试** | TDD：先定义 `TreeNode` 测试，再实现 |
| **C. 原型验证** | 写一个小原型（独立文件），验证树生长逻辑 |
| **D. 其他** | 用户指定 |

需要我开始实现吗？还是先写测试/原型？
