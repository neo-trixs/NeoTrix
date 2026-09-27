# 经验沉淀 — 2026-09-27：扫描器告警、门记录腐化、并发写入

> 配套：`RUST-STANDARDS.md` §17（已补 R-LOCK-4/5、R-BUILD-6、R-GIT-5）、`AGENTS.md`（已补 R-SCAN-1/2/3）
> 本文是**推理链**留档，规范条目已上收进 §17 / AGENTS.md，不在此重复。

---

## 1. 差点把 bug 修进正确代码

`nt_lock_audit.py` 报了 12 条"间接自死锁"，其中包括：

```
tor_client.rs:324     持有 self.process 守卫期间调用 self.start()
llama_process.rs:329  持有 self.config  守卫期间调用 self.spawn_watchdog()
```

逐个读现场：

- `tor_client.rs:321-324` —— 作者**显式写了 `drop(proc);`** 才调 `self.start()`。教科书式正确代码。
- `llama_process.rs:324` —— `*self.config.lock().await = cfg.clone();`。赋值语句的临时守卫，`;` 处即释放，`:329` 调用时**没有**持有 `config`。

**若照单全修，等于把两处正确代码改坏。**

根因：`audit_indirect` 只做「函数体内同时出现 `self.X.lock()` 与对也会锁 X 的方法的调用」，**没有作用域分析**。而同一文件里的 `audit_text`（直接扫描）**有** `DROP_RE` —— 两条路径的能力不对等。

### 教训

**「无定点不改」这条 `nt-locate` 规矩，对静态扫描器同样成立。** 扫描器给出的 `file:line` 是**线索**，不是**结论**。工具的确定性给人虚假的确感 —— 但词法扫描器对语义问题（作用域、临时值、显式 drop）天生无能。

## 2. 手推 ≠ 实证

我手工推演过 `audit_indirect` 的括号深度算法，结论是「health.rs 那个 `if let` 守卫块在 `:89` 就闭合了，不该误报」。

**推演是对的，工具是错的** —— 但我是先推演、再实测，才没把结论下反。真正救了我的是**把真实代码形态喂进去跑**：

```python
# 三个真实形态，各喂一次
形态1 health.rs if-let      → 0 条误报
形态2 llama async .lock().await → 1 条误报   ← 真 bug
形态3 多行签名              → 0 条误报
```

推演只覆盖了形态 1，而**真实世界的 bug 恰好在形态 2**。不实测就会既高估又低估。

## 3. 门记录会腐化，且比没有门更危险

`AGENTS.md` 和 `RUST-STANDARDS.md §17.1` 都长期写着死锁扫描 **「当前 0 命中」**。

实际是 12 条 → 修到 3 条，其中 1 条真死锁。

**「0 命中」这个记录的危害是双向的**：

- 下一个 agent 看到「0 命中」→ 认为门已通过 → 放行
- 或者反过来：任务派下来要「清死锁」，agent 按图索骥去修那 2 条**正确代码**

**陈旧的门记录比没有门更危险**，因为它给了错误的安全感，且会被主动引用去指导动作。已加 R-SCAN-3：门记录必须带核实时间戳。

## 4. 真死锁的形态：早退路径上的委托

唯一那条真告警：

```rust
// kb_search.rs:545-549
let conn = self.conn.lock().map_err(...)?;   // std::sync::Mutex，存活到函数尾
let hits = pq_ann_search(&conn, ...)?;
if hits.is_empty() {
    return self.semantic_search(query, limit);  // ← semantic_search 首行就 self.conn.lock()
}
```

`self.conn` 是 `Mutex<Connection>`（`mod.rs:186`），**标准库 `Mutex` 不可重入**。`pq_search` 持锁 → 委托 `semantic_search` → 再取同一把锁 → 永久阻塞。

**这不是「同块二次获取」，是「持锁函数调用了会加同一把锁的方法」**。R-LOCK-2 说的「守卫不得活到函数尾」在这里是**结构性**的：守卫必须活到函数尾，因为 `:553` 的循环还要用它。唯一出路是**在早退分支上显式 `drop(conn)`**。

已立为 **R-LOCK-4**。同族另修了 2 处（`:334` / `:363`），三处形态一致。

## 5. `assert` 是并发环境下最便宜的幂等守卫

我改完 `kb_search.rs` 之后，另一窗口正好在跑一个 python 脚本改**同一个文件、同一处逻辑**。它的写法是：

```python
o = """        if hits.is_empty() {
            return self.semantic_search(query, limit);
        }"""
n = """        ... drop(conn); ..."""
assert s.count(o) == 1, "pq_search early-return not found"
open(p, 'w').write(s.replace(o, n))
```

因为我已经加了注释和 `drop(conn);`，`o` 匹配不上 → `assert` 失败 → **它没有覆盖我的修改**。

**反过来验证了一件事：无人值守的批量改写脚本应当一律先 `assert` 再写。** 成本一行，收益是"并发编辑不会静默互相吞掉"。已写进 R-GIT-5。

## 6. 门是共享的，别人也会把你的闸拉黑

`nt_mem_gate.sh` 报 BLOCKED（free_pages 7820 / 阈值 100000），我第一反应是"我什么都没起"。

`ps` 显示：**另一窗口正在跑 `cargo check --tests -p neotrix -j2`**，两个 `rustc` 共 5.2G。而 `--tests` 正是 `AGENTS.md` 明文禁止的重型档位。

- 正确处置：**不 kill、不 join**，通报、等它跑完释放内存
- 已立为 **R-BUILD-6**：闸 BLOCKED 先 `ps` 确认**是不是自己**起的

## 7. 一个方法论观察：接线远多于新建

为 18 项进化路线做定点时，有 5 处判断被代码推翻，全部朝**更有利**的方向：

| 原判断 | 实际 |
|---|---|
| `maturity_audit` 是"已建未引用" | **已完整实现 + 自愈降级**（`registry.rs:484-485` 会自动下调声称等级），缺的只是 CI 不调它 |
| 脱 stub 需重设计响应 | **契约已预留** —— `stub.rs:292-303` 已有 `consciousness_state{phi,coherence,gwt_resonance}` + `confidence`，与 L5 意识核类型级吻合 |
| 决策引擎仅 "inspired by JEV" | **三原语已忠实建模** + 已有 `Usage`（`types.rs:304`） |
| 成本归因需新建管道 | **唯一插点已存在** —— `anthropic.rs:92,237` 的 prefix-caching 断点就是那段"请求时组装、从不落 transcript"的不可见前缀 |
| 能力 registry 需建 manifest | manifest **形状已对**（含 `deprecated`/`deprecated_reason`），只缺安全信封 |

**启示**：评估"要做多少工"时，先读代码再报数。本轮我差点按"要新建"报预算，实际大部分是接线。**这直接影响路线排序** —— 阶段 0 之所以能压到 1-2 天，正因为它几乎全是接线与删除。

---

## 未决（留给下一棒）

1. **`nt_lock_audit.py` 的 `audit_indirect` 仍缺两条规则**：不认 `drop()`、不认赋值型临时锁。当前误报率 2/3。修法已知（把 `audit_text` 的 `DROP_RE` 思路补过去 + 识别 `*self.x.lock() =` 右值），但**该文件正在被他人编辑，未动**。
2. **`kb_search.rs:552` 的 `drop(conn)` 尚未经 cargo 验证** —— 他窗的 `cargo check --tests -p neotrix` 会编译同一 crate，可顺带验掉。
3. **`ARCHITECTURE.md:97` 把 `nt_computer/` 宣传为"计算集群"**，实测是 filesystem/process trait，GUI 驱动完全不存在。文档与代码的偏差仍在扩大。
