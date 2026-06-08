source_id: daizhigev20
url: https://github.com/garychowcmu/daizhigev20
category: knowledge_corpus
analyzed: true (structure only)
depth: directory listing
status: classified
priority: high
neotrix_phase: 1.3, 2

## Content Overview
殆知阁古代文献 TXT 全集.
Raw classical Chinese texts in TXT format.
~1.5GB+ estimated, 37 commits, 1.6k stars.

## Directory Structure
├── 佛藏/ (Buddhist canon)
├── 儒藏/ (Confucian canon)
├── 医藏/ (Medical canon)
├── 史藏/ (Historical canon)
│   ├── 正史/ (二十四史)
│   ├── 编年/ (资治通鉴 etc.)
│   ├── 传记/
│   ├── 别史/
│   ├── 史评/
│   ├── 地理/
│   ├── 政书/
│   ├── 职官/
│   ├── 诏令奏议/
│   ├── 纪事本末/
│   ├── 经世文编/
│   ├── 目录/
│   ├── 载记/
│   └── 志存记录/
├── 子藏/ (Philosophers)
├── 易藏/ (I Ching) ← HIGH PRIORITY for E8
├── 艺藏/ (Arts)
├── 诗藏/ (Poetry)
├── 道藏/ (Daoist canon)
└── 集藏/ (Collected works)

## Crawl Plan
- [ ] git clone (large, ~1.5GB)
- [ ] Extract 易藏/周易 for E8 hexagram ↔ 卦辞 mapping
- [ ] Index file list by category
- [ ] Phase 2: VSA embedding for KB seed

## Relevance to NeoTrix
- 易藏: direct input to E8 hexagram reasoning (Phase 1.3)
- 史藏: historical cases for world model (Phase 2)
- 儒/道/佛藏: philosophical foundation for value alignment (Phase 2)
