# nt-lang

## Purpose
NeoTrix 语言编译器

## Trigger Words
- nt-lang
- language
- compiler
- 编译器
- DSL
- 领域特定语言

## Content
- 中间表示 (IR)
- 代码生成 (codegen)
- Rust 后端
- 测试解析器

## Commands
```bash
nt-lang build <file.nt>    # 编译单个文件
nt-lang build-all          # 编译所有文件
```

## Location
⛔ **`crates/nt-lang/` 已于 `2bbed32c`（2026-09-28）删除**（5 文件/273 行，删前状态：
只有 `[[bin]]` 无 `[lib]`、0 个主树 manifest 依赖 ⇒ 孤儿）。本 skill 保留为
**历史说明**，下列 `## Commands` **当前不可用**。

## 当前可用部分
DSL 夹具仍在库：`neotrix-core/test_suites/{vsa_quantized,cross_modal,vsa_engine}.nt`
（3 文件）。这些是 NT 语言的输入样本，但**当前没有编译器能编译它们** ——
`neotrix-core/test_suites/` 无对应的 runner 消费它们。

## 若要复活
补 `[lib]` + 接线到主树 manifest，或另建 crate。设计与路线见
`docs/plans/2026-09-20-nt-lang-evolution-roadmap.md`（「测试生成器 → 声明式 DSL」
的在制品定位）。⛔ 历史上曾有一条「不要删」的裁决，但晚到一天、被删除覆盖。
