# NeoTrix 架构整合计划

> **日期**: 2026-09-08
> **目标**: 按功能类型整合目录结构

---

## 一、当前问题

### 1.1 l2_perception/nt_world (11 目录)

| 目录 | 文件数 | 内容 |
|------|--------|------|
| nt_world_crawl | 22 | 爬虫/抓取 |
| nt_world_browse | 5 | 浏览器 |
| nt_world_browse_auto | 4 | 自动浏览 |
| nt_world_osint | 12 | 情报收集 |
| nt_world_absorber | 5 | 内容吸收 |
| nt_world_sense | 12 | 感知分析 |
| nt_world_jepa | 14 | 联合嵌入 |
| nt_world_model | 6 | 模型推理 |
| nt_world_map | 5 | 地图映射 |
| nt_world_cleanup | 4 | 清理 |
| nt_world_media_source | 1 | 媒体源 |

### 1.2 l6_meta (7 目录)

| 目录 | 内容 |
|------|------|
| nt_meta | 元认知 |
| nt_governance | 治理 |
| nt_memory | 记忆 |
| nt_nexus | 枢纽 |
| nt_repair | 修复 |
| nt_mind | 思维 |
| nt_act | 行动 |

---

## 二、整合方案

### 2.1 l2_perception/nt_world → 5 目录

```
nt_world/
├── crawl/      爬虫类 (crawl + browse + browse_auto)
├── osint/      情报类 (osint + absorber)
├── sense/      感知类 (sense + jepa + model)
├── explore/    探索类 (map + cleanup)
└── source/     源类 (media_source)
```

### 2.2 l6_meta → 4 目录

```
l6_meta/
├── coordination/  协调类 (meta + governance)
├── memory/        记忆类 (memory + nexus)
├── healing/       修复类 (repair)
└── evolution/     进化类 (mind + act)
```

### 2.3 l1_action → 3 目录 (已简洁)

```
l1_action/
├── tools/     工具类 (nt_act)
├── io/        接口类 (nt_io)
└── storage/   存储类 (nt_memory)
```

### 2.4 l3_embodiment → 3 目录 (已简洁)

```
l3_embodiment/
├── body/      身体类 (nt_physical)
├── feel/      情感类 (nt_feel)
└── safety/    安全类 (nt_shield)
```

---

## 三、执行计划

### Phase 1: l2_perception/nt_world 整合
1. 创建 crawl/ 目录，移入 crawl + browse + browse_auto
2. 创建 osint/ 目录，移入 osint + absorber
3. 创建 sense/ 目录，移入 sense + jepa + model
4. 创建 explore/ 目录，移入 map + cleanup
5. 更新 mod.rs

### Phase 2: l6_meta 整合
1. 创建 coordination/ 目录，移入 meta + governance
2. 创建 memory/ 目录，移入 memory + nexus
3. 创建 healing/ 目录，移入 repair
4. 创建 evolution/ 目录，移入 mind + act
5. 更新 mod.rs

### Phase 3: 验证
1. 编译检查
2. 依赖图验证
