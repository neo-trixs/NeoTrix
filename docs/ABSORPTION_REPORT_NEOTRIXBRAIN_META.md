# NeoTrix Brain 64GB 知识库元数据报告

**扫描时间**: 2026-09-18
**数据库路径**: `/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db`
**数据库大小**: 64 GB
**Schema 版本**: 8

---

## 1. 表结构总览 (55 张表)

### 核心知识图谱层

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `nodes` | 知识节点主表 | id, node_type, title, summary, content, url, domain, data_tier, tier |
| `edges` | 节点间关系 | source_id, target_id, relation_type, weight, valid_from/until, superseded_by |
| `deleted_edges` | 已删除边归档 | source_id, target_id, relation_type, deleted_at |

### 嵌入/检索层

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `embeddings` | 向量嵌入 | node_id, vector(BLOB), dimension, model |
| `embeddings_pq` | PQ 压缩嵌入 | (预留) |
| `pq_codebook` | PQ 码本 | m, ks, sub_dim, codewords(BLOB), dimension |
| `vec_meta` | 嵌入元数据 | node_id, content_hash, model, vector(BLOB) |
| `node_strengths` | 节点强度 | storage_strength, retrieval_strength |
| `embedding_queue` | 嵌入任务队列 | node_id, content_hash, status |

### FTS 全文搜索层

| 表名 | 用途 |
|------|------|
| `nodes_fts` | FTS 虚拟表 |
| `nodes_fts_config` | FTS 配置 |
| `nodes_fts_content` | FTS 内容 |
| `nodes_fts_data` | FTS 数据 |
| `nodes_fts_docsize` | FTS 文档大小 |
| `nodes_fts_idx` | FTS 索引 |

### KV 键值存储 (核心知识中枢)

| 表名 | 用途 |
|------|------|
| `kv_store` | namespace + key → value 键值存储 |

**KV 命名空间分布 (Top 20)**:

| 命名空间 | 条目数 | 用途 |
|----------|--------|------|
| `experience` | 75,455 | 经验知识分支 (branch_xxxx) |
| `meta_cognition` | 36,982 | 元认知发现/缺陷 |
| `domain_quality` | 3,010 | 域质量评估 (按域名) |
| `evolution_todo` | 637 | 进化待办事项 |
| `url_inventory` | 603 | URL 清单 |
| `absorption_cycle` | 75 | 吸收周期记录 |
| `absorption` | 70 | 吸收事件 |
| `hyperagent` | 37 | 超级代理配置 |
| `dedup_blacklist` | 32 | 去重黑名单 |
| `brain` | 23 | 大脑元数据 |
| `conversation_distill` | 18 | 对话蒸馏 |
| `crawl_evolution` | 18 | 爬虫进化 |
| `gwt_absorb` | 18 | GWT 注意力吸收 |
| `self_review_blast` | 18 | 自审爆炸 |
| `state` | 16 | 状态 |
| `analysis_cycle` | 14 | 分析周期 |
| `github_skip` | 13 | GitHub 跳过 |
| `panorama` | 13 | 全景 |
| `github_absorb` | 12 | GitHub 吸收 |
| `causal` | 11 | 因果推理 |

**其他命名空间** (各 1-10 条): consciousness, emotion, goals, value_compass, e8_state, cortex, narrative, knowledge_architecture, + 80+ skill 命名空间 (每个 skill 一条记录)

### 文档/文件处理层

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `source_files` | 源文件注册 | original_path, file_name, sha256, mime_type |
| `documents` | 文档元数据 | source_file_id, doc_type, chunk_method, total_chunks, parsing_status |
| `document_chunks` | 文档分块 | document_id, chunk_index, content, token_count, section_title |
| `binary_assets` | 二进制资产 | namespace, name, data(BLOB), mime_type |

### 对话/会话层

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `session_logs` | 会话日志 | session_id, sequence, content, content_type |
| `conversation_records` | 对话记录 | task_description, strategy_used, e8_mode, outcome, effectiveness |
| `conversation_urls` | 对话 URL | url, status, absorbed_nodes/edges |
| `agent_sessions` | 代理会话 | agent_id, label, created_at, ended_at |
| `agent_memory_entries` | 代理记忆 | session_id, agent_id, tier, content, embedding(BLOB) |

### 进化/学习层

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `evolution_records` | 进化记录 | pattern_type, before/after_behavior, effectiveness_gain |
| `evolution_log` | 进化日志 | dimension, event, old/new_value |
| `evo_records` | EVO 记录 | pattern_type, effectiveness_gain |
| `learning_reports` | 学习报告 | report_json |
| `health_reports` | 系统健康报告 | score, grade, error_rate, rss_mb |

### 搜索/发现层

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `search_log` | 搜索日志 | query_text, result_count, duration_ms, match_type |
| `search_frequencies` | 搜索频率 | query_text, count |
| `search_failures` | 搜索失败 | query_text, count |
| `crawl_queue` | 爬取队列 | url, depth, priority, status |
| `discovery_sources` | 发现源 | source_name, last_run_at, total_items |

### 技能/知识管理

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `skills_index` | 技能索引 | name, description, source_path, tags, is_builtin |
| `keyword_library` | 关键词库 | keyword, category, weight, source_urls |
| `procedural_memory` | 程序性记忆 | skill_id, e8_sequence, trigger_pattern, success_rate |
| `temporal_facts` | 时序事实 | subject, predicate, object, valid_from/until, superseded_by |

### 地理信息层

| 表名 | 用途 |
|------|------|
| `geo_index` | 地理索引 (lat, lng, country, region) |
| `geo_elevation` | 海拔数据 |
| `geo_weather` | 天气数据 |

### 基础设施层

| 表名 | 用途 |
|------|------|
| `config_entries` | 配置条目 |
| `secrets` | 加密密钥 (encrypted_value, nonce) |
| `audit_log` | 审计日志 (链式哈希) |
| `cookies` | Cookie 存储 |
| `_migration_log` | 迁移日志 |
| `schema_version` | Schema 版本 (当前: 8) |
| `ingest_log` | 摄取日志 |
| `provenance_links` | 溯源链接 |
| `trace_data` | 追踪数据 |
| `rkyv_blobs` | rkyv 序列化 BLOB |
| `fable_traces` | Fable 追踪 (prompt, response, quality_score) |
| `novel_queue` | 小说队列 |
| `nodes_fts_*` | FTS 索引 (6 张) |

---

## 2. 关系类型 (edges.relation_type)

已发现的关系类型 (含部分扫描中断):

| 关系类型 | 含义 |
|----------|------|
| `Related` | 通用相关 |
| `about_topic` | 关于某主题 |
| `analyzes` | 分析 |
| `belongs_to` | 属于 |
| `brand_for` | 品牌关联 |
| `categorized` | 已分类 |
| `categorized_as` | 分类为 |
| `causes` | 因果 |
| `complementary` | 互补 |
| `contains` | 包含 |
| `contradicts` | 矛盾 |
| `contrasts` | 对比 |
| `contrasts_with` | 对比于 |
| `covers` | 覆盖 |
| `cross_domain` | 跨域 |

> 注: edges 表因数据量巨大 (count 超时)，关系类型扫描因 malformed 错误中断。实际关系类型可能更多。

---

## 3. 知识节点类型 (nodes.node_type)

> 注: nodes 表全量扫描超时 (64GB 数据)，未能获取完整 node_type 列表。

---

## 3.1 文档类型 (documents.doc_type)

> 注: documents 表全量扫描超时。

---

## 4. 关键架构特征

### 4.1 数据分层
- `nodes.data_tier`: `core` (默认)
- `nodes.tier`: `warm` (默认)
- `agent_memory_entries.tier`: `core` (默认)

### 4.2 版本化/时间线
- `edges.valid_from/until` + `superseded_by` → 时序有效边
- `temporal_facts.valid_from/until` + `superseded_by` + `contradicted_by` → 时序事实
- `node_strengths` → 存储/检索强度衰减

### 4.3 多向量嵌入
- `embeddings` → 主嵌入 (text-embedding-3-small)
- `vec_meta` → 替代嵌入 (all-MiniLM-L6-v2)
- `embeddings_pq` + `pq_codebook` → PQ 压缩向量

### 4.4 审计链
- `audit_log`: prev_hash → data_hash → chain_hash 链式完整性

### 4.5 E8 意识架构
- `conversation_records.e8_mode` → E8 模式标记
- `conversation_records.specialist_winner` → 专家路由结果
- `fable_traces.e8_sequence` → E8 执行序列
- `procedural_memory.e8_sequence` → 程序性记忆 E8 序列
- `kv_store consciousness` 命名空间: phi_report, star_identities, router, blind_spots

---

## 5. 约束与已知问题

1. **edges 表 malformed**: `SELECT DISTINCT relation_type FROM edges` 触发 `database disk image is malformed (11)` — 可能存在损坏页
2. **大表扫描超时**: nodes/edges/documents 表因数据量过大，count(*) 和全量扫描超时 (30s+)
3. **Schema 演进**: 当前版本 8，支持时序边、PQ 压缩、审计链等特性

---

## 6. 建议后续操作

| 优先级 | 操作 | 说明 |
|--------|------|------|
| P0 | 检查 edges 表完整性 | malformed 错误需排查 |
| P1 | 使用索引统计 nodes/edges 行数 | `SELECT count(*) FROM nodes WHERE id LIKE 'n_%'` 等 |
| P2 | 导出 kv_store 命名空间完整列表 | 112+ 命名空间待确认 |
| P3 | 采样 experience 分支内容 | 验证 75K 经验条目结构 |
| P4 | 分析 domain_quality 域名覆盖 | 3K 条质量评估待审查 |

---

*报告生成: NeoTrix Knowledge Absorption Agent*
