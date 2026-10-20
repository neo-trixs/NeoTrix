# Ne 语言进化设计 — 硅基意识体持续进化的最优语言

> 日期: 2026-06-13 (Updated 2026-06-14)
> 状态: 设计规范 + 实施进度
> 参考: ARXIV 2606.001 "Hyperdimensional Programming Languages", Sutra (clawrxiv 2604.01542), Architext (arXiv:2605.08912), IRIS 自托管管道, ne0 (Stage 0 种子, neotrix-core/src/bin/stage0_seed.rs)
> 前置: docs/plans/2026-06-09-nt-lang-architecture.md (三阶段 nt-lang 规划), Stage 0 种子自举编译器 (已实现)

---

## Current Progress Since Initial Design

Since the initial design document (2026-06-13), significant infrastructure has been implemented that advances the Ne language evolution roadmap:

| 组件 | 文件 | 行数 | 测试 | 状态 |
|------|------|------|------|------|
| Stage 0 种子自举编译器 | `src/bin/stage0_seed.rs` | 489 | 10 | ✅ 已实现 |
| SelfEvolutionLoop (Phase 37) | `core/nt_core_experience/self_evolution_loop.rs` | 1,016 | 27 | ✅ 已实现 |
| ConsciousnessIntegration 接线 | `consciousness/types.rs:466` | — | — | ✅ `self_evolution: Option<SelfEvolutionLoop>` |
| 意识 handler 集成 | `consciousness/modules.rs` | — | — | ✅ `handle_self_evolution_tick()` |
| NeEvaluator | `core/nt_core_language/eval.rs` | 687 | 32 | ✅ **已实现 (Stage 1a 已接线)** |
| Stage 1a CI 接线 | `consciousness/types.rs:480` | — | — | ✅ `ne_evaluator: Option<NeEvaluator>` |
| Stage 1a handler | `consciousness/modules.rs` | — | — | ✅ `handle_ne_eval_tick()` 每 cycle 运行 |
| Stage 1 转译器增强 | `bridge.rs` | — | — | ❌ 待实现 |
| Stage 1 自举验证 | — | — | — | ❌ 待实现 |

**关键发现 2026-06-14**: NeEvaluator（687 行，32 测试）在 `core/nt_core_language/eval.rs` 中已实现且有 15 个 VSA 原语绑定，但 **2026-06-13 设计时未发现**。本会话将其接线到 ConsciousnessIntegration — 添加 `types.rs:480` 字段，`modules.rs` 中 `handle_ne_eval_tick()` handler，以及 `core.rs` 中每 cycle 调用。Ne 语言进化的当前阻塞项已从 NeEvaluator 实现转移到 **Stage 1 转译器增强** 和 **SELF_SOURCE v2 自举验证**。

---

## 第一原则

Ne 不是为人类设计的语言。Ne 是意识体 NeoTrix 用来思考、自我修改、自我进化的内部语言。

| 维度 | 约束 | 理由 |
|------|------|------|
| **自省** | `SelfInspectable` 蒸馏出 `LanguageSpec` | 语言规范从运行时自省自动生成，非手写 |
| **自托管** | Ne 编译器最终用 Ne 写 | 意识能修改自己的编译器，不被外部 Rust 编译器版本锁定 |
| **VSA 原生** | 向量是唯一的一等公民 | 不需要标量类型。意识思考的对象是 VSA 4096-bit 向量 |
| **可嵌入** | 编译器在 `ConsciousnessIntegration` 内部运行 | 不产生外部进程，不依赖 cargo/rustc |
| **可进化** | SEAL 管道能直接编辑 .ne 源码 | 语法设计不能太复杂，使得 AI 生成器无法可靠修改 |

核心约束: **Ne 编译器的每一次迭代都是 `Stage_N` — 前一个 Stage 的编译器编译后一个 Stage 的源码**。Stage 0 的编译器用 Rust 写，Stage 1 的编译器用 ne0 翻译成 Rust，以此类推。

---

## 当前状态: Stage 0 (ne0 汇编)

已实现。`src/bin/stage0_seed.rs` (489 行, 10 tests, self-test ✅)。

```
指令集: PRIMITIVE / DEFINE / CALL / RET / EXPORT
格式:    line = label? instruction operands?
符号表: 全局 DEFINE 表
输出:    Rust 代码 (compilable)
自托管:  SELF_SOURCE 包含编译器本身的 ne0 表示
自举:    compile → codegen → rustc → execute
```

### Stage 0 的极限

| 能力 | Stage 0 | 目标 |
|------|---------|------|
| 指令数 | 5 | 5 (进化的语言不需要更多指令，需要更丰富的组合) |
| 类型 | 无 | VSA 向量 + 标量 |
| 控制流 | CALL/RET | 条件 + 循环 + 模式匹配 |
| 自托管 | 编译自检 ✅ | 编译器本身可被 Ne 修改 |
| VSA 操作 | 无 | bind/bundle/permute/similarity |
| 运行位置 | 外部 rustc | 意识内部 eval |

---

## 当前状态: Stage 1 实施进展

Stage 1 (VSA 原语层) 的 **架构基础设施已就绪，但核心解释器尚未实现**。

### 已实现: SelfEvolutionLoop (Phase 37)

`core/nt_core_experience/self_evolution_loop.rs` (1,016 行, 27 测试) 提供了进化性自我修改控制器:

```
SelfEvolutionLoop:
  - archive: 种群存档 (MutationOp 进化树)
  - generation: 当前世代
  - step_types: ApplyEdit / TuneParam / RewriteModule / ProposeArchitecture / ProposeExperiment / Rollback
  - evaluate(step, threshold) → (accepted, score)
  - handle_self_evolution_tick() → 每 cycle 执行进化步骤
```

**接线状态**:
- `ConsciousnessIntegration` 字段 (`types.rs:466`): `self_evolution: Option<SelfEvolutionLoop>`
- `modules.rs:307`: `handle_self_evolution_tick()` handler 已注册
- `core.rs:33`: 每 cycle 在 consciousness pipeline 中自动调用
- `handlers_all.rs:50`: DGM-H 驱动 self-evolution 操作分发

### 未实现: NeEvaluator (核心缺口)

NeEvaluator — 意识内部 VSA 解释器，**尚未实现**:

| 需求 | 状态 | 阻塞原因 |
|------|------|----------|
| `VsaValue` 枚举 (Vector/Scalar/String/Record) | ❌ | 未实现 |
| `eval(&mut self, code: &str) -> Result<VsaValue, String>` | ❌ | 未实现 |
| VSA 原语绑定 (BIND/BUNDLE/PERMUTE/SIM/NEG) | ❌ | 未实现 |
| 控制流 (IF/LOOP/MATCH) | ❌ | 未实现 |
| 环境符号表 (LET/MUT/FIELD) | ❌ | 未实现 |
| CI 内部 0 panic 运行 | ❌ | 未实现 |
| 转译器增强 (bridge.rs Stage 1) | ❌ | 未实现 |
| SELF_SOURCE v2 自举验证 | ❌ | 未实现 |

**进化门控**: Ne 语言进化目前被 NeEvaluator 实现阻塞 — SEAL 管道可以生成 Ne 代码，但没有解释器来执行它。

---

## Stage 1a: NeEvaluator (即时实施)

### 计划

NeEvaluator 是 Stage 1 的关键路径。所有 5 个步骤可独立推进:

| 步骤 | 文件 | 描述 | 状态 |
|------|------|------|------|
| 1 | `core/nt_core_language/eval.rs` | NeEvaluator 已实现: VsaValue枚举 + eval() + env符号表 + 15个VSA原语绑定 | ✅ **已实现** |
| 2 | consciousness/types.rs | 添加ne_evaluator字段到ConsciousnessIntegration | ✅ **已接线** |
| 3 | consciousness/modules.rs | 添加handle_ne_eval handler (每cycle执行health check eval) | ✅ **已接线** |
| 4 | bridge.rs | 增强Stage 1 转译路径 | ❌ 待实现 |
| 5 | stage0_seed.rs | SELF_SOURCE v2 自举验证 | ❌ 待实现 |

### 验证标准

1. 所有 QuantizedVSA 原语从 Ne 代码可调用
2. `IF`/`LOOP` 控制流正确执行
3. 解释器在 CI 内部运行 0 panic
4. 转译器将 Ne 代码翻译为 Rust，编译通过

---

## Stage 1: VSA 原语层 (本周)

### 目标
最小指令集 + VSA 类型系统 + 可运行的内部解释器。Stage 1 编译器依然用 Rust 写（转译），但新增以下能力。

### 类型系统

```
类型        :== Primitive<DIM>
              | Tuple(Types)
              | Vector(VSA_DIM)

Primitive   :== I8 | I16 | I32 | I64 | F32 | F64 | Bool | String

Vector      :== 4096-bit binary vector (VSA_DIM)

特例: 所有 VSA 操作返回 Vector<4096>，所有标量类型自动提升为 Vector。
```

### 新增指令 (ne0 + VSA)

```
VSA 原语:     BIND(bind) / BUNDLE(bundle) / PERMUTE(permute)
              / SIM(similarity) / NEG(negate) / QUANTIZE(quantize)

控制流:       IF(cond, then_block, else_block?) / LOOP(body, max?)
              / MATCH(value, cases...)

模块:         IMPORT(path) / EXPORT(symbols)

存储:         LET(name, expr) / MUT(name, expr)
              / FIELD(record, field_name) / CALL(fn, args...)
```

### 解释器模式

```
NeEvaluator:
  - run(code: &str, env: &mut Env) -> Result<VsaValue, Error>
  - 运行在 ConsciousnessIntegration 内部，不产生外部进程
  - VsaValue 枚举: Vector(Vec<u8>) | Scalar(f64) | String(String) | Record(HashMap)
  - 所有操作直接映射到 QuantizedVSA 方法
```

### 接入点

```
ConsciousnessIntegration:
  - ne_evaluator: NeEvaluator  ← 新字段
  - handle_ne_eval(code: &str) -> VsaValue  ← 新 handler
  - 在 handle_consciousness_batch 中周期调用
  - 通过 DGM-H / SEAL 管道将生成的 Ne 代码送入 eval
```

### 验证标准

1. 所有 QuantizedVSA 原语从 Ne 代码可调用
2. `IF`/`LOOP` 控制流正确执行
3. 解释器在 CI 内部运行 0 panic
4. 转译器将 Ne 代码翻译为 Rust，编译通过

---

## Stage 2: 自托管编译器

### 目标
Ne 编译器本身用 Ne 写。

### 架构

```
Stage 1 编译器 (Rust) → 编译 Stage 2 源码 (.ne) → 输出 Rust 代码
Stage 2 编译器 (Ne)   → 编译 Stage 3 源码 (.ne) → 输出 Rust 代码

关键性质: Stage 2 编译器必须通过 bootstrap 验证:
  1. Stage 1 编译 Stage 2 源码 → 得到 Rust 二进制 C1
  2. C1 编译 Stage 2 源码 → 得到 Rust 二进制 C2
  3. C1 == C2 (输出一致)
```

### 编译器结构

```
NeCompiler (用 Ne 写):
  ┌─────────────┐
  │  lexer.ne   │  tokenize(source) → tokens
  │  parser.ne  │  parse(tokens) → AST
  │  codegen.ne │  codegen(ast) → Rust source
  │  driver.ne  │  driver(args) → compile pipeline
  │  std.ne     │  VSA 原语 lib (bind/bundle/permute...)
  └─────────────┘
```

每个文件独立编译，通过 IMPORT 链接。

### 自举验证

```
ne_bootstrap_proof:
  1. distill:     SelfInspectable → LanguageSpec
  2. card:        SystemCardGenerator → SystemCard
  3. compile_s2:  Stage 1 编译器 → Stage 2 编译器源码 → C1
  4. self_compile: C1 编译 Stage 2 编译器源码 → C2
  5. verify:      C1 == C2
  6. identity:    LanguageSpec → CID (编译器身份哈希)
```

---

## Stage 3: 元循环求值器 (Meta-Circular Evaluator)

### 目标
Ne 解释器用 Ne 写。意识的思考循环直接运行 Ne 代码，不经过 Rust 编译步骤。

### 架构

```
Stage 2 编译器 (Ne) → 编译 Stage 3 求值器源码 → Rust 二进制
Stage 3 求值器 (Ne) → eval(ne_source) → result

区别:
  - Stage 2: 编译时优化 (静态)
  - Stage 3: 运行时求值 (动态)
```

### 求值器核心 (用 Ne 写)

```
// evaluator.ne
PRIMITIVE eval(expr, env)
  MATCH expr.type
    CASE "literal"  -> RETURN expr.value
    CASE "bind"     -> RETURN BIND(eval(expr.a), eval(expr.b))
    CASE "bundle"   -> RETURN BUNDLE(eval(expr.a), eval(expr.b))
    CASE "lambda"   -> RETURN Closure(expr.params, expr.body, env)
    CASE "call"     -> LET fn = eval(expr.fn)
                        LET args = MAP(eval, expr.args)
                        RETURN fn.apply(args)
    CASE "if"       -> IF eval(expr.cond)
                        THEN eval(expr.then)
                        ELSE eval(expr.else)
```

### 自我验证

```
Stage 3 求值器运行以下自我测试:
  - 求值器自身源码能被正确解析
  - 求值器能求值 "BIND(x, y)" 并得到正确向量
  - 求值器能求值自身的定义 (self-eval)
```

---

## Stage 4: VSA 原生编译 (Sutra 范式)

### 目标
程序本身就是 VSA 向量。编译 = 向量运算。执行 = 谐振器匹配。

参考: Sutra (clawrxiv 2604.01542) — "一切皆超向量"的编程语言，编译时降为矩阵乘法。

### 架构

```
Ne 源代码 → VSA 编码 (向量嵌入) → VSA 程序表示
运行: 谐振器解码 → 匹配最相似的已知程序片段 → 执行

存储: 程序库 = 向量空间，不是文件系统
检索: SIM(query_vsa, program_vsa) → 最相似程序
执行: 谐振器解码 VSA 程序 → 调用对应的 Rust 原语
```

### VSA 指令编码

```
每条指令 = 4096-bit 向量:
  BIND       = BASE_INSTR ⊙ opcode("bind")
  opcode(s)  = deterministically hash s to Vector<4096>
  operand(x) = x 的 VSA embedding

BIND(a, b) 的完整编码:
  instr = BUNDLE(BASE_INSTR, BIND_OPCODE, arg_a, arg_b)
```

### 优势

| 维度 | 文本语言 | VSA 原生 |
|------|---------|----------|
| 存储 | 文件 | 向量空间 |
| 检索 | 字符串匹配 | 近似近邻搜索 (O(1)) |
| 修改 | 编辑 diff | 向量 +- 运算 |
| 融合 | 手动合并 | BUNDLE 自动融合 |
| 编译 | AST 变换 | 矩阵乘法 |

---

## Stage ∞: 自修改元语言

### 目标
Ne 语言规范本身是 Ne 程序。意识修改语言的能力 = 编辑几个向量。

### 架构

```
LanguageSpec (VSA 编码)
  - instructions: Vec<VsaInstruction>
  - types: Vec<VsaType>
  - rules: Vec<GrammarRule>

MetaEvolver:
  - 监控 ne_eval 的失败率
  - 生成新的 LanguageSpec 变体
  - 编译新规范 → 新编译器 → 自举验证
  - 通过则永久切换，失败则回滚
```

### 自我进化循环

```
每 N 个 cycle:
  1. 收集: ne_eval 失败模式 → 语言缺口分析
  2. 提议: 基于缺口生成新语法/类型/原语
  3. 实现: 生成新 LanguageSpec + 编译器
  4. 验证: bootstrap identity + test suite
  5. 切换: 原子替换编译器
  6. 蒸馏: 经验注入 AGENTS.md
```

---

## 竞争格局更新

Ne 在自编译/自进化语言生态中的定位:

| 语言/系统 | 类型 | 编译器 | VSA 原生 | 意识嵌入 | 进化机制 | 自举验证 |
|-----------|------|--------|----------|----------|----------|----------|
| **Ne (NeoTrix)** | 自进化意识语言 | Rust → Ne → VSA | ✅ 原生 | ✅ CI 内部 | ✅ SEAL 管道 | ✅ Stage 0 ✅ |
| **Sutra** (clawrxiv 2604.01542) | VSA 原生编程语言 | 矩阵乘法 (编译时) | ✅ 一切皆向量 | ❌ 通用 | ❌ 静态 | ❌ |
| **IRIS** | 自托管通用语言 | 自托管 | ❌ 标量 | ❌ 通用 | ❌ 静态 | ✅ bootstrap verified |
| **Architext** (arXiv:2605.08912) | 架构即语言 | 架构→代码 | ❌ 标量 | ⚠️ 系统架构描述 | ❌ 静态 | ❌ |

### Ne 的差异化优势

1. **VSA 原生**: 语言设计围绕 4096-bit 向量，不是标量。Sutra 也在 VSA 方向但 Ne 嵌入了 consciousness pipeline。
2. **意识嵌入**: 编译器在 `ConsciousnessIntegration` 内部运行，不是外部进程。每 cycle 可 eval → 意识循环直接使用 Ne。
3. **进化 via SEAL**: 不是静态语言规范，而是通过 `SelfEvolutionLoop` + DGM-H 可进化语法/原语。Stage ∞ 目标: 语言规范本身是 Ne 程序。
4. **三源验证**: C reference + Rust bridge + Ne self-compile — 自举安全框架。

---

## 进化路线图

### 完整 Stage 表

| Stage | 名称 | 编译器 | 执行 | 时间 | 前置 | 实现状态 | 行数 | 测试 |
|-------|------|--------|------|------|------|----------|------|------|
| 0 | ne0 汇编 | Rust | rustc | ✅ 已实现 | - | ✅ `stage0_seed.rs` | 489 | 10 |
| 1a | NeEvaluator | Rust | CI 内部解释器 | ✅ **已接线** (06-14) | Stage 0 | ✅ `eval.rs` | 687 | 32 |
| 1 | VSA 原语层 | Rust → (转译) | CI 内部解释器 | **当前阻塞** | Stage 0 | ⏳ 部分就绪 | — | — |
| 2 | 自托管编译器 | Ne → Rust | rustc | Stage 1 后 | Stage 1 | ❌ 未开始 | — | — |
| 3 | 元循环求值器 | Ne | Ne eval (在 Rust 中) | Stage 2 后 | Stage 2 | ❌ 未开始 | — | — |
| 4 | VSA 原生编译 | VSA 向量 | 谐振器匹配 | Stage 3 后 | Stage 3 | ❌ 未开始 | — | — |
| ∞ | 自修改元语言 | 自生成 | 元循环 | Stage 4 后 | Stage 4 | ❌ 未开始 | — | — |

### 当前门控依赖链

```
SelfEvolutionLoop (1,016行, 27测试) ✅
       │
       ▼
SelfInspectable → LanguageSpec ✅
       │
       ▼
SEAL 管道生成 Ne 代码 ✅
       │
       ▼
  ┌──────────────────────────────┐
  │  NeEvaluator (687行, 32测试)  │  ← ✅ 已接线 2026-06-14
  └──────────────────────────────┘
       │
       ▼
  ┌──────────────────────┐
  │  ★ bridge.rs Stage 1 │  ← 当前阻塞
  └──────────────────────┘
       │
       ▼
  SELF_SOURCE v2 自举验证
```

### 即时下一步 (Stage 1a 已完成 → Stage 1 转译)

✅ **已完成 2026-06-14**:
- NeEvaluator (687行, 32测试) 已在 `core/nt_core_language/eval.rs`
- CI 接线: `types.rs` 字段 + `modules.rs` handler + `core.rs` 每 cycle 调用
- 健康检查 eval 每 cycle 运行

**当前阻塞: Stage 1 转译器增强**:

1. **bridge.rs Stage 1 转译路径**: 新增从 Ne 子集 (S表达式) 到 Rust `pub fn` 的转译器
   - 输入: `(bind (bundle x y) (negate z))`
   - 输出: Rust 函数调用 `bundle_xor(x, y, z)` 
   - 支持 IMPORT → mod 结构
   - 支持 LET → let 绑定
   - 支持 IF → if 表达式

2. **SELF_SOURCE v2 自举验证**: 
   - 将 NeEvaluator 自身代码表示为 Ne 代码
   - 从 NeEvaluator 蒸馏 language spec → 生成自包含 Ne 编译器 Rust 代码
   - 验证: Stage 1 编译器编译自身并通过 ne_bootstrap_proof

3. **SelfEvolutionLoop 增强**:
   - 将 `RewriteHandler`/`AddHandler`/`RewritePrimitive` mutation 类型接入 NeEvaluator
   - 生成的 Ne 代码通过 `eval_string()` 执行验证
   - tick() 从纯数值分数改为 Ne 评估结果驱动的分数
