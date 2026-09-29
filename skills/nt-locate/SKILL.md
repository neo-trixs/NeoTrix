---
name: nt-locate
description: 精准定位到代码点再下刀 — 选择器/组件名/sourceFile → 文件:行三层定位，微操作闭环
origin: NeoTrix
triggers: 定位, 定点, 微操作, 精准修改, locate, micro-edit, pinpoint, 定点修改
condition: task:edit
---

# nt-locate — 精准定位微操作（Agentation 思想吸收）

人类点选/标注必须落地到可微操作的代码点。改码前先定点，读上下文再下刀。

## 工具

`scripts/ops/nt_locate.py`（只读零依赖，rg 优先）：

```bash
python3 scripts/ops/nt_locate.py --component CrystalIterState --source-file handlers_crystal.rs --root .
python3 scripts/ops/nt_locate.py --selector ".sidebar > button.primary" --root .
python3 scripts/ops/nt_locate.py --selftest   # 自检（含 AST 层）
python3 scripts/ops/nt_locate.py --no-ast ... # 关 AST（纯 grep，更快）
```

## 四层定位（cocoindex AST 思想吸收）

- L1（100 分）：sourceFile basename 直达文件，行内再定 token 行
- L2（50+ 分）：token 覆盖度排名
- L3（10 分）：selector 原串全文兜底
- **L4（AST，+25/定义行 120 分）**：tree-sitter 解析 .rs/.py，命中行标最内层作用域
  （`fn/impl/struct`），component 命中定义名直接给定义行；缺包自动降级

- L1（100 分）：sourceFile basename 直达文件，行内再定 token 行
- L2（50+ 分）：选择器拆 token，按覆盖度排名
- L3（10 分）：selector 原串全文兜底

## 微操作闭环

```
点选/标注 → nt_locate 定点 → 读上下文（R-P16 先读后写）→ 最小改动
  → rustfmt 自查 → 单测/cargo 验证 → handoff 记账
```

## 铁律

- 无定点不下刀：拿不到 file:line 就继续找，不许凭印象改
- Agentation 标注到来时：annotation(JSON/Markdown) → `agentation_to_cards.py` 入库 →
  本 skill 按 element/component 定点 → 修 → 单测
