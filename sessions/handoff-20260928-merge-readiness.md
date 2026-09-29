# MERGE-READINESS — `fix/bitemporal-and-layer-ratchet` → 主干

> 实测于 2026-09-28，分支 @ `905898c2`（20 commits），主干 @ `0b047c6a`。
> 探测方式：`git merge --no-commit --no-ff` 试合并，随后 `merge --abort` 复原
> （**未留任何改动**，工作树已验干净）。

## 1. 结论

| 项 | 结果 |
|---|---|
| 试合并 | **可合，仅 1 个真冲突**（`AGENTS.md`） |
| 冲突文件 | `AGENTS.md`（内容冲突，人工裁决） |
| `Auto-merging` 静默区 | 2 个 `.rs` + 基线文件，**已逐一核实安全**（见 §3） |
| 门基线 | 合并前后均 **8 known / PASS 0 new** |

## 2. 唯一冲突：AGENTS.md

两边都在同一文件的「三道闸」区域加了内容（我加分层门 8 known 记录，
主干加 R-DISK-1~4 磁盘回收规约与死引用清零）。

**裁决建议**：**两侧都保留**，按小节归位，不删任何一方。
我方那段的标题是「目录/命名门」下的分层门条目；主干那两段是独立的
回收规约条目。冲突标记应只覆盖同一小节内相邻的列表项。

合并后**必须**重跑并核对（见 §4）。

## 3. 静默 `Auto-merging` 已核实（这是 L8 说的高发区）

| 文件 | 我改了什么 | 主干改了什么 | 同位置？ | 判定 |
|---|---|---|---|---|
| `l5_cognition/nt_core_cad_consciousness.rs` | 1 行改道 | 2 行改道 | 同文件小范围 | 人工复核合并结果 |
| `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | 1 行（`scheduler` 字段类型改道） | 19+3 行，全部是**新增注释块 + 资产路径 bugfix** | **否**（我改类型行，主干加注释） | ✅ 安全 |
| `scripts/layer-deps-baseline.txt` | 逐批下调至 8 | 未动 | — | ✅ **未被合并触碰**，无静默降级 |

> 三个 .rs 的 auto-merge 后仍必须靠 `cargo check` 兜底 —— 文本不冲突
> **不等于**语义不冲突（这正是 L15 要防的：换错类型 `cargo check` 也不报错）。

## 4. 合并后验证清单（照做，别跳）

```bash
cd .worktrees/ratchet                      # 或新开干净 worktree
sh scripts/ops/nt_mem_gate.sh              # 非 0 禁止起构建
export CARGO_BUILD_JOBS=2
cargo check --tests -p neotrix             # 期望 exit=0
cargo test -p neotrix --lib -- --test-threads=4
#   期望 12154 passed / 0 failed / 39 ignored
cargo check -p neotrix --features ios-bridge   # 期望 exit=0（第二棵树专用门）
bash scripts/check-layer-deps.sh --strict       # 期望 8/8 PASS 0 new
bash scripts/check-naming.sh                    # advisory，基线 1,646
```

任一项不符 ⇒ **回滚合并，不要留下半合状态**。

## 5. 合并姿势建议

推荐**在干净 worktree 里合**（主工作树已被他窗 WIP 占用）：

```bash
git worktree add --detach .worktrees/merge-test HEAD
cd .worktrees/merge-test
git merge --no-ff fix/bitemporal-and-layer-ratchet
# 解 AGENTS.md 冲突 → 跑 §4 全套 → 通过再在主干上落 merge commit
```

## 6. 为什么我没有直接合

- 用户尚未拍板合入时机（主干 48 commits 前移，涉及他窗大量重构）；
- 主干上**有他窗未提交 WIP**（`nt_core_capability_tree` 断链），
  在主树合会把「他窗的破」与「我的合并」混在一个 index 里，
  正是 L7-b 记录的那类事故（共享 index 竞争 / 卷走他人暂存项）。
