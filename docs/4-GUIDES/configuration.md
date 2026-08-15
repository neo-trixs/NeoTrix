# Configuration

## Config File

`~/.config/neotrix/config.toml`

```toml
provider = "opencode"
api_key = "enc:..."        # 加密存储 (AES-256-GCM), 见下文
default_model = "opencode/gpt-4o-mini"
color_mode = "dark"
```

## API Key 加密存储

API 密钥不落明文。`neotrix config encrypt-keys` 会加密所有明文 `api_key`/`secret` 字段，
`decrypt-keys` 用于解密（谨慎使用）。加密值以 `enc:` 前缀存储。

密钥来源优先级 (见 `nt_shield/key_encryption.rs`):

1. **OS keychain** (macOS Keychain / libsecret), 生产默认
2. `NEOTRIX_VAULT_KEY` 环境变量 — 显式提供 32 字节密钥
3. 机器派生密钥 — **仅显式 opt-in**: `NEOTRIX_ALLOW_MACHINE_KEY=1`

### 解密失败处理

启动时若无法用当前密钥解密旧加密值, 会打印警告 `[config] warning: failed to decrypt api_key`
并**降级继续运行** (不阻断启动)。常见原因: keychain 条目失效/重装系统。
修复: 重新 `neotrix config encrypt-keys` (用新密钥重加密), 或设置 `NEOTRIX_VAULT_KEY`
提供确定性的密钥。

## Environment Variables

Variables prefixed `NEOTRIX_`:
- `NEOTRIX_PROVIDER` — LLM provider
- `NEOTRIX_API_KEY` — API key (优先于 config.toml)
- `NEOTRIX_BASE_URL` — LLM base URL (自定义端点)
- `NEOTRIX_MODEL` — LLM model
- `NEOTRIX_EMBEDDING_API_KEY` — Embedding API key (语义搜索)
- `NEOTRIX_EMBEDDING_BASE_URL` — Embedding base URL
- `NEOTRIX_EMBEDDING_MODEL` — Embedding model
- `NEOTRIX_EMBEDDING_DIMENSION` — Embedding dimension
- `NEOTRIX_API_TOKEN` — HTTP serve 模式 Bearer 认证 token (保护 `/api/*`)
- `NEOTRIX_VAULT_KEY` — 32 字节 AES 密钥 (替代 OS keychain)
- `NEOTRIX_ALLOW_MACHINE_KEY` — 显式允许机器派生密钥 (默认拒绝, C-3 加固)
- `NEOTRIX_NETWORK_UNBLOCK` — 设为 `1` 放开 LLM 网络访问 (默认沙箱隔离拦截外部调用)
- `NEOTRIX_COT_*` — CoT 推理参数 (MODEL/MAX_TOKENS/TEMPERATURE/THINKING_BUDGET/STRUCTURED)
- `NEOTRIX_DISPATCH_*` — 多 agent 调度参数 (CONCURRENCY/AGGRESSION/TIMEOUT/MAX_SUBTASKS/VERIFIER/COT/HIDE_INTERNAL)
- `NEOTRIX_CLOUD_ENDPOINT` — 云同步端点
- `NEOTRIX_DISCOVER_LOCAL` / `NEOTRIX_DISCOVER_PROBE` — 本地服务发现
- `NEOTRIX_QUIET` — 抑制启动配置打印
