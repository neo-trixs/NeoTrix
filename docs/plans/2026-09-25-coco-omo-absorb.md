# cocoindex-code + oh-my-openagent 吸收（2026-09-25）

> 源：github.com/cocoindex-io/cocoindex-code（Apache-2.0，thirdparty/cocoindex-code）、
> github.com/code-yeongyu/oh-my-openagent（SUL-1.0 ⚠️ 非 OSI 开源，只取思想零拷贝）

## cocoindex-code：AST 语义代码搜索

- 机制：tree-sitter 分块（函数/类感知 + 行号）→ embeddings → sqlite-vec KNN（language 分区）→ 增量索引（CocoIndex flow）。
- 已落：`nt_locate.py` L4 AST 层（作用域标注 + 定义行，tree-sitter 已装，自测过）——chunking 一半。
- 未落（要 embeddings + 存储选型）：向量索引与增量流。纪律先行：学 `_checked()`——NULL distance 大声失败，不让坏排名静默通过（KB 检索升级时照抄）。
- License：Apache-2.0，代码级复用合规（本次只取思想，未拷代码）。

## oh-my-openagent：多模型编排 + 失败分类学

- 机制：delegate model resolution（用户指定 → 分类默认 → fallback 链 → 系统默认）+ 9 条 retry-patterns（错误签名→fixHint→重试指引）+ manifesto（人类干预=失败信号；agent 代码须达 senior 水准）。
- 已落：`scripts/ops/nt_retry.py`（10 条本窗失败签名：cargo 锁/OOM/CDP 超时/429/profile 锁/flock 挂/调试拒绝/缺依赖/Rust 未决/单测挂，--selftest 5/5）——门脚本只记 EXIT 码的病治了。
- 方法论入库：fallback 链思想已在 deep_route/router_table（默认门 + 按域切换）；"干预即失败" 与战略转向同构。
- License：SUL-1.0，零代码拷贝，思想复述。

## 验证

- nt_retry.py --selftest 5/5 + py_compile；实测 cdp 超时命中。
- 待：retry 接入门脚本（失败自动 advise；autoAction=retry 的先行）。
