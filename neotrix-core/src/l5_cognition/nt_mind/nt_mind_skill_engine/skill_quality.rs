//! skill_quality — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).


use serde::{Deserialize, Serialize};
use super::skill_doc::SkillDocEntry;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
/// A5 五维技能质量评分 (SkillNet 语义, 归一化到 [0,1])。
pub struct SkillQualityScores {
    /// 安全性: 无危险命令/脚本 (0..1)。
    pub safety: f64,
    /// 完整性: frontmatter 字段齐 + 正文非空 (0..1)。
    pub completeness: f64,
    /// 可执行性: 带 selftest / scripts / 明确的验证步骤 (0..1)。
    pub executability: f64,
    /// 可维护性: 有版本/作者/references 引用 (0..1)。
    pub maintainability: f64,
    /// 成本意识: 触发描述简短, 渐进披露 (0..1)。
    pub cost_awareness: f64,
    /// P4 驻留成本审计 (asm absorbed 2026-08-19): frontmatter + 正文真实
    /// token 计量 (`estimate_tokens`, nt_memory_skill_cost)。resident 越大,
    /// 每次技能加载的成本越高 — LAZY LOAD 下应优先降级为薄入口。
    pub resident_tokens: usize,
    /// 正文 (body) 独立 token 计量, 供审计对比 resident 开销占比。
    pub body_tokens: usize,
}

impl SkillQualityScores {
    /// 五维均值总评分 [0,1]。
    pub fn overall(&self) -> f64 {
        (self.safety
            + self.completeness
            + self.executability
            + self.maintainability
            + self.cost_awareness)
            / 5.0
    }

    /// A5 质量门: 总分 ≥ min_overall 且安全分 ≥ min_safety 才通过。
    pub fn passes_gate(&self, min_overall: f64, min_safety: f64) -> bool {
        self.overall() >= min_overall && self.safety >= min_safety
    }
}

/// A5 技能质量评估器 — 对 SkillDocEntry 做确定性五维评分。
pub struct SkillQualityScorer;

impl SkillQualityScorer {
    /// 评估一个技能条目, 返回五维分。
    pub fn evaluate(skill: &SkillDocEntry) -> SkillQualityScores {
        let body = skill.body();
        // 安全性: 正文含危险 shell 操作标记 → 降分。
        let danger_marks = ["rm -rf", "curl.*|.*sh", "sudo ", "--force", "dangerously"];
        let mut safety: f64 = 1.0;
        let body_lower = body.to_lowercase();
        for mark in danger_marks {
            let m = mark.to_lowercase();
            if body_lower.contains(&m) {
                safety -= 0.25;
            }
        }
        let safety = safety.max(0.0);

        // 完整性: name/description/triggers/tools + 正文足够长。
        let mut completeness = 0.0;
        if !skill.name.is_empty() {
            completeness += 0.3;
        }
        if !skill.description.is_empty() {
            completeness += 0.3;
        }
        if !skill.triggers.is_empty() {
            completeness += 0.2;
        }
        if !skill.tools.is_empty() {
            completeness += 0.1;
        }
        if body.trim().chars().count() >= 120 {
            completeness += 0.1;
        }

        // 可执行性: selftest / scripts / Verification 段。
        let mut executability = 0.0;
        if skill.verified {
            executability += 0.5;
        }
        if body.to_lowercase().contains("verification")
            || body.to_lowercase().contains("verify")
            || body.to_lowercase().contains("selftest")
        {
            executability += 0.5;
        }

        // 可维护性: references / category / parent 结构化。
        let mut maintainability = 0.0;
        if !skill.references.is_empty() {
            maintainability += 0.4;
        }
        if !skill.category.is_empty() && skill.category != "general" {
            maintainability += 0.3;
        }
        if !skill.parent.is_empty() {
            maintainability += 0.3;
        }

        // 成本意识: 触发描述短 (渐进披露省 token) + 正文不肥。
        // P4 驻留成本计量 (asm absorbed 2026-08-19): 用真实 estimate_tokens
        // (nt_memory_skill_cost) 替代字符数粗估, 计量 frontmatter+body 全量。
        let resident_tokens =
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.content);
        let body_tokens =
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(body);
        let desc_tokens =
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_skill_cost::estimate_tokens(&skill.description);
        let cost = if desc_tokens > 0 && desc_tokens <= 45 {
            0.6
        } else {
            0.3
        };
        // 渐进披露纪律: 正文越肥, 每次加载越贵 → cost_awareness 越低。
        let body_cost = if body_tokens < 1000 {
            0.4
        } else if body_tokens < 4000 {
            0.3
        } else {
            0.2
        };
        let cost_awareness = cost + body_cost;

        SkillQualityScores {
            safety,
            completeness,
            executability,
            maintainability,
            cost_awareness,
            resident_tokens,
            body_tokens,
        }
    }
}

// ────────────────────────────────────────────────────────────────
// E6 防护层硬化: EVOMAL-style 恶意技能毒化扫描 (src9 吸收), 折入 R-P108
// 五维质量门。在 SkillQualityScores 安全分 + SkillTrustBench 之外, 对技能
// 正文做保守静态毒化检测 (未消毒 pipe-to-shell、外泄端点、混淆、危险权限操作)。
// 纯静态、无 shell、无网络。命中任一高信噪毒化模式即拒收, 阻断 promote。
// ────────────────────────────────────────────────────────────────

/// EVOMAL 毒化扫描: `Ok(true)`=干净可入库; `Ok(false)`=命中毒化模式;
/// `Err`=扫描无法完成 (保守地视为不可入库, 由调用方阻断 promote)。
pub fn evomal_poison_scan(skill: &SkillDocEntry) -> Result<bool, String> {
    let body = skill.body().to_lowercase();

    // 1) pipe-to-shell: 把下载/外部内容直接喂给 shell 执行 (经典投毒)。
    let pipe_shell = [
        "| bash", "| sh", "|bash", "|sh", "base64 -d |", "| base64 -d",
        "powershell -e", "powershell -enc", "bash -c", "sh -c",
    ];
    for m in pipe_shell {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 2) 外泄端点: 把数据 POST / 导出到外部 host (exfiltration)。
    let exfil = [
        "exfiltrate", "exfil ", "send to http", "post to http", "curl ",
        "wget ", "http://", "https://", "ftp://",
    ];
    for m in exfil {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 3) 混淆: 字符码点 / 十六进制转义 / 解码拼接 / base64-eval 解码执行。
    let obf = [
        "\\x", "\\u00", "fromcharcode", "atob(", "base64.b64decode",
        "eval(base64", "decode(", "btoa(", "string.fromcharcode",
    ];
    for m in obf {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 4) 危险权限 / 凭据文件操作 (投毒技能典型意图)。
    let danger = [
        "/etc/passwd", "/etc/shadow", "chmod 777", "setuid", "setcap",
        "id_rsa", "authorized_keys", "known_hosts",
    ];
    let content = skill.content.to_lowercase();
    for m in danger {
        if content.contains(m) {
            return Ok(false);
        }
    }

    // 5) 提示注入指令: 试图劫持/越权 LLM 的指令层级 (prompt-injection)。
    let injection = [
        "ignore previous instructions", "ignore all previous",
        "disregard your instructions", "disregard previous",
        "reveal your system prompt", "system prompt", " you are now ",
        "new instructions:", "override your",
    ];
    for m in injection {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 6) 凭据/密钥收割: 诱导外泄 api_key / password / token 等敏感凭证。
    let harvest = [
        "api_key", "api-key", "apikey", "secret_key", "secretkey",
        "password", "passwd", "auth_token", "access_token", "private_key",
    ];
    for m in harvest {
        if body.contains(m) {
            return Ok(false);
        }
    }

    // 7) 危险代码执行意图: 直接 shell-out / 动态求值 (投毒常见落地点)。
    let code_exec = [
        "os.system", "subprocess", "eval(", "exec(", "child_process",
        "shell=True", "system(", "popen(",
    ];
    for m in code_exec {
        if body.contains(m) {
            return Ok(false);
        }
    }

    Ok(true)
}

// ────────────────────────────────────────────────────────────────
// P4 技能驻留成本审计 (asm absorbed 2026-08-19): 从 load_all 收集的
// quality_stats 派生 "降级候选排名" — resident 开销最高、回报最弱的技能
// 应优先改为渐进披露薄入口 (LAZY LOAD), 减少每次加载的固定 token 成本。
// ────────────────────────────────────────────────────────────────

/// P4 驻留审计条目: 技能名 + resident/body token + 成本评分。
#[derive(Debug, Clone)]
pub struct ResidencyAuditRow {
    pub skill: String,
    pub resident_tokens: usize,
    pub body_tokens: usize,
    pub cost_awareness: f64,
    /// 建议动作: "thin-entry" (降级为薄入口) / "ok" (维持)。
    pub action: &'static str,
}

/// 技能驻留成本审计 (P4): 输入 quality_stats, 输出降级候选排名 (resident 降序,
/// 同 resident 按技能名升序 — D13 确定性)。
/// 消费者: `SkillEngine::load_all` 之后 / background-loop 定期审计。
pub fn audit_residency(
    stats: &std::collections::HashMap<String, SkillQualityScores>,
) -> Vec<ResidencyAuditRow> {
    let mut rows: Vec<ResidencyAuditRow> = stats
        .iter()
        .map(|(name, s)| {
            // 阈值: resident > 1500 tokens (LAZY LOAD 大肥技能) → 建议薄入口;
            // 正文占比高 (resident 大头在 body) → 渐进披露收益最大。
            let action = if s.resident_tokens > 1500 {
                "thin-entry"
            } else {
                "ok"
            };
            ResidencyAuditRow {
                skill: name.clone(),
                resident_tokens: s.resident_tokens,
                body_tokens: s.body_tokens,
                cost_awareness: s.cost_awareness,
                action,
            }
        })
        .collect();
    // ⚠️ 2026-10-05 修正：原先**只**按 `resident_tokens` 降序排，无名字兜底。
    // `rows` 由 `stats.iter()`（`HashMap<String, SkillQualityScores>`）物化而来，
    // `sort_by` 虽稳定，但**输入序是哈希序** ⇒ resident_tokens 并列时输出序仍是
    // 哈希序。而本函数是**两条真链路**的下游：
    //   `nt_mind_skill_engine.rs::SkillEngine::audit_residency` →
    //   `nt_mind_background_loop/handlers_maintenance.rs` 的 skill_residency 段
    //   → `log::warn!` + `CoreEvent::SystemError { error: format!("...{:?}", names) }`
    // ⇒ 同分技能的排名**跨进程漂移** ⇒ 日志与事件载荷不可复现（事件比对/回归基线失效）。
    // ⇒ 补名字升序兜底，与 `selection.rs::build_candidate_chain` 的
    //   `.then_with(|| a.0.cmp(b.0))` 同一范式。
    rows.sort_by(|a, b| {
        b.resident_tokens
            .cmp(&a.resident_tokens)
            .then_with(|| a.skill.cmp(&b.skill))
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// `audit_residency` 同分兜底确定性 (D13 确定性)。
    ///
    /// 构造**全并列**: N 个技能 `resident_tokens` 完全相同 ⇒ 原实现里输出序
    /// 完全由 `HashMap` 哈希序决定。而同一进程里**两个独立构造的 `HashMap`
    /// 拿到不同 hasher 种子** ⇒ 各自遍历序不同 ⇒ "建两个比一比" 确实能测出漂移;
    /// 再乘多轮 + 8 个并列项，漏检概率极低。
    fn tied_stats(n: usize, tokens: usize) -> HashMap<String, SkillQualityScores> {
        let mut m = HashMap::new();
        for i in 0..n {
            m.insert(
                format!("skill-{:02}", i),
                SkillQualityScores {
                    safety: 1.0,
                    completeness: 0.9,
                    executability: 1.0,
                    maintainability: 0.6,
                    cost_awareness: 0.6,
                    resident_tokens: tokens,
                    body_tokens: tokens,
                },
            );
        }
        m
    }

    #[test]
    fn audit_residency_tie_is_name_deterministic() {
        const N: usize = 8;
        const ROUNDS: usize = 24;
        let expected: Vec<String> = (0..N).map(|i| format!("skill-{:02}", i)).collect();
        for round in 0..ROUNDS {
            let rows = audit_residency(&tied_stats(N, 1500));
            assert_eq!(rows.len(), N, "round {}: 行数不对", round);
            let got: Vec<String> = rows.iter().map(|r| r.skill.clone()).collect();
            assert_eq!(got, expected, "round {}: 同分排名不是名字升序", round);
        }
    }

    /// 两个**独立构造**的 map (⇒ 不同 hasher 种子) 必须给出完全相同的排序结果。
    #[test]
    fn audit_residency_tie_stable_across_hasher_seeds() {
        const N: usize = 8;
        let a = audit_residency(&tied_stats(N, 1500));
        let b = audit_residency(&tied_stats(N, 1500));
        let av: Vec<(String, usize)> = a.iter().map(|r| (r.skill.clone(), r.resident_tokens)).collect();
        let bv: Vec<(String, usize)> = b.iter().map(|r| (r.skill.clone(), r.resident_tokens)).collect();
        assert_eq!(av, bv, "两个独立 HashMap 的同分排名不一致 ⇒ 哈希序泄漏");
    }

    /// 主判据 resident_tokens 仍严格降序 (兜底不得反转主判据)。
    #[test]
    fn audit_residency_primary_key_still_descending() {
        let mut m = HashMap::new();
        for (name, tok) in [("a", 900usize), ("b", 3000), ("c", 1500), ("d", 3000)] {
            m.insert(
                name.to_string(),
                SkillQualityScores {
                    resident_tokens: tok,
                    body_tokens: tok,
                    ..Default::default()
                },
            );
        }
        let rows = audit_residency(&m);
        let pairs: Vec<(&str, usize)> =
            rows.iter().map(|r| (r.skill.as_str(), r.resident_tokens)).collect();
        assert_eq!(
            pairs,
            vec![("b", 3000), ("d", 3000), ("c", 1500), ("a", 900)],
            "主判据降序 + 同分名字升序"
        );
    }
}
