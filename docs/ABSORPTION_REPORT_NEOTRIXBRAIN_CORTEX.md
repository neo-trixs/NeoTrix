# NeoTrix Brain — Cortex Archive 吸收报告

**吸收时间**: 2026-09-18  
**数据源**: `/Volumes/NeoTrixBrain/cortex-archive/`  
**总容量**: ~114 GB

---

## 📊 文件统计

| 类型 | 文件数 | 总容量 | 格式 |
|------|--------|--------|------|
| PMTiles (地图) | 50 | 15 GB | .pmtiles |
| Wikipedia (维基) | 3 | 14 GB | .zim |
| ZIM (离线知识) | 63 | 85 GB | .zim |
| **合计** | **116** | **114 GB** | - |

---

## 🗺️ PMTiles — 地图数据

**覆盖范围**: 美国全境 (按 Census Bureau 区域划分)  
**数据源**: OpenStreetMap  
**更新日期**: 2026-08-23

### 区域分布

| 区域 | 州数 | 文件示例 |
|------|------|----------|
| East North Central | 5 | Illinois, Indiana, Michigan, Ohio, Wisconsin |
| East South Central | 4 | Alabama, Kentucky, Mississippi, Tennessee |
| Mid-Atlantic | 3 | New Jersey, New York, Pennsylvania |
| Mountain Region | 8 | Arizona, Colorado, Idaho, Montana, Nevada, New Mexico, Utah, Wyoming |
| New England | 6 | Connecticut, Maine, Massachusetts, New Hampshire, Rhode Island, Vermont |
| Pacific Region | 5 | Alaska, California, Hawaii, Oregon, Washington |
| South Atlantic | 9 | Delaware, Florida, Georgia, Maryland, North/South Carolina, Virginia, West Virginia |
| West North Central | 7 | Iowa, Kansas, Minnesota, Missouri, Nebraska, North/South Dakota |
| West South Central | 4 | Arkansas, Louisiana, Oklahoma, Texas |

### 用途
- 离线地图渲染 (MapLibre/PMTiles)
- 无需瓦片服务器的矢量地图服务
- 支持全美道路/建筑/POI 查询

---

## 📚 Wikipedia — 离线维基百科

| 文件 | 版本 | 容量 | 说明 |
|------|------|------|------|
| wikipedia_en_all_mini_2026-06.zim | 2026-06 | 12 GB | 英文全量 (Mini版) |
| wikipedia_en_top_mini_2026-06.zim | 2026-06 | 316 MB | 热门文章 (Mini版) |
| wikipedia_en_top_nopic_2026-06.zim | 2026-06 | 2.1 GB | 热门文章 (无图版) |

### 版本说明
- **Mini**: 文章文本压缩，保留关键图片
- **NoPic**: 纯文本，无图片，体积更小
- **All**: 英文维基全量文章 (~680万篇)

---

## 🧠 ZIM — 离线知识库

**总计**: 63 个 ZIM 文件，涵盖 6 大领域

### 领域分布

#### 1. Agriculture & Food (7 文件)
- Comprehensive: Learning Self-Reliance, Project Gutenberg Agriculture
- Standard: Cooking Q&A, Food for Preppers, Gardening Q&A
- Essential: Based.Cooking, FOSS Cooking

#### 2. Computing & Technology (16 文件)
- Comprehensive: Docker, Electronics Q&A, Linux, Robotics Q&A
- Standard: Arduino, Git, Node.js, Raspberry Pi, React
- Essential: CSS, HTML, JavaScript, Python, freeCodeCamp

#### 3. DIY & Repair (4 文件)
- Comprehensive: iFixit Repair Guides (3.3 GB)
- Standard: DIY & Home Improvement Q&A (1.9 GB)
- Essential: Motor Vehicle Maintenance, Woodworking

#### 4. Education & Reference (13 文件)
- Comprehensive: LibreTexts (Business, Engineering, Geosciences, Humanities), TED, Wikibooks
- Standard: LibreTexts (Biology, Chemistry, Math, Physics), TED-Ed, Wikiversity

#### 5. Medicine (8 文件)
- Comprehensive: LibrePathology, LibreTexts Medicine, Wikipedia Medicine
- Essential: CDC Health, Medical Library, Military Medicine, NHS A-Z
- Standard: MedlinePlus

#### 6. Survival & Preparedness (5+ 文件)
- Comprehensive: CD3WD Technology, Canadian Prepper, Hundred Rabbits, Post-Disaster

### 分级说明
- **Comprehensive**: 完整文档库，含图片
- **Standard**: 核心内容，部分图片
- **Essential**: 精简版，纯文本

---

## 🔧 技术规格

### PMTiles
```
格式: PMTiles v3 (单文件瓦片存档)
压缩: ZSTD
坐标: EPSG:3857 (Web Mercator)
图层: 矢量 (OpenMapTiles schema)
```

### ZIM
```
格式: ZIM (ZIM Internet Media)
库: openzim/libzim
读取: kiwix-serve / kiwix-desktop
索引: Xapian 全文搜索
```

---

## 💡 NeoTrix 集成建议

| Cortex 数据 | NeoTrix 模块 | 用途 |
|-------------|--------------|------|
| PMTiles | nt_world 地理感知 | 离线地图查询 |
| Wikipedia ZIM | nt_memory 知识库 | 知识问答增强 |
| ZIM 知识库 | nt_core 推理 | 领域知识支撑 |

---

**报告生成**: NeoTrix Knowledge Absorption Agent  
**状态**: ✅ 已完成吸收，数据就绪