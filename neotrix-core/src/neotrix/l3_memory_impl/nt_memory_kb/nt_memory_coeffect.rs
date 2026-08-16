//! coeffect 依赖声明表 — 吸收自 cordiverse/paper §3.2 (reactive coeffects)
//! 与 §3.2.1 Def 22/23 (coeffect context Σ)。
//!
//! 机制:
//! - 依赖 = 类型化部分函数表 (k:K)⇀𝒱k: 每 key 有静态类型标签 (inject 依赖)。
//! - `set` 走 effect (可撤回): "coeffect operations are effects, and
//!   effects are revertible" (P18) — 所有依赖变更经 RevertibleContext 记录。
//! - 前置条件 (Def 23): set(k,v) 要求 k∉dom(σ) (不可重复提供); unset 要求
//!   k∈dom(σ) (不可撤销不存在项)。
//! - 通知 (Def 26): 状态转移按规格分类 activating / deactivating / neutral;
//!   激活触发 effect 执行, 去激活触发 accumulator 恢复。
//!
//! NeoTrix 消费方 (R-P79): **NT-MEMORY KB 注册表** — 技能/插件/能力声明
//! `inject` 依赖 key, 注入走 effect 通道 (可回滚), 前置条件防重复提供。
//! 持久化到 `kv_store` namespace `coeffect_deps` (与 knowledge.db 同源)。

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use crate::core::nt_core_context::revertible::{ClosureEffect, RevertibleContext};
use crate::neotrix::nt_memory_kb::nt_memory_unify::{kv_delete, kv_get, kv_set};

/// coeffect 依赖表的持久化 namespace。
pub const COEFFECT_NS: &str = "coeffect_deps";

/// 单个依赖声明: key + 提供方 + 值 + 前置条件状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoeffectBinding {
    /// 依赖 key (类型化依赖表的键)。
    pub key: String,
    /// 提供方 (fiber/技能名, 单源纪律: 每 key 唯一 provider)。
    pub provider: String,
    /// 值 (JSON 字符串序列化)。
    pub value: String,
}

impl CoeffectBinding {
    pub fn new(key: impl Into<String>, provider: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            provider: provider.into(),
            value: value.into(),
        }
    }
}

/// 内存态 coeffect 表 — 类型化依赖表 (Def 22: (k:K)⇀𝒱k)。
/// 所有 set/unset 走 RevertibleContext (可撤回), 满足 Def 23 前置条件。
#[derive(Debug, Clone, Default)]
pub struct CoeffectRegistry {
    /// key → (provider, value)。单源: 每 key 唯一 provider。
    dom: HashMap<String, (String, String)>,
}

impl CoeffectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&(String, String)> {
        self.dom.get(key)
    }

    pub fn provider_of(&self, key: &str) -> Option<&str> {
        self.dom.get(key).map(|(p, _)| p.as_str())
    }

    pub fn keys(&self) -> Vec<&str> {
        self.dom.keys().map(String::as_str).collect()
    }

    /// dom 内 key 集合 (用于 activating/deactivating 判定)。
    pub fn dom(&self) -> HashSet<&str> {
        self.dom.keys().map(String::as_str).collect()
    }

}

/// 通知分类 (Def 26): activating / deactivating / neutral。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoeffectNotif {
    Activating,
    Deactivating,
    Neutral,
}

impl CoeffectNotif {
    /// 状态转移按规格分类: σ⊭d∧σ'⊧d → Activating; σ⊧d∧σ'⊭d → Deactivating。
    pub fn classify(had_before: bool, has_after: bool) -> Self {
        match (had_before, has_after) {
            (false, true) => CoeffectNotif::Activating,
            (true, false) => CoeffectNotif::Deactivating,
            _ => CoeffectNotif::Neutral,
        }
    }
}

/// 通知回调: 依赖 key 激活/去激活时被调用。
/// `Activating` → 执行 effect (accumulator 记录); `Deactivating` → accumulator 恢复。
pub type NotifyFn = Box<dyn Fn(&str, CoeffectNotif) + Send + Sync>;

/// 事务化注入批次 — 一组依赖声明原子提交, 任一前置条件违反整体回滚。
///
/// 语义对齐 §3.2 "coeffect operations are effects, and effects are revertible":
/// 所有 set 走 RevertibleContext (∂Γ), unset 为 inverse, 失败 recover 到批前状态。
pub struct CoeffectTx<'a> {
    ctx: RevertibleContext<'a, &'a mut CoeffectRegistry>,
    applied: Vec<CoeffectBinding>,
    notifies: Vec<NotifyFn>,
}

impl<'a> CoeffectTx<'a> {
    /// 开始事务 (借用注册表; 生命周期受 `'a` 约束)。
    pub fn begin(reg: &'a mut CoeffectRegistry) -> Self {
        Self {
            ctx: RevertibleContext::new(reg),
            applied: Vec::new(),
            notifies: Vec::new(),
        }
    }

    /// 注册通知回调 (每个 set/unset 后按分类触发)。
    pub fn on_notify(mut self, f: NotifyFn) -> Self {
        self.notifies.push(f);
        self
    }

    /// 事务化 set: 前置条件违反 → 整体回滚 (all-or-nothing)。
    pub fn set(&mut self, binding: CoeffectBinding) -> Result<(), String> {
        let existing = self.ctx.state().dom.get(&binding.key).cloned();
        if let Some((existing_provider, _)) = existing {
            self.ctx.recover();
            return Err(format!(
                "coeffect precondition violated: key '{}' already provided by '{}'",
                binding.key, existing_provider
            ));
        }
        let key = binding.key.clone();
        let provider = binding.provider.clone();
        let value = binding.value.clone();
        self.ctx.track(ClosureEffect::new(
            format!("set:{}", key),
            {
                let key = key.clone();
                let provider = provider.clone();
                let value = value.clone();
                move |reg: &mut &mut CoeffectRegistry| {
                    reg.dom.insert(key.clone(), (provider.clone(), value.clone()));
                }
            },
            {
                let key = key.clone();
                move |reg: &mut &mut CoeffectRegistry| {
                    reg.dom.remove(&key);
                }
            },
        ));
        self.applied.push(binding);
        for n in &self.notifies {
            n(&key, CoeffectNotif::classify(false, true));
        }
        Ok(())
    }

    /// 事务化 unset: 撤销不存在项 → 整体回滚。
    pub fn unset(&mut self, key: &str) -> Result<(), String> {
        let key_owned = key.to_string();
        let Some(prev) = self.ctx.state().dom.get(key).cloned() else {
            self.ctx.recover();
            return Err(format!(
                "coeffect precondition violated: cannot unset nonexistent key '{}'",
                key
            ));
        };
        self.ctx.track(ClosureEffect::new(
            format!("unset:{}", key_owned),
            {
                let key = key_owned.clone();
                move |reg: &mut &mut CoeffectRegistry| {
                    reg.dom.remove(&key);
                }
            },
            {
                let key = key_owned.clone();
                let prev = prev.clone();
                move |reg: &mut &mut CoeffectRegistry| {
                    reg.dom.insert(key.clone(), prev.clone());
                }
            },
        ));
        self.applied.retain(|b| b.key != key);
        for n in &self.notifies {
            n(&key_owned, CoeffectNotif::classify(true, false));
        }
        Ok(())
    }

    /// 显式回滚到批前状态 (撤销全部 set/unset)。
    pub fn rollback(&mut self) {
        self.ctx.recover();
        self.applied.clear();
    }

    /// 成功提交 — 返回本次注入的绑定列表 (持久化由调用方负责)。
    pub fn commit(self) -> Vec<CoeffectBinding> {
        self.applied
    }

    /// 检查注册表当前状态 (借用读)。
    pub fn state(&self) -> &CoeffectRegistry {
        self.ctx.state()
    }
}

/// 持久化: 将一次已提交的绑定写入 `kv_store` namespace `coeffect_deps`。
/// 失败不阻断 (持久化尽力而为; 内存态已生效)。
pub fn persist_bindings(conn: &Connection, bindings: &[CoeffectBinding]) -> Result<usize, String> {
    let mut n = 0;
    for b in bindings {
        let serialized = format!("{}|{}", b.provider, b.value);
        kv_set(conn, COEFFECT_NS, &b.key, &serialized)?;
        n += 1;
    }
    Ok(n)
}

/// 从 `kv_store` 恢复绑定到内存注册表。
pub fn load_bindings(conn: &Connection, reg: &mut CoeffectRegistry) -> Result<usize, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM kv_store WHERE namespace=?1 ORDER BY key")
        .map_err(|e| format!("load_bindings prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![COEFFECT_NS], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("load_bindings query: {}", e))?;
    let mut n = 0;
    for row in rows {
        let (key, serialized) = row.map_err(|e| format!("load_bindings row: {}", e))?;
        if let Some((provider, value)) = serialized.split_once('|') {
            reg.dom.insert(key, (provider.to_string(), value.to_string()));
            n += 1;
        }
    }
    Ok(n)
}

/// 清除持久化的依赖表 (namespace 整体删除)。
pub fn clear_bindings(conn: &Connection) -> Result<usize, String> {
    let mut n = 0;
    let keys = kv_get_all_keys(conn)?;
    for k in keys {
        if kv_delete(conn, COEFFECT_NS, &k)? {
            n += 1;
        }
    }
    Ok(n)
}

/// 列出 namespace 全部 key (供 clear 使用)。
fn kv_get_all_keys(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT key FROM kv_store WHERE namespace=?1 ORDER BY key")
        .map_err(|e| format!("kv_get_all_keys prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![COEFFECT_NS], |row| row.get::<_, String>(0))
        .map_err(|e| format!("kv_get_all_keys query: {}", e))?;
    let mut keys = Vec::new();
    for row in rows {
        keys.push(row.map_err(|e| format!("kv_get_all_keys row: {}", e))?);
    }
    Ok(keys)
}

/// 便捷读取: 从持久化读单个绑定 (不加载进内存表)。
pub fn get_persisted_binding(conn: &Connection, key: &str) -> Result<Option<CoeffectBinding>, String> {
    match kv_get(conn, COEFFECT_NS, key)? {
        Some(serialized) => {
            if let Some((provider, value)) = serialized.split_once('|') {
                Ok(Some(CoeffectBinding::new(key, provider, value)))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut reg = CoeffectRegistry::new();
        let bindings = {
            let mut tx = CoeffectTx::begin(&mut reg);
            tx.set(CoeffectBinding::new("db", "knowledge", "{\"path\":\"kb.db\"}")).unwrap();
            tx.commit()
        };
        assert_eq!(bindings.len(), 1);
        assert_eq!(reg.provider_of("db"), Some("knowledge"));
        assert!(reg.get("db").is_some());
    }

    #[test]
    fn test_set_duplicate_precondition_violated() {
        let mut reg = CoeffectRegistry::new();
        let err = {
            let mut tx = CoeffectTx::begin(&mut reg);
            tx.set(CoeffectBinding::new("db", "knowledge", "v1")).unwrap();
            // 重复提供 → 前置条件违反 (Def 23: k∉dom), 整批 all-or-nothing 回滚
            tx.set(CoeffectBinding::new("db", "other", "v2")).unwrap_err()
        };
        assert!(err.contains("already provided"));
        // all-or-nothing: 整批回滚, 第一条 db 也撤销
        assert!(reg.dom.is_empty(), "整批回滚后 dom 应为空: {:?}", reg.dom);
    }

    #[test]
    fn test_unset_nonexistent_precondition_violated() {
        let mut reg = CoeffectRegistry::new();
        let err = {
            let mut tx = CoeffectTx::begin(&mut reg);
            tx.unset("nope").unwrap_err()
        };
        assert!(err.contains("nonexistent"));
        assert!(reg.dom.is_empty());
    }

    #[test]
    fn test_batch_rollback_restores_previous() {
        let mut reg = CoeffectRegistry::new();
        let err = {
            let mut tx = CoeffectTx::begin(&mut reg);
            tx.set(CoeffectBinding::new("a", "p1", "1")).unwrap();
            tx.set(CoeffectBinding::new("b", "p2", "2")).unwrap();
            // 第三次违反前置条件 → 整批回滚 (a, b 全部撤销)
            tx.set(CoeffectBinding::new("a", "p3", "3")).unwrap_err()
        };
        assert!(err.contains("already provided"));
        assert!(reg.dom.is_empty(), "回滚后 dom 应为空: {:?}", reg.dom);
    }

    #[test]
    fn test_notify_activating_deactivating() {
        let mut reg = CoeffectRegistry::new();
        let seen_ref: std::sync::Arc<std::sync::Mutex<Vec<(String, CoeffectNotif)>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        {
            let seen_clone = seen_ref.clone();
            let mut tx = CoeffectTx::begin(&mut reg);
            tx = tx.on_notify(Box::new(move |k, n| {
                seen_clone.lock().unwrap().push((k.to_string(), n));
            }));
            tx.set(CoeffectBinding::new("db", "k", "v")).unwrap();
            tx.unset("db").unwrap();
            tx.commit();
        }
        let seen = seen_ref.lock().unwrap().clone();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].1, CoeffectNotif::Activating);
        assert_eq!(seen[1].1, CoeffectNotif::Deactivating);
    }

    #[test]
    fn test_persist_and_load_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        let _ = crate::neotrix::nt_memory_kb::nt_memory_schema::initialize(&conn);
        let _reg = CoeffectRegistry::new();
        let bindings = vec![CoeffectBinding::new("db", "knowledge", "{\"path\":\"kb.db\"}")];
        assert_eq!(persist_bindings(&conn, &bindings).unwrap(), 1);
        let mut restored = CoeffectRegistry::new();
        assert_eq!(load_bindings(&conn, &mut restored).unwrap(), 1);
        assert_eq!(restored.provider_of("db"), Some("knowledge"));
        assert!(get_persisted_binding(&conn, "db").unwrap().is_some());
        assert!(get_persisted_binding(&conn, "nope").unwrap().is_none());
        assert_eq!(clear_bindings(&conn).unwrap(), 1);
        assert!(load_bindings(&conn, &mut restored).unwrap() == 0);
    }
}