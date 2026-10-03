# 涌现接线：已定位的 panic 缺陷（2026-10-03）

> 配套 `EMERGENCE-ROADMAP-2026-10-03.md` 附录 A。
> 本文**只做定位与补丁形状**，**不含未验证的改动** —— 改签名需同时动
> 5 个生产者 + 5 个调用点 + 闭包返回类型，未跑完 `--tests` 故不落盘。

## 一、先更正我自己的两个假阴性（本轮实测）

我在核对附录 A 时连错两次，**都由搜索范围过窄造成**：

| 我的断言 | 实测 | 根因 |
|---|---|---|
| 「`trade_product_spec` **没有生产者**」 | ⛔ 错。生产者在 `l4_emotion/nt_memory/nt_trade_product_spec.rs:949` | 我只 `rg` 了 `l1_action/`，而根节点住在 `l4_emotion/` |
| 「接线**还没做**」 | ⛔ 错。`consciousness_runtime.rs:953-1020` **已按拓扑序接好**，并带 `must_be_registered` 断言 | 我按函数名搜调用方，只看到 `use` 导入行就下了结论 |

⇒ **纪律**：跨层搜索必须覆盖**全部候选层**（本仓根节点跨 `l1_action` / `l4_emotion` 两层），
否则「找不到」会被误读成「不存在」。

## 二、⛔ 真缺陷：5 个能力节点生产者在生产代码里 `.expect()`

| 文件:行 | 现状 |
|---|---|
| `l4_emotion/nt_memory/nt_trade_product_spec.rs:955` | `.expect("Failed to register product_spec capability")` |
| `l1_action/nt_act/nt_act_trade/quote_negotiation.rs:298` | `.expect("Failed to register quote_negotiation capability")` |
| `l1_action/nt_act/nt_act_trade/production_logistics.rs` | `.expect("Failed to register …")` |
| `l1_action/nt_act/nt_act_trade/finance_compliance.rs` | `.expect("Failed to register …")` |
| `l1_action/nt_act/nt_act_trade/full_cycle.rs` | `.expect("Failed to register …")` |

**这不是理论风险** —— `consciousness_runtime.rs:1073` 自己记录了它**已经发生过一次**：

```
panicked at nt_trade_product_spec.rs:955:
Failed to register product_spec capability: AlreadyExists("NT-MEMORY::trade::trade_product_spec")
```

⛔ 违反本仓硬规则「生产代码禁 `unwrap`/`expect`/`panic!`」。

## 三、⚠️ 更正附录 A.4 第 4 点：「拓扑序错了会**静默失败**」是**错的**

`CapabilityTreeRegistry::register`（`registry.rs:168`）的 `Err` 面恰好 **2 种**：

```
:170  Err(RegistryError::AlreadyExists(node.id))
:205  Err(RegistryError::CircularDependency(node.id, "circular".into()))
```

⇒ 顺序错 ⇒ `AlreadyExists` ⇒ 现有 `.expect()` ⇒ **直接 panic**，
**不是**附录 A.4.4 所说的「幂等成功、静默失败」。
⇒ 该点描述会让人以为「最坏情况只是不生效」，因而**低估接线风险**。

## 四、为什么不能只把 `AlreadyExists` 当幂等成功

若把 `AlreadyExists` 静默吞掉，剩下 `CircularDependency` 只能 `eprintln!` 放过去
⇒ 那是**吞错误**，而 `CircularDependency` 恰恰是**真 bug 信号**（依赖成环）。
⇒ 故**必须改签名**才能完整处理：

```rust
// 5 个生产者统一改为
pub fn register_x_capability(
    registry: &mut CapabilityTreeRegistry,
) -> Result<CapabilityNode, RegistryError> {
    let node = /* 原构造不变 */;
    registry.register(node.clone())?;      // ⛔ 去掉 .expect()
    Ok(node)
}
```

调用点（`consciousness_runtime.rs:994/1001/1008/1013/1020`）改用 `?`。
⚠️ **连带要求**：`:985` 的闭包当前标注 `-> (usize, usize)`，
加了 `?` 之后**必须**改成 `-> Result<(usize, usize), _>`，
并确认 `with_registry` 的错误类型与 `RegistryError` 的合并方式（`?` 需要 `From`）。
⇒ **这是该补丁唯一的隐藏工作量**，也是本轮不落盘的原因。

## 五、另两处同区域弱点（同批可查）

1. **check-then-act 竞态**：调用点是 `already(reg, ID)` 判空后再 `register()`。
   共享注册表下两者之间存在窗口 ⇒ 窗口内被别的路径注册 ⇒ 触发上面的 panic。
   ⇒ 去掉 `.expect()` 后这变成**可处理的错误**而非 panic，是本补丁的附带收益。
2. ⛔ **`debug_assert!` 在 release 被编译掉**（`:995` 等 5 处）
   ⇒ release 构建下「注册了但没生效」**无人发现**。
   ⇒ 与 `:1029` 的 `0 < newly < 5` 运行时检查不同，那条只覆盖「部分成功」，
   **不覆盖「全部 debug_assert 被编译掉」的情形**。

## 六、并发与可动性（已核）

上述 6 个文件在核对时**全部 clean**（无未提交改动），mtime 停在 9/28–10/3，
⇒ **无并发冲突**，可安全实施上述补丁。
