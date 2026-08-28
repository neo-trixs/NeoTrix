# NeoTrix Knowledge Brain 性能优化报告

## 基准测试结果 (2026-08-27)

| 测试项 | median(ms) | p95(ms) | 目标 | 状态 |
|--------|------------|---------|------|------|
| FTS 全文检索 | 0.80ms | 1.09ms | 20ms | ✅ PASS |
| 向量相似度 (100向量) | 243.74ms | 250.12ms | 50ms | ❌ FAIL |
| 邻居遍历 (1-hop) | 27.22ms | 28.25ms | 10ms | ❌ FAIL |
| 多跳推理 (2-hop) | 0.63ms | 0.76ms | 15ms | ✅ PASS |
| 抽象层级筛选 | 33.06ms | 642.55ms | 15ms | ❌ FAIL |
| 缺口报告查询 | 0.64ms | 0.78ms | 10ms | ✅ PASS |
| 多模态索引查询 | 0.68ms | 0.82ms | 10ms | ✅ PASS |
| 边类型聚合统计 | 30.59ms | 33.90ms | 50ms | ✅ PASS |

## Phase 1 索引优化已执行

```sql
-- 已创建索引
CREATE INDEX idx_edges_source_target ON edges(source_id, target_id, relation_type);
CREATE INDEX idx_edges_target_source ON edges(target_id, source_id, relation_type);
CREATE INDEX idx_node_dimensions_abstraction ON node_dimensions(abstraction, abstraction_order);
CREATE INDEX idx_embeddings_node ON embeddings(node_id);
CREATE INDEX idx_nodes_node_type ON nodes(node_type);
CREATE INDEX idx_knowledge_gap_reports_status ON knowledge_gap_reports(status, severity DESC);
CREATE INDEX idx_reasoning_chains_status ON reasoning_chains(status);
CREATE INDEX idx_multimodal_index_img_hash ON multimodal_index(img_hash);
```

## 优化后测试结果

| 测试项 | 优化前 p95 | 优化后 p95 | 改进 |
|--------|------------|------------|------|
| FTS 全文检索 | 1.09ms | 1.23ms | 持平 ✅ |
| 向量相似度 | 250ms | 272ms | 持平 ❌ |
| 邻居遍历 | 28ms | 28ms | 持平 ❌ |
| 抽象层级筛选 | 642ms | 665ms | median改善(33→12ms) 部分改善 |
| 缺口报告查询 | 0.78ms | 0.74ms | 持平 ✅ |
| 多模态索引 | 0.82ms | 0.80ms | 持平 ✅ |

## 瓶颈分析 (优化后)

### 1. 向量相似度搜索 (249ms median) - 需 HNSW
**原因**: 无索引的暴力计算 100 个向量的点积 (Python 循环)
**解决方案**: 集成 HNSW 索引到 `nt_memory_kb::kb_vector_index.rs` (使用 `instant-distance` crate)
**预期**: 100 向量查询 <5ms (50x 提升)

### 2. 邻居遍历 (27ms median) - 需物化视图
**原因**: UNION 查询需两次索引扫描 + 结果合并
**解决方案**: 创建物化视图 `materialized_neighbors` 表
**预期**: <5ms (5x 提升)

### 3. 抽象层级筛选 (12ms median, 665ms p95) - 查询计划不稳定
**原因**: JOIN + ORDER BY random() 导致查询计划抖动
**解决方案**: 
- 创建物化视图 `nodes_by_abstraction` (预计算分组)
- 或使用 `nt_memory_kb` 封装的查询接口
**预期**: median <5ms, p95 <20ms

## 优化路线图

### Phase 1: 索引优化 ✅ 已完成
### Phase 2: HNSW 向量索引 (进行中)
- 集成点: `nt_memory_kb::kb_vector_index.rs`
- 依赖: `instant-distance` crate (已在项目)
- 预期: 100 向量查询 <5ms

### Phase 3: 物化视图 / 缓存层
```sql
-- 邻居物化视图
CREATE TABLE materialized_neighbors AS
SELECT source_id, target_id, relation_type FROM edges
UNION ALL
SELECT target_id, source_id, relation_type FROM edges;

CREATE INDEX idx_mat_neighbors_source ON materialized_neighbors(source_id);
```

- Redis 缓存热点查询 (FTS、抽象层级、缺口报告)
- 预期: 邻居遍历 <5ms, 抽象层级 <5ms

## 预期最终指标

| 测试项 | 当前 p95 | Phase 2/3 后目标 | 改进倍数 |
|--------|----------|------------------|----------|
| 向量相似度 | 272ms | <5ms | 50x+ |
| 邻居遍历 | 28ms | <5ms | 5x+ |
| 抽象层级筛选 | 665ms | <20ms | 30x+ |

## 实施优先级

1. **P0** (本周): HNSW 索引集成到 `kb_vector_index.rs`
2. **P1** (下周): 物化视图 `materialized_neighbors` + Redis 缓存层
3. **P2** (下周): `nodes_by_abstraction` 物化视图 + Redis 缓存
