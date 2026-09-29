# awesome-astra-prompts 吸收（2026-09-24，轻量）

> 源：https://github.com/TripoGrowthLab/awesome-astra-prompts（289★，43 commits，thirdparty/awesome-astra-prompts，HEAD 5496050）
> 实质：200+ GPT-6 Astra 3D 提示词的策展目录（游戏/Blender/交互世界 × 14 语言，日更两次），提示词本体在 tripo3d.ai 链接后，第三方内容**不授权复用**。

## 权利红线（RIGHTS.md 明示）

- MIT 只覆盖自研 tooling + 编辑文档；第三方 prompts/images/videos/code 各归原主，“公开仓库不等于复用许可”。
- 结论：**零提示词拷贝**。只吸收方法论层（taxonomy + 流水线纪律），提示词模板用自己的话重写。

## 可吸收的三件（license-clean）

1. **Catalog taxonomy**：games / Blender scenes / interactive worlds × 14 locale catalogs + with-code 索引（12 个带源码）。
   → NeoTrix 3D/CAD（three_d_render、cad_absorb）缺的就是提示词分类面；按下表建。
2. ** living-catalog 纪律**：CMS → sync → catalogs + `sync-manifest.json`（schemaVersion/count/limit/去重/文件清单断言）+ media checksums + 每日两次 workflow + validate.mjs 门。
   → 映射到我们的 smelt/evals：repo_cards 进度文件加 manifest（总数/去重/校验和），evals 加门。
3. **3D 提示词模板（自研表述）**：subject（主体+动作+场景）/ style（渲染风）/ composition（构图视角）/ lighting-material（光照材质）/ constraints（否定项：no fake UI、无水印）/ provenance（来源链接 + 许可标注必填）。
   模板是方法论复述，不含任何原文提示词。

## 不做的事

- 不批量抓 tripo3d.ai 提示词正文（权利+反爬双红线）。
- 不把 276 条目导成记忆卡（标题党条目无实质，只有索引价值）。

## 写法解剖（读原文提炼，只记结构；示例为自研）

一条高完成度 3D 任务提示词 = 7 段式（源站自带 Prompt breakdown 佐证：先定核心产出→声明可编辑性→闭环测试→长活分期）：

1. Goal：一句话产出 + 硬约束（语言/引用归属/不调用外部 API）；
2. Visual direction：调色/材质/相机位/灯光/UI 排布，一次给全；
3. World：空间布局 anchor（路径/地标/动静分区）；
4. Asset inventory：命名槽位 + 单槽规格 + 复用次数 + 回退规则（占位模型先行，逐槽替换，状态逐槽跟踪）；
5. Mechanics：**全部量化**（速度/半径/时长/冷却/伤害），可测才可验；
6. Implementation：技术栈 + 动画系统 + 性能手段（池化/实例化/帧测）；
7. Acceptance：逐条映射回 §5 + 回退保留 + 与参考图对比 + 性能实报。

自研示例骨架（茶具场景，原创表述）：Goal（一套茶具静物 WebGL 页）→ Visual（暖木桌/侧光/45° 俯视）→ World（桌中茶壶、南侧两杯、北侧窗光）→ Slots（teapot×1/cups×2， procedural 占位先行）→ Numbers（旋转 0.5rad/s、缩放 0.8–1.2、60fps）→ Stack（Three.js ES modules、静态构建）→ Acceptance（旋转/缩放/回退/对比参考图四项勾选）。

## 验证

- 本文档 + taxonomy 落盘；thirdparty clone 在位供查。
- `nt_3d_task_spec.rs` 已落码（7 段式构造 + 验收单生成 + 缺件诊断，3 单测，rustfmt 干净；cargo test 等内存）。
  缺口闭环：引擎会建模渲染，现会“说话”下任务书。
