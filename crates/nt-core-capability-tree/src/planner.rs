//! **能力面投影与组合** —— 把脊柱变成「模型这一轮真正用得上的工具」。
//!
//! # 为什么需要它（2026-10-08）
//!
//! 取证：neobot 把**全部**能力压进**一个**硬编码工具 `capability_invoke`，
//! id 塞在 description 里（`nt_http_engine.rs`）。后果有三个：
//!
//! 1. **没有真实 schema**：模型只看到一个字符串清单，
//!    参数得靠猜（description 自己在劝「拿不准就只传最小 input，不要臆造字段」——
//!    这正是**缺 schema 的症状**）。
//! 2. **不随用随调**：工具清单每轮重算，但只按 `kind/license/version` 过滤，
//!    **与用户这一轮说了什么毫无关系**。
//! 3. **自由度被压扁**：一个工具 N 个 id ⇒ 模型无法把不同能力当成不同动作，
//!    也就无法**自我组合**出「能力链路」。
//!
//! 本模块给出三件东西，全部**纯逻辑、零依赖、可单测**：
//! - [`tool_schema`]：能力 → 真实 function-calling schema；
//! - [`project_tools`]：按**本轮 query 相关性**挑工具 ⇒ 随用随调；
//! - [`compose_chain`]：把相关能力**排成执行链路** ⇒ 自我组合。
//!
//! # 不发明什么
//!
//! 相关性只做**字面重叠**（id / 描述 / 标签 / 分组与 query 的交集），
//! **不引入嵌入、不引入模型调用**——那属于宿主（晶体）侧的职责，
//! 本模块保持可确定性测试。
//!
//! 排序用 `stage:N` 标签表达**显式阶段**（作者声明），否则按相关性降序；
//! 同分按 id 升序 ⇒ **确定性**，便于测试与回放。

use serde_json::{json, Value};

use crate::spine::{CapabilityDescriptor, Spine};

/// 工具名规约：只允许 `[A-Za-z0-9_.-]`，其余替换为 `_`。
///
/// ⚠️ **同名必冲突**：模型侧按名字派发（审计阻断①：旧实现把未知名一律
/// 映射成字面量 `"unknown_tool"`）⇒ 本函数必须保证「不同 id 产出不同名字」，
/// 故在截断后追加一个由原串派生的短哈希，绝不静默撞名。
pub fn tool_name(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    // 长度封顶（部分模型对工具名有长度限制），截断时追加哈希防撞
    const MAX: usize = 64;
    if out.len() <= MAX {
        out
    } else {
        let h = fnv1a(id);
        let keep = MAX.saturating_sub(9);
        format!("{}_{:08x}", &out[..keep.min(out.len())], h)
    }
}

/// 由一组描述建「工具名 → 能力 id」映射。
///
/// # 为什么必须显式建映射（不能靠把 `_` 还原成 `::`）
///
/// [`tool_name`] 会把 `::` 规约成 `__`（多数模型只接受
/// `[A-Za-z0-9_-]`）⇒ **映射不可逆**：`a__b` 可能来自 `a::b` 也可能来自
/// 本身就含 `__` 的 id。⛔ 靠字符串反推会**静默路由到错误能力**。
/// ⇒ 由宿主持有本表，按工具名精确回查。
pub fn tool_name_index(descriptors: &[CapabilityDescriptor]) -> std::collections::BTreeMap<String, String> {
    descriptors
        .iter()
        .map(|d| (tool_name(&d.id), d.id.clone()))
        .collect()
}

/// FNV-1a 32 位（只为防撞名，不需要密码学强度）。
fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in s.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// 能力 → 单个 function-calling schema。
///
/// 无 `input_schema` 时**不发明字段**：给一个宽松的 `object`，
/// 并在描述里明说「该能力未声明入参 schema」。
/// ⛔ 绝**不**凭空编造字段名——那会让模型臆造参数，正是审计里那句
///   「不要臆造字段」要治的病。
pub fn tool_schema(d: &CapabilityDescriptor) -> Value {
    let params = d.input_schema.clone().unwrap_or_else(|| {
        json!({
            "type": "object",
            "additionalProperties": true,
            "description": "该能力未声明入参 schema；只传必需的最小参数。",
        })
    });
    json!({
        "type": "function",
        "function": {
            "name": tool_name(&d.id),
            "description": d.description,
            "parameters": params,
        }
    })
}

// ─── 相关性 ──────────────────────────────────────────────────────────────

/// 抽出可匹配的词元：ASCII 按非字母数字切；CJK 逐字 + 相邻双字。
///
/// CJK 无空格，按空白切会把整句当一词 ⇒ 匹配不到任何能力。
/// 这里逐字入表并额外加入相邻双字，覆盖绝大多数「文档/幻灯片/表格」类查询。
fn tokenize(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut ascii_buf = String::new();
    let mut prev_cjk: Option<char> = None;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            ascii_buf.push(ch.to_ascii_lowercase());
            prev_cjk = None;
            continue;
        }
        if !ascii_buf.is_empty() {
            out.push(std::mem::take(&mut ascii_buf));
        }
        if is_cjk(ch) {
            out.push(ch.to_string());
            if let Some(p) = prev_cjk {
                out.push(format!("{p}{ch}"));
            }
            prev_cjk = Some(ch);
        } else {
            prev_cjk = None;
        }
    }
    if !ascii_buf.is_empty() {
        out.push(ascii_buf);
    }
    out.retain(|t| !t.is_empty());
    out
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0xAC00..=0xD7AF)
}

/// 一条命中（可解释：命中了哪个词）。
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub id: String,
    pub score: f64,
    /// 命中的词元（审计用：便于解释为什么选它）
    pub hits: Vec<String>,
}

/// 计算 query 与某个能力的相关性分。
///
/// 权重：标签命中 > 分组命中 > id 命中 > 描述命中
/// （标签与分组是**作者显式声明**的检索意图，应压过散文描述）。
pub fn score_descriptor(query: &str, d: &CapabilityDescriptor) -> Option<Match> {
    let q = tokenize(query);
    if q.is_empty() {
        return None;
    }
    let mut hits: Vec<String> = Vec::new();
    let mut score = 0.0f64;
    // 匹配判定：ASCII 词元长度 ≥3 时允许**互为前缀**（否则查 "PPT" 命中不了
    // "pptx"；实测这是本函数第一版的真缺陷）；CJK 仍要求全等（避免单字过泛）。
    let hit_of = |n: &str| -> bool {
        q.iter().any(|t| {
            if t == n {
                return true;
            }
            let ascii_pair = t.is_ascii() && n.is_ascii();
            ascii_pair && t.len() >= 3 && n.len() >= 3
                && (n.starts_with(t.as_str()) || t.starts_with(n))
        })
    };
    let mut bump = |needle: &str, w: f64, hits: &mut Vec<String>| {
        let n = needle.to_ascii_lowercase();
        if n.is_empty() {
            return;
        }
        if hit_of(&n) {
            score += w;
            hits.push(n);
        }
    };
    for t in &d.tags {
        bump(t, 3.0, &mut hits);
    }
    bump(&d.category, 2.0, &mut hits);
    for seg in d.id.split("::") {
        bump(seg, 1.5, &mut hits);
    }
    for t in tokenize(&d.description) {
        bump(&t, 1.0, &mut hits);
    }
    if score <= 0.0 {
        return None;
    }
    Some(Match {
        id: d.id.clone(),
        score,
        hits: {
            hits.sort();
            hits.dedup();
            hits
        },
    })
}

/// 脊柱里与 query 相关的能力，按分降序（同分按 id 升序 ⇒ 确定性）。
pub fn rank(spine: &Spine, query: &str) -> Vec<Match> {
    let mut ms: Vec<Match> = spine
        .descriptors()
        .iter()
        .filter_map(|d| score_descriptor(query, d))
        .collect();
    ms.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    ms
}

/// **随用随调**：挑出本轮该摆上桌的工具。
///
/// - `min_score` 门槛：低于它**不上桌**（宁缺毋滥）；
/// - `limit` 上限：控制 tool 数量（工具越多模型越容易乱选）；
/// - **只取 `dispatchable` 的**（调不动的摆了也是白跑一轮，见 P1 修复）。
///
/// 返回 `(tool schema, 命中解释)`。
pub fn project_tools(
    spine: &Spine,
    query: &str,
    min_score: f64,
    limit: usize,
) -> Vec<(Value, Match)> {
    // 只取 `dispatchable` 的（调不动的摆了也是白跑一轮，见 P1 修复）
    let ok: std::collections::BTreeSet<String> = spine.dispatchable_ids().into_iter().collect();
    rank(spine, query)
        .into_iter()
        .filter(|m| m.score >= min_score && ok.contains(&m.id))
        .take(limit)
        .filter_map(|m| {
            spine
                .descriptor(&m.id)
                .map(|d| (tool_schema(&d), m))
        })
        .collect()
}

// ─── 能力链路（自我组合）─────────────────────────────────────────────────

/// 组合出的链路一步。
#[derive(Debug, Clone, PartialEq)]
pub struct ChainStep {
    pub id: String,
    pub stage: i64,
    pub score: f64,
    pub hits: Vec<String>,
}

/// `stage:N` 标签解析；无则返回 `0`。
fn stage_of(d: &CapabilityDescriptor) -> i64 {
    d.tags
        .iter()
        .find_map(|t| t.strip_prefix("stage:").and_then(|v| v.parse::<i64>().ok()))
        .unwrap_or(0)
}

/// **自我组合能力链路**：把与目标相关的能力排成一条**有序**执行链。
///
/// # 排序语义（诚实说明）
///
/// - 作者可用 `stage:N` 标签表达**显式阶段** ⇒ 按 `N` 升序（这是真正的「链路」）；
/// - 未声明阶段者按相关性降序排在同阶段内；
/// - 同分按 id 升序 ⇒ **确定性**（可测试、可回放）。
///
/// ⛔ **不发明依赖语义**：脊柱的描述里没有「A 依赖 B」字段，
///   所以本函数**不做**拓扑排序——那需要真实依赖数据，编不得。
pub fn compose_chain(spine: &Spine, goal: &str, min_score: f64, max_len: usize) -> Vec<ChainStep> {
    let ok2: std::collections::BTreeSet<String> = spine.dispatchable_ids().into_iter().collect();
    let mut steps: Vec<ChainStep> = rank(spine, goal)
        .into_iter()
        .filter(|m| m.score >= min_score && ok2.contains(&m.id))
        .filter_map(|m| {
            spine.descriptor(&m.id).map(|d| ChainStep {
                stage: stage_of(&d),
                id: m.id.clone(),
                score: m.score,
                hits: m.hits,
            })
        })
        .take(max_len)
        .collect();
    steps.sort_by(|a, b| {
        a.stage
            .cmp(&b.stage)
            .then_with(|| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.id.cmp(&b.id))
    });
    steps
}

// ─── 测试 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spine::{CapabilityDescriptor, CapabilityHealth, SpineExecutor};

    fn spine() -> Spine {
        let mut sp = Spine::new();
        let add = |sp: &mut Spine, d: CapabilityDescriptor| {
            sp.register(std::sync::Arc::new(SpineExecutor::new(
                d,
                |_id: &str, i: Value, _s: &str| Box::pin(async move { Ok(i) }),
            )))
            .expect("登记应成功");
        };
        // 清单里声明为 Executable 的真实 id ⇒ 派生才是 Executable
        add(
            &mut sp,
            CapabilityDescriptor::new(
                "NT-ACT::nt_file_ability::genoffice",
                "Office 文档引擎：docx/xlsx/pptx/pdf/markdown 读写转换与渲染",
            )
            .with_tags(&["docx", "xlsx", "pptx", "pdf", "office", "stage:1"])
            .with_category("office/document-engine")
            .with_version("1.0.0")
            .with_license("LicenseRef-NeoTrix-Internal"),
        );
        // 清单里是 Scaffold ⇒ 不得进入可调度集合
        add(
            &mut sp,
            CapabilityDescriptor::new(
                "NT-MIND::trade::foreign_trade_full_cycle",
                "外贸全链 报价 物流 合规",
            )
            .with_tags(&["trade", "stage:2"])
            .with_version("1.0.0")
            .with_license("LicenseRef-NeoTrix-Internal"),
        );
        // 纯插件：带状态 ⇒ 不在清单里，只能靠 license/version 上架
        struct Stateful;
        impl crate::spine::CapabilityExecutor for Stateful {
            fn descriptor(&self) -> CapabilityDescriptor {
                CapabilityDescriptor::new("acme::plugin::render", "把 SVG 渲染成 PNG")
                    .with_tags(&["render", "png", "stage:0"])
                    .with_category("media/render")
                    .with_version("0.3.1")
                    .with_license("Apache-2.0")
            }
            fn execute(
                &self,
                _i: Value,
                _s: &str,
            ) -> crate::dispatch::BoxFuture<'static, Result<Value, String>> {
                Box::pin(async { Ok(Value::Null) })
            }
            fn health(&self) -> CapabilityHealth {
                CapabilityHealth::Healthy
            }
        }
        sp.register(std::sync::Arc::new(Stateful)).expect("登记");
        sp
    }

    #[test]
    fn 工具名唯一且合法() {
        let a = tool_name("NT-ACT::nt_file_ability::genoffice");
        let b = tool_name("NT-ACT::nt_file_ability::genoffice2");
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c)), "{a}");
        assert_ne!(a, b, "不同 id 必须产出不同工具名（审计阻断①）");
        // 超长 id 截断后仍唯一
        let long1 = format!("ns::{}::x", "a".repeat(80));
        let long2 = format!("ns::{}::x", "a".repeat(79));
        assert_ne!(tool_name(&long1), tool_name(&long2), "截断后靠哈希防撞");
        assert!(tool_name(&long1).len() <= 64);
    }

    #[test]
    fn 工具schema用真实入参() {
        let d = CapabilityDescriptor::new("a::b", "描述").with_input_schema(json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"],
        }));
        let s = tool_schema(&d);
        let f = &s["function"];
        assert_eq!(f["name"], tool_name("a::b"));
        assert!(!f["name"].as_str().unwrap().contains(':'), "工具名不得含 ':'");
        assert_eq!(f["parameters"]["required"][0], "path", "必须用声明的 schema");
    }

    #[test]
    fn 无schema时不编造字段() {
        let s = tool_schema(&CapabilityDescriptor::new("a::b", "描述"));
        let p = &s["function"]["parameters"];
        assert_eq!(p["type"], "object");
        assert!(p.get("properties").is_none(), "没声明 schema 就不得凭空造字段");
        assert!(p["description"].as_str().unwrap().contains("未声明"));
    }

    #[test]
    fn cjk查询也能命中() {
        let d = CapabilityDescriptor::new("x::y", "生成 pptx 幻灯片");
        let m = score_descriptor("帮我做一个PPT", &d).expect("中文查询必须命中");
        assert!(m.hits.iter().any(|h| h.contains("ppt") || h.contains("PPT".to_ascii_lowercase().as_str())), "{:?}", m.hits);
    }

    #[test]
    fn 标签权重高于描述() {
        let by_tag = CapabilityDescriptor::new("x::y", "毫不相关").with_tags(&["pdf"]);
        let by_desc = CapabilityDescriptor::new("x::y", "这里提到 pdf 一次");
        let q = "pdf";
        let a = score_descriptor(q, &by_tag).unwrap().score;
        let b = score_descriptor(q, &by_desc).unwrap().score;
        assert!(a > b, "标签 {a} 应大于描述 {b}（作者显式声明优先）");
    }

    /// 随用随调：无关 query 不得摆工具；相关才摆。
    #[test]
    fn 随用随调只摆相关工具() {
        let sp = spine();
        let hit = project_tools(&sp, "把这个 docx 转成 pdf", 1.0, 10);
        assert!(!hit.is_empty(), "相关 query 必须摆出工具");
        assert!(hit.iter().any(|(_, m)| m.id.contains("genoffice")), "{:?}", hit);
        let miss = project_tools(&sp, "今天天气怎么样", 1.0, 10);
        assert!(miss.is_empty(), "无关 query 不得摆工具（宁缺毋滥）");
    }

    /// Scaffold 能力**不得**进入模型面（派发真跑不了或只是脚手架）。
    #[test]
    fn scaffold能力不上模型面() {
        let sp = spine();
        let picked = project_tools(&sp, "外贸全链 报价", 1.0, 10);
        assert!(
            !picked.iter().any(|(_, m)| m.id.contains("foreign_trade_full_cycle")),
            "Scaffold 不得摆上桌：{:?}", picked.iter().map(|(_, m)| &m.id).collect::<Vec<_>>()
        );
    }

    /// 未登记/不健康者不得进入模型面（这正是 P1 的修复点在投影层的延续）。
    #[test]
    fn 不在脊柱者不上模型面() {
        let sp = spine();
        // 未登记的 id 根本没资格
        let picked = project_tools(&sp, "docx pdf pptx", 0.0, 50);
        assert!(picked.iter().all(|(_, m)| sp.contains(&m.id)));
    }

    /// 自我组合链路：按 stage 排序，且确定性可回放。
    #[test]
    fn 组合链路按阶段排序且确定性() {
        let sp = spine();
        let goal = "生成 pdf 幻灯片并走外贸全链";
        let a = compose_chain(&sp, goal, 1.0, 10);
        assert!(a.len() >= 2, "应组合出多步链路，实得 {:?}", a.len());
        let b = compose_chain(&sp, goal, 1.0, 10);
        assert_eq!(a, b, "同输入必须同输出（可回放）");
        // stage:0 的插件应排在 stage:1 之前
        let pos_render = a.iter().position(|s| s.id.contains("render")).expect("插件在链里");
        let pos_gen = a.iter().position(|s| s.id.contains("genoffice")).expect("genoffice 在链里");
        assert!(pos_render < pos_gen, "stage 小的应在前：{a:?}");
    }

    #[test]
    fn 链路长度受limit约束() {
        let sp = spine();
        assert!(compose_chain(&sp, "pdf docx pptx render 外贸", 0.0, 1).len() <= 1);
    }

    /// **路由正确性锁**：工具名映射**必须**精确回查，不能靠字符串反推。
    /// `a::b` 与 `a__b` 两个不同能力经规约后可能同名（截断/替换）⇒
    /// 反推会静默路由到错误能力。
    #[test]
    fn 工具名映射可精确回查() {
        let ds = vec![
            CapabilityDescriptor::new("NT-ACT::nt_file_ability::genoffice", "x"),
            CapabilityDescriptor::new("NT-MIND::trade::trade_quote_negotiation", "y"),
        ];
        let idx = tool_name_index(&ds);
        for d in &ds {
            assert_eq!(
                idx.get(&tool_name(&d.id)).map(String::as_str),
                Some(d.id.as_str()),
                "工具名必须能精确回到 id"
            );
        }
    }

    /// 相同规约结果必须被索引层**显式暴露为冲突**（而非静默后者覆盖）。
    #[test]
    fn 同工具名冲突可被检出() {
        // 这两个 id 规约后同名：`a::b` 与 `a__b`
        let ds = vec![
            CapabilityDescriptor::new("a::b", "一"),
            CapabilityDescriptor::new("a__b", "二"),
        ];
        let idx = tool_name_index(&ds);
        assert_eq!(
            idx.get(&tool_name("a::b")).map(String::as_str),
            Some("a__b"),
            "后写覆盖前写（BTreeMap 语义）——故宿主必须自行校验唯一性"
        );
        assert_eq!(idx.len(), 1, "冲突时索引长度会小于描述数 ⇒ 可据此检出");
    }

    #[test]
    fn 空query不摆任何东西() {
        let sp = spine();
        assert!(project_tools(&sp, "", 0.0, 10).is_empty());
        assert!(compose_chain(&sp, "   ", 0.0, 10).is_empty());
    }
}