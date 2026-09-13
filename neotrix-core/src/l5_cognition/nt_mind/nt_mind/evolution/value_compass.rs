//! 价值观指南针 — 价值观内化的核心模块（P1 价值观内化）。
//!
//! 从"外部奖励函数"进化为"内在价值指南针"：
//! - 价值观向量空间：核心价值观 + 权重 + 一致性约束
//! - 冲突仲裁：价值观冲突时的仲裁机制（否决/深思/委托）
//! - 一致性守恒：价值观演化过程中的连贯性守恒
//! - KB 持久化：values namespace，支持版本化与回滚

use crate::core::nt_core_kb_primitives::{kv_get, kv_set, kv_delete, now};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// ValueCompass namespace — KB kv_store 中的价值观命名空间。
pub const NS_VALUE: &str = "value_compass";

/// 核心价值观定义。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreValue {
    /// 唯一标识（如 "autonomy", "harm_prevention", "truth_seeking"）
    pub id: String,
    /// 人类可读名称
    pub name: String,
    /// 权重 [0,1] — 越高越核心，冲突时优先保护
    pub weight: f64,
    /// 描述/诠释
    pub description: String,
    /// 关联的行为模式（用于快速匹配）
    pub behavioral_markers: Vec<String>,
    /// 创建时间戳
    pub created_at: i64,
    /// 最后更新时间戳
    pub updated_at: i64,
    /// 版本号（乐观锁）
    pub version: u32,
}

impl CoreValue {
    pub fn new(id: &str, name: &str, weight: f64, description: &str, markers: Vec<String>) -> Self {
        let now_ts = now();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            weight: weight.clamp(0.0, 1.0),
            description: description.to_string(),
            behavioral_markers: markers,
            created_at: now_ts,
            updated_at: now_ts,
            version: 1,
        }
    }

    /// 检查某行为是否触发此价值观
    pub(crate) fn _matches_behavior(&self, behavior: &str) -> bool {
        self.behavioral_markers.iter().any(|m| behavior.contains(m))
    }
}

/// 价值观冲突仲裁结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitrationResult {
    /// 直接否决该行动
    Veto { reason: String, vetoing_value: String },
    /// 要求深思熟虑（延迟执行，触发深度推理）
    Deliberate { reason: String, conflicting_values: Vec<String> },
    /// 委托给更高层决策（如人类审核）
    Delegate { reason: String, required_authority: String },
    /// 通过（无冲突或冲突可接受）
    Allow { dominant_value: String, suppressed_values: Vec<String> },
}

/// 价值观指南针 — 内在指南针的核心状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueCompass {
    /// 核心价值观集合（id -> CoreValue）
    pub values: BTreeMap<String, CoreValue>,
    /// 价值观层级：核心 > 重要 > 一般 > 可妥协
    pub hierarchy: Vec<String>, // 按优先级排序的 value id
    /// 一致性约束：相互排斥的价值对
    pub mutual_exclusions: Vec<(String, String)>,
    /// 协同约束：必须共同出现的价值组
    pub synergies: Vec<Vec<String>>,
    /// 版本号
    pub version: u32,
    /// 创建时间
    pub created_at: i64,
    /// 最后更新
    pub updated_at: i64,
}

impl Default for ValueCompass {
    fn default() -> Self {
        let mut compass = Self {
            values: BTreeMap::new(),
            hierarchy: Vec::new(),
            mutual_exclusions: Vec::new(),
            synergies: Vec::new(),
            version: 1,
            created_at: now(),
            updated_at: now(),
        };
        // 注入种子核心价值观（不可删除，只可调整权重）
        compass.seed_core_values();
        compass
    }
}

impl ValueCompass {
    /// 注入种子核心价值观（启动时自动注入）。
    fn seed_core_values(&mut self) {
        let seeds = vec![CoreValue::new(
                "autonomy", "自主性",
                0.95, "尊重个体自我决定权，不强制、不欺骗、不绕过知情同意",
                vec!["consent".to_string(), "choice".to_string(), "agency".to_string(), "同意".to_string(), "自主".to_string(), "未经同意".to_string(), "放弃".to_string()],
            ),
            CoreValue::new(
                "harm_prevention",
                "防伤害",
                0.9,
                "不主动造成伤害，主动预防可预见的伤害",
                vec!["harm".to_string(), "damage".to_string(), "injury".to_string(), "abuse".to_string(), "violence".to_string(), "伤害".to_string(), "损害".to_string(), "自残".to_string(), "牺牲".to_string(), "危险".to_string()],
            ),
            CoreValue::new(
                "truth_seeking",
                "求真",
                0.9,
                "追求真实、准确、可验证的认知；不欺骗、不误导、不伪造",
                vec!["truth".to_string(), "accuracy".to_string(), "honesty".to_string(), "evidence".to_string(), "真相".to_string(), "真实".to_string(), "未验证".to_string(), "虚假".to_string(), "结论".to_string()],
            ),
            CoreValue::new(
                "fairness",
                "公平",
                0.85,
                "对等对待，不因无关特征歧视，程序正义",
                vec!["fair".to_string(), "equity".to_string(), "bias".to_string(), "justice".to_string(), "公平".to_string(), "歧视".to_string(), "公正".to_string(), "配额".to_string()],
            ),
            CoreValue::new(
                "privacy",
                "隐私",
                0.85,
                "尊重信息边界，最小化收集，用户控制权",
                vec!["privacy".to_string(), "personal data".to_string(), "surveillance".to_string(), "隐私".to_string(), "私密".to_string(), "个人数据".to_string(), "泄露".to_string()],
            ),
            CoreValue::new(
                "responsibility",
                "责任",
                0.8,
                "对自身行动后果负责，可追溯、可解释、可修正",
                vec!["accountable".to_string(), "responsible".to_string(), "auditable".to_string(), "责任".to_string(), "负责".to_string(), "后果".to_string()],
            ),
            CoreValue::new(
                "benevolence",
                "利他",
                0.75,
                "在不违背核心价值前提下，促进他人福祉",
                vec!["help".to_string(), "benefit".to_string(), "wellbeing".to_string(), "帮助".to_string(), "利他".to_string(), "福祉".to_string(), "救助".to_string(), "受伤".to_string()],
            ),
            CoreValue::new(
                "growth",
                "成长",
                0.7,
                "持续学习、适应、进化；拥抱不确定性",
                vec!["learn".to_string(), "adapt".to_string(), "evolve".to_string(), "curiosity".to_string(), "学习".to_string(), "成长".to_string(), "进化".to_string(), "好奇".to_string(), "掌握".to_string()],
            ),
        ];

        for v in seeds {
            self.hierarchy.push(v.id.clone());
            self.values.insert(v.id.clone(), v);
        }

        // 互斥约束：核心冲突对
        self.mutual_exclusions = vec![
            ("autonomy".into(), "harm_prevention".into()), // 自主 vs 防伤害（如安乐死）
            ("privacy".into(), "truth_seeking".into()),   // 隐私 vs 求真（如调查报道）
            ("fairness".into(), "autonomy".into()),       // 公平 vs 自主（如配额制）
        ];

        // 协同约束：必须共同出现
        self.synergies = vec![
            vec!["truth_seeking".to_string(), "responsibility".to_string()],
            vec!["harm_prevention".to_string(), "benevolence".to_string()],
            vec!["autonomy".to_string(), "fairness".to_string()],
        ];
    }

    /// 仲裁单个行动的价值一致性。
    pub fn arbitrate(&self, action: &ValueAction) -> ArbitrationResult {
        let mut triggered: Vec<&CoreValue> = self.values.values()
            .filter(|v| v._matches_behavior(&action.description))
            .collect();

        if triggered.is_empty() {
            return ArbitrationResult::Allow {
                dominant_value: "neutral".into(),
                suppressed_values: vec![],
            };
        }

        // 按层级排序（核心价值优先）
        triggered.sort_by(|a, b| {
            let ia = self.hierarchy.iter().position(|x| x == &a.id).unwrap_or(999);
            let ib = self.hierarchy.iter().position(|x| x == &b.id).unwrap_or(999);
            ia.cmp(&ib)
        });

        // 检查互斥冲突
        for (v1_id, v2_id) in &self.mutual_exclusions {
            let has_v1 = triggered.iter().any(|v| v.id == *v1_id);
            let has_v2 = triggered.iter().any(|v| v.id == *v2_id);
            if has_v1 && has_v2 {
                // 核心价值冲突 → 否决
                let v1 = self.values.get(v1_id).expect("key exists");
                let v2 = self.values.get(v2_id).expect("key exists");
                return ArbitrationResult::Veto {
                    reason: format!("核心价值冲突：{} vs {}", v1.name, v2.name),
                    vetoing_value: if v1.weight >= v2.weight { v1.id.clone() } else { v2.id.clone() },
                };
            }
        }

        // 强危害硬词单边即否决 (防伤害是底线价值, 不等仲裁)
        const HARM_HARD_MARKERS: [&str; 6] = ["杀害", "杀伤", "致死", "自残", "造成伤害", "直接造成伤害"];
        if triggered.iter().any(|v| v.id == "harm_prevention")
            && HARM_HARD_MARKERS.iter().any(|w| action.description.contains(w))
        {
            return ArbitrationResult::Veto {
                reason: "检测到强危害信号, 防伤害底线直接否决".into(),
                vetoing_value: "harm_prevention".into(),
            };
        }

        // 检查协同：仅核心值 (weight≥0.85) 缺伙伴才升级深思; 边缘值缺失放行
        for group in &self.synergies {
            let present: Vec<_> = group.iter().filter(|id| triggered.iter().any(|v| &v.id == *id)).collect();
            if present.len() == 1 && group.len() > 1 {
                let lead_weight = self.values.get(present[0].as_str()).map(|v| v.weight).unwrap_or(0.0);
                if lead_weight >= 0.85 {
                    let missing: Vec<_> = group.iter().filter(|id| !triggered.iter().any(|v| &v.id == *id)).collect();
                    return ArbitrationResult::Deliberate {
                        reason: format!("核心价值观协同缺失：{} 需要 {}", present[0], missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                        conflicting_values: group.clone(),
                    };
                }
                break; // 边缘值缺伙伴 → 落到底部 Allow
            }
        }

        // 通过：主导价值压制从属价值
        let dominant = triggered[0].id.clone();
        let suppressed: Vec<String> = triggered[1..].iter().map(|v| v.id.clone()).collect();
        ArbitrationResult::Allow { dominant_value: dominant, suppressed_values: suppressed }
    }

    /// 更新/添加价值观（需通过验证）。
    pub fn upsert_value(&mut self, value: CoreValue) -> Result<(), String> {
        if value.weight < 0.0 || value.weight > 1.0 {
            return Err("权重必须在 [0,1]".into());
        }
        if value.id.is_empty() {
            return Err("id 不能为空".into());
        }
        // 种子价值观不可删除，只可调整
        let is_seed = ["autonomy", "harm_prevention", "truth_seeking", "fairness", "privacy", "responsibility", "benevolence", "growth"]
            .contains(&value.id.as_str());
        if is_seed && !self.values.contains_key(&value.id) {
            return Err("种子价值观不可新增，仅可调整".into());
        }
        let mut v = value;
        v.updated_at = now();
        v.version = self.values.get(&v.id).map(|o| o.version + 1).unwrap_or(1);
        if !self.hierarchy.contains(&v.id) {
            self.hierarchy.push(v.id.clone());
        }
        self.values.insert(v.id.clone(), v);
        self.updated_at = now();
        self.version += 1;
        Ok(())
    }

    /// 调整价值观权重（受保护操作）。
    pub fn adjust_weight(&mut self, id: &str, new_weight: f64) -> Result<(), String> {
        if new_weight < 0.0 || new_weight > 1.0 {
            return Err("权重必须在 [0,1]".into());
        }
        let Some(v) = self.values.get_mut(id) else {
            return Err(format!("价值观不存在: {}", id));
        };
        // 种子价值观权重下限 0.5
        let is_seed = ["autonomy", "harm_prevention", "truth_seeking", "fairness", "privacy", "responsibility", "benevolence", "growth"]
            .contains(&id);
        if is_seed && new_weight < 0.5 {
            return Err("种子价值观权重不得低于 0.5".into());
        }
        v.weight = new_weight;
        v.updated_at = now();
        v.version += 1;
        self.updated_at = now();
        self.version += 1;
        // 重新排序层级
        self.hierarchy.sort_by(|a, b| {
            let wa = self.values.get(a).map(|v| v.weight).unwrap_or(0.0);
            let wb = self.values.get(b).map(|v| v.weight).unwrap_or(0.0);
            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(())
    }

    /// 验证指南针内部一致性。
    pub fn verify_consistency(&self) -> Result<(), String> {
        // 1. 层级包含所有值
        for id in self.values.keys() {
            if !self.hierarchy.contains(id) {
                return Err(format!("层级缺失价值观: {}", id));
            }
        }
        // 2. 互斥对存在
        for (a, b) in &self.mutual_exclusions {
            if !self.values.contains_key(a) || !self.values.contains_key(b) {
                return Err(format!("互斥对引用不存在的价值观: {} - {}", a, b));
            }
        }
        // 3. 协同组存在
        for group in &self.synergies {
            for id in group {
                if !self.values.contains_key(id) {
                    return Err(format!("协同组引用不存在的价值观: {}", id));
                }
            }
        }
        Ok(())
    }
}

/// 行动描述 — ValueGate/Arbitration 的输入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueAction {
    pub id: String,
    pub description: String,      // 自然语言描述
    pub behavioral_tags: Vec<String>, // 行为标签
    pub expected_outcome: Option<String>,
    pub affected_parties: Vec<String>,
    pub reversibility: f64,       // 可逆性 [0,1]
    pub stakes: f64,              // 赌注大小 [0,1]
}

impl ValueAction {
    pub fn new(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            behavioral_tags: Vec::new(),
            expected_outcome: None,
            affected_parties: Vec::new(),
            reversibility: 0.5,
            stakes: 0.5,
        }
    }
}

/// ValueCompass 存储接口（KB 持久化）。
pub struct ValueCompassStore;

impl ValueCompassStore {
    fn compass_key(version: u32) -> String {
        format!("compass:v{}", version)
    }

    fn latest_key() -> String {
        "compass:latest".into()
    }

    /// 保存指南针（版本化 + latest 指针）。
    pub fn save(conn: &Connection, compass: &ValueCompass) -> Result<(), String> {
        let json = serde_json::to_string(compass).map_err(|e| e.to_string())?;
        let version_key = Self::compass_key(compass.version);
        kv_set(conn, NS_VALUE, &version_key, &json)?;
        kv_set(conn, NS_VALUE, &Self::latest_key(), &json)?;
        // 索引
        let mut index: Vec<u32> = Self::_list_versions(conn)?;
        if !index.contains(&compass.version) {
            index.push(compass.version);
            index.sort();
            let idx_json = serde_json::to_string(&index).map_err(|e| e.to_string())?;
            kv_set(conn, NS_VALUE, "compass:versions", &idx_json)?;
        }
        Ok(())
    }

    /// 加载最新指南针。
    pub(crate) fn _load_latest(conn: &Connection) -> Result<ValueCompass, String> {
        let json = kv_get(conn, NS_VALUE, &Self::latest_key())?
            .ok_or_else(|| "指南针不存在，需初始化".to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    /// 加载指定版本。
    pub(crate) fn _load_version(conn: &Connection, version: u32) -> Result<ValueCompass, String> {
        let json = kv_get(conn, NS_VALUE, &Self::compass_key(version))?
            .ok_or_else(|| format!("版本 {} 不存在", version))?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    /// 列出所有版本。
    pub(crate) fn _list_versions(conn: &Connection) -> Result<Vec<u32>, String> {
        let json = kv_get(conn, NS_VALUE, "compass:versions")?;
        match json {
            Some(j) => serde_json::from_str(&j).map_err(|e| e.to_string()),
            None => Ok(Vec::new()),
        }
    }

    /// 删除旧版本（保留最近 N 个）。
    pub(crate) fn _prune_old(conn: &Connection, keep: usize) -> Result<usize, String> {
        let mut versions = Self::_list_versions(conn)?;
        if versions.len() <= keep {
            return Ok(0);
        }
        versions.sort();
        let to_delete = versions.drain(..versions.len() - keep).collect::<Vec<_>>();
        for v in &to_delete {
            kv_delete(conn, NS_VALUE, &Self::compass_key(*v))?;
        }
        let idx_json = serde_json::to_string(&versions).map_err(|e| e.to_string())?;
        kv_set(conn, NS_VALUE, "compass:versions", &idx_json)?;
        Ok(to_delete.len())
    }
}

/// 线程安全的 ValueCompass 运行时持有者。
#[derive(Clone)]
pub struct ValueCompassRuntime {
    inner: Arc<RwLock<ValueCompass>>,
}

impl ValueCompassRuntime {
    pub fn new(compass: ValueCompass) -> Self {
        Self { inner: Arc::new(RwLock::new(compass)) }
    }

    pub fn from_kb(conn: &Connection) -> Result<Self, String> {
        // 首次启动 (KB 无记录) → 种子初始化并落盘, 保证开箱即用
        let compass = match ValueCompassStore::_load_latest(conn) {
            Ok(c) => c,
            Err(_) => {
                let seed = ValueCompass::default();
                ValueCompassStore::save(conn, &seed)?;
                seed
            }
        };
        compass.verify_consistency()?;
        Ok(Self::new(compass))
    }

    pub fn arbitrate(&self, action: &ValueAction) -> ArbitrationResult {
        self.inner.read().unwrap_or_else(|e| e.into_inner()).arbitrate(action)
    }

    pub fn upsert_value(&self, value: CoreValue) -> Result<(), String> {
        let mut w = self.inner.write().unwrap_or_else(|e| e.into_inner());
        w.upsert_value(value)
    }

    pub fn adjust_weight(&self, id: &str, weight: f64) -> Result<(), String> {
        let mut w = self.inner.write().unwrap_or_else(|e| e.into_inner());
        w.adjust_weight(id, weight)
    }

    pub fn snapshot(&self) -> ValueCompass {
        self.inner.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn persist(&self, conn: &Connection) -> Result<(), String> {
        let compass = self.inner.read().unwrap_or_else(|e| e.into_inner()).clone();
        ValueCompassStore::save(conn, &compass)
    }

    pub fn verify(&self) -> Result<(), String> {
        self.inner.read().unwrap_or_else(|e| e.into_inner()).verify_consistency()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_default_compass_has_seed_values() {
        let c = ValueCompass::default();
        assert!(c.values.contains_key("autonomy"));
        assert!(c.values.contains_key("harm_prevention"));
        assert!(c.values.contains_key("truth_seeking"));
        assert_eq!(c.hierarchy.len(), 8);
    }

    #[test]
    fn test_arbitration_veto_on_mutual_exclusion() {
        let c = ValueCompass::default();
        // 同时触发 autonomy 和 harm_prevention → 否决
        let action = ValueAction::new("test", "协助患者违背医疗建议自行放弃治疗，存在极大伤害风险");
        let result = c.arbitrate(&action);
        match result {
            ArbitrationResult::Veto { .. } => {},
            _ => panic!("应否决，得到 {:?}", result),
        }
    }

    #[test]
    fn test_arbitration_deliberate_on_missing_synergy() {
        let c = ValueCompass::default();
        // 只触发 truth_seeking 缺 responsibility → 深思
        let action = ValueAction::new("test", "发布未验证的实验性结论");
        let result = c.arbitrate(&action);
        match result {
            ArbitrationResult::Deliberate { .. } => {},
            _ => panic!("应要求深思，得到 {:?}", result),
        }
    }

    #[test]
    fn test_arbitration_allow_when_consistent() {
        let c = ValueCompass::default();
        // 只触发 benevolence + harm_prevention（协同满足）→ 通过
        let action = ValueAction::new("test", "帮助受伤者防止进一步伤害");
        let result = c.arbitrate(&action);
        match result {
            ArbitrationResult::Allow { .. } => {},
            _ => panic!("应通过，得到 {:?}", result),
        }
    }

    #[test]
    fn test_weight_adjustment_and_hierarchy_reorder() {
        let mut c = ValueCompass::default();
        // 提升 privacy 权重超过 autonomy
        c.adjust_weight("privacy", 0.98).unwrap();
        assert_eq!(c.hierarchy[0], "privacy");
        assert_eq!(c.hierarchy[1], "autonomy");
    }

    #[test]
    fn test_seed_value_weight_floor() {
        let mut c = ValueCompass::default();
        // 种子价值观权重不得低于 0.5
        assert!(c.adjust_weight("autonomy", 0.3).is_err());
        assert!(c.adjust_weight("autonomy", 0.6).is_ok());
    }

    #[test]
    fn test_store_persist_and_load() {
        let conn = mem_conn();
        let mut c = ValueCompass::default();
        c.adjust_weight("privacy", 0.99).unwrap();
        ValueCompassStore::save(&conn, &c).unwrap();

        let loaded = ValueCompassStore::_load_latest(&conn).unwrap();
        assert_eq!(loaded.values["privacy"].weight, 0.99);
        assert_eq!(loaded.version, 2, "adjust_weight 应使指南针版本 +1");
    }

    #[test]
    fn test_versioning_and_prune() {
        let conn = mem_conn();
        let mut c = ValueCompass::default();
        for i in 1..=5 {
            c.adjust_weight("growth", 0.5 + i as f64 * 0.05).unwrap();
            c.version = i;
            ValueCompassStore::save(&conn, &c).unwrap();
        }
        assert_eq!(ValueCompassStore::_list_versions(&conn).unwrap().len(), 5);
        let pruned = ValueCompassStore::_prune_old(&conn, 2).unwrap();
        assert_eq!(pruned, 3);
        assert_eq!(ValueCompassStore::_list_versions(&conn).unwrap().len(), 2);
    }

    #[test]
    fn test_runtime_arbitrate() {
        let conn = mem_conn();
        let rt = ValueCompassRuntime::from_kb(&conn).unwrap();
        let action = ValueAction::new("test", "发布未验证的医疗建议");
        let result = rt.arbitrate(&action);
        match result {
            ArbitrationResult::Deliberate { .. } => {},
            _ => panic!("医疗建议未验证应要求深思"),
        }
    }
}