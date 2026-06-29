import { ExperienceTree } from '../core/experience-tree';

const tree = new ExperienceTree();

// === Core architecture (existing) ===
tree.addNode('vsa', 'VSA 4096-bit', '统一向量表征 · MAP/BSC/HRR/FHRR 四模型', 0.95);
tree.addNode('e8', 'E8 64-state', '推理核 · 64态流形推理', 0.92, 'vsa');
tree.addNode('hypercube', 'HyperCube', '知识超立方体 · 6维索引', 0.88, 'vsa');
tree.addNode('gwt', 'GWT Workspace', '全局工作空间 · 注意力焦点', 0.85, 'vsa');
tree.addNode('seal', 'SEAL 进化', '自我进化 · GEPA Pareto + MetaSEAL', 0.90, 'e8');
tree.addNode('gepa', 'GEPA 反射', 'Trace→Reflect→Mutate→Evaluate→Select', 0.82, 'seal');
tree.addNode('hebbian', 'Hebbian 记忆', '联想记忆 + DistillationAgent', 0.80, 'seal');
tree.addNode('iit', 'IIT Φ 8', '8路并行MIP采样 · 意识度量', 0.78, 'hypercube');
tree.addNode('consensus', 'P2P 共识', 'Banach不动点迭代 + λ收缩', 0.75, 'hypercube');
tree.addNode('freeenergy', '自由能好奇', 'Active Inference · N_deficit驱动', 0.82, 'gwt');
tree.addNode('council', '身份委员会', '匿名LLM · 身份轮换池', 0.74, 'hypercube');
tree.addNode('veto', 'VETO 否决门', '自由不选择 · VolitionEngine', 0.70, 'seal');
tree.addNode('governance', '治理哈希链', 'SEAL操作审计 · 防篡改账簿', 0.68, 'seal');

// === Knowledge Absorption Capability (new) ===
tree.addNode('absorption', '吸收引擎 v2', '仓库理解 + 知识吸收 · 4阶段管道', 0.88, 'gwt');
tree.addNode('codegraph', 'CodeGraph 结构索引', '实体/边图 · 带可信度的类型化关系', 0.85, 'absorption');

// Absorbed papers & tools
tree.addNode('rpg_encoder', 'RPG-Encoder ICML 2026', '意图↔实现双向循环 · 95.7%拓扑开销缩减', 0.92, 'absorption');
tree.addNode('codebase_mem', 'Codebase-Memory', '66语种 Tree-Sitter KG · 14 MCP工具 · 10×少token', 0.88, 'absorption');
tree.addNode('aoci', 'AOCI 符号-语义索引', '单遍结构蓝图 · 工业任务零缺陷 · 4-130× token缩减', 0.90, 'absorption');
tree.addNode('larger', 'LARGER 词法锚定图检索', '词法搜索锚入仓库图 · +13.9 Acc@5', 0.85, 'absorption');
tree.addNode('fastcode', 'FastCode 侦查优先', '探索→消费解耦 · 侦查先行 · 单步高价值上下文', 0.87, 'absorption');

// Derived insights & defects
tree.addNode('d1_no_index', 'D1: 无结构索引', '当前逐文件线性读取 → 需预建结构图', 0.70, 'codegraph');
tree.addNode('d2_no_multiphase', 'D2: 无多阶段吸收', '缺乏Scout→Extract→Deep-read→Synthesize流程', 0.75, 'codegraph');
tree.addNode('d3_no_persist', 'D3: 无跨会话持久化', '每会话重头理解 → 需增量演化拓扑', 0.72, 'codegraph');
tree.addNode('d5_no_compression', 'D4: 无蓝图压缩', '全文本输入 → 需符号+语义双重表示', 0.68, 'codegraph');

// === Phase pipeline completed (new) ===
tree.addNode('phases', '吸收管线完成', 'scout→extract→deep-read→synthesize 全闭环', 0.90, 'absorption');
tree.addNode('source_ground', '源扎根', '每个知识条目携带来源和可信度', 0.85, 'phases');
tree.addNode('defect_track', '缺陷追踪', '知识→下游缺陷显式链接', 0.82, 'phases');

// === Absorption quality metrics (new) ===
tree.addNode('quality', '吸收质量', '每阶段质量评分 · 来源可信度 · 完整性追踪', 0.78, 'phases');

interface Props { compact?: boolean }

export default function ExperienceTreeView({ compact }: Props) {
  function renderNode(id: string, depth = 0): JSX.Element | null {
    const node = tree.getNode(id);
    if (!node) return null;
    return (
      <div key={id}>
        <div className="tree-row" style={{ paddingLeft: depth * 10 }}>
          <span className="tree-dot" style={{ opacity: 0.3 + node.confidence * 0.7 }} />
          <span className="tree-label">{node.label}</span>
          {!compact && <span className="tree-desc">{node.description}</span>}
          <span className="tree-conf">{(node.confidence * 100).toFixed(0)}%</span>
        </div>
        {node.children.map(c => renderNode(c, depth + 1))}
      </div>
    );
  }

  return (
    <div className="sidebar-placeholder">
      {!compact && (
        <div className="view-header" style={{ marginBottom: 8 }}>
          <h3 style={{ fontSize: 11, margin: 0 }}>Experience Tree</h3>
          <span className="view-meta">{tree.size()} nodes</span>
        </div>
      )}
      <div className="tree-container">
        {tree.getRoots().map(r => renderNode(r.id))}
      </div>
    </div>
  );
}
