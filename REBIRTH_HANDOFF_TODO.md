# NeoTrix 蜕皮重生 — 任务交接清单

> 生成时间: 2026-09-15 | 生成者: AI Agent
> 状态: 架构设计+分析完成，待修复执行

---

## 当前状态

| 指标 | 数值 |
|------|------|
| 编译错误 | 918个（因349新文件+40模块未接线） |
| 安全漏洞 | 6个cargo audit（2个HIGH） |
| panic!() | 112个生产代码 |
| unsafe | 87个 |
| unwrap() | 3,332个 |
| 已修复 | ed25519特征, ort条件化, panic清理11文件 |
| 已创建 | 5个安全模块, 10个KB吸收分支 |

---

## Phase 0: 根因修复 (立即执行, 1天)

- [ ] 清除cargo锁: `rm -rf target/debug/.cargo-lock && pkill -f cargo`
- [ ] 修复 `ed25519-dalek` 特征: 确认 `rand_core` 替代 `rand`
- [ ] 修复 `memmap2` 依赖: 确认版本0.9.11与rkyv兼容
- [ ] 修复 `ort` 可选依赖: 确保 `#[cfg(feature="onnx")]` 正确
- [ ] 创建缺失模块: `stealth_middleware.rs`, `vault.rs` 或从mod.rs移除引用
- [ ] 修复 `tokio` 未链接: 确保所有使用tokio的文件有 `use tokio;`

## Phase 0-B: 模块接线 (2-3天)

- [ ] 修复 `neotrix-core/src/l1_action/nt_memory/typed_memory/conflict.rs` — 类型不匹配
- [ ] 修复 `neotrix-core/src/l1_action/nt_memory/typed_memory/kb.rs` — rusqlite API
- [ ] 修复 `neotrix-core/src/l1_action/nt_memory/typed_memory/multitier.rs` — 类型推断
- [ ] 修复 `neotrix-core/src/l1_action/nt_memory/typed_memory/forgetting.rs` — borrow
- [ ] 修复 `neotrix-core/src/l1_action/nt_io/nt_io_provider/config/external_config.rs` — lifetime
- [ ] 修复 `neotrix-core/src/l3_embodiment/nt_shield/http_intercept/mod.rs` — lifetime+序列化
- [ ] 修复 `neotrix-core/src/l6_meta/nt_meta/whale.rs` — AtomicU64 Clone
- [ ] 修复 `neotrix-core/src/entry.rs` — borrow/move
- [ ] 修复 `neotrix-core/src/l3_embodiment/nt_shield/osint/mod.rs` — 未使用变量+borrow
- [ ] 修复 `neotrix-core/src/l2_perception/nt_world/ocr/pdf_to_text_pipeline.rs` — 未使用变量

## Phase 1: 安全漏洞修复 (1天)

- [ ] 修复 `ed25519-dalek` 1.0.1 RUSTSEC-2022-0093 → 升级
- [ ] 修复 `wasmtime` 41/42 RUSTSEC-2026-0269 sandbox escape → 升级
- [ ] 修复 `rustc-serialize` RUSTSEC-2022-0004 → 移除
- [ ] 修复 `curve25519-dalek` 3.2.0 RUSTSEC-2024-0344 → 升级
- [ ] 添加KB文件锁 → 防止多进程数据库损坏
- [ ] 替换 `std::process::exit(0)` → 信号量优雅shutdown
- [ ] 修复EventBus mutex poisoning → 添加恢复机制
- [ ] 实现circuit breaker → CLOSED/OPEN/HALF_OPEN

## Phase 2: panic!()治理 (2天)

- [ ] `safety_kernel.rs` 12个panic!() → Result<(), SecurityError>
- [ ] `nt_core_dispatch.rs` panic → Err(DispatchError)
- [ ] `nt_core_event.rs` panic → Err(EventError)
- [ ] `nt_core_gwt/independence.rs` panic → Err(GwtError)
- [ ] `nt_core_error/mod.rs` + `recovery.rs` panic → Err(IoError/RecoveryError)
- [ ] `nt_core_guard_chain.rs` panic → Err(GuardError)
- [ ] `nt_core_grounded_gate.rs` 4个panic → Err(GateError)
- [ ] `nt_core_orch_agent.rs` panic → Err(OrchestrationError)
- [ ] `native_bus.rs` panic → Err(BusError)
- [ ] `consciousness_bridge.rs` panic → Err(BridgeError)
- [ ] `value_gate.rs` 2个panic → Err(PolicyViolation)
- [ ] 其余90个panic!() → 逐一替换

## Phase 3: 冗余清理 (1天)

- [ ] 扫描 `*_ecs.rs`, `*_scene.rs`, `*_signal.rs` 等重复模块
- [ ] 统一到单一实现
- [ ] 清理 `#[allow(dead_code)]` 超过10次的模块
- [ ] 检测theater modules (D44)
- [ ] 验证所有模块有消费者(Dark Forest)

## Phase 4: 缺失模块实现 (3-5天)

- [ ] `nt_act::AcpProtocol` — ACP协议
- [ ] `nt_memory::VectorIndex` — 向量检索
- [ ] `nt_act::AsyncToolExecutor` — 异步工具执行
- [ ] `nt_act::DeferredLoader` — 延迟加载
- [ ] `nt_shield::BinaryAnalyzer` — 二进制分析
- [ ] `nt_shield::MitigationAuditor` — 缓解措施审计
- [ ] `nt_shield::SinkAnalyzer` — 危险调用分析
- [ ] `nt_shield::VulnerabilityPipeline` — 漏洞管道
- [ ] `nt_world::FunctionRecovery` — 函数恢复
- [ ] `nt_world::CFGBuilder` — CFG构建
- [ ] `nt_core::TLCMLayerCorrection` — TLCM层校正
- [ ] `nt_mind::RISEReflector` — RISE反思
- [ ] `nt_mind::MidTurnSteering` — 中途转向
- [ ] `nt_meta::RuntimeMonitor` — 运行时监控
- [ ] `nt_meta::EvolvingEvaluator` — 评估器进化

## Phase 5: 跨域错位修复 (1天)

- [ ] NT-ACT ↔ NT-IO agent执行边界明确
- [ ] NT-CORE ↔ NT-MIND E8/SEAL边界明确
- [ ] NT-WORLD ↔ NT-SHIELD 浏览器分工明确
- [ ] NT-MEMORY ↔ NT-NEXUS 记忆边界明确
- [ ] 三重错误层次 → 实现From转换
- [ ] 6层vs3层代码混用 → 迁移到6层命名

## Phase 6: 测试验证 (2天)

- [ ] `cargo test -p neotrix --lib` 全绿
- [ ] SelfTest覆盖率 > 80%
- [ ] 性能基准测试通过
- [ ] 安全审计通过(0 unsafe)

## Phase 7: 生产就绪 (1天)

- [ ] Tauri桌面端修复(黑屏→正常)
- [ ] CLI命令补全/帮助
- [ ] 日志系统分级/轮转/格式化
- [ ] 配置热加载

---

## 多Agent执行方案

```
Agent1(Build) ──→ cargo check+test+clippy (每次commit)
Agent2(Audit) ──→ D1-D63 FPAM审查 (每5 cycle)
Agent3(Security)──→ cargo audit+渗透测试 (每周)
Agent4(Memory)───→ KB health+经验树 (60s tick)
Agent5(World)────→ OCR+browser+OSINT (每日)
Agent6(IO)───────→ providers+routing (每日)
Agent7(Mind)─────→ SEAL+WHALE+RSI (每cycle)
Agent8(Redundancy)──→ 冗余清理 (每月)
```

## 验证命令

```bash
# 修复后验证
cargo check -p neotrix --lib
cargo test -p neotrix --lib
cargo clippy -p neotrix --all-targets
cargo audit

# 经验树
neotrix-experience hub
neotrix-experience route-verify --clean
```

---

## 已完成的工作（交接）

1. ✅ 200+外部URL分析完成
2. ✅ 融合架构设计完成(DEFINITIVE_FUSION_PLAN.md)
3. ✅ 全量评测完成(NEOTRIX_EVALUATION.md)
4. ✅ 10个KB经验分支已写入
5. ✅ 11文件panic!()清理完成
6. ✅ 5个安全模块已创建(mitigation_auditor, sink_analyzer, vulnerability_pipeline, function_recovery, cfg_builder)
7. ✅ ed25519-dalek特征修复
8. ✅ ort依赖条件化
9. ✅ 复杂度分类器验证通过(969行)
10. ✅ 投机解码验证通过

---

*交接完成。请接收者从Phase 0开始执行。*
