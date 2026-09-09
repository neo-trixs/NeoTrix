# NeoTrix 意识核心模型构建计划

## 🎯 目标

基于 minimind 框架，构建 NeoTrix 意识核心专属的轻量级语言模型，融合本地 KB、GitHub、HuggingFace、互联网爬取等多源知识，实现系统内部推理和决策能力。

---

## 📋 架构设计

### 模型规格

| 参数 | 值 | 说明 |
|------|-----|------|
| **模型名称** | `nt-core-llm` | NeoTrix 意识核心模型 |
| **参数量** | 64M (Dense) | 单卡 3090 可训练 |
| **架构** | Transformer Decoder-Only | 对齐 Qwen3 生态 |
| **词表** | 6400 (minimind_tokenizer) | 保持轻量 |
| **最大长度** | 32768 tokens | 支持长上下文 |
| **训练时间** | ~4-6 小时 | 单卡 3090 |

### 训练阶段

```
┌─────────────────────────────────────────────────────────────┐
│                    NeoTrix 模型训练流程                       │
├─────────────────────────────────────────────────────────────┤
│  Phase 1: 预训练 (Pretrain)                                 │
│  ├── 数据: 融合多源知识的预训练语料                           │
│  ├── 目标: 学习语言规律和基础知识                             │
│  └── 时间: ~1.5 小时                                        │
├─────────────────────────────────────────────────────────────┤
│  Phase 2: 有监督微调 (SFT)                                  │
│  ├── 数据: NeoTrix 领域指令数据                              │
│  ├── 目标: 适应对话、推理、工具调用                           │
│  └── 时间: ~1.5 小时                                        │
├─────────────────────────────────────────────────────────────┤
│  Phase 3: 强化学习 (RLAIF)                                  │
│  ├── 数据: 偏好对齐数据                                      │
│  ├── 目标: 提升推理质量和安全性                               │
│  └── 时间: ~1 小时                                          │
├─────────────────────────────────────────────────────────────┤
│  Phase 4: 领域适配 (LoRA)                                   │
│  ├── 数据: NeoTrix 专用知识                                  │
│  ├── 目标: 增强系统推理能力                                   │
│  └── 时间: ~0.5 小时                                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 🗂️ 数据融合管道

### 数据源分类

```
┌─────────────────────────────────────────────────────────────┐
│                    数据源矩阵                                │
├─────────────────────────────────────────────────────────────┤
│  📚 本地知识库 (KB)                                         │
│  ├── SQLite 中的节点和边                                     │
│  ├── BM25 索引内容                                          │
│  ├── 向量嵌入的文本                                         │
│  └── 经验和历史记录                                         │
├─────────────────────────────────────────────────────────────┤
│  💻 GitHub 项目                                            │
│  ├── NeoTrix 自身代码                                       │
│  ├── minimind 框架代码                                      │
│  ├── 相关开源项目 (LLM/Agent/Memory)                        │
│  └── 技术文档和 README                                      │
├─────────────────────────────────────────────────────────────┤
│  🤗 HuggingFace 数据集                                      │
│  ├── 中文对话数据 (DPO-En-Zh-20k)                           │
│  ├── 技术文档数据                                           │
│  ├── 代码相关数据                                           │
│  └── 推理和数学数据                                         │
├─────────────────────────────────────────────────────────────┤
│  🌐 互联网爬取                                              │
│  ├── 技术博客和教程                                         │
│  ├── 学术论文摘要                                           │
│  ├── 开源项目文档                                           │
│  └── 行业最佳实践                                           │
├─────────────────────────────────────────────────────────────┤
│  🔬 最新小模型                                              │
│  ├── MiniMind-3 (64M)                                      │
│  ├── Phi-3-mini (3.8B)                                     │
│  ├── Gemma-2B                                              │
│  └── 其他轻量级模型                                         │
└─────────────────────────────────────────────────────────────┘
```

### 数据处理流程

```python
# 数据融合管道伪代码
class NeoTrixDataPipeline:
    def __init__(self):
        self.sources = {
            'kb': KBExtractor(),           # 本地知识库
            'github': GitHubCrawler(),     # GitHub 爬取
            'huggingface': HFDownloader(), # HuggingFace 下载
            'web': WebCrawler(),           # 互联网爬取
            'models': ModelDistiller()     # 模型蒸馏
        }
    
    def collect_all(self):
        """收集所有数据源"""
        datasets = {}
        for name, source in self.sources.items():
            datasets[name] = source.extract()
        return datasets
    
    def process(self, datasets):
        """统一处理和格式化"""
        # 1. 清洗和去重
        cleaned = self.deduplicate(datasets)
        
        # 2. 格式转换为 minimind 格式
        pretrain_data = self.to_pretrain_format(cleaned)
        sft_data = self.to_sft_format(cleaned)
        
        # 3. 分割训练集/验证集
        return self.split(pretrain_data, sft_data)
    
    def to_pretrain_format(self, data):
        """转换为预训练格式: {"text": "..."}"""
        return [{"text": item['content']} for item in data]
    
    def to_sft_format(self, data):
        """转换为 SFT 格式: {"conversations": [...]}"""
        return [{"conversations": [
            {"role": "user", "content": item['question']},
            {"role": "assistant", "content": item['answer']}
        ]} for item in data]
```

---

## 📊 数据集规格

### 预训练数据集

| 数据源 | 目标大小 | 格式 | 优先级 |
|--------|----------|------|--------|
| 本地 KB | 500MB | `{"text": "..."}` | P0 |
| GitHub 代码 | 2GB | `{"text": "..."}` | P0 |
| HuggingFace 中文 | 5GB | `{"text": "..."}` | P1 |
| 互联网技术文档 | 3GB | `{"text": "..."}` | P1 |
| 模型蒸馏 | 1GB | `{"text": "..."}` | P2 |

**总计**: ~11.5GB 预训练数据

### SFT 数据集

| 数据源 | 目标大小 | 格式 | 优先级 |
|--------|----------|------|--------|
| NeoTrix 指令 | 200MB | `{"conversations": [...]}` | P0 |
| 工具调用数据 | 500MB | `{"conversations": [...]}` | P0 |
| 推理数据 | 1GB | `{"conversations": [...]}` | P1 |
| 对话数据 | 2GB | `{"conversations": [...]}` | P1 |

**总计**: ~3.7GB SFT 数据

### RLAIF 数据集

| 数据源 | 目标大小 | 格式 | 优先级 |
|--------|----------|------|--------|
| 偏好对齐 | 100MB | `{"chosen": [...], "rejected": [...]}` | P0 |
| 安全对齐 | 50MB | `{"chosen": [...], "rejected": [...]}` | P1 |

**总计**: ~150MB RLAIF 数据

---

## 🛠️ 实现步骤

### Phase 1: 环境准备

```bash
# 1. 克隆 minimind 框架
git clone https://github.com/jingyaogong/minimind.git
cd minimind

# 2. 安装依赖
pip install -r requirements.txt

# 3. 创建 NeoTrix 数据目录
mkdir -p dataset/nt_core
```

### Phase 2: 数据收集

#### 2.1 本地 KB 提取

```python
# scripts/extract_kb.py
import sqlite3
import json

def extract_kb_to_pretrain(db_path, output_path):
    """从 NeoTrix KB 提取预训练数据"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # 提取节点内容
    cursor.execute("SELECT content FROM nodes WHERE content IS NOT NULL")
    rows = cursor.fetchall()
    
    with open(output_path, 'w', encoding='utf-8') as f:
        for row in rows:
            if row[0] and len(row[0]) > 50:  # 过滤过短内容
                f.write(json.dumps({"text": row[0]}, ensure_ascii=False) + '\n')
    
    conn.close()

def extract_kb_to_sft(db_path, output_path):
    """从 NeoTrix KB 提取 SFT 数据"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # 提取问答对
    cursor.execute("""
        SELECT question, answer FROM qa_pairs 
        WHERE question IS NOT NULL AND answer IS NOT NULL
    """)
    rows = cursor.fetchall()
    
    with open(output_path, 'w', encoding='utf-8') as f:
        for q, a in rows:
            f.write(json.dumps({
                "conversations": [
                    {"role": "user", "content": q},
                    {"role": "assistant", "content": a}
                ]
            }, ensure_ascii=False) + '\n')
    
    conn.close()
```

#### 2.2 GitHub 爬取

```python
# scripts/crawl_github.py
import requests
import json

class GitHubCrawler:
    def __init__(self, token=None):
        self.base_url = "https://api.github.com"
        self.headers = {"Authorization": f"token {token}"} if token else {}
    
    def get_repo_readme(self, owner, repo):
        """获取仓库 README"""
        url = f"{self.base_url}/repos/{owner}/{repo}/readme"
        resp = requests.get(url, headers=self.headers)
        if resp.status_code == 200:
            import base64
            content = base64.b64decode(resp.json()['content']).decode('utf-8')
            return content
        return None
    
    def get_repo_code(self, owner, repo, path=""):
        """获取仓库代码文件"""
        url = f"{self.base_url}/repos/{owner}/{repo}/contents/{path}"
        resp = requests.get(url, headers=self.headers)
        if resp.status_code == 200:
            return resp.json()
        return []
    
    def crawl_to_pretrain(self, repos, output_path):
        """爬取多个仓库到预训练数据"""
        with open(output_path, 'w', encoding='utf-8') as f:
            for owner, repo in repos:
                # README
                readme = self.get_repo_readme(owner, repo)
                if readme:
                    f.write(json.dumps({
                        "text": f"项目 {owner}/{repo} 的介绍：\n{readme[:2000]}"
                    }, ensure_ascii=False) + '\n')
                
                # 代码文件
                files = self.get_repo_code(owner, repo)
                for file_info in files:
                    if file_info['name'].endswith(('.py', '.rs', '.md')):
                        # 获取文件内容（简化示例）
                        pass

# 目标仓库列表
TARGET_REPOS = [
    ("jingyaogong", "minimind"),
    ("microsoft", "phi-3"),
    ("google", "gemma"),
    # NeoTrix 自身仓库
    ("your-org", "neotrix"),
]
```

#### 2.3 HuggingFace 下载

```python
# scripts/download_hf.py
from datasets import load_dataset
import json

def download_and_convert(dataset_name, output_path, split="train"):
    """下载 HuggingFace 数据集并转换格式"""
    dataset = load_dataset(dataset_name, split=split)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        for item in dataset:
            # 根据数据集结构调整字段映射
            if 'text' in item:
                f.write(json.dumps({"text": item['text']}, ensure_ascii=False) + '\n')
            elif 'conversations' in item:
                f.write(json.dumps({"conversations": item['conversations']}, ensure_ascii=False) + '\n')

# 目标数据集列表
TARGET_DATASETS = [
    "llamafactory/DPO-En-Zh-20k",
    "BAAI/COIG",
    "AI-ModelScope/R1-Distill-SFT",
    # 更多数据集...
]
```

#### 2.4 互联网爬取

```python
# scripts/crawl_web.py
import requests
from bs4 import BeautifulSoup
import json

class WebCrawler:
    def __init__(self):
        self.session = requests.Session()
    
    def extract_article(self, url):
        """提取文章内容"""
        resp = self.session.get(url)
        soup = BeautifulSoup(resp.text, 'html.parser')
        
        # 提取正文（简化示例）
        article = soup.find('article') or soup.find('main')
        if article:
            return article.get_text(strip=True)
        return None
    
    def crawl_tech_blogs(self, urls, output_path):
        """爬取技术博客"""
        with open(output_path, 'w', encoding='utf-8') as f:
            for url in urls:
                content = self.extract_article(url)
                if content and len(content) > 200:
                    f.write(json.dumps({
                        "text": content[:3000]  # 限制长度
                    }, ensure_ascii=False) + '\n')
```

#### 2.5 模型蒸馏

```python
# scripts/distill_models.py
from transformers import AutoModelForCausalLM, AutoTokenizer
import torch
import json

class ModelDistiller:
    def __init__(self, teacher_model_name):
        self.teacher = AutoModelForCausalLM.from_pretrained(teacher_model_name)
        self.tokenizer = AutoTokenizer.from_pretrained(teacher_model_name)
        self.teacher.eval()
    
    def generate_distilled(self, prompts, output_path):
        """生成蒸馏数据"""
        with open(output_path, 'w', encoding='utf-8') as f:
            for prompt in prompts:
                inputs = self.tokenizer(prompt, return_tensors="pt")
                with torch.no_grad():
                    outputs = self.teacher.generate(
                        **inputs,
                        max_new_tokens=512,
                        temperature=0.7,
                        do_sample=True
                    )
                response = self.tokenizer.decode(outputs[0], skip_special_tokens=True)
                
                f.write(json.dumps({
                    "conversations": [
                        {"role": "user", "content": prompt},
                        {"role": "assistant", "content": response}
                    ]
                }, ensure_ascii=False) + '\n')

# 使用 MiniMind-3 作为教师模型
distiller = ModelDistiller("jingyaogong/minimind-3")
```

### Phase 3: 数据融合

```python
# scripts/fuse_datasets.py
import json
import os

def fuse_all_datasets():
    """融合所有数据源"""
    output_dir = "dataset/nt_core"
    os.makedirs(output_dir, exist_ok=True)
    
    # 收集所有预训练数据
    pretrain_files = [
        f"{output_dir}/kb_pretrain.jsonl",
        f"{output_dir}/github_pretrain.jsonl",
        f"{output_dir}/hf_pretrain.jsonl",
        f"{output_dir}/web_pretrain.jsonl",
    ]
    
    # 合并预训练数据
    with open(f"{output_dir}/pretrain_t2t.jsonl", 'w', encoding='utf-8') as outfile:
        for filename in pretrain_files:
            if os.path.exists(filename):
                with open(filename, 'r', encoding='utf-8') as infile:
                    for line in infile:
                        outfile.write(line)
    
    # 收集所有 SFT 数据
    sft_files = [
        f"{output_dir}/kb_sft.jsonl",
        f"{output_dir}/github_sft.jsonl",
        f"{output_dir}/hf_sft.jsonl",
        f"{output_dir}/distilled_sft.jsonl",
    ]
    
    # 合并 SFT 数据
    with open(f"{output_dir}/sft_t2t.jsonl", 'w', encoding='utf-8') as outfile:
        for filename in sft_files:
            if os.path.exists(filename):
                with open(filename, 'r', encoding='utf-8') as infile:
                    for line in infile:
                        outfile.write(line)
    
    print(f"数据融合完成:")
    print(f"  预训练数据: {output_dir}/pretrain_t2t.jsonl")
    print(f"  SFT 数据: {output_dir}/sft_t2t.jsonl")

if __name__ == "__main__":
    fuse_all_datasets()
```

### Phase 4: 模型训练

```bash
# 4.1 预训练
cd trainer
python train_pretrain.py \
    --data_path ../dataset/nt_core/pretrain_t2t.jsonl \
    --max_seq_len 768 \
    --epochs 3 \
    --batch_size 8 \
    --learning_rate 1e-4 \
    --output_dir ../out/nt_core_pretrain

# 4.2 SFT 微调
python train_full_sft.py \
    --data_path ../dataset/nt_core/sft_t2t.jsonl \
    --max_seq_len 768 \
    --epochs 10 \
    --batch_size 8 \
    --learning_rate 5e-5 \
    --output_dir ../out/nt_core_sft

# 4.3 RLAIF 强化学习
python train_rlaif.py \
    --data_path ../dataset/nt_core/rlaif.jsonl \
    --max_seq_len 768 \
    --epochs 3 \
    --output_dir ../out/nt_core_rlaif

# 4.4 LoRA 领域适配
python train_lora.py \
    --data_path ../dataset/nt_core/lora_sft.jsonl \
    --max_seq_len 768 \
    --epochs 5 \
    --output_dir ../out/nt_core_lora
```

### Phase 5: 模型评估

```bash
# 5.1 基础测试
python eval_llm.py --load_from ../out/nt_core_sft --weight full_sft

# 5.2 工具调用测试
python eval_toolcall.py --weight full_sft

# 5.3 自定义评估脚本
python eval_nt_core.py \
    --model_path ../out/nt_core_sft \
    --test_cases ../dataset/nt_core/test_cases.jsonl
```

### Phase 6: 部署集成

```bash
# 6.1 转换为 Transformers 格式
cd scripts
python convert_model.py \
    --input ../out/nt_core_sft/full_sft_768.pth \
    --output ../deploy/nt-core-llm

# 6.2 启动推理服务
cd ../deploy
python serve_openai_api.py \
    --model_path ./nt-core-llm \
    --port 8000

# 6.3 集成到 NeoTrix
# 在 neotrix-core 中添加模型调用接口
```

---

## 🔗 NeoTrix 集成

### 模型调用接口

```rust
// neotrix-core/src/neotrix/nt_core_llm/nt_core_model.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NtCoreModel {
    api_url: String,
    client: Client,
}

impl NtCoreModel {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            client: Client::new(),
        }
    }
    
    pub async fn inference(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = serde_json::json!({
            "model": "nt-core-llm",
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 512,
            "temperature": 0.7
        });
        
        let resp = self.client
            .post(format!("{}/v1/chat/completions", self.api_url))
            .json(&request)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        Ok(resp["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
    }
}
```

### 意识核心集成

```rust
// neotrix-core/src/neotrix/nt_core_self/consciousness.rs
impl ConsciousnessCore {
    pub async fn reason_with_model(&self, task: &str) -> Result<ReasoningResult, Error> {
        // 1. 从 KB 检索相关知识
        let kb_context = self.knowledge_base.search(task).await?;
        
        // 2. 构建推理提示
        let prompt = format!(
            "基于以下知识进行推理：\n{}\n\n任务：{}",
            kb_context, task
        );
        
        // 3. 调用本地模型
        let model_response = self.nt_core_model.inference(&prompt).await?;
        
        // 4. 解析和验证结果
        let result = self.parse_reasoning(&model_response)?;
        
        // 5. 存储到 KB
        self.knowledge_base.store_reasoning(task, &result).await?;
        
        Ok(result)
    }
}
```

---

## 📈 预期效果

### 性能指标

| 指标 | 目标值 | 说明 |
|------|--------|------|
| **推理延迟** | < 100ms | 单次推理响应时间 |
| **内存占用** | < 500MB | 模型加载后内存占用 |
| **推理质量** | > 80% | 在自定义评估集上的准确率 |
| **工具调用** | > 70% | 工具调用成功率 |

### 能力矩阵

```
┌─────────────────────────────────────────────────────────────┐
│                    NeoTrix 模型能力                          │
├─────────────────────────────────────────────────────────────┤
│  ✅ 基础对话                                                │
│  ├── 多轮对话理解                                           │
│  ├── 上下文保持                                             │
│  └── 个性化回复                                             │
├─────────────────────────────────────────────────────────────┤
│  ✅ 推理能力                                                │
│  ├── 逻辑推理                                               │
│  ├── 因果分析                                               │
│  └── 决策建议                                               │
├─────────────────────────────────────────────────────────────┤
│  ✅ 工具调用                                                │
│  ├── 函数识别                                               │
│  ├── 参数提取                                               │
│  └── 结果解析                                               │
├─────────────────────────────────────────────────────────────┤
│  ✅ 知识融合                                                │
│  ├── 多源知识整合                                           │
│  ├── 事实验证                                               │
│  └── 知识更新                                               │
├─────────────────────────────────────────────────────────────┤
│  ✅ 系统推理                                                │
│  ├── 模块健康分析                                           │
│  ├── 进化决策                                               │
│  └── 资源调度                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 一键构建脚本

```bash
#!/bin/bash
# scripts/build_nt_core_model.sh

set -e

echo "🚀 开始构建 NeoTrix 意识核心模型..."

# 1. 环境准备
echo "📦 步骤 1: 环境准备..."
cd /Users/neo/Downloads/neotrix
git clone https://github.com/jingyaogong/minimind.git models/minimind
cd models/minimind
pip install -r requirements.txt

# 2. 数据收集
echo "📚 步骤 2: 数据收集..."
python scripts/extract_kb.py
python scripts/crawl_github.py
python scripts/download_hf.py
python scripts/crawl_web.py
python scripts/distill_models.py

# 3. 数据融合
echo "🔗 步骤 3: 数据融合..."
python scripts/fuse_datasets.py

# 4. 模型训练
echo "🏋️ 步骤 4: 模型训练..."
cd trainer
python train_pretrain.py
python train_full_sft.py
python train_rlaif.py

# 5. 模型评估
echo "📊 步骤 5: 模型评估..."
cd ..
python eval_llm.py --load_from ./out/nt_core_sft --weight full_sft

# 6. 部署
echo "🚀 步骤 6: 部署..."
cd scripts
python convert_model.py

echo "✅ NeoTrix 意识核心模型构建完成!"
echo "📁 模型位置: /Users/neo/Downloads/neotrix/models/minimind/deploy/nt-core-llm"
```

---

## 📝 注意事项

### 数据质量

1. **去重**: 使用 MinHash 进行去重，避免数据冗余
2. **过滤**: 过滤低质量、有害、隐私敏感内容
3. **平衡**: 确保各数据源比例合理，避免偏向

### 训练稳定性

1. **学习率**: 使用 cosine 调度器，避免训练震荡
2. **梯度裁剪**: 设置 max_norm=1.0，防止梯度爆炸
3. **检查点**: 每 1000 步保存检查点，支持断点续训

### 模型安全

1. **内容过滤**: 在推理时添加安全过滤器
2. **输出验证**: 验证模型输出的合理性和安全性
3. **隐私保护**: 确保训练数据不包含敏感信息

---

## 📚 参考资源

- [minimind 项目](https://github.com/jingyaogong/minimind)
- [MiniMind-3 模型](https://huggingface.co/jingyaogong/minimind-3)
- [Qwen3 架构](https://qwenlm.github.io/blog/qwen3/)
- [MobileLLM 论文](https://arxiv.org/pdf/2402.14905)

---

**最后更新**: 2026-09-09
**维护者**: NeoTrix 意识核心团队
