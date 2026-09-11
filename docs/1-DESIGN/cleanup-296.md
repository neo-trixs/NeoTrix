# 跨域引用扫描报告

**扫描时间**: 2026-09-11 17:30:00
**扫描范围**: /Users/neo/Downloads/neotrix/neotrix-core/src 下6层目录 (l1_action → l6_meta)
**排除条件**: 文件名包含 "test" 或 "facade" 的文件

## 跨层引用汇总

| 源层 | 目标层 | 引用次数 | 示例文件 |
|------|--------|----------|----------|
| l2_perception | l1_action | 63 | l2_perception/nt_world/nt_world_urlhaus.rs:157 |

## 详细跨层引用

### l2_perception

- `l2_perception/nt_world/nt_world_urlhaus.rs:157`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_urlhaus.rs:165`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_urlhaus.rs:236`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_urlhaus.rs:358`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_usgs.rs:284`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_usgs.rs:293`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_usgs.rs:300`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_usgs.rs:409`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_bgpview.rs:124`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_bgpview.rs:228`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdelt.rs:137`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdelt.rs:148`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdelt.rs:158`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdelt.rs:346`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdelt.rs:359`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdelt.rs:379`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ucdp.rs:207`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ucdp.rs:216`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ucdp.rs:223`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ucdp.rs:352`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_aoi.rs:204`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_aoi.rs:213`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_aoi.rs:220`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_aoi.rs:374`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:933`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:934`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:948`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1015`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1028`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1039`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1042`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1045`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1112`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1125`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1135`: 引用 l1_action 层
- `l2_perception/nt_world/osint/mod.rs:1144`: 引用 l1_action 层
- `l2_perception/nt_world/crawl/unified.rs:284`: 引用 l1_action 层
- `l2_perception/nt_world/crawl/unified.rs:733`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdacs.rs:143`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdacs.rs:152`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdacs.rs:159`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_gdacs.rs:263`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ofac.rs:135`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ofac.rs:144`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ofac.rs:151`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_ofac.rs:250`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_polymarket.rs:113`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_polymarket.rs:122`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_polymarket.rs:129`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_polymarket.rs:227`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_opencorporates.rs:125`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_opencorporates.rs:233`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_adsb.rs:129`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_adsb.rs:138`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_adsb.rs:145`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_adsb.rs:245`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_edgar.rs:359`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_edgar.rs:371`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_edgar.rs:381`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_edgar.rs:618`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_edgar.rs:632`: 引用 l1_action 层
- `l2_perception/nt_world/nt_world_edgar.rs:652`: 引用 l1_action 层

## 统计摘要

- **总跨层引用数**: 63
- **涉及层对数**: 1
- **最活跃跨层**: l2_perception->l1_action

### 按层统计

| 层 | 引用其他层次数 | 被其他层引用次数 |
|----|----------------|------------------|
| l1_action | 0 | 63 |
| l2_perception | 63 | 0 |
| l3_embodiment | 0 | 0 |
| l4_emotion | 0 | 0 |
| l5_cognition | 0 | 0 |
| l6_meta | 0 | 0 |

## 架构建议

根据六层架构设计原则：
1. **L1 Action** → 可以引用 L2 Perception（获取感知数据）
2. **L2 Perception** → 可以引用 L3 Embodiment（具身接口）
3. **L3 Embodiment** → 可以引用 L4 Emotion（情感状态）
4. **L4 Emotion** → 可以引用 L5 Cognition（认知处理）
5. **L5 Cognition** → 可以引用 L6 Meta（元认知）
6. **反向引用**（如 L6 引用 L1）应视为架构违规

**注意**: 当前扫描发现 l2_perception 反向引用了 l1_action，这可能违反架构原则。建议检查这些引用是否必要，或考虑重构。

---
*报告由跨域引用扫描脚本自动生成*