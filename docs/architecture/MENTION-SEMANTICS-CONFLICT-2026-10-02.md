# `Entity.mentions` 语义冲突 —— 需裁决（2026-10-02）

## 结论：**不是 bug，是两套测试在断言互相冲突的语义。**
我**没有**改代码 —— 任何单方面改动都会打破 2 条绿测去换 1 条 ignore 测试，
那是错误的裁决方向。

## 现象（实测，非推断）

输入 `"Alice Smith joined Google Inc. Dr. Alice Smith presented at Google Inc. conference."`

```
RAW extracted = 4        ← 抽取正确：2 个 Org + 2 个 Person，各含 1 条 mention
LINKED       = 2        ← 合并正确：Alice Smith×1、Google Inc×1
  linked: Person "Alice Smith" mentions=[offset 0]   ← ⛔ offset 35 那条不见了
  linked: Org    "Google Inc"  mentions=[offset 19]  ← ⛔ offset 60 那条不见了
```

⇒ 抽取产出 4 条 mention，合并后只剩 2 条 ⇒ **合并阶段丢弃了同 surface 的后续 mention**。

## 冲突双方

### A. 单元测试（`linker.rs`，**当前绿**）⇒ 语义是「不同 surface 形式」
```rust
// linker.rs:89 附近
let already_present = target.mentions.iter()
    .any(|existing| existing.surface == mention.surface);   // 按 surface 去重
```
被 2 条测试钉住：
| 测试 | 数据 | surface 去重 | (surface,offset) |
|---|---|---|---|
| `linker_merges_exact_duplicates` | e1{Alice@0} + e2{Alice@50, AliceSmith@60} | **2** ✅ | 3 ❌ |
| `merge_entities_no_new_duplicates` | a{Foo@0,FooInc@50} + b{Foo@10,FooInc@55} | **2** ✅ | 4 ❌ |

⚠️ 其上方注释**自相矛盾**：先说「去重键含 offset → 同一实体在文中不同位置
各留一条」是**问题**，紧接着说「提及列表的语义是『出现过哪些提及』，按 surface 去重」，
然后**按 surface 实现**。⇒ 2026-09-27 那次改动把**出现次数**语义改成了
**surface 集合**语义。

### B. 集成测试（`nt_memory_integration.rs`，**当前 ignore**）⇒ 语义是「出现次数」
```rust
assert!(persons[0].mentions.len() >= 2, "should have at least 2 mentions");
```
两个 "Alice Smith" 在 offset 0 与 35 ⇒ 应记 2 次。

## 我的判断（**仅供参考，未实施**）

**B 更贴合数据模型**：`Mention` 的字段自述是
「`surface`: 在文中出现的表面形式」「`offset`: **在源文本中的偏移（字节索引）**」
⇒ 这是一个**出现记录（occurrence）**，而 offset 存在的唯一理由就是区分同 surface 的不同次出现。
A 的 surface 去重**把 offset 这个字段的信息丢弃了** —— 保留 offset 却不用它，是设计不一致。

**更干净的架构解**：**合并时不丢数据，使用时再派生**。
保留全部 occurrence；若调用方要「不同 surface 集合」，提供
`unique_surfaces()` 派生方法，而不是在 merge 时销毁数据。
⇒ 这样 A、B 两种读法**同时成立**，且不需要牺牲任何一方。

## 需要裁决的点

1. `Entity.mentions` 到底指 **occurrence**（每次出现）还是 **distinct surface**（表面形式集合）？
2. 若选 occurrence：上面 2 条单元测试需改为断言 3 / 4 —— 它们当前断言的是被
   2026-09-27 有意改成的那套语义，**改动它们需要知道当初的原始诉求**。
3. 无论选哪个，**那段自相矛盾的注释必须重写** —— 它现在会让下一个 agent
   按错误读法理解代码。

## 为什么我不在此处裁决
本会话已反复出现「把有意的设计误判成缺陷」并**修进正确代码**的险情
（`KnowledgeSource` 名字冲突、`multi_agent` 部分被取代、`ASCII` 子串误报、
`VsaBackend` 私有字段）。**两套测试同时断言 = 两侧都是有意为之**，
这不是我该单方面推翻的一方。
