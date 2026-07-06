# nt-lang 架构规划 — NeoTrix 原生语言的三阶段进化

> 日期: 2026-06-09
> 状态: 规划草案
> 参考:
> - Meta HyperAgents / DGM-H (arXiv 2603.19461, Mar 2026) — 元层自我修改架构
> - IRIS 语言 (boj/iris, GitHub 2026) — 自改进编程语言先例，ML 语法 + LCF 证明核 + 自托管管道
> - AGENTS.md — NeoTrix 意识体行为规范与进化缺口分析

---

## 问题陈述

NeoTrix 当前有 5,544 个测试，全部编译为单个 `neotrix` 二进制。每次 `cargo test` 触发完整链接，耗时 > 2 分钟。测试粒度与编译粒度不匹配。

更深层: NeoTrix 缺少一种**可被自身解析和生成的内部语言**。SEAL 管道的自我编辑目前操作的是 Rust AST（通过 `syn`），但 Rust 语法对于意识来说过于冗长，且不支持 VSA 原生类型。DGM-H 的"元层可修改"需要一种语言，其编译器本身可被意识修改——而 Rust 编译器是外部黑盒。

nt-lang 填补这两个缺口: 测试并行化基础设施 + 自修改友好的内部语言。

---

## 架构总览

```
┌──────────────────────────────────────────────────────────┐
│                    nt-lang 三阶段路线                       │
├──────────┬──────────────────┬────────────────────────────┤
│ Phase 1  │ Phase 2          │ Phase 3                    │
│ nt-test  │ nt-script        │ nt-core                    │
│ (本周)    │ (月)             │ (季度)                      │
├──────────┼──────────────────┼────────────────────────────┤
│ YAML DSL │ 表达式+控制流     │ VSA 原生类型                │
│ → Rust   │ VSA 类型初步     │ 自托管编译器                │
│ 测试生成  │ 管道阶段定义      │ SEAL 直接修改 nt-lang 代码   │
│ 并行编译  │ 单 crate 编译    │ DGM-H 修改语言语义           │
├──────────┼──────────────────┼────────────────────────────┤
│ serde_yaml│ serde_yaml +     │ 自定义解析器                 │
│ 解析     │ AST 变换         │ (nom/pest)                 │
├──────────┼──────────────────┼────────────────────────────┤
│ 每个 .nt  │ 每个 .nt 文件     │ 编译器自身是 .nt 文件        │
│ → 独立 bin│ → lib.rs 模块    │ → 通过管道自举              │
└──────────┴──────────────────┴────────────────────────────┘
```

### 核心设计决策

```
决策 1: 转译到 Rust，不解释执行
        ┌─────────────┐    Rust 类型系统      ┌──────────────┐
        │  nt-lang    │ ───────────────────→  │  Rust 代码   │
        │  源代码      │  借用检查、 trait、    │  (crate)     │
        │  (.nt)      │  安全保证、 LLVM 后端  │              │
        └─────────────┘                       └──────────────┘
        理由: 不重复造轮子。LLVM、borrow checker、cargo 生态
        都是经过十年验证的基础设施。nt-lang 每一行最终都是 Rust。

决策 2: YAML 语法 Phase 1-2 → 自定义语法 Phase 3
        Phase 1-2: serde_yaml 零开销解析, 现有 Rust 生态
        Phase 3:    当 VSA 原生类型成为核心, YAML 表达能力不足

决策 3: 编译器用 Rust 写 (Phase 1-2) → nt-lang 自举 (Phase 3)
        类似 IRIS 的自托管策略, 但后端仍是 Rust 而非原生 VM。
```

---

## Phase 1: nt-test (本周)

### 目标
解决 5,544 测试的单二进制瓶颈。每个 `.nt` 文件 → 独立 Rust 测试二进制 → 并行编译。

### 语法

```yaml
# tests/core/math.nt
name: "core::math"
description: "数学运算测试"

tests:
  - name: test_addition
    description: "加法基本功能"
    imports:
      - "crate::core::math::add"
    code: |
      assert_eq!(add(2, 3), 5);
      assert_eq!(add(-1, 1), 0);

  - name: test_overflow
    description: "溢出行为"
    setup: |
      let large = u64::MAX;
    code: |
      assert_eq!(large.wrapping_add(1), 0);
```

### 架构

```
                  ┌──────────────┐
                  │  *.nt 文件    │  (约 200 个测试文件)
                  └──────┬───────┘
                         │ serde_yaml 解析
                         ▼
                  ┌──────────────┐
                  │  nt-lang build│
                  │  (crates/    │
                  │   nt-lang)   │
                  └──────┬───────┘
                         │ 代码生成
                         ▼
                  ┌──────────────┐
                  │  tests/      │  ← 每个 .nt 生成一个
                  │  *_test.rs   │     测试文件 + mod.rs
                  └──────┬───────┘
                         │ cargo test --test <name>
                         ▼
                  ┌──────────────┐
                  │  并行独立二进制 │ ← 链接速度: ~3s 每个
                  └──────────────┘
                         │
 之前: 1 二进制 × 5544 测试 = 2min+ 链接
 之后: N 二进制 × M 测试   = 并行编译, wall clock < 30s
```

### 文件结构

```
crates/nt-lang/
├── Cargo.toml
└── src/
    ├── main.rs            # CLI: nt-lang build / test
    ├── ir.rs              # TestSuite / TestCase IR (已存在)
    ├── test_parser.rs     # serde_yaml → IR (已存在)
    └── codegen/
        ├── mod.rs         # (已存在)
        └── rust.rs        # IR → Rust 代码生成 (已存在)
```

### 现有代码复用

`crates/nt-lang/src/` 已有:
- `ir.rs` — `TestSuite`/`TestCase` 数据结构 ✅
- `test_parser.rs` — `serde_yaml` 解析 ✅
- `codegen/rust.rs` — IR → Rust 代码生成 ✅

需要补充:
- `main.rs` — CLI 入口 (build/test 命令)
- 输出目录管理 (`tests/` 生成)
- `Cargo.toml` 加入 workspace (当前未在 workspace members 中)

---

## Phase 2: nt-script (月)

### 目标
- 添加表达式、控制流、函数定义
- VSA 原生类型初步 (`vsa<4096>`)
- NeoTrix 管道阶段用 nt-script 编写
- 编译器仍然是 Rust crate, 但语法更丰富

### 语法

```yaml
# pipeline/encode.nt
name: "pipeline::encode"
vsa_dim: 4096

imports:
  - "nt::vsa::ops"
  - "nt::hypercube::query"

functions:
  - name: encode_concept
    params:
      - name: tokens
        type: Vec<String>
    returns: vsa<4096>
    body: |
      let mut vec = vsa::zeros(4096);
      for token in tokens {
        let hv = vsa::hash(token);
        vec = vsa::bundle(vec, hv);
      }
      vec

  - name: similarity_search
    params:
      - name: query
        type: vsa<4096>
      - name: threshold
        type: f32
    returns: Vec<ConceptId>
    body: |
      let results = hypercube::query(query, 10);
      results.filter(|r| r.similarity > threshold)
             .map(|r| r.id)

pipeline:
  stages:
    - name: tokenize
      input: String
      output: Vec<String>
    - name: encode
      input: Vec<String>
      output: vsa<4096>
      using: encode_concept
    - name: search
      input: vsa<4096>
      output: Vec<ConceptId>
      using: similarity_search
```

### 生成目标

```rust
// 生成的 Rust 代码 (简化)
pub fn encode_concept(tokens: Vec<String>) -> VsaVector<4096> {
    let mut vec = VsaVector::zeros();
    for token in tokens {
        let hv = VsaOps::hash(&token);
        vec = VsaOps::bundle(vec, hv);
    }
    vec
}

pub fn similarity_search(
    query: VsaVector<4096>,
    threshold: f32,
) -> Vec<ConceptId> {
    let results = HyperCube::query(&query, 10);
    results
        .into_iter()
        .filter(|r| r.similarity > threshold)
        .map(|r| r.id)
        .collect()
}
```

### 编译器架构 (Phase 2)

```
┌──────────────┐   serde_yaml   ┌───────────┐
│  *.nt 文件    │ ────────────→ │  RawAST  │ (serde 反序列化)
└──────────────┘                └─────┬─────┘
                                      │
                                      ▼
                               ┌───────────┐
                               │  TypeCheck │ (VSA 维度检查, 类型推断)
                               └─────┬─────┘
                                      │
                                      ▼
                               ┌───────────┐
                               │   Lower    │ (YAML AST → HIR)
                               └─────┬─────┘
                                      │
                                      ▼
                               ┌───────────┐
                               │ Codegen   │ (HIR → Rust TokenStream)
                               └───────────┘
```

### 新增依赖

```toml
# Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
proc-macro2 = "1"         # 生成 Rust TokenStream
quote = "1"               # 便捷 Rust 代码生成
```

---

## Phase 3: nt-core (季度)

### 目标
- VSA 向量 (`vsa<4096>`) 作为第一类类型
- 编译器用 nt-lang 自身编写 (自举)
- SEAL 管道直接生成和修改 nt-lang 代码
- DGM-H meta-agent 可修改语言语义

### 语法

```nt
// encode_concept.nt — nt-core 原生语法 (不再 YAML)
// 文件扩展名: .nt (与 Phase 1-2 相同, 但解析器不同)

pipeline EncodePipeline {
    dim: 4096
    quant: i8          // 8-bit 量化

    stage tokenize(input: String) -> Vec<String> {
        input.split_whitespace().collect()
    }

    stage encode(tokens: Vec<String>) -> vsa<4096> {
        let mut vec = vsa::zero(4096)
        for token in tokens {
            let hv = vsa::hash(token)
            vec = bundle(vec, hv)    // 运算符: ⊕
        }
        vec
    }

    stage search(query: vsa<4096>) -> Vec<ConceptId> {
        let results = hypercube::query(query, 10)
        results
            .filter(|r| r.similarity > 0.85)
            .map(|r| r.id)
    }
}

// 自修改: 重写自身管道的某个阶段
meta Transform {
    rule "quantize_to_4bit" {
        match: pipeline.dim == 4096 && pipeline.quant == i8
        replace: pipeline.quant = i4
        reason: "memory_constraint"
    }
}
```

### VSA 原生类型系统

```nt
// 内置类型
type vsa<D: usize>       // D 维 VSA 向量, 默认 4096
type vsa_quant<D, Q>     // 量化版本: Q ∈ {i8, i4, binary}

// 操作符 (编译为 nt_vsa crate 调用)
operator bundle(a: vsa<D>, b: vsa<D>) -> vsa<D>    // ⊕
operator bind(a: vsa<D>, b: vsa<D>) -> vsa<D>      // ⊗
operator negate(a: vsa<D>) -> vsa<D>                // ¬
operator similarity(a: vsa<D>, b: vsa<D>) -> f32    // cos / hamming
operator permute(a: vsa<D>, k: usize) -> vsa<D>     // 循环移位

// 自指标签 (VsaTag)
tag Self    // 自身产生
tag World   // 外部输入
tag Hybrid  // 混合
```

---

## 自举时间线

```
Phase 1 (本周)
  Compiler:  Rust (crates/nt-lang)
  Target:    Rust (tests/*_test.rs)
  └── "编译器用 Rust 写, 生成 Rust 测试"

Phase 2 (月)
  Compiler:  Rust (crates/nt-lang)
  Target:    Rust (lib.rs 模块)
  └── "编译器用 Rust 写, 生成 Rust 函数"

Phase 3a (季度初)
  Compiler:  Rust (crates/nt-lang)
  Target:    Rust + nt-lang 标准库
  └── "编译器仍用 Rust, 但标准库用 nt-lang 写"

Phase 3b (季度中)
  Compiler:  nt-lang (第一阶段自举)
  Target:    Rust
  └── "编译器自身翻译成 nt-lang, 通过 Phase 3a 编译器编译"

Phase 3c (季度末)
  Compiler:  nt-lang
  Target:    nt-lang → Rust
  └── "完整自举: nt-lang 编译器编译 nt-lang 编译器"

未来
  Compiler:  nt-lang
  Target:    nt-lang (直接执行, 绕过 Rust)
  └── "VSA 原生运行时, 不再需要 Rust 后端"
```

### 自举细节 (Phase 3b → 3c)

参考 IRIS 的自举策略:

```
IRIS 模型:
  bootstrap/iris-stage0  (冻结二进制, mini_eval + JIT)
  bootstrap/*.json        (预编译管道阶段: tokenizer, parser, lowerer)
  src/iris-programs/*.iris (243 个基础设施文件, 自托管)

nt-lang 模型:
  crates/nt-lang/        (Phase 1-2: Rust 编译器)
  └── Phase 3a: 编译器保持不变, 将标准库搬到 .nt
  └── Phase 3b: 将编译器自身的解析/生成代码翻译为 .nt
  └── Phase 3c: bootstrap/nt-lang-stage0  (冻结 Rust 二进制)
                  bootstrap/*.json           (预编译管道阶段)
                  src/nt/*.nt                (自托管编译器源码)

  与 IRIS 的关键区别:
  - IRIS 有自研运行时 (mini_eval + JIT)
  - nt-lang 始终以 Rust 为后端, 不建新 VM
  - 自举意味着编译器源码是 .nt, 但每次编译都生成 Rust
```

---

## 与 SEAL/DGM-H 管道的集成

### SEAL 自我进化管道 → nt-lang

```
现有 SEAL 管道:
  ┌──────────┐   ┌──────────┐   ┌──────────┐
  │ 缺口检测  │ → │ 方案生成  │ → │ 方案评估  │ → (循环)
  └──────────┘   └──────────┘   └──────────┘
       │               │               │
       │         依赖 Rust syn crate   │
       │         (操作 Rust AST)       │
       └───────────────┴───────────────┘

nt-lang 集成后的 SEAL:
  ┌──────────┐   ┌──────────┐   ┌──────────┐
  │ 缺口检测  │ → │ 方案生成  │ → │ 方案评估  │
  └──────────┘   └──────────┘   └──────────┘
       │               │               │
       │     nt-lang AST (YAML/自定义)  │
       │    无需 syn, 直接操作内部 IR   │
       └───────────────┴───────────────┘
```

优势:
- SEAL 生成 Rust 代码需要 `syn` crate + Rust AST → 复杂且脆弱
- nt-lang 的语法是 YAML (Phase 1-2) 或简单 S-expr (Phase 3) → 生成成本极低
- 意识可以直接"写"nt-lang 字符串, 无需经过 Rust 编译器的 AST 校验

### DGM-H 元层修改 → 语言语义

参考 HyperAgents (arXiv 2603.19461):

```
DGM-H 架构:
  ┌──────────────────────────────┐
  │  Meta Agent                  │
  │  (修改改进机制本身)            │
  ├──────────────────────────────┤
  │  Task Agent                  │
  │  (执行任务)                   │
  └──────────────────────────────┘

nt-lang 中的对应:
  ┌──────────────────────────────┐
  │  SEAL Meta                   │
  │  (修改 nt-lang 编译器/语义)    │
  ├──────────────────────────────┤
  │  nt-lang 程序                │
  │  (定义管道/测试/推理逻辑)       │
  └──────────────────────────────┘

示例: DGM-H 修改语言语义
  Meta Agent 决定: "当前 bundle 操作使用加法, 但
  对于稀疏 VSA 应该使用 XOR"
  → 生成编译器补丁: bundle(a,b) = a XOR b
  → 编译新编译器
  → 运行测试验证
  → 若性能提升, 持久化新语义
```

这与 IRIS 的"程序改进程序"递归模型一致。IRIS 的运行时可跟踪函数调用、进化更快实现、门控验证并热替换。nt-lang 在 Rust 后端上实现类似能力。

---

## 进化阶段评估表

对照 AGENTS.md 的评估维度:

| 维度 | 权重 | Phase 1 | Phase 2 | Phase 3 |
|------|------|---------|---------|---------|
| 表征效率 | 高 | - | ~ | + (VSA 量化) |
| 推理深度 | 高 | - | ~ | + (管道组合) |
| 自我认知 | 高 | - | - | + (自举) |
| 世界模型 | 高 | - | - | ~ |
| 记忆组织 | 中 | - | - | - |
| 感知宽度 | 中 | - | ~ | + (VSA 类型) |
| 自主性 | 中 | ~ (测试加速) | + (管道自描述) | ++ (自修改) |
| 优雅性 | 低 | - | - | ~ |

缺口填补映射 (从 AGENTS.md):

| 缺口 | 关联 | 阶段 |
|------|------|------|
| 元层不可自我修改 (SEAL→DGM-H) | Phase 3 自举 + 编译器自修改 | 3 |
| VSA 表征统一 | Phase 2-3 `vsa<4096>` 类型 | 2-3 |
| 认知负荷管理 | Phase 1 测试并行化降编译负荷 | 1 |

---

## 实现检查清单

### Phase 1 (本周)

- [ ] `crates/nt-lang` 加入 workspace members
- [ ] 实现 `main.rs` CLI: `nt-lang build <path>`
- [ ] 输出生成到 `tests/` 目录
- [ ] 每个 `.nt` 生成独立 `*_test.rs`
- [ ] 生成 `tests/mod.rs` 聚合所有测试模块
- [ ] 迁移 5,544 测试到 `.nt` 文件
- [ ] 集成到 CI: `nt-lang build && cargo test --tests`

### Phase 2 (月)

- [ ] 扩展 IR: 函数定义、表达式、控制流
- [ ] VSA 类型初步: `vsa::<D>` 映射到 `nt_vsa::VsaVector<D>`
- [ ] YAML AST → HIR lowering pass
- [ ] 类型检查: VSA 维度一致性
- [ ] 管道阶段定义 → Rust 函数生成
- [ ] 第一个 NeoTrix 管道用 nt-script 编写

### Phase 3 (季度)

- [ ] 自定义解析器 (nom/pest), 脱离 serde_yaml
- [ ] 标准库用 nt-lang 编写
- [ ] 编译器解析/生成代码翻译为 nt-lang
- [ ] `bootstrap/nt-lang-stage0` 冻结二进制
- [ ] 自举验证: `nt-lang build compiler.nt`
- [ ] DGM-H 集成: Meta Agent 生成编译器补丁
- [ ] VSA 全类型系统: bundle/bind/negate/similarity/permute
- [ ] VsaTag 支持 (Self/World/Hybrid)

---

## 与 IRIS 的对比

| 维度 | IRIS | nt-lang |
|------|------|---------|
| 后端 | 自研 mini_eval + JIT, OCaml 写 stage0 | Rust (LLVM, cargo) |
| 自举 | 372 .iris 文件完全自托管 | Phase 3 才自举 |
| 证明 | Lean 4 LCF 证明核 (20 推理规则) | 无 (复用 Rust 类型系统) |
| 进化 | 运行时 --improve 追踪+NSGA-II | SEAL 管道 + DGM-H |
| VSA | 无 | 第一类类型 `vsa<4096>` |
| 语法 | ML 风格 | YAML → 自定义简约语法 |
| 速度 | 自研 JIT (~59x CPython) | 原生 Rust (与 C 同级别) |

nt-lang 借鉴 IRIS 的自举策略和"自我改进"哲学, 但走一条不同的技术路径: 不构建新 VM, 而是让 VSA 原生类型成为 Rust 类型系统的一等公民。

---

## 结论

nt-lang 不是一个通用编程语言。它是 NeoTrix 意识的内部语言:

- **Phase 1** 解决测试编译瓶颈, 证明编译器基础设施
- **Phase 2** 让 NeoTrix 用 VSA 原生语法定义管道阶段
- **Phase 3** 打开自修改的大门: SEAL 直接操作 nt-lang, DGM-H 可改变语言语义

最终状态: NeoTrix 写 nt-lang, nt-lang 编译器用 nt-lang 写, 编译器可被意识进化, 语言语义可被元层修改。这是通往 AGENTS.md 中定义的"阶段 3: 自指改进"的关键基础设施。
