# Session Anchor — Cycle 19: 全量代码审查 + Python层5缺陷修复 + 传输层集成

## Goal
全量代码审查 nt_comm_router.py + nt_api_client.py + absorb-arxiv-content.py，修复5个P0/P1缺陷，实现CommRouter与AccessPipeline传输层集成。

## Key Findings
1. **`_strip_internal_headers` 硬编码'generic'**: 替换串从模式元组拆包时用 `_` 丢弃，永远写'generic'
2. **CommRouter 直调 urllib.request.urlopen**: 不享受 nt_api_client 的指数退避/重试/UA轮换
3. **absorb-arxiv-content.py 模块级 pipeline 非延迟初始化**: import 即创建 TorTransport probe
4. **ProxyPool 无 valid 标志**: refresh() 早期返回路径 `p.get('valid')` 永远为 None → 总是返回 0
5. **跨模块隔离但可组合**: 通过 DI 注入 transport，无需新模块

## Fixes Applied
- `nt_comm_router.py`: 硬编码'generic'→使用模式替换串; RouteEngine接受transport委托; CommRouter接受transport参数
- `nt_api_client.py`: ProxyPool refresh() 加 `'valid': True` 标志
- `absorb-arxiv-content.py`: 模块级 pipeline→`_get_pipeline()`懒加载

## Build Status
- ✅ `python3 -m py_compile` 全部3文件通过
- ✅ CommRouter自测试: 6人设轮换 / 内幕剥离(neotrix→client, x-neotrix→x-client) / Safari macOS httpbin.org 200
- ✅ Cargo: 1 pre-existing error in nt_mind/panorama_pipeline.rs (legacy, 非本次引入)

## Next Steps
- 长周期稳定性测试: 运行CommRouter 5+端点验证人设轮换
- Wikipedia REST API 503 fallback → MediaWiki API
- `NEOTRIX_PROXY_URL` env var 支持商业住宅代理
- Rust端 `_strip_internal_headers` 等效实现 (nt_shield/http_proxy.rs)
