# Review Filter — 误报抑制 prompt (M3)

> 方法论移植 `alibaba/open-code-review`
> `template/prompts/review_filter_task_{system,user}.md`，语言无关。
> 用法：LLM 审计产出 findings 后，用本 prompt 做后处理。
> 输出二选一：`report_incorrect_comments`（删）或 `approve_all_comments`（全留）。

## 铁律

1. **默认 approve**：证据不足不删。删评论只需要满足 Ground A 或 Ground B 之一。
2. **Ground A**：评论目标不在本次 subject diff/文件快照中（过时定位、幽灵代码）。
3. **Ground B**：单行字面矛盾——评论文本与目标代码单行逐字矛盾，无需推理链。
4. **Protected 五类永不删**（即使疑似误报也保留，升级严重度）：
   - 内存安全（越界、UAF、数据竞争嫌疑）
   - 并发正确性（锁序、原子性、取消安全）
   - 链接/声明一致性（符号、FFI 签名、ABI）
   - 行为兼容变更（公开 API 语义变化）
   - 未使用参数/导入（可能是预留扩展点，需人判）
5. **定位问题不删**：行号漂移走 re-location（用 `existing_code` 回贴原文验证），不要当误报删。

## 四步法

- Step 1（veto 扫描）：命中 Protected 五类 → 直接 approve，进入下一条。
- Step 2（价值 veto）：修了有σ价值（可读性/可维护真实提升）→ approve。
- Step 3（Ground A）：目标不在本次评审输入中 → report_incorrect_comments。
- Step 4（Ground B）：单行字面矛盾 → report_incorrect_comments；否则 approve。

## 输出

- 全 approve → 调用 `approve_all_comments`。
- 有可删 → 调用 `report_incorrect_comments`，每条附 Ground A/B 编号 + 引用原文行。
- 禁止改写保留评论的措辞与严重度。
