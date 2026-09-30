# 建议 1 可行性验证：调用边（2026-09-30 实测）

> 目的：在动手前验证「委托给编译器拿调用边」这条路在本机是否走得通。
> **纪律：已证 / 未证分开写。** 任何未实证的能力不写成结论。
> 环境：`rustc 1.94.0 (Homebrew)` · **无 rustup · 无 nightly** · `RUSTC_BOOTSTRAP=1` 可用。

---

## 1. 结论先行

| 问题 | 答案 | 证据 |
|---|---|---|
| 本机能否拿到编译器自己的解析结果？ | ✅ **能** | 见 §2、§3 |
| 需要 nightly / rustup 吗？ | ✅ **不需要** | `RUSTC_BOOTSTRAP=1` 在 stable 上解锁 `-Z`，实测通过 |
| 同名诱饵能否混淆？ | ✅ **结构上不可能** | 目标是带 crate 哈希的 `DefId` |
| 现有文本传输能否规模化？ | ❌ **不能** | 31 KB/源码行 ⇒ 全树约 **27 GB** |
| 边能否用正则抽出来？ | ❌ **未证，且我抽错了** | 见 §5 |

⇒ **机制成立，传输层与抽取器是真工程，不是配置项。**

---

## 2. 已证 ①：rustdoc JSON（无需 nightly）

```
RUSTC_BOOTSTRAP=1 cargo rustdoc -- -Z unstable-options --output-format json
  → target/doc/nt_probe.json
```

- `format_version: 57`，含 `root` / `index` / `paths` / `external_crates` / `target`
- `paths` 是**全仓解析基础**：`{id} → {path: [...], kind: ...}`，
  样例 `['nt_probe','inner','deep'] kind=function`
- **⛔ 但不含函数体**：`ItemEnum::Function` 只有
  `sig` / `generics` / `header` / `has_body: true`
  ⇒ **它是签名索引，不是体索引**。靠它拿不到调用边。

## 3. 已证 ②：typed HIR 含编译器已解析的调用目标

```
RUSTC_BOOTSTRAP=1 cargo rustc --lib -- -Zunpretty=hir-tree
```

**真实排版**（实测抄录，不是推测）：

```
Call(
    PathSegment {
        ident: to_lowercase#0,
        res: Err,                       ← 该处未解析
        ...
    },
    Expr { kind: Path( Resolved( None, Path { res: Local(...), ... } ) ) },
)

res: Def(
    Mod,
    DefId(20:0 ~ serde[cf58]),           ← 编译器自己的解析，带 crate 哈希
)
```

- `res:` 取值分布（扫前 20 MB）：`Def` 8164 / `Local` 1136 / `Err` 294 /
  `SelfTyAlias` 288 / `PerNS` 218 / `PrimTy` 210
- **`res: Def` 全文 34,478 处**
- 目标形如 `DefId(0:3 ~ nt_probe[05df]::leaf)`
  ⇒ **同名诱饵（`CrystalCycle` 那种）在结构上不可能发生**
  ⇒ 这正是外部 `codegraph` 的命题「委托，不重写」在 Rust 上的对应物

## 4. 已证 ③：文本传输不可规模化

| crate | 源码行 | hir-tree 体积 | 耗时 |
|---|---:|---:|---:|
| `nt_probe`（玩具） | 13 | 41.5 KB | <1 s |
| `nt-core-capability-tree` | 4,858 | **151 MB**（另测 144 MB） | 8.12 s（含依赖编译） |

⇒ **≈31 KB / 源码行**。按此推算 `neotrix-core`（**867,000 行**）
⇒ **约 27 GB** 文本。

- 作为**一次性生成物**并流式处理（不落盘）：时间上可接受（151 MB / 8 s 是编译主导，
  非导出主导）
- 作为**CI 每次跑**：不可接受
- ⇒ 需要更紧凑的传输：`.rmeta`（二进制，含完整体 + 已解析 DefId）
  或 rust-analyzer 自己的索引。**本次未测成**（`--emit=metadata -o …` 未产出独立文件，
  stderr 未展开即中止），**故不写成结论**。

## 5. 未证 ④：抽取器我做错了，这本身就是结论

我先写了一个行级正则 filter（假设 `id: DefId(…)` 紧邻 `Call(`），在
4,858 行、源码内 225+ 处 `::fn(` 形态调用的 crate 上：

```
#ITEMS 1269   ← 归属识别正常
#CALLS 0      ← 边一条没抽到
```

**这不是「没有边」，是 filter 找错了字段**（真实解析在 `PathSegment.res`，
不在 `id:` 上）。按本仓纪律：**零命中不构成不存在的证据** —— 我今天已经因此
误判过两次。

⇒ **结论：HIR 文本格式需要真正的解析器，不是行正则。**
这是本建议的主要工作量，也是它**没有被本轮完成**的原因。

---

## 6. 修正本文档此前的措辞

`CAPABILITY-GAP-2026-09-30.md` §建议 1 原写：

> 「先探 `cargo +nightly rustc -Zunpretty=…` 与 `RUSTC_BOOTSTRAP=1 cargo rustc
> -- -Zemit=…` 是否能在本机拿到编译器自己的元数据（**先验证可行再动手**）」

验证已完成，答案需要修正两点：

1. **不需要 `+nightly`** —— 本机根本没有 rustup，`RUSTC_BOOTSTRAP=1` 已足够。
   原措辞会让人以为要装 nightly。
2. **「拿到元数据」成立，但「据此建图」不成立** —— 元数据里**确实有**编译器解析
   （34,478 处 `res: Def`），但**把它变成边表**需要一个解析器，且文本传输
   按全树规模不可用。原措辞把「机制成立」说成了「路径已定」，**过于乐观，已更正**。

## 7. 下一步的判据（可证伪）

若继续做，第一道门应是：

> 解析器在 `nt-core-capability-tree`（4,858 行、源码 225+ 处 `::fn(` 形态）上
> 抽出的边数，必须与**人工清点**的同口径边数一致（±2%）。
> 抽不出 ⇒ 换 `.rmeta`/rust-analyzer 路线，不要在 HIR 文本上继续投入。

## 8. 与外部资料的对应

| 外部做法 | 本机对应 | 状态 |
|---|---|---|
| `codegraph` 用 `scip-typescript` / `go/packages` 拿边 | `-Zunpretty=hir-tree` 的 `res: Def` | ✅ 机制已证，抽取未证 |
| 「端点非真节点就丢弃，宁缺勿错」 | `res: Err` / `Local` 自然落到「未解析」侧 | ✅ 可直接照抄 |
| `arXiv 2606.22417`：跨 ≥3 文件改动收益最大 | 本仓主要工程动作恰在此档 | ✅ 价值判断不变 |
| `grafel`：39 语言、跨仓、消息总线拓扑 | 本仓 11 crate 无跨边 | ⛔ 仍是缺口 |

未推送：按指令仅本地提交。

---

## 附录 · 抽取器实测：排除了哪些字段（2026-09-30 追加）

在 6 MB HIR 样本上逐个试过，**三个候选字段全部否证**：

| 假设 | 样本实测 | 否证 |
|---|---|---|
| `id: DefId(…)` 紧邻 `Call(` | 边 0 条 | 该字段在该格式中不承载调用目标 |
| `PathSegment.res` | 边 0 条 | 它是**段级**解析，样本中为 `res: Err`（未解析）；不是调用目标 |
| `Path.resolved_paths` | **0 次出现** | 该字段在此 HIR dump 格式中**不存在** |

同时纠正一处计数口径错误：`Call(` 的字符串计数会把 `MethodCall(` 一并算进去
⇒ 必须用 `(?:Method)?Call\(` 且**子串匹配**（`Call(` 不在行首；
我第一次用 `startswith` 又漏了，本会话同类失误第 5 次）。

样本统计（6 MB）：`(?:Method)?Call(` 60 处 · `res: Def(` 2,724 处
⇒ **`res: Def` 的绝大多数不在调用块内**，它们来自 use 路径 / 类型 / impl 项。
所以「`res: Def` 很多」**不能**推出「调用边容易抽」。

### 结论（比上一版更具体）

- ✅ 已证：编译器解析**确实在 dump 里**（`res: Def(Kind, DefId(crate ~ path))`）
- ⛔ 未证：**调用目标在哪个字段**
- ⇒ 这已是**解析器级**任务，不是正则任务。
  继续在文本上试字段，等于在 144 MB 上盲搜，成本不可控。

### 下一位接手者的具体起点（省掉我这次试错的 3 个字段）

1. 先在一份**极小**输入上定位字段（玩具 crate，41 KB，秒级迭代），
   不要在 144 MB 上试 —— 我这次用大样本是低效的起点。
2. 判据用 **`Call(` 总数 vs 抽出边数之比**，自证而不必人工清点：
   抽 0 条 = 提取器坏；抽到 Call 数的高比例 = 可用。
3. 若字段定位仍不成，转 `.rmeta`（二进制）或 rust-analyzer 索引，
   **不要**继续在 HIR 文本上投入。

---

## 附录 · `check-silent-failure` 的**实测盲区**（2026-09-30）

量化（不是推测）：把 `GATED` 的 `fs::` / `kv_set` 形态换成**方法调用形态**
（`self.x(...)`、`kb.y(...)`）后，新增候选 **304 处**。

其中**命中全量审计已点名的 6 个高危点**：

| 位置 | 丢弃的调用 | 审计结论 |
|---|---|---|
| `tiered_memory/tier_archival.rs:305` | `self.remove(id)` | `prune()` 返回候选数而非删除数 ⇒ 报告虚假删除量 |
| `nt_memory_knowledge_assets.rs:321/398/473` | `kb.update_node_metadata(...)` | 丢弃后无条件 `report.imported += 1` ⇒ **导入报告显示全部成功** |
| `nt_world_github_absorber.rs:730` | `self.kb.upsert_edge(...)` | GraphRAG 关系边丢失 ⇒ 多跳查询静默少召回 |
| `nt_memory_pipeline.rs:687` | `kb.upsert_edge(...)` | 同上 |

⇒ **审计发现真 bug 的那一类形态，恰好是本门覆盖不到的那一类。**
这不是巧合：门按「看起来像落库/落盘」写，审计按「丢弃后调用方会误信什么」找。

### 为什么不无脑扩到 304

304 里含大量**非 `Result` 返回**的调用（`cache.get_semantic(&emb)`、
`self.entries.pop_front()`、`server_handle.join()` 语义各异）。
直接扩 ⇒ 噪声门 ⇒ 又是「亮而无效」。

### 已做与建议

- ✅ 已修 `knowledge_assets.rs:321`（同函数 7 行之上就有正确范式，
  `report.errors.push` + `continue`，此处是漏网）
- ⛔ 其余 298 处**不在本轮动** —— 逐条判「丢弃后调用方是否会误信」是语义判断，
  不是模式匹配；与本轮否掉「死代码门」「doc 承诺门」同理。
- 📌 若要做，判据必须是「**调用方是否据此改变行为**」，
  而非「这是不是落库调用」。
