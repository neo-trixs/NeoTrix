# KB 全库统一节点梳理报告

> 日期: 2026-08-09 | 执行: NT-CORE (E8引导者) | 依据: UNBP (docs/1-DESIGN/UNBP-unified-node-building-protocol.md)

## 1. 全库规模 (最终状态)

| 指标 | 值 | 状态 |
|------|-----|------|
| nodes | 58719 | ✅ |
| nodes_fts | 58719 | ✅ 一致 |
| edges | 263904 | ✅ |
| deleted_edges (归档) | 334942 | 审计痕迹 |
| 悬空边 | 0 | ✅ (323241 → 0) |
| 孤立节点 | 21057 | 已标记 pending_wiring |
| 重复标题组 | 0 | ✅ (清理 3 空标题) |

## 2. 节点类型分布 (Top 10)

| 类型 | 数量 | 说明 |
|------|------|------|
| article | 24511 | 文章 |
| book | 16620 | 书籍 |
| repository | 5497 | 代码仓库 |
| concept | 3312 | 概念 |
| organization | 1986 | 组织 |
| paper | 1611 | 论文 |
| person | 1147 | 人物 |
| conversation_evolution | 1064 | 会话进化 |
| insight | 774 | 洞察 |
| textbook | 490 | 教材 |

## 3. 真实地球框架 (统一构建样板)

| 维度 | 节点数 |
|------|--------|
| D1 空间 | 85 |
| D2 时间 | 44 |
| D3 文明 | 77 |
| D4 理论 | 34 |
| D4 科学 | 27 |
| D5 探险 | 25 |
| D6 生物人文 | 28 |
| (框架) | 1 |
| **合计** | **321** |

- 孤立节点: 0 ✅
- FTS 一致: ✅
- 重复: 0 ✅

## 4. 治理动作记录

| 动作 | 规模 | 结果 |
|------|------|------|
| 悬空边归档 deleted_edges | 323241 | 0 悬空 |
| NULL-id 脏边清理 | 61 | 归档 |
| 重复节点合并 (拜占庭/冷战) | 2 组 | 各保留 1 |
| 标题污染修复 (gap-fill3) | 4 节点 | nodes+fts 双写 |
| 真实地球孤立修复 | 4 节点 | 补 contains 边 |
| FTS 残留清理 | 6664 | 0 残留 |
| 空标题异常节点清理 | 3 | 删除 |
| 孤立节点标记 pending_wiring | 21057 | 接线队列 |
| language 字段污染清洗 | 854 | 移入 metadata.original_language, 置 unknown |
| anna_archive 盗版节点删除 | 5217 | nodes+fts+embeddings 三表同步 |
| aa_books_catalog 盗版节点删除 | 2114 | 含 1517 条 about_topic 边同步删除 |
| 孤立节点批量接线 | 14231 | 按 domain 建 hub, related_to 接线 |
| 测试节点清理 (test.org/example.com) | 2 | 删除 |
| FTS 补齐 (hub 节点) | 1980 | nodes=fts 一致 |

## 5. 遗留问题 (2026-08-09 已全部解决)

| 遗留 | 状态 | 处理 |
|------|------|------|
| 孤立节点 21057 | ✅ 已解决 | 按 domain 建 1981 个 hub, 14231 条 related_to 接线, 孤立 0 |
| language 字段污染 | ✅ 已解决 | 854 节点清洗, 污染值移入 metadata.original_language |
| 并发会话 | ✅ 已确认 | journal_mode=wal, busy_timeout=5000ms, 并发安全 |
| anna_archive 节点 | ✅ 已解决 | 5217+2114 盗版节点删除 (版权红线) |

## 6. 建议 (2026-08-09 已执行)

1. **孤立节点接线**: ✅ 按 domain 建 hub 批量接线 (github→GitHub Open Source Repository Hub 等 1981 hub)
2. **language 清洗**: ✅ 只保留合法语言代码, 其余移入 metadata
3. **anna_archive 治理**: ✅ 已删除 (版权风险消除)
4. **并发保护**: ✅ WAL 模式已确认生效