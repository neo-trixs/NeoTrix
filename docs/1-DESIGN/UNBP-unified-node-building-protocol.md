# UNBP — Unified Node Building Protocol (统一节点构建协议)

> 状态: accepted | 日期: 2026-08-09 | 作者: NT-CORE (E8引导者)
> 触发: KB 统一构建审计发现 323241 悬空边 + 22021 孤立节点 + 2 组重复节点

## 1. 上下文 (Context)

KB (SQLite, `/Users/neo/.neotrix/knowledge.db`) 经多会话、多批次、多工具写入后出现三类数据孤岛问题:

| 问题 | 规模 | 根因 |
|------|------|------|
| 悬空边 (source/target 指向不存在节点) | 323241 条 (48%) | 早期导入节点被清理但边未归档; 部分边 id 为 NULL |
| 孤立节点 (无任何边) | 22021 个 | 早期吸收 book/article/repository 未接线 |
| 重复节点 (同 URL/标题) | 2 组 (真实地球框架) | URL 锚点变体 (`#zh-full`) 绕过 absorb-node 去重 |

根因: **无统一构建协议** — 节点写入通道多样 (Rust CLI / Python 直写 / 历史脚本), 无规范化、无去重门、无接线门。

## 2. 决策 (Decision)

建立 **UNBP (Unified Node Building Protocol)** — 统一节点构建协议, 作为 KB 写入的唯一规范。

### 2.1 单一写入通道 (Single Write Path)

```
所有节点写入 → neotrix-experience absorb-node (Rust CLI)
                ├── URL 规范化去重 (去 # 锚点 + 尾斜杠 + 大小写)
                ├── nodes + nodes_fts 双写 (同 rowid)
                └── 返回 inserted/duplicated/mapped 计数
```

- **禁止** Python 直写 `nodes` 表 (R-P97 纪律扩展)
- Python 仅允许: 数据 prep (生成 JSON) / edges 写入 / 升级 (必须同步 FTS)

### 2.2 URL 规范化规则 (URL Normalization)

absorb-node 去重前必须规范化 URL:

```
1. 去 fragment: url.split('#')[0]  (关键! #zh-full/#RealEarth4D 是视角标记非唯一性)
2. 去尾斜杠: rstrip('/')
3. 域名小写
4. 保留 query (部分 API URL 依赖)
```

**锚点策略**: 同一 URL 不同视角 (如 `#zh-full` 中文全量) 若需独立节点, 必须:
- 标题差异化 (含视角词), 且
- 显式声明 `meta.anchor_variant = true`, 否则视为重复合并

### 2.3 去重门 (Dedup Gate)

absorb-node 落盘前检查 (优先级从高到低):

1. **URL 规范化后精确匹配** → duplicate (跳过)
2. **标题精确匹配** → duplicate (跳过)
3. **标题 LIKE 匹配 + 同维度** → 警告 (人工确认)

### 2.4 接线门 (Wiring Gate)

每个新节点必须满足以下之一, 否则标记 `meta.pending_wiring`:

- 属于某框架 (contains 边到框架节点)
- 有 ≥1 条 related_to/part_of 边
- 是框架节点本身 (如真实地球四维世界观)

**孤立节点禁止**: 新节点落库后 24h 内必须接线, 否则进入 `pending_wiring` 队列。

### 2.5 维度规范 (Dimension Canon)

真实地球框架节点标题规范:

```
真实地球·{维度}·{名称}（{副标题}）
维度 ∈ {空间D1, 时间D2, 文明D3, 理论D4, 科学D4, 探险D5, 生物人文D6}
```

- 维度前缀**单一来源** (禁止 `真实地球·D3文明·真实地球·文明维度D3·` 重复)
- 标题生成必须用模板函数, 禁止手拼

### 2.6 边规范 (Edge Canon)

- 边 id 生成: `{batch}_{md5(source|type|target)[:16]}`
- 边必须带 `metadata.batch` (溯源)
- 悬空边禁止: 写入前校验 source/target 存在

## 3. 后果 (Consequences)

### 正面
- 重复节点归零 (URL 规范化去重)
- 悬空边归零 (写入前校验)
- 孤立节点可追踪 (pending_wiring 队列)
- 全库 nodes = FTS 恒等 (双写纪律)

### 负面
- absorb-node 需增加 URL 规范化逻辑 (Rust 改动)
- 历史遗留 22021 孤立节点需逐步接线 (非本协议范围, 单独治理)

## 4. 备选方案 (Alternatives)

| 方案 | 拒绝理由 |
|------|---------|
| 全量重建 KB | 破坏 6.5 万节点历史数据, 不可逆 |
| 仅清理不建协议 | 治标不治本, 未来仍会重复/孤立 |
| Python 统一写入层 | 与 R-P97 (Rust 唯一写入) 冲突 |

## 5. 验收标准 (Acceptance)

- [x] 悬空边 = 0 (323241 → 0, 已归档 deleted_edges)
- [x] 真实地球框架孤立 = 0
- [x] 真实地球框架重复 = 0 (2 组合并)
- [x] 全库 nodes = FTS
- [ ] absorb-node URL 规范化 (Rust 待实现)
- [ ] 历史孤立节点接线 (后续治理)

## 6. 治理记录 (Governance Log)

| 日期 | 操作 | 结果 |
|------|------|------|
| 2026-08-09 | 悬空边归档 deleted_edges | 323241 → 0 |
| 2026-08-09 | NULL-id 边清理 | 61 条归档 |
| 2026-08-09 | 重复节点合并 (拜占庭/冷战) | 2 组 → 各 1 |
| 2026-08-09 | 标题污染修复 (4 节点) | nodes+fts 双写 |
| 2026-08-09 | 孤立节点修复 (真实地球 4 个) | 补 contains 边 |