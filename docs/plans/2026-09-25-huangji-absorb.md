# 皇極經世吸收 — 晶体神经元图谱种子（2026-09-25）

> 管线：搜研 → 机制 → license → 最小落点 → 自测 → 文档（本件）→ handoff → KB。
> 铁律：不开全量预训练；公有领域零拷贝风险（邵雍 1011–1077，逝世超百年且 1931 年前出版）。

## 1. 来源（CCL： wikisource 四库本目录 / wikipedia 皇極經世 / 刘钢先天易图考 / 识典古籍沈大成疏）

- 体量：十二卷六十四篇 = 元會運世 34 + 声音律呂 16 + 观物内 12 + 观物外 2。
- 时间：1 元 = 12 會 = 360 運 = 4320 世 = 129600 年（一會 10800 / 一運 360 / 一世 30）。
- 生成：加一倍法 2⁰太極→2¹兩儀→2²四象→2³八卦→2⁶六十四卦（二进制字典序，莱布尼茨 1703 印证）。
- 先天数：乾一兑二离三震四巽五坎六艮七坤八；经世取象：乾日兑月离星震辰坤水艮火坎土巽石。
- 阶段：皇（道）-帝（德）-王（功）-伯（力）；开物于寅，闭物于戌；方法论：以物观物。

## 2. 本仓现状（动刀前定点）

- `nt_crystal_core/knowledge_graph.rs`（KnowledgeGraphManager：节点/边四型/实体链接/时序查询）
  **孤儿**：未在 mod.rs 注册，全仓零引用，从未参与编译。
- 晶体核心无神经元图谱种子；`nt_meta` 注册在位（D1 门已验）可作反例参照。

## 3. 落点（最小，纯函数零模型）

- 新文件 `nt_crystal_core/nt_huangji_atlas.rs`：`seed_huangji_atlas(&mut KnowledgeGraphManager)`，
  45 节点 / 44 边（人书 2 / 加一倍 5 / 八卦 8+取象 8 / 元 1+十二會 12+運世 2 / 四阶段 4+开闭 2 / 方法论 1；
  边：因果 5 / 取象语义 8 / 时序 29 / 书-理语义 1+人书因果 1）。
- embedding 确定性 4 维 [层阶/序号/基数对数/0]；metadata 全中文键（先天數/經世取象/年數/治道）。
- 注册：mod.rs 尾部加 `pub mod knowledge_graph;` + `pub mod nt_huangji_atlas;`（两行，邻域零碰）。
- 单测 6 个：总数 / 先天数 / 取象 / 元會數 / 时序全覆盖 / 因果链；禁 unwrap/expect/panic（含测试）。

## 3.5 深度吸收第二刀（标签剥离 + 缺口补齐，同窗 16:00）

- `nt_cosmo_frames.rs`（新）：Scale（扇出串+展开上限防实例爆炸）/ Doubling（2^n 选幂链）/
  Phase（相序权重衰减）/ link_grounding（符号→现象语义边）。零硬编码标签，名全经闭包注入。
  附 4 单测。atlas 改写到 frames 上，45/44 与单测断言不变。
- `knowledge_graph.rs` 缺口三补：`find_by_label` 精确查找（atlas 私有 helper 上移去重）；
  `extract_entities` 收 CJK（旧只收拉丁大写首字母，中文永不建索引）；`neighbors(id, 边型?)`
  出边遍历（推理消费主入口）。附 3 单测；旧 3 单测行为不变。
- 注册加 `pub mod nt_cosmo_frames;` 一行。

## 4. 验证

- `huangji_gate.sh`：`cargo test -j1 -p neotrix --lib -- knowledge_graph nt_huangji_atlas`，reporter 接 huangji。
- 验收：11+6 全绿（knowledge_graph 3 旧 + atlas 6 新，首编译即入树）。

## 5. 后续（不 war）

- experience-tree neuron() 可同源喂皇極实体（entity_index 拉丁分词对中文无效，中文节点暂只走 label 精确路径）。
- 声音律呂（天声地音）/ 十二辟卦值年：第二块拼图，按需再种。
