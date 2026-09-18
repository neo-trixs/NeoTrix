# NeoTrixBrain 知识合并报告

**日期**: 2026-09-18
**源数据库**: `/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db` (64GB)
**目标**: `~/.neotrix/knowledge.db`

## 合并统计

| 类别 | 记录数 |
|------|--------|
| kv_store | 4,977 |
| session_log | 1,236 |
| evolution_snapshot | 9 |
| meta_cognition | 7 |
| dgm_evolution | 7 |
| external_kb (索引) | 3 |
| **总计** | **6,239** |

## 源数据库概况

- **kv_store 总量**: 117,264 条
- **合并比例**: ~5.3% (按类别筛选重要条目)
- **其他表**: session_logs, crawl_queue, kb_nodes, kb_edges, embeddings, agents 等

## 数据分类

### kv_store (4,977)
- `snapshot_*` — GWT 进化快照 (reward/coherence/autonomy)
- `cycle_report_*` — 爬取周期报告 (nodes/edges/failures)
- `nt-*` — 爬取阶段状态
- `meta_proposal_*` — 元认知提案
- `dgm_proposal_*` — DGM 进化提案
- `path`, `chrome-profile` — 配置项

### session_log (1,236)
- NeoTrixBrain 会话内容 (markdown)
- 时间范围: 2026-07 ~ 2026-08

### 索引条目 (3)
1. **ntbrain_index** — 数据库结构索引
2. **ntbrain_cortex** — cortex-archive 离线知识
3. **ntbrain_medical** — 医疗知识图谱

## 查询示例

```sql
-- 查看所有进化快照
SELECT title, insight FROM experience 
WHERE category = 'evolution_snapshot' ORDER BY timestamp;

-- 查看爬取周期
SELECT title, insight FROM experience 
WHERE category = 'crawl_cycle' ORDER BY timestamp DESC LIMIT 10;

-- 搜索特定关键词
SELECT title, insight FROM experience 
WHERE insight LIKE '%HyperCube%' OR title LIKE '%HyperCube%';
```

## 后续行动

- [ ] 定期同步增量数据
- [ ] 建立跨 KB 查询接口
- [ ] 将关键洞察接入 experience-tree 吸收流程
