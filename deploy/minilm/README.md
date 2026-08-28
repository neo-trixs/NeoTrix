# MiniLM Embedding 服务部署

## 快速启动

```bash
cd deploy/minilm
docker-compose up -d
```

## 验证服务

```bash
# 健康检查
curl http://localhost:8237/health

# 嵌入测试
curl -X POST http://localhost:8237/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs": ["Hello world", "System design patterns"]}'
```

## 环境变量配置

NeoTrix 会自动检测 `NEOTRIX_EMBEDDING_API_KEY` 和 `NEOTRIX_EMBEDDING_ENDPOINT`：

```bash
export NEOTRIX_EMBEDDING_ENDPOINT=http://localhost:8237
export NEOTRIX_EMBEDDING_API_KEY=  # 可选，如果服务需要认证
```

## 性能参数

| 参数 | 值 | 说明 |
|------|-----|------|
| 模型 | all-MiniLM-L6-v2 | 384维，平衡速度/质量 |
| 批大小 | 32 | 并行处理能力 |
| 并发请求 | 128 | 吞吐能力 |
| 内存限制 | 4GB | 容器内存上限 |

## GPU 加速 (可选)

如果有 NVIDIA GPU，取消 `docker-compose.yml` 中 `minilm-gpu` 服务的注释，并注释掉 CPU 版本。

## 嵌入质量对比

| 方法 | 维度 | 语义质量 | 速度 |
|------|------|----------|------|
| hash-kernel | 384 | 低 | 极快 |
| MiniLM (CPU) | 384 | 高 | 中等 |
| MiniLM (GPU) | 384 | 高 | 极快 |
| OpenAI text-embedding-3-small | 1536 | 最高 | 慢 (网络) |

## 重新生成所有嵌入

服务启动后，运行重新嵌入脚本：

```bash
export NEOTRIX_EMBEDDING_ENDPOINT=http://localhost:8237
python3 scripts/regenerate_all_embeddings.py
```
