# NeoTrixBrain 数据探索报告

## 📊 数据库概况

| 组件 | 路径 | 大小 | 说明 |
|------|------|------|------|
| **主数据库** | `/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db` | **64 GB** | 核心知识库 |
| **地图瓦片** | `cortex-archive/pmtiles/` | **~32 GB** | 美国各州地图数据 |
| **Wikipedia** | `cortex-archive/wikipedia/` | **~15 GB** | 英文维基百科 |
| **ZIM 知识库** | `cortex-archive/zim/` | **~177 GB** | 多领域知识合集 |

**总计**: ~288 GB 知识数据

---

## 🗄️ 主数据库结构

### 表统计

| 表名 | 记录数 | 说明 |
|------|--------|------|
| `nodes` | 极大 (>100万) | 知识节点 |
| `edges` | 极大 | 关系边 |
| `embeddings` | 235,143 | 向量嵌入 |
| `kv_store` | 117,264 | 键值存储 |
| `session_logs` | 3,152 | 会话日志 |
| `health_reports` | 2,012 | 健康报告 |
| `evolution_records` | 13 | 进化记录 |
| `procedural_memory` | 有数据 | 技能记忆 |

### 节点类型 (已发现)

| 节点类型 | 示例 |
|----------|------|
| `concept` | Mathematics, Algebra, Geometry, Calculus, Logic |

### 关系类型

- 需要进一步探索 (表较大，查询超时)

### 向量嵌入

- **数量**: 235,143 个嵌入向量
- **维度**: 待确认
- **模型**: text-embedding-3-small

---

## 📚 Cortex-Archive 知识库

### 1. Wikipedia (15 GB)

| 文件 | 大小 | 内容 |
|------|------|------|
| `wikipedia_en_all_mini_2026-06.zim` | 12 GB | 英文维基百科完整版 |
| `wikipedia_en_top_nopic_2026-06.zim` | 2.1 GB | 热门文章 (无图) |
| `wikipedia_en_top_mini_2026-06.zim` | 316 MB | 热门文章精华 |

### 2. ZIM 知识合集 (177 GB)

#### 计算机与技术

| 文件 | 大小 | 内容 |
|------|------|------|
| `Docker_Documentation.zim` | 1.7 MB | Docker 文档 |
| `Linux_Documentation.zim` | 546 KB | Linux 文档 |
| `CSS_Documentation.zim` | 4.7 MB | CSS 文档 |
| `HTML_Documentation.zim` | 1.6 MB | HTML 文档 |
| `JavaScript_Documentation.zim` | 2.6 MB | JavaScript 文档 |
| `Python_Documentation.zim` | 4.2 MB | Python 文档 |
| `Git_Documentation.zim` | 1.5 MB | Git 文档 |
| `Node.js_Documentation.zim` | 1.4 MB | Node.js 文档 |
| `freeCodeCamp.zim` | 7.6 MB | freeCodeCamp 教程 |
| `Arduino_Q_A.zim` | 247 MB | Arduino 问答 |
| `Robotics_Q_A.zim` | 233 MB | 机器人技术问答 |
| `Electronics_Q_A.zim` | 3.9 GB | 电子学问答 |

#### 农业与食品

| 文件 | 大小 | 内容 |
|------|------|------|
| `Learning_Self-Reliance__Homesteading.zim` | 3.7 GB | 自给自足/家园生活 |
| `Project_Gutenberg__Agriculture.zim` | 4.2 GB | 古登堡农业书籍 |
| `Based.Cooking.zim` | 15 MB | 基础烹饪 |
| `FOSS_Cooking.zim` | 23 MB | 开源烹饪 |
| `Cooking_Q_A.zim` | 226 MB | 烹饪问答 |
| `Food_for_Preppers.zim` | 93 MB | 食物储备 |
| `Gardening_Q_A.zim` | 882 MB | 园艺问答 |

### 3. 地图瓦片 (32 GB)

美国各州 PMTiles 格式地图数据，可用于地理信息推理。

---

## 🧠 数据价值分析

### 对 Any-to-Any 模型的价值

| 数据类型 | 训练阶段 | 价值 |
|----------|----------|------|
| **Wikipedia** | Pretrain | ⭐⭐⭐⭐⭐ 通用知识基础 |
| **技术文档** | Pretrain + SFT | ⭐⭐⭐⭐⭐ 代码能力 |
| **问答数据** | SFT | ⭐⭐⭐⭐⭐ 对话能力 |
| **健康报告** | SFT | ⭐⭐⭐⭐ 系统诊断推理 |
| **进化记录** | SFT | ⭐⭐⭐⭐ 自我进化推理 |
| **技能记忆** | SFT + Tool Call | ⭐⭐⭐⭐ 工具调用能力 |
| **会话日志** | SFT | ⭐⭐⭐ 对话模式学习 |
| **嵌入向量** | KB 融合 | ⭐⭐⭐⭐⭐ 语义搜索增强 |
| **地图瓦片** | Vision | ⭐⭐⭐ 地理可视化 |

### 推荐训练数据组合

```
┌─────────────────────────────────────────────────────────────────┐
│                    推荐训练数据组合                               │
├─────────────────────────────────────────────────────────────────┤
│  Phase 1: 预训练 (Pretrain)                                     │
│  ├── Wikipedia 精华 (5GB)                                       │
│  ├── 技术文档 (2GB)                                             │
│  ├── 问答数据 (1GB)                                             │
│  └── 总计: ~8GB                                                 │
├─────────────────────────────────────────────────────────────────┤
│  Phase 2: SFT 微调                                              │
│  ├── 对话数据 (1GB)                                             │
│  ├── 工具调用数据 (500MB)                                        │
│  ├── 诊断推理数据 (200MB)                                        │
│  ├── 进化推理数据 (100MB)                                        │
│  └── 总计: ~1.8GB                                               │
├─────────────────────────────────────────────────────────────────┤
│  Phase 3: RLAIF + 领域适配                                       │
│  ├── 偏好对齐数据 (100MB)                                        │
│  ├── NeoTrix 专属知识 (500MB)                                    │
│  └── 总计: ~600MB                                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🚀 数据提取管道

### ZIM 文件处理

```python
# 需要安装: pip install libzim

from libzim import Archive

def extract_zim_content(zim_path, output_path):
    """提取 ZIM 文件内容"""
    archive = Archive(zim_path)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        for article in archive.iter_articles():
            if article.is_redirect:
                continue
            
            title = article.title
            content = article.get_text_content()
            
            if content and len(content) > 100:
                import json
                f.write(json.dumps({
                    "text": f"{title}\n\n{content[:3000]}",
                    "source": f"zim:{article.namespace}",
                    "title": title
                }, ensure_ascii=False) + '\n')
```

### 数据库提取

```python
import sqlite3
import json

def extract_knowledge_nodes(db_path, output_path, limit=100000):
    """提取知识节点"""
    conn = sqlite3.connect(db_path)
    cursor = conn.execute("""
        SELECT id, node_type, title, summary, content
        FROM nodes
        WHERE content IS NOT NULL
        AND LENGTH(content) > 100
        LIMIT ?
    """, (limit,))
    
    with open(output_path, 'w', encoding='utf-8') as f:
        for row in cursor:
            text = f"""节点: {row[2]}
类型: {row[1]}
摘要: {row[3] or ''}
内容: {row[4][:2000]}"""
            
            f.write(json.dumps({
                "text": text,
                "source": f"kb_node:{row[0]}"
            }, ensure_ascii=False) + '\n')
    
    conn.close()

def extract_conversations(db_path, output_path):
    """提取对话记录 → SFT 数据"""
    conn = sqlite3.connect(db_path)
    cursor = conn.execute("""
        SELECT id, task_description, actions_taken
        FROM conversation_records
        WHERE task_description IS NOT NULL
    """)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        for row in cursor:
            conversations = [
                {"role": "user", "content": row[1]},
                {"role": "assistant", "content": row[2] or ''}
            ]
            
            f.write(json.dumps({
                "conversations": conversations
            }, ensure_ascii=False) + '\n')
    
    conn.close()

def extract_health_reports(db_path, output_path):
    """提取健康报告 → 诊断推理数据"""
    conn = sqlite3.connect(db_path)
    cursor = conn.execute("""
        SELECT id, report_data, timestamp
        FROM health_reports
        WHERE report_data IS NOT NULL
    """)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        for row in cursor:
            try:
                report = json.loads(row[1])
                conversations = [
                    {"role": "user", "content": "分析系统健康状态"},
                    {"role": "assistant", "content": f"健康报告:\n{json.dumps(report, indent=2)}"}
                ]
                
                f.write(json.dumps({
                    "conversations": conversations
                }, ensure_ascii=False) + '\n')
            except:
                continue
    
    conn.close()
```

---

## 📋 下一步

### 数据准备任务

1. **提取 Wikipedia** → `dataset/neoTrixBrain/wikipedia_pretrain.jsonl`
2. **提取技术文档** → `dataset/neoTrixBrain/tech_pretrain.jsonl`
3. **提取问答数据** → `dataset/neoTrixBrain/qa_sft.jsonl`
4. **提取知识节点** → `dataset/neoTrixBrain/kb_pretrain.jsonl`
5. **提取对话记录** → `dataset/neoTrixBrain/conversations_sft.jsonl`
6. **提取健康报告** → `dataset/neoTrixBrain/diagnostic_sft.jsonl`
7. **融合所有数据** → `dataset/neoTrixBrain/pretrain_t2t.jsonl` + `sft_t2t.jsonl`

### 预计数据量

| 阶段 | 目标大小 | 实际可用 |
|------|----------|----------|
| 预训练 | 8 GB | ✅ 充足 |
| SFT | 1.8 GB | ✅ 充足 |
| RLAIF | 600 MB | ✅ 充足 |

**结论**: 数据非常充足，可以构建高质量的 Any-to-Any 模型。

---

**最后更新**: 2026-09-09
