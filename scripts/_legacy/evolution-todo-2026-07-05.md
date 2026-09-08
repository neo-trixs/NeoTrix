# NeoTrix 自我进化 TODO — 2026-07-05

## 状态总览

| 维度 | 状态 |
|------|------|
| 吸收管道 | PID 51231 运行中 (Cycle 253, ~1h/10h) |
| KB 节点 | 80,247 (24,932 有内容, 55,315 空) |
| KB 边 | 267,877 |
| KB 嵌入 | 0 (需 NEOTRIX_EMBEDDING_API_KEY) |
| 爬取队列 | 0 pending (全部完成) |
| Rust build | ✅ 0 errors, 0 warnings |
| Clippy | ✅ 0 warnings |

## 本轮已修复

### Blockers (P0)
1. **Duplicate URL 黑名单** → 消除 81% 冗余 ERROR (24K/10h)
2. **GitHub 失败跳过** → 3 repos 永久跳过, 不再每 cycle 重试
3. **Log flush 缓冲修复** → `flush=True` 确保实时可见

### Features (P1)
4. **Wikipedia 随机发现** → 固定 58 topics 耗尽后自动使用 Random API
5. **Wikipedia Concept 填充** → 每 cycle 填充 8 个空 Concept/Insight/Article 节点

### 研究吸收
6. **E8/GWT/SAE 7 份论文** → 7 Insight + 7 Concept + 21 edges 注入 KB

### Rust
7. **clippy `needless_borrowed_reference`** → nt_memory_search.rs:195

## 待办清单

### P0 — 阻塞 (需人工介入)
| # | 项 | 说明 |
|---|-----|------|
| 1 | **嵌入生成** | 设 `NEOTRIX_EMBEDDING_API_KEY` 后运行 `kb-generate-embeddings.py` — 24,932 节点待嵌入 |
| 2 | **55K 空节点填充** | 需要外部队列策略: (a) Wikipedia API 批量查询标题, (b) ArXiv 批量吸收, (c) 种子发现 |

### P1 — 高优先级
| # | 项 | 说明 |
|---|-----|------|
| 3 | **Wikipedia Fill 速率控制** | 当前每 cycle 8 个节点填充, 10h = 28,800 空节点可填满 — 需验证 |
| 4 | **GitHub API Token 集成** | 设 `NEOTRIX_GITHUB_TOKEN` 后 rate limit 5000/hr → 可吸收更多仓库 |
| 5 | **爬取队列种子注入** | 55K 空节点中 Insight/Concept/Article 无 URL, 需从 Wikipedia/ArXiv 种子生成 URL |

### P2 — 优化
| # | 项 | 说明 |
|---|-----|------|
| 6 | **多进程管道** | 当前单线程每 cycle ~3s, 改用 `concurrent.futures` 并行 Wikipedia 请求 |
| 7 | **KB 边自动构建** | 新节点入库时自动连接相关节点 (相同 domain/topic) |
| 8 | **E8/GWT/SAE 源码集成** | 将研究 Findings 转化为 Rust 实现: E8GraphResonator, Spectral Seeding, TopK SAE |
| 9 | **元认知自修复** | 检测到 `DUPLICATE_NODE` 后自动删除重复节点而非仅记录 |

### P3 — 长远
| # | 项 | 说明 |
|---|-----|------|
| 10 | **P0 嵌入脚本文档** | 运行 `python3 scripts/kb-generate-embeddings.py` |
| 11 | **cargo clean 后验证** | 每 3 轮自进化循环执行一次 `cargo clean && cargo check --lib` |
| 12 | **桌面端 UI 覆盖** | 34/63 CLI 命令无 Tauri UI (54%) |
