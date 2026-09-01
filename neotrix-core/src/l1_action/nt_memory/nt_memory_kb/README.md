# KnowledgeBase — NeoTrix 持久化知识层

## 架构
```
KnowledgeBase (SQLite WAL + FTS5)
├── store.rs    — 节点/边/队列 CRUD
├── search.rs   — FTS5 混合搜索 + BM25 回退
├── graph.rs    — BFS 最短路径 + 子图 + 社区发现
├── pipeline.rs — Wikipedia/ArXiv/GitHub 爬取
├── seed.rs     — 文明基础种子 (88 节点 + 89 边)
├── integration.rs — WebMiner/KnowledgeEngine 桥接
├── consciousness_interface.rs — E8/GWT 意识查询
└── schema.rs   — DDL + FTS5 索引
```

## 关键决策
- SQLite WAL + FTS5 (非内存) — 跨会话持久
- FTS5 为主 + BM25 回退 — 精度+召回兼顾
- `insert_or_get_node` 先查 URL, 无 URL 则按 title+type 去重
- PLACE IN `neotrix/` 层 — 依赖 rusqlite/reqwest, 违反 core 零依赖规则

## BM25 回退
当 FTS5 返回结果数 < limit 时自动触发:
1. 从 SQLite 读取全部节点
2. 构建 BM25Index (lazy, dirty 标记)
3. 查询 BM25, 去重合并到 FTS5 结果

## 去重
`dedup_nodes()`: 合并无 URL 的重复 title+type 节点, 迁移边后删除
`insert_or_get_node()`: 自动按 title+type 去重 (无 URL 时)

## 守护进程
`neotrix-kb-crawl`: 每 6h 自循环爬取
launchd: `scripts/com.neotrix.kb-crawl.plist`
管理: `scripts/manage-kb-crawl.sh {install|uninstall|status|log}`

## 性能
- FTS5: ~0.16ms/query (短语精确查询)
- BM25: ~0.33ms/query (分词召回查询)  
- BM25 索引构建: ~4ms/400 节点 (lazy, dirty 时触发)
