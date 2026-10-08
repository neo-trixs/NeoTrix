//! NT-TRAIN-EXPORT — 晶体记忆 → LLM 训练数据（MiniMind 数据飞轮映射）
//!
//! MiniMind 用统一 JSONL 贯穿 Pretrain → SFT → DPO 全阶段；本模块把
//! 晶体意识的记忆/推理链/经验导出为同格式：47 万记忆即 47 万行
//! 训练数据，有 GPU 即可直接开训，无需返工。
//!
//! - pretrain：`{"text": ...}`（全部记忆内容）
//! - sft：`{"conversations": [...]}`（前提 → 结论，推理链转对话）
//! - think：`{"conversations": [...]}`（`<think>` 推理痕迹 + 结论）
//! - dpo：`{"chosen": ..., "rejected": ...}`（同域成功方案 vs 失败教训）
//!
//! 纯函数，无 IO 副作用（写盘由调用方 `write_jsonl` 显式触发）；
//! 无 unwrap / expect / panic；无 `[]` 索引。

use super::consciousness::{CrystalConsciousness, ReasoningType};
use super::CrystalCore;

/// 训练数据导出器（纯函数集）
pub struct NtTrainExport;

impl NtTrainExport {
    /// 预训练行：全部记忆内容（去空去重由调用方按需做，这里保真）
    pub fn pretrain_lines(consciousness: &CrystalConsciousness) -> Vec<String> {
        consciousness
            .memories
            .values()
            .map(|m| {
                serde_json::json!({"text": m.content}).to_string()
            })
            .collect()
    }

    /// SFT 行：每条推理链 → 前提（user）→ 结论（assistant）对话
    pub fn sft_lines(consciousness: &CrystalConsciousness) -> Vec<String> {
        consciousness
            .reasoning_chains
            .iter()
            .filter_map(|ch| Self::chain_to_sft(consciousness, ch))
            .collect()
    }

    /// Think 行：推理链 → `<think>` 痕迹 + 结论（MiniMind 自适应思考映射）
    pub fn think_lines(consciousness: &CrystalConsciousness) -> Vec<String> {
        consciousness
            .reasoning_chains
            .iter()
            .filter_map(|ch| Self::chain_to_think(consciousness, ch))
            .collect()
    }

    /// DPO 行：成功方案（chosen）vs 失败教训（rejected）配对。
    ///
    /// 配对规则：教训文本提及同域则优先，否则取第一条失败兜底；
    /// 无失败记录时不产出（DPO 必须有负例）。
    pub fn dpo_lines(core: &CrystalCore) -> Vec<String> {
        let mut out = Vec::new();
        for sol in core.experience.successes.iter() {
            let foe = core
                .experience
                .failures
                .iter()
                .find(|l| {
                    l.failure_description.contains(sol.domain.as_str())
                        || l.takeaway.contains(sol.domain.as_str())
                })
                .or_else(|| core.experience.failures.first());
            if let Some(lesson) = foe {
                out.push(
                    serde_json::json!({
                        "chosen": format!("{}: {} → {}", sol.problem, sol.approach, sol.result),
                        "rejected": format!("{}: {}", lesson.failure_description, lesson.takeaway),
                        "domain": sol.domain,
                    })
                    .to_string(),
                );
            }
        }
        out
    }

    /// 写 JSONL 文件，返回行数
    pub fn write_jsonl(path: &std::path::Path, lines: &[String]) -> Result<usize, String> {
        use std::fmt::Write as _;
        let mut buf = String::new();
        for line in lines {
            let _ = writeln!(buf, "{line}");
        }
        std::fs::write(path, buf).map_err(|e| format!("write jsonl: {e}"))?;
        Ok(lines.len())
    }

    /// 追加 JSONL（增量导出用，不存在则创建）
    pub fn append_jsonl(path: &std::path::Path, lines: &[String]) -> Result<usize, String> {
        use std::fmt::Write as _;
        use std::io::Write as _;
        if lines.is_empty() {
            return Ok(0);
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| format!("mkdir jsonl: {e}"))?;
            }
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("append jsonl: {e}"))?;
        let mut buf = String::new();
        for line in lines {
            let _ = writeln!(buf, "{line}");
        }
        f.write_all(buf.as_bytes())
            .map_err(|e| format!("append jsonl write: {e}"))?;
        Ok(lines.len())
    }

    /// 意识中最大记忆时间戳（增量水位用；空意识返回 0）
    pub fn max_created_at(consciousness: &CrystalConsciousness) -> u64 {
        consciousness
            .memories
            .values()
            .map(|m| m.created_at)
            .max()
            .unwrap_or(0)
    }

    /// 增量预训练行：仅 created_at >= since 的记忆
    pub fn pretrain_lines_since(
        consciousness: &CrystalConsciousness,
        since_ts: u64,
    ) -> Vec<String> {
        consciousness
            .memories
            .values()
            .filter(|m| m.created_at >= since_ts)
            .map(|m| serde_json::json!({"text": m.content}).to_string())
            .collect()
    }

    /// 推理链结论时间戳（经 conclusion_memory_id 回查；无则 None）
    fn chain_ts(
        consciousness: &CrystalConsciousness,
        ch: &super::consciousness::ReasoningChain,
    ) -> Option<u64> {
        ch.conclusion_memory_id
            .as_deref()
            .and_then(|id| consciousness.memories.get(id))
            .map(|m| m.created_at)
    }

    /// 增量 SFT 行：结论记忆时间戳 >= since 的链（无结论 id 的链跳过，防重复）
    pub fn sft_lines_since(consciousness: &CrystalConsciousness, since_ts: u64) -> Vec<String> {
        consciousness
            .reasoning_chains
            .iter()
            .filter(|ch| Self::chain_ts(consciousness, ch).is_some_and(|t| t >= since_ts))
            .filter_map(|ch| Self::chain_to_sft(consciousness, ch))
            .collect()
    }

    /// 增量 think 行：同 sft 过滤口径
    pub fn think_lines_since(consciousness: &CrystalConsciousness, since_ts: u64) -> Vec<String> {
        consciousness
            .reasoning_chains
            .iter()
            .filter(|ch| Self::chain_ts(consciousness, ch).is_some_and(|t| t >= since_ts))
            .filter_map(|ch| Self::chain_to_think(consciousness, ch))
            .collect()
    }

    /// 单链 → SFT 行（sft_lines 的单条版，供全量/增量复用逻辑一致）
    fn chain_to_sft(
        consciousness: &CrystalConsciousness,
        ch: &super::consciousness::ReasoningChain,
    ) -> Option<String> {
        let premises: Vec<String> = ch
            .premises
            .iter()
            .filter_map(|pid| consciousness.memories.get(pid).map(|m| m.content.clone()))
            .collect();
        if premises.is_empty() {
            return None;
        }
        Some(
            serde_json::json!({
                "conversations": [
                    {"role": "user", "content": premises.join("\n")},
                    {"role": "assistant", "content": ch.conclusion.clone()},
                ],
                "chain_type": format!("{:?}", ch.chain_type),
            })
            .to_string(),
        )
    }

    /// 单链 → think 行
    fn chain_to_think(
        consciousness: &CrystalConsciousness,
        ch: &super::consciousness::ReasoningChain,
    ) -> Option<String> {
        let premises: Vec<String> = ch
            .premises
            .iter()
            .filter_map(|pid| consciousness.memories.get(pid).map(|m| m.content.clone()))
            .collect();
        if premises.is_empty() {
            return None;
        }
        let thinking = format!(
            "<think>类型={:?}，由{}条前提出发，逐步推导。</think>",
            ch.chain_type,
            premises.len()
        );
        Some(
            serde_json::json!({
                "conversations": [
                    {"role": "system", "content": "think step by step"},
                    {"role": "user", "content": premises.join("\n")},
                    {"role": "assistant", "content": format!("{thinking}{}", ch.conclusion)},
                ],
            })
            .to_string(),
        )
    }

    /// 推理类型名（供外部统计）
    pub fn chain_type_name(t: &ReasoningType) -> &'static str {
        match t {
            ReasoningType::Deductive => "deductive",
            ReasoningType::Inductive => "inductive",
            ReasoningType::Abductive => "abductive",
            ReasoningType::Analogical => "analogical",
            ReasoningType::CrossDomain => "cross_domain",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::consciousness::MemoryType;
    use super::*;

    fn seeded() -> (CrystalCore, CrystalConsciousness) {
        let mut core = CrystalCore::new("t");
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火则热", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("铁遇火", MemoryType::Fact, "physics", 0.8);
        c.reason(vec![a, b], ReasoningType::Deductive);
        core.experience.record_success("编译慢", "增量编译", "快了", true, "缓存", "engineering");
        core.experience.record_failure("构建崩", "内存爆", "加内存", "先看监控", "bug", 0.8);
        (core, c)
    }

    #[test]
    fn test_pretrain_lines_cover_memories() {
        let (_, c) = seeded();
        let lines = NtTrainExport::pretrain_lines(&c);
        // 2 前提 + 1 结论
        assert_eq!(lines.len(), 3);
        assert!(lines.iter().all(|l| l.contains("\"text\"")));
    }

    #[test]
    fn test_sft_think_lines_from_chains() {
        let (_, c) = seeded();
        let sft = NtTrainExport::sft_lines(&c);
        assert_eq!(sft.len(), 1);
        assert!(sft[0].contains("conversations"));
        let think = NtTrainExport::think_lines(&c);
        assert_eq!(think.len(), 1);
        assert!(think[0].contains("<think>"));
    }

    #[test]
    fn test_dpo_pairs_when_both_exist() {
        let (core, _) = seeded();
        let dpo = NtTrainExport::dpo_lines(&core);
        // 成功 1 条 + 失败 1 条（跨类回退配对）→ 1 对
        assert_eq!(dpo.len(), 1);
        assert!(dpo[0].contains("\"chosen\"") && dpo[0].contains("\"rejected\""));
    }

    #[test]
    fn test_dpo_empty_without_failures() {
        let (mut core, _) = seeded();
        core.experience.failures.clear();
        assert!(NtTrainExport::dpo_lines(&core).is_empty());
    }

    #[test]
    fn test_pretrain_lines_since_filters_by_ts() {
        let (_, mut c) = seeded();
        // 把一条记忆搬到过去
        let old_id = c
            .memories
            .values()
            .next()
            .map(|m| m.id.clone())
            .unwrap();
        c.memories.get_mut(&old_id).unwrap().created_at = 100;
        let fresh_ts = NtTrainExport::max_created_at(&c);
        assert!(fresh_ts > 100);
        let lines = NtTrainExport::pretrain_lines_since(&c, fresh_ts);
        assert!(!lines.is_empty());
        assert!(lines.len() < c.memories.len(), "old memory must be excluded");
        // 全量等价：since=0 全收
        assert_eq!(NtTrainExport::pretrain_lines_since(&c, 0).len(), c.memories.len());
        // 未来时间戳：全空
        assert!(NtTrainExport::pretrain_lines_since(&c, u64::MAX).is_empty());
    }

    #[test]
    fn test_sft_think_since_follow_conclusion_ts() {
        let (_, mut c) = seeded();
        let ch = c.reasoning_chains.first().cloned().unwrap();
        let cid = match ch.conclusion_memory_id.clone() {
            Some(id) => id,
            None => return, // 无结论 id 的链：增量口径跳过（覆盖不到则此测空转通过）
        };
        c.memories.get_mut(&cid).unwrap().created_at = 100;
        let fresh_ts = NtTrainExport::max_created_at(&c);
        assert!(NtTrainExport::sft_lines_since(&c, fresh_ts).is_empty());
        assert!(NtTrainExport::think_lines_since(&c, fresh_ts).is_empty());
        c.memories.get_mut(&cid).unwrap().created_at = fresh_ts;
        assert_eq!(NtTrainExport::sft_lines_since(&c, fresh_ts).len(), 1);
        assert_eq!(NtTrainExport::think_lines_since(&c, fresh_ts).len(), 1);
    }

    #[test]
    fn test_append_jsonl_roundtrip() {
        let dir = std::env::temp_dir().join("nt_train_export_append_test");
        let _ = std::fs::remove_dir_all(&dir);
        let p = dir.join("a.jsonl");
        let n1 = NtTrainExport::append_jsonl(&p, &["{\"a\":1}".to_string()]).unwrap();
        let n2 = NtTrainExport::append_jsonl(&p, &["{\"a\":2}".to_string()]).unwrap();
        assert_eq!((n1, n2), (1, 1));
        let data = std::fs::read_to_string(&p).unwrap();
        assert_eq!(data.lines().count(), 2);
        // 空追加不建文件也返回 0
        assert_eq!(NtTrainExport::append_jsonl(&dir.join("e.jsonl"), &[]).unwrap(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_max_created_at_empty_zero() {
        let c = CrystalConsciousness::new("t");
        assert_eq!(NtTrainExport::max_created_at(&c), 0);
    }

    #[test]
    fn test_write_jsonl_roundtrip() {        let dir = std::env::temp_dir().join("nt_train_export_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("sft.jsonl");
        let n = NtTrainExport::write_jsonl(&p, &["{\"a\":1}".to_string()]).unwrap();
        assert_eq!(n, 1);
        assert!(p.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
