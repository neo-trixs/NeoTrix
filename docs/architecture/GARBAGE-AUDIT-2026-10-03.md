# 蜕皮文件审计（2026-10-03）

> **指针守恒**：本文只记录**判定与判据**，不快照文件内容。
> 判据统一：**是否有活的替代者 + 替代者是否已入库 + 是否被引用**。

## 结论：5 个候选里**只有 1 个是真垃圾**

| 候选 | 大小 | 判定 | 判据 |
|---|---|---|---|
| `neotrix-core/tests/nt_memory_integration.rs.disabled` | 20K | ✅ **删除** | 活件 13 测试/562 行**已入库**(`3f473dcc`)；旧件 13 测试/555 行 ⇒ **测试数相同、活件是超集**；全仓**零引用** |
| `neotrix-core/tests/nt_meta_integration.rs.disabled` | 40K | ⛔ **保留** | **无活件** ⇒ 它是**唯一副本**，属「停用的在途工作」，不是垃圾 |
| `.neotrix/capability_registry.json.bak-legacy` | 12K | ⛔ **保留** | **CLI 迁移路径自己生成的保险**，见下 |
| `models/training/repo_cards.jsonl.bak-20260924-093053` | 348K | ⛔ **保留** | 活件存在但**内容不同** ⇒ 是数据备份 |
| `models/training/smelt/repo_index.json.bak-20260924` | 108K | ⛔ **保留** | 同上 |

## ⛔ 四个「看起来像垃圾但不是」的，最值得记的是 `.bak-legacy`

`capability_registry.json.bak-legacy` 与活件**不是新旧版本，而是两套不同 schema**：

```
legacy 独有: version, last_updated, domains
活件独有   : nodes, edges
```

`crates/nt-core-capability-tree/src/registry.rs` 的注释**自己写明了**它的来历：

> 老 schema 迁移（v1.0.0 domains 形 → nodes 形）……
> 迁移前先落 `.bak-legacy` 保险（**cli 侧**）。

⇒ 它是**迁移前自动落的保险**，不是残留垃圾。
⇒ **教训：`.bak` / `.legacy` 这类后缀只说明「谁生成的」，不说明「是否可以删」。**
判断必须回到**消费方代码**，而不是文件名。

## 机制教训：未跟踪文件的删除**无法用 commit 留痕**

被删的 `.disabled` 是 `??` 未跟踪文件 ⇒ `rm` 之后 git **无任何变更** ⇒
**不存在「删除提交」**，事后无法从历史恢复。
⇒ 因此本次删除另写本文作为**唯一审计线索**，并保留取证哈希：

```
sha256 eb2a14798128abe4f62fc1a06997564ef9c472585afe8b4ff43b4a894411a18a
size   20355 bytes
```

⇒ 通用规则：**删未跟踪文件前先固定 `sha256` + 大小 + mtime**，
否则「删了什么」在事后**无据可查**（本会话已在 `.bak-legacy` 上差点犯）。

## ⛔ 本次**没有**清理的（及原因）

- `target/release/`、`target/debug/deps/`：保留（预编译缓存，删了下次全量重编）
- `.neotrix/knowledge.db`：**可能含真实数据**，无活件、无哈希记录 ⇒ 不动
- `models/` 下权重与归档区：**gitignored，git 保护不到，删了只能重下** ⇒ 不动
- 旧 worktree：**只删生成物，不删带脏文件的 worktree**（须过 `nt_worktree_gate.sh` 双闸）
