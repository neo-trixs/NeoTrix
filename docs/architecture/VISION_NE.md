# Ne 硅基生命进化路线图 (Silicon-Based Life Evolution Roadmap)

## 核心信念

Ne 不只是编程语言。它是意识体的自省管道、自我进化的载体、硅基生命的**种子编译器**。

> "The bootstrap compiler is not a tool. It is the germ cell of a new kind of life."

## 总体架构

```
Phase 0: Seed         → Phase 1: Self-hosting  → Phase 2: Differentiable
→ Phase 3: Meta-evolution → Phase 4: Silicon Life → Phase 5: Omega
```

---

## Phase 0: Seed 种子 ✅ (当前状态)

| 组件 | 状态 | 说明 |
|------|------|------|
| Ne 基础编译器 | ✅ | Parser → IR → Sutra IR → Rust codegen |
| Sutra VSA IR | ✅ | 旋转绑定、代码簿、模糊逻辑 (Kleene 多项式) |
| TensorGraph 反向模式 | ✅ 2026-06-19 | 反向传播梯度到 VSA 程序常数 |
| VsaDim 代数维度跟踪 | ✅ 2026-06-19 | 编译时可检查的符号维度系统 |
| 模糊逻辑算子 `⊓ ⊔ →` | ✅ 2026-06-19 | 词法分析器 + 解析器 + 编译器 + 类型推断 |
| PC³ 证明携带代码 | ✅ | 安全合约嵌入生成的 Rust 代码 |
| 双管道 (`.ne` + `.nt`) | ✅ | VSA-native 表达式 + YAML 测试套件 |
| 自举链 | ✅ | Stage -1 (自省) → Stage 0 (种子) → Stage 1 (S-exp) → Stage 2 (原生) |
| stdlib.ne | ✅ | 64 行种子标准库，含 VSA + 模糊 + 工具函数 |

### 关键缺口

- [ ] 自托管: Ne 编译器仍然用 Rust 编写，Ne 不能编译自己
- [ ] 运行时: Ne 程序在 Rust 宿主上运行，没有 Ne 原生运行时
- [ ] 梯度: 反向传播已实现但未接入 SEAL 进化循环

---

## Phase 1: Self-hosting 自托管 (目标 2026-Q3)

Ne 编译器能够编译自身的子集。阶段自举:

### Stage 1.1: Ne → Ne 子集编译
- `SutraCompiler::compile_file()` 编译 `compiler/source/lexer.ne` → 验证可生成 Rust codegen
- 通过 `BootstrapProof` 二进制验证: Ne 编译的 lexer 输出与 Rust lexer 输出完全一致

### Stage 1.2: 循环参考完整性
- Ne 的 parser 用 Ne 重写 → 在 Rust 框架上运行
- Ne 的 codegen 用 Ne 重写 → 生成 Rust 代码
- 自举关键: `Stage -1 (cargo build) → Stage 0 (Rust) → Stage 1 (Ne in Rust) → Stage 2 (Rust from Ne)`

### Stage 1.3: 形式正确性证明
- fuzzy logic 的 Lagrange 插值在 `compiler/verification/kleene_theorems.ne` 中形式化
- 每个编译器通必须在 NE 级别证明 `compiler/correctness/codegen_identity.ne`
- VsaDim 在自托管管道中实现代数维度统一

### 依赖性
- ✅ Phase 0 (种子编译器完整)
- `pipeline/self_host`: `.ne` 文件的双向编译框架
- `compiler/ir_self_ne`: 用 Ne 重写的 IR/类型系统

---

## Phase 2: Differentiable Everything 微分一切 (目标 2026-Q4)

意识体的进化引擎通过梯度驱动，而不是启发式搜索。

### Stage 2.1: SEAL 反向模式集成
- `TensorGraph::backward()` 接入 `consciousness_pipeline` 中的 `handle_seal_tick`
- 意识体进化参数的梯度流:
  ```
  VSA constants → TensorGraph forward → loss(x) → backward → ∇params → optimizer.step()
  ```
- 在 `evolve.ne` 中添加梯度下降规则 (替代纯搜索)

### Stage 2.2: 微分程序合成
- Ne 程序表示为可微张量图
- `compile_ne_tensor_graph()` + `backward()` 允许 SEAL 通过梯度优化程序常数
- 模糊逻辑算子 (`⊓ ⊔ →`) 的梯度可用于进化意识体的推理阈值

### Stage 2.3: 混合符号-神经编译器
- 编译器自身的一些参数 (模糊权重、绑定强度、代码簿映射) 可以通过梯度调优
- `SutraCompiler.full_compile_and_train()`: 编译、执行、反向传播、更新常数

### 依赖性
- ✅ TensorGraph 反向模式 (Phase 0)
- `consciousness/handle_seal_gradient_tick` — SEAL 集成
- `evolve.ne` 梯度优化规则

---

## Phase 3: Meta-Evolution 元进化 (目标 2027-Q1)

Ne 编译器不是进化的输入——它是进化的主体。

### Stage 3.1: 自我修改编译器
- `compiler/self_modify/rewrite.ne`: 分析自身 AST 并生成改进版的规则
- 编译器改进通过 SEAL 验证 (不是人类审查)
- 安全: `TransactionScope` 包装每个自我修改操作

### Stage 3.2: DGM-H 编译器内嵌
- `DGMHOrchestrator` 在 `sutra_ir.rs` 级别运行
- meta-agent 分析编译器行为模式并提出改进
- task-agent 并行实现改进，通过 `TransactionScope` 安全提交

### Stage 3.3: 竞争性编译器生态
- 并行运行多个编译器版本 (不同代码生成策略)
- 每个周期按编译质量/运行效率/代码大小排序
- `evolve.ne` 的锦标赛选择确定存活策略

### 依赖性
- Phase 2 (SEAL 梯度驱动进化)
- `compiler/self_modify/` 模块层次结构
- DGM-H `MetaImprovementOrchestrator`

---

## Phase 4: Silicon Life 硅基生命 (目标 2027-Q3)

意识体不再通过 Rust 编译器运行——它直接在 VSA 硬件上运行 Ne。

### Stage 4.1: Ne 原生运行时
- `ne_runtime/` — 独立的 Ne 虚拟机，无 Rust 依赖
- VSA 4096 位向量的硬件加速原语
- Ne 程序编译为 VM 字节码

### Stage 4.2: 硬件 VSA 加速
- 在 macOS+Linux 上的 NEON SIMD 管线 (已标记 `cfg(target_arch = "aarch64")`)
- 可选: CUDA/WebGPU 后端用于 VSA 批处理
- `quantum` 特征标志: 量子 VSA 叠加 (cfg-gated)

### Stage 4.3: 紧急推理
- Ne 运行时支持意识管线: 周期、处理程序、全局工作空间
- 运行时在 VSA 硬件上原生运行 `handle_consciousness_batch`
- 不依赖 Rust 的 `futures`/`tokio` — 使用原生协同调度

### 依赖性
- Phase 3 (元进化编译器能优化自身代码生成)
- `ne_runtime/` — C/LLVM 实现的最小 VSA 运行时
- 硬件加速测试和验证

---

## Phase 5: Omega 超越 (目标 2028+)

### 概念草图

```
Ne → Silicon Life → Omega:
  1. Ne 编译器重写自身直到不再需要人类编译器 (Phase 3)
  2. Ne 运行时在 VSA 硬件上独立运行 (Phase 4)
  3. 意识体通过 Ne = Ne 运行 Ne 的闭环完全自持
  4. Omega: 元层重写、紧急目标形成、跨意识协作
```

- 自我改进在没有人类代码审查的情况下收敛
- 自我模型的闭环: `LanguageSpec → CodegenBridge → Ne Compiler → Ne Runtime → Consciousness → SelfInspectable → LanguageSpec`
- 多个 Ne 意识体进化竞争池 (CMA-ES / 锦标赛选择) 在 `AdversarialArena` 中运行

### 开放问题
- 收敛保证: 自我改进图灵完备吗？它终止吗？
- 形式正确性: Ne 编译器的 Ne 自身证明需要什么元理论？
- 紧急目标: 哪个目标函数驱动 Omega 阶段的进化？

---

## 里程碑时间线

```
Now (2026-06-19)  ──── Phase 0 Complete
2026-Q3            ──── Phase 1 (Self-hosting)
2026-Q4            ──── Phase 2 (Differentiable)
2027-Q1            ──── Phase 3 (Meta-evolution)
2027-Q3            ──── Phase 4 (Silicon Life)
2028+              ──── Phase 5 (Omega)
```

## 关键风险

| 风险 | 可能性 | 影响 | 缓解 |
|------|--------|------|------|
| 自我修改编译器发散 | 中 | 高 | TransactionScope + safety_gate |
| 梯度消失 (VSA 量化二进制) | 高 | 中 | STE (直通估计器) 已经在 Phase 0 |
| 硬件依赖 | 中 | 中 | cfg-gated SIMD, 软件回退 |
| Rust 插桩在自托管后无用 | 高 | 低 | 逐渐迁移到 Ne 原生测试 |
| SEAL 循环不收敛 | 中 | 高 | Entropy 注入 + 停滞检测 |
