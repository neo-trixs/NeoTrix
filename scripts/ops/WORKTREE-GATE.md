# nt_worktree_gate.sh —— worktree 门

多窗口并行时的 worktree 约束与磁盘体检。**不代替判断，只强制证据。**

## 为什么存在

2026-09-28 实测事故（`RUST-STANDARDS.md` §17.7 R-DISK-1~7）：

- 22 个 worktree 占 **28G**，其中 **16.6G 是 `target/` 编译产物**，真实工作区每个 < 1M
- 15 个 worktree 合计 **850 处未提交改动**，**不在任何提交里** ——
  `git branch -a --contains` 判不出来，删目录即永久丢失（单个曾有 758 处）
- 因只看"已合入分支"就删，我误删过 `ratchet`（4 处脏文件），靠 patch 兜底才恢复

## 用法

```sh
sh scripts/ops/nt_worktree_gate.sh check              # 体检 + 分类（只读）
sh scripts/ops/nt_worktree_gate.sh clean              # 清 target（零风险）
sh scripts/ops/nt_worktree_gate.sh prune              # 干跑：报告哪些可删
sh scripts/ops/nt_worktree_gate.sh prune --force      # 实删（须双闸 + 兜底）
```

## 退出码

| 码 | 含义 | 该做什么 |
|---|---|---|
| 0 | 无事 | — |
| 2 | 有 cargo 在跑（仅 `clean` 模式） | 让位（`R-DISK-4`） |
| 3 | **仅** target 累计超阈值 | `clean` 零风险回收 |
| 4 | 有 worktree 带未提交改动（**含**体积超阈值时也报此码） | `prune` 会先兜底，**别手删** |

### ⭐ 两条判据**同时**成立时，「脏」优先（2026-10-01 修正）

⛔ 旧实现是「体积 ⇒ `exit 3`」**先命中并 return**，脏判据的 `exit 4`
**永远到不了** ⇒ 退出码**只反映体积**。
实测这不是理论问题：2 个脏 worktree、6 处未提交改动时 rc=3，
而 `.githooks/pre-push` 的 `case` 里 `3)` 只打印建议、`LEAK=0`
⇒ **pre-push 当时正在放行**这些改动。掩盖 bug 是**活的**。

现在：**两条判据都会打印**（体积信息不丢），退出码取**更危险的**那个。
⚠️ **行为变化**：两者同时成立时 rc 由 3 变 4 ⇒ pre-push **开始真正拦截**。
这是修复，但属「突然变严」—— 首次遇到请先读本节。

### 主树**只报告、不阻断**

`check` 会打印**主树**的未提交处数与 target 体积，但**不计入**退出码。
⛔ 刻意不纳入阻断：实测主树长期有十几处未提交与上百 GB target，
纳入后退出码会**永久**非 0 ⇒ `.githooks/pre-push` 会在**几乎每次 push**
上硬失败 ⇒ 把诊断改进变成流程阻断。**那些改动该怎么处理由人决定**，
不该由门替人决定。

### ⚠️ `.githooks/pre-push` 与本表是**强耦合**

`pre-push` 按**具体码值** `case`，且必须保留 `*)` **fail-closed 兜底** ——
否则门一改判据，未知码会**静默落空**，脏 worktree 拦截**无声消失**。
改本表时**必须**同步改 hook。

## 三条硬约束（脚本已内建，人工也必须遵守）

1. **`prune` 双闸**：HEAD 须含于某分支；**近 3h 无 `.rs` 改动**这一条
   **仅在该 worktree 本身有脏文件时才要求**（2026-09-29 修正，代码已如此）。
   ⓘ 本文档此前写的是「**且** 近 3h 无 `.rs` 改动」⇒ **与代码漂移**，已改。
   理由：mtime 判据对**干净** worktree 是误报（别人在主树写 `.rs` 会污染
   它的时间戳直觉），去掉误报让门更准确，不是放松。
   "已合入分支" ≠ "可删" —— 未提交改动不在任何提交里。
2. **`prune` 兜底**：有脏文件时先导出 patch + 未跟踪清单，并用
   `git apply --check --reverse` 校验可回放，**校验不过就拒绝删**。
3. **判据同向**：任一判据指向"有工作"即放弃。R-DISK-7 的教训 ——
  本轮我曾把 `lsof +D` 无输出误读成"无进程占用"，而它对近 3k 文件的目录
  会超时无输出；同时 mtime 判据写死了时间窗。**两者矛盾时不得下结论。**

## 兜底档

`prune` 自动写到 `.neotrix/worktree-salvage/`（可用 `NT_WT_SALVAGE` 改路径）：

```
<name>.patch            git diff（已跟踪文件）
<name>.untracked.txt    status --porcelain | grep '^??'（patch 不含未跟踪内容）
```

回放：`git -C <worktree> apply --3way <name>.patch`

## 环境变量

| 变量 | 默认 | 用途 |
|---|---|---|
| `NT_WT_TARGET_MB` | 1024 | target 累计超此值（MB）即提示可回收 |
| `NT_WT_SALVAGE` | `.neotrix/worktree-salvage` | 兜底 patch 输出目录 |
