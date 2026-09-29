# Agentation 吸收（2026-09-24）

> 源：https://github.com/benjitaylor/agentation（4.7k★，克隆于 thirdparty/agentation，HEAD 0e3236e）
> 一句话：agent-agnostic 可视化反馈工具——人类在页面上点选标注，产出带 selector/位置/上下文的结构化 markdown，经 MCP 同步给 coding agent 做精准 grep 定位。

## 能力映射（NeoTrix 归位）

| Agentation 件 | NeoTrix 家 | 接口 |
|---|---|---|
| Annotation（selector/path/comment/kind/intent/severity） | 记忆卡片（cocoons） | models/training/agentation_to_cards.py → {url,title,content,domain=agentation-feedback} → `nt-train-export -- --ingest`（content-hash 幂等） |
| MCP tools（get_pending/ack/resolve/watch） | 人机回环（ntbrowse/Colab 浏览器工作流） | 先文件摆渡（本脚本），直连 MCP client 待 nt_act 补 client 侧（现 mcp_protocol 只有 server 侧） |
| skills/agentation（Next.js 挂载） | 不适用（NeoTrix 非 Next.js 前端） | 只取 two-session-workflow 方法论：标注→selector→grep→修→resolve |
| skills/agentation-self-driving | 对话吸收桥（D2 结晶输入） | 同上，方法论层复用 |

## License 红线

PolyForm Shield 1.0.0（非宽松开源）：**零代码拷贝**，只做文件/MCP 协议级互操作 + 方法论复述。本次吸收无 license 风险。

## 验证

- `agentation_to_cards.py --selftest` OK（resolved 默认跳过，稳定 URL，字段齐）。
- ingest 通道已由 arch3 实证（new=26/175/1645）；首批真实标注到来即跑 ingest（命令见上）。

## 下一步

1. 首个真实标注导出 → 跑转换 + ingest → cocoons。
2. nt_act 补 MCP client 侧后，直连 agentation-mcp（watch 订阅替代文件摆渡）。
3. ✅ 已落（思想二阶段）：scripts/ops/nt_locate.py —— 选择器/组件名/sourceFile → 文件:行
   （L1 sourceFile 直达 100 分 → L2 token 覆盖排名 → L3 原串兜底，只读零依赖，--selftest 过）。
   微操作闭环：人类点选/Agentation 标注 → nt_locate 定点 → 读上下文 → 最小改动 → 单测验证。
