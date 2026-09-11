# Cleanup #227 — L3→L1 跨域引用修复

**日期**: 2026-09-11
**范围**: `l3_embodiment/` 目录下 6 处 `use crate::l1_action` 跨层引用
**模式**: L3 具身层直接引用 L1 行动层类型 → 改为通过 `l3_embodiment::l1_facade` 门面访问

## 背景

六层架构中，L3 具身层不应直接 `use crate::l1_action::*`。`l3_embodiment/l1_facade.rs` 已作为单一事实源 re-export 门面存在，其他 L3 文件（如 `audit.rs`、`risk_assessor.rs`、`api_proxy.rs`）已迁移到 facade 模式。

## 修复清单 (6/6)

### 1. vault.rs — L1Error/L1Result (sandbox feature)

**文件**: `l3_embodiment/nt_shield/nt_shield/vault.rs:11`
**Before**: `// use crate::l1_action::nt_io::nt_l1_error::{L1Error, L1Result};`
**After**: `use crate::l3_embodiment::l1_facade::{L1Error, L1Result};`
**说明**: 325 行 AES-256-GCM 加密凭证保险库，大量使用 `L1Error`/`L1Result`。注释掉的导入导致类型通过隐式路径可用，改为显式 facade 引用。

### 2. keyvault.rs — L1Error/L1Result (sandbox feature)

**文件**: `l3_embodiment/nt_shield/nt_shield/keyvault.rs:13`
**Before**: `// use crate::l1_action::nt_io::nt_l1_error::{L1Error, L1Result};`
**After**: `use crate::l3_embodiment::l1_facade::{L1Error, L1Result};`
**说明**: 343 行密钥保险库（支持文件/keyring 双后端），同 vault.rs 情况。

### 3. nt_shield_sandbox_entry.rs — L1Error/L1Result (sandbox feature)

**文件**: `l3_embodiment/nt_shield/nt_shield_sandbox_entry.rs:3`
**Before**: `// use crate::l1_action::nt_io::nt_l1_error::{L1Error, L1Result};`
**After**: `use crate::l3_embodiment::l1_facade::{L1Error, L1Result};`
**说明**: 430 行沙箱入口（Local/Docker/WASM/Remote 四模式），使用 `L1Error::Wasm`、`L1Error::Command` 等变体。

### 4. tor_client.rs — L1Error/L1Result (stealth-net feature)

**文件**: `l3_embodiment/nt_shield/nt_shield_stealth_net/tor_client.rs:9`
**Before**: `// use crate::l1_action::nt_io::nt_l1_error::{L1Error, L1Result};`
**After**: `use crate::l3_embodiment::l1_facade::{L1Error, L1Result};`
**说明**: 631 行 Tor SOCKS5 客户端，使用 `L1Error::Network`、`L1Error::Command`、`L1Error::Config` 等变体。

### 5. nt_shield_mcp_security.rs — McpTransport (dead reference)

**文件**: `l3_embodiment/nt_shield/nt_shield/nt_shield_mcp_security.rs:10`
**Before**: `// use crate::l1_action::nt_act::types::McpTransport; // TODO: 待迁移至 L1`
**After**: *(删除整行)*
**说明**: `McpTransport` 类型在代码库中不存在（grep 零命中）。TODO 注释引用了已删除的类型。此文件无任何对 `McpTransport` 的实际使用。

### 6. proxy_pool.rs — KbProvider (dead reference)

**文件**: `l3_embodiment/nt_shield/nt_shield_stealth_net/proxy_pool.rs:14`
**Before**: `// use crate::l1_action::nt_act::nt_l1_shared_types::KbProvider;`
**After**: *(删除整行)*
**说明**: `KbProvider` trait 在代码库中不存在（grep 零命中）。但文件 72 行和 100 行仍引用 `Arc<dyn KbProvider>`。**注意**: 此文件编译依赖 `stealth-net` feature，当前 `cargo check`（default features）不触发此路径。`KbProvider` 引用是技术债，需单独清理。

## 验证

```bash
# 跨层引用清零
grep -rn "use crate::l1_action" l3_embodiment/ --include="*.rs" | grep -v test | grep -v facade
# → 0 matches

# facade 使用一致性（与其他 L3 文件对齐）
grep -rn "l1_facade" l3_embodiment/ --include="*.rs" | grep -v test | grep -v 'l1_facade.rs' | grep -v mod.rs
# → vault.rs, keyvault.rs, sandbox_entry.rs, tor_client.rs + 已有 audit.rs, risk_assessor.rs, api_proxy.rs
```

## 技术债

| 项目 | 优先级 | 说明 |
|------|--------|------|
| `proxy_pool.rs` 的 `Arc<dyn KbProvider>` | P1 | 类型不存在，`stealth-net` feature 启用时编译会失败。需定义 `KbProvider` trait 或移除引用。 |
| `vault.rs`/`keyvault.rs` 隐式类型来源 | P2 | 注释掉的导入下类型仍可用，说明存在隐式 re-export 路径。修复后改为显式 facade 引用，更清晰。 |

## 影响范围

- **feature gates**: `sandbox` (vault, keyvault, sandbox_entry), `stealth-net` (tor_client, proxy_pool)
- **零行为变更**: 仅导入路径重定向，不改逻辑
- **向后兼容**: facade 是已有稳定 API，无 breaking change
