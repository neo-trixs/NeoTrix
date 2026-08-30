# CollectionMerge 通用抽象设计 (2026-08-20)

## 1. 背景与目标

矩阵缺口 (file-kind-operation-matrix) 显示 docx/pptx **无多文档合并**。Excel 已闭环
的范式 (SheetMode 三策略 + SchemaStore 数据化 + 统一 CLI/意识核心入口) 应抽象为
**CollectionMerge 通用模式** (R-P42: 强化现有节点, 非平行适配器) — 让 docx/pptx/pdf
合并走同一接口, 而非各写一套。

本文档为**设计落地**, 不含代码。落地方案需等并行会话释放 neotrix-core 编译权。

## 2. 统一接口设计

```
CollectionMerge<T>:
  inputs:  &[PathBuf]              // 文档集合
  strategy: MergeStrategy          // FirstOnly / Preferred / AllSheets 泛化
  schema:  Option<MergeSchemaJson> // 结构化提取规则 (复用 SchemaStore)
  output:  PathBuf
```

策略泛化 (复用 SheetMode 三模式, 语义对齐):
| SheetMode | CollectionMerge 语义 |
|-----------|----------------------|
| `FirstSheet` | 只取首个文档 (透传) |
| `Preferred`  | 同名资源冲突时取"修改版"优先 |
| `AllSheets`  | 全部文档拼接 (合并) |

分层实现 (按格式能力选择深度):
- **L0 文本级**: 提取纯文本 → 拼接 → 输出 txt/md (所有格式可用, 无结构保真)
- **L1 结构级**: OPC part 合并 (docx/pptx) / 对象图重建 (pdf, 已闭环)
- **L2 语义级**: DoclingDocument 统一文档模型 (文献调研建议, 未来)

## 3. 可行性分析 (实证)

### docx (L1 结构级 — 可行, OPC part 层)
office_oxide `EditablePackage` (core/editable.rs) 已公开:
- `get_part` / `set_part` — part 读写
- `part_rels` — part 关系读取
- `content_types` — 内容类型 (只读)

**合并算法**:
1. 以首文档为基座, 打开其 EditablePackage
2. 对后续文档: 读 `document.xml`, 提取 `<w:body>` 全部 `<w:p>` 子节点, 追加到基座 `<w:body>`
3. **资源冲突**: 各文档 `word/media/*` 需重命名 (前缀 `docN_`) 避免覆盖 — 在
   zip/part 层重写 `document.xml.rels` 引用 + 新增重命名 part
4. 合并 `word/styles.xml` (缺失样式定义) / `word/fontTable.xml`

**关键阻塞点**: `Relationships` (core/relationships.rs) 为只读 (无 add),
`ContentTypes` 只读 — 资源 part 新增 + rels 重写需:
- (a) 在 neotrix-core 用独立 zip 库 (zip crate) 直接操作 OPC, 或
- (b) 给 office_oxide 提 PR 扩展 EditablePackage 的 add_part/add_relationship
  (R-P79: 吸收即生产接线, 但依赖外部 crate 发布周期)

**风险**: 无合并单元格语义保留 (docx 段落级拼接已满足 90% 场景);
分节符 (`<w:sectPr>`) 需处理避免页面设置冲突。

### pptx (L1 结构级 — 可行, 幻灯片级粒度)
- 读 `ppt/slides/slideN.xml` + `_rels/slideN.xml.rels`
- 合并 = 追加 slide part + 更新 `ppt/presentation.xml` 的 `<p:sldIdLst>`
- 资源重命名同上; 母版/主题复用基座
- **粒度**: 整幻灯片追加 (无跨幻灯片内容拼接), 比 docx 更简单

### pdf (L1 — 已闭环 2026-08-20)
`merge_pdfs` 已实现: 对象图重建 + 单一 Catalog/Pages。CollectionMerge 接口
包一层即可统一入口。

### xlsx (L1 — 已闭环)
`merge_tables_with_mode` + SheetMode + SchemaStore 已是 CollectionMerge 原型,
保留为 Excel 特化入口, 通用接口转发。

## 4. 接口形态 (Rust)

```rust
// nt_file_ability/merge.rs 新增
pub enum MergeStrategy { FirstOnly, Preferred, All }

pub struct CollectionMergeRequest {
    pub inputs: Vec<PathBuf>,
    pub strategy: MergeStrategy,
    pub schema: Option<MergeSchemaJson>,
    pub output: PathBuf,
}

pub enum MergeOutcome {
    /// (格式, 已合并条目数, 输出说明)
    Text { items: usize, note: String },
    Docx { items: usize, parts: usize },
    Pptx { slides: usize },
    Pdf { pages: usize },
    Xlsx { rows: usize, note: String },
}
```

分发规则 (按输入格式):
- 全部 Pdf → merge_pdfs (已闭环)
- 全部 xlsx/csv/tsv → merge_tables_with_mode (已闭环)
- 全部 docx → merge_docx_opc (本设计 L1 落地)
- 全部 pptx → merge_pptx_opc (本设计 L1 落地)
- 混合格式 → 回退 L0 文本级拼接 (extract_text 聚合)

## 5. 实施顺序 (依赖关系)

| # | 任务 | 依赖 | 预估 |
|---|------|------|------|
| 1 | CollectionMergeRequest 接口 + 分发器 (转发已闭环路径) | 无 | 小 |
| 2 | docx L1: 段落级 body 拼接 + sectPr 处理 | zip part 层可用性确认 | 中 |
| 3 | pptx L1: slide 追加 + presentation.xml 更新 | 2 的资源重命名复用 | 中 |
| 4 | 资源冲突重命名 (docN_ 前缀) + rels 重写 | 2/3 共通 | 中 |
| 5 | 混合格式 L0 回退 (extract_text 聚合) | 无 | 小 |
| 6 | 单测: 三策略 + 资源冲突 + 多文档 + 失败路径 | 2-5 | 按 schema_tests 10 例模式 |
| 7 | CLI `/file merge` (统一入口) + 意识核心 `doc_merge` 路由 | 1-6 | 小 |
| 8 | office_oxide 上游 PR (EditablePackage::add_part) 或 zip crate 依赖评估 | 2 | 外部 |

## 6. 风险与缓解

| 风险 | 缓解 |
|------|------|
| office_oxide Relationships 只读, 无法写 rels | 方案 (a) 独立 zip 操作 OPC; (b) 上游 PR。推荐先 (a) 做原型 (zip crate 已是传递依赖) |
| docx sectPr 分节冲突 | 合并时剥离后续文档首节 sectPr, 保留基座 |
| 资源 part 重命名后 rels 目标失效 | 逐文档重写 document.xml.rels, 冲突媒体加 docN_ 前缀 |
| 并行会话编译阻塞 | 本设计待编译权释放后落地; 先完成接口/文档 (非侵入) |

## 7. 架构一致性 (矩阵建议 #1 落地)

CollectionMerge = 矩阵第 5 节"范式归一"的具体化: 一个入口, 按格式分派到
既有闭环 (xlsx/pdf) + 新闭环 (docx/pptx), 全部登记 CapabilityRegistry +
意识核心路由 (Dark Forest: 有消费者)。schema 领域化继续走 SchemaStore
(每文件类型独立 schema.json)。

## 8. 实施状态 (2026-08-21 更新)

| # | 任务 | 状态 | 验证 |
|---|------|------|------|
| 1 | CollectionMergeRequest 接口 + 分发器 | ✅ 完成 | 单测通过 (pdf/docx/pptx/xlsx/混合/L0) |
| 2 | docx L1: 段落级 body 拼接 + sectPr 处理 | ✅ 完成 | 3 单测 + 真实 DOCX 144 段落 |
| 3 | pptx L1: slide 追加 + presentation.xml 更新 | ✅ 完成 | 单测 + 真实 PPTX 38 slides |
| 4 | 资源冲突重命名 (docN_ 前缀) + rels 重写 | ✅ 完成 | 媒体冲突测试通过 + rId 重编号验证 |
| 5 | 混合格式 L0 回退 (extract_text 聚合) | ✅ 完成 | pdf+docx 混合测试通过 |
| 6 | 单测: 三策略 + 资源冲突 + 多文档 + 失败路径 | ✅ 完成 | All/Preferred/FirstOnly + dry_run + 边界 |
| 7 | CLI `/file merge` + 意识核心 `doc_merge` 路由 | ✅ 完成 | 端到端真实文件验证 + dispatch 5 测试 |
| 8 | office_oxide 上游 PR | ⏳ 待定 | 外部依赖，当前 zip 层方案已可用 |
| — | **新增: Preferred 策略真正实现** | ✅ 完成 | 文件名识别"修改版"优先作为基座 |
| — | **新增: dry_run 预览模式** | ✅ 完成 | 不写文件仅返回预览信息 |
| — | **新增: 单测覆盖三策略 + dry_run + 边界** | ✅ 完成 | 8 个测试场景全覆盖 |

> 说明: 任务 1-7 对应原设计实施顺序。新增三项为本次迭代强化补充。全量回归受并行会话编译阻塞暂未跑通，待并行会话稳定后补跑。

---

## 8. 状态

设计完成 (2026-08-20)。待并行会话释放编译权后按第 5 节顺序实施。