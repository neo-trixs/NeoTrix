# 评测验证信条（VERIFICATION）

> 取自 rakazo `docs/agent-verification.md` 的方法论（Apache-2.0，思想无禁令），
> 按本仓口径重写。 GAIA-mini 真源在 `evals/gaia_mini/`。

## 三条铁律

1. **没跑 ≠ 通过**。缺 live 凭证、缺沙箱、缺模型时，该项记「未跑」，
   永不记绿。把「没有观察到失败」当成「验证通过」正是本仓假绿教训 §4 的全部内容。
2. **自称完成不算数**。模型/智能体说"做完了"，必须读产物本身：
   文件在不在、记录落没落库、副作用有没有 —— 断言打在产物上，不打在陈述上。
3. **失败先分类再归因**。红了先判是五者之几再动手：
   智能体错 / 产品错 / 提供方错 / harness 错 / 没跑完。
   直奔提示词/代码改，等于用修 bug 的方式修环境。

## 分层（按本仓现状裁剪上游七层）

| 层 | 真东西 | 替身 | 命令 |
|---|---|---|---|
| 单元/契约 | 纯函数与契约 | 无（不许 mock 真逻辑） | `cargo test -p neotrix-neobot -p neobot-desktop` |
| 门禁 | 脚本断言 | 无（门自己要先被变异验证） | `node scripts/ops/nt_check_*.mjs` |
| stub-boot 真渲染 | 生产包 + Chrome | IPC 桩（数据是编的，几何是真的） | layout / interact 两门 |
| 真机 | Tauri 应用 + WKWebView | 无（Chrome 不是 WKWebView，不冒充） | 缺口，见 STATUS §5 |
| 模型实测 | 真凭证真跑 | 无；缺凭证即「未跑」 | 按需，不进 CI |

## 红线

- Fixture 必须合成；不清不楚的数据不进仓（看不清 redistribution 条款的第三方 eval 数据不引）。
- 清空历史不能重置预算/计数（上游原话：tool budget 跨 clearing 保留）。
- 修 harness 引起基线变化时，先修 harness 再重跑基线，不改单子迎合实现。
