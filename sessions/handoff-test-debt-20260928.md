# Handoff — 剩余三笔债（含精确诊断，2026-09-28）

> 承接自「单元测试 57 → 3」那一轮。3 条测试失败 + 92 处分层违规，
> 都不是小修，需要产品判断或阶段级工程，故在此留下**已验证的诊断**而非猜测。

## 1. 会话标识

- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-09-28
- 落点 commit：`0d92477a`（测试账本已棘轮到 3 条，无 flaky）

## 2. 三笔债的现状

### 债 A · `nodes` 表无法存双时间历史（2 条测试）

**失败测试**
```
l0_substrate::nt_core_kb_primitives::tests::test_node_history
l0_substrate::nt_core_kb_primitives::tests::test_nodes_as_of_returns_committed
        → UNIQUE constraint failed: nodes.id
```

**已验证的根因**（`neotrix-core/src/l0_substrate/nt_core_kb_primitives.rs:188`）

```sql
id TEXT PRIMARY KEY,     -- ← 主键不含时间维
```

- `nodes_as_of()`（:685）与 `node_history()`（:834）**本来就按 `transaction_time`
  过滤与排序，并期望同 `id` 返回多行** —— 查询函数是对的，schema 与之矛盾。
- 测试插入同 `id` 的 3 个版本（v1@tx100 / v2@tx200 / v3@tx300）⇒ 第二个就撞 UNIQUE。
- **结论：schema 的 bug，测试表达的意图是正确的。**

**我试过改成复合主键 `(id, transaction_time)`，又暴露两个更深的问题，已回退**
（不做半迁移的 schema）：

1. `edges.source_id/target_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE`
   （同文件 :248-249）是**真外键**。SQLite 要求被引用列是唯一键，复合主键下该外键失效
   ⇒ 报 `foreign key mismatch - "edges" referencing "nodes"`。
2. `nodes_as_of()` 的文档注释明写「latest version visible at that time」，
   **实现却返回 `transaction_time <= as_of` 的全部版本**（`ORDER BY id, transaction_time DESC`
   但没有「每个 id 只取最新」的逻辑）⇒ 实测 `left: 2, right: 1`。
   复合主键让多版本共存后，这个既有缺陷立刻暴露。

**完整修复所需**（= `TODO.md` 2.2「真双时间迁移」）
- [ ] `nodes` 主键改 `(id, transaction_time)`
- [ ] `edges` 外键改为引用 `(id, transaction_time)`（注意 edges 自身也有 `transaction_time`，
      外键语义要想清楚：一条边应绑定到节点的哪个版本？）
- [ ] 修 `nodes_as_of()` 使其真正取「每个 id 在 as_of 时刻的最新版本」
- [ ] **既有 DB 的数据迁移**：`CREATE TABLE IF NOT EXISTS` 不会改已存在的表，
      线上库仍是单列主键 ⇒ 需要真实迁移脚本，且要处理已存的历史数据
- [ ] 复核 `exp_absorb.rs` 的 `UPDATE nodes SET metadata=? WHERE id=?`
      （复合主键下会更新该 id 的**所有**版本 —— 对 metadata 良性，但需确认）

### 债 B · Noise 握手 responder 无法计算 `es`（1 条测试）

**失败测试**
```
l3_embodiment::nt_shield/nt_shield_ztnet/crypto/noise_handshake.rs::tests::full_handshake
        → InvalidState（_create_message2）
```

**已验证的根因**（`noise_handshake.rs:190-194`）

```rust
let es = ephemeral.diffie_hellman(
    self.remote_static_public.as_ref().ok_or(_NoiseError::InvalidState)?,
);
```

responder 执行 `_create_message2` 时要算 `es = DH(自己的 ephemeral, 对端 static)`，
但按 Noise **IK** 模式，responder **根本不知道 initiator 的静态密钥** ——
它正是被加密在 message 2 里（`enc_static`，:186）才送达的。
所以 `self.remote_static_public` 对 responder 恒为 `None` ⇒ `InvalidState`。

> 顺带：该文件 :71-80 的协议名注释已自陈
> 「协议名与 Noise spec 的 `Noise_IKpsk2_25519_ChaChaPoly_SHA256`(39 字节) 不一致」，
> 说明这块实现**从未与 spec 对齐验证过**。

**建议路径**（属密码学实现，不宜顺手改）
- [ ] 先决定：本模块是要真对齐 Noise IK spec，还是明确降级为「自有简化握手」并改名/改注释
- [ ] 若对齐：`es` 应由 **initiator** 在 `_consume_message2` 时用**自己的** static 计算；
      responder 侧不计算 `es`。`ee`/`s` 的处理也需按 spec 复核
- [ ] 该模块目前**无生产调用方**（`TODO.md` §六 已记录 JEV/决策面未接线），
      故可安全改；但也意味着改完仍需自证（见下）
- [ ] 建议补「握手对称性」测试：双方各自算出的 chaining key / symmetric key 必须相等
      （当前测试只走 initiator 一侧，验不到这类不对称）

### 债 C · 分层依赖违规 92 处

`bash scripts/check-layer-deps.sh --strict` → `PASS: 0 new violation(s); 92 known/recorded`
（账本在 `scripts/layer-deps-baseline.txt`，已接 CI，`--strict` 拦新增）

11 个类别，实测分布：
```
L1→L2  L1→L3  L1→L4  L1→L5  L1→L6
L2→L3  L2→L4  L2→L5
L3→L4  L3→L5  L3→L6
L4→L5  L5→L6
```

**棘轮已就位**（`--update-baseline` 只应向下棘轮），所以每修一处都能看见数字下降。
**不要**为了数字好看而整体解禁或把基线调大。

**建议的切法**（按文件聚类，不按类别硬啃）
1. 先挑**叶子违规**：单个文件只违反 1 条规则的先修，改动可控
2. 优先修**有 facade 替代路径**的：门已排除 `!*facade*` / `!*l1_facade*` / `!traits.rs`，
   说明这三类是官方认可的跨层通道 —— 很多违规可能只需把引用改走 facade
3. `neotrix-core/src/l1_action/nt_io/nt_io_web/api.rs` 违规密度较高（7+），
   但该文件正被并发编辑，需先确认归属再动

## 3. 本轮的方法论留痕（比结论更耐用）

- **干净检出是唯一可信的地面真相。** 主工作树因含他人未提交修复而「看起来正常」；
  本轮 57 条测试失败里 **37 条在工作树里早已修好、只是从未入库**。
- **实测优先于推断。** 「工作树能编译」≠「可独立提交」：拷入 `entry/mod.rs` 会拉进
  `dialog.rs`，后者需要 HEAD 没有的 `neotrix-neobot` 依赖，错误数从 2 **反弹到 58**。
- **半迁移比不迁移更糟。** 债 A 改到一半暴露出外键与查询的更深问题 ⇒ 果断回退并记录。
- **flaky 的根因往往不是环境。** 两条 flaky 分别是「随机选 profile」
  （1/4 概率同）与「HashMap 迭代顺序随机 + 得分并列」—— 都是**按构造就会随机失败**。
  加重试或放宽阈值都没用，消掉不确定性才有用。
- **测试若断言一个不存在的能力，它比没有测试更坏。** 本轮修掉的「坏测试」有 5 处：
  `publish()` 两分支都返回 `success:false` 却断言 `success`；
  注释说「趋势内」却传 150（序列下一���恰是 115）；注释说「注入轻微方差」但 std≈0.45；
  断言依赖宿主机器健康度；自检期望与实现的默认值矛盾。
- **加固校验器 > 放宽断言。** `CadWiringEvidenceSelfTest` 原本只查「非空且含冒号」，
  于是**注释里的冒号**骗过了它；加固为「文件真实存在」后立刻抓出 4 条假证据。

## 4. 纪律提醒

- 全仓 6+ 处测试用 `std::env::set_var("HOME", ...)` 改**进程全局**环境变量，
  与读 `HOME` 的测试竞态（已修好 `skill_loader` / `checkpoint` 两处受害测试，
  但根因未除）。正解是给所有 HOME 改写点加一把共享 `Mutex`
  （`agent.rs:506` 的 `TEST_MCP_SERIAL` 就是同类先例）——
  当时因 `cipher.rs` 属他窗在制品而未做，**这是一个已知的未完项**。
- `apps/neobot-desktop/frontend/` 23 个文件 + `gen/` 4 个仍未入库（归属裁决，
  见 `handoff-disease-list-20260927.md:103`）。`cargo check` 不需要它们，
  但 `tauri build` 会。
- 本轮全程未碰他窗在制品：957 个脏文件与 17 个 worktree 原样保留。
