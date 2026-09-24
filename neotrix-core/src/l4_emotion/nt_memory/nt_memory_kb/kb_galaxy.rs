//! kb_galaxy — 从 `nt_memory_kb/mod.rs` 拆分 (行为零变更).
//! `impl KnowledgeBase` 按域搬运, 原文逐行, 仅补可见性/导入。


use super::KnowledgeBase;
use super::nt_memory_galaxy_hygiene;
use super::nt_memory_weave;

impl KnowledgeBase {
    /// 星系卫生代码强制 (T3 生产接线): 跨 namespace 校验真实 hub 的
    /// 幽灵分支 / 沉寂星辰 / 缺失 hub。由 BackgroundLoop 周期调用。
    pub fn galaxy_hygiene_check(
        &self,
        config: &nt_memory_galaxy_hygiene::GalaxyHygieneConfig,
    ) -> nt_memory_galaxy_hygiene::GalaxyHygieneReport {
        match self.conn.lock() {
            Ok(conn) => nt_memory_galaxy_hygiene::galaxy_hygiene_check(&conn, config),
            Err(e) => {
                let mut report = nt_memory_galaxy_hygiene::GalaxyHygieneReport::default();
                report
                    .findings
                    .push(format!("[error] KB lock failed: {}", e));
                report
            }
        }
    }

    /// 原生星辰唤醒 (T3 生产接线): 技能激活事件落盘星辰活跃度。
    pub fn galaxy_wake_star(&self, ns: &str) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_galaxy_hygiene::galaxy_wake_star(&conn, ns)
    }

    /// 跨会话织网一次 (NT-NEXUS T3 生产接线): 图谱维护 + 连接强化。
    /// 由 BackgroundLoop 周期调用, 与会话收尾 (experience absorb) 互补。
    pub fn weave_once(&self) -> Result<nt_memory_weave::WeaveReport, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_weave::weave_once(&conn)
    }

    /// 记录跨会话桥接 (NT-NEXUS): 显式连接两个 session 的共享主题。
    pub fn record_session_bridge(
        &self,
        session_a: &str,
        session_b: &str,
        implicit_keyword: &str,
        bridge_note: &str,
    ) -> Result<nt_memory_weave::SessionBridge, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_weave::record_session_bridge(
            &conn,
            session_a,
            session_b,
            implicit_keyword,
            bridge_note,
        )
    }

    /// 连接两个 pattern (NT-NEXUS): 命中强化 / 未命中新建。返回 (新建, 强化)。
    pub fn connect_patterns(
        &self,
        from_pattern: &str,
        to_pattern: &str,
    ) -> Result<(bool, bool), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_weave::connect_patterns(&conn, from_pattern, to_pattern)
    }

    /// 星辰 Persona 读取 (cumora.ai "Personas, not prompts" 借鉴): 返回 hub 的
    /// `persona` 子对象 `{role, voice, system_prompt}`。
    pub fn galaxy_get_persona(&self, ns: &str) -> Result<Option<serde_json::Value>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_galaxy_hygiene::galaxy_get_persona(&conn, ns)
    }

    /// 星辰 Persona 写入 (cumora.ai "Personas, not prompts" 借鉴): 部分更新
    /// hub 的 `persona` 子对象 (仅写入提供的字段)。
    pub fn galaxy_set_persona(
        &self,
        ns: &str,
        role: Option<&str>,
        voice: Option<&str>,
        system_prompt: Option<&str>,
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_galaxy_hygiene::galaxy_set_persona(&conn, ns, role, voice, system_prompt)
    }

    /// 沉寂星辰扫描: 返回 `(ns, 上次活跃 epoch, invocations)`。
    pub fn galaxy_wake_scan(&self, staleness_days: u64) -> Vec<(String, Option<u64>, u64)> {
        match self.conn.lock() {
            Ok(conn) => nt_memory_galaxy_hygiene::galaxy_wake_scan(&conn, staleness_days),
            Err(_) => Vec::new(),
        }
    }

    /// 发现真实星系 hub (各技能 namespace 的 key='hub')。
    pub fn galaxy_list_hubs(&self) -> Vec<(String, serde_json::Value)> {
        match self.conn.lock() {
            Ok(conn) => nt_memory_galaxy_hygiene::galaxy_list_hubs(&conn),
            Err(_) => Vec::new(),
        }
    }
}
