//! nt_route_features — D1 深特征路由（M1+M5，权重 0.3）。
//!
//! 目标进来先抽深层特征（意图动词/约束/资源域），再选工具/方法，
//! 而不是按字面关键词路由。纯规则零模型；无命中回退字面逻辑（fail-open）。
//!
//! 设计：`docs/plans/2026-09-24-d1-deep-feature-routing.md`。

use std::collections::HashSet;

/// 深特征抽取结果。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeepFeatures {
    /// 意图动词（search/compare/synthesize/debug/execute/navigate/extract/measure）
    pub verbs: Vec<String>,
    /// 约束（time_limit/cost_budget/lang/modality/offline/approval）
    pub constraints: Vec<String>,
    /// 资源域（repo/paper/web/kb/colab/browser/shell/eval/logs/sidecar）
    pub domains: Vec<String>,
}

impl DeepFeatures {
    /// 是否抽到任何特征（空则调用方回退字面路由）。
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty() && self.constraints.is_empty() && self.domains.is_empty()
    }
}

/// （特征词 → 规范名）查表，中英同义。顺序无关，命中即收。
const VERB_TABLE: &[(&[&str], &str)] = &[
    // D1 对照组补（2026-09-25）：英文 B 侧 "how to …" / "check whether …" 无动词命中
    // → 同义对被判空特征。how/check 归 search（找用法/查状态），OLD-5 单测已验无回归。
    (
        &["找", "查", "搜", "search", "find", "look", "how", "check"],
        "search",
    ),
    (&["比", "对比", "compare", "versus", "vs"], "compare"),
    (
        &["写", "总结", "生成", "write", "summarize", "generate"],
        "synthesize",
    ),
    (&["修", "报错", "fix", "debug", "repair"], "debug"),
    (&["跑", "执行", "run", "execute"], "execute"),
    (&["打开", "前往", "open", "navigate", "goto"], "navigate"),
    (&["提取", "解析", "extract", "parse"], "extract"),
    (&["测", "评估", "measure", "eval", "benchmark"], "measure"),
];

const CONSTRAINT_TABLE: &[(&[&str], &str)] = &[
    (&["限时", "快", "urgent", "quick"], "time_limit"),
    (&["便宜", "免费", "free", "cheap"], "cost_budget"),
    (&["中文", "chinese"], "lang_zh"),
    (&["英文", "english"], "lang_en"),
    (
        &["图", "音频", "代码", "image", "audio", "code"],
        "modality",
    ),
    (&["离线", "本地", "offline", "local"], "offline"),
    (&["审批", "确认", "approve", "confirm"], "approval"),
];

const DOMAIN_TABLE: &[(&[&str], &str)] = &[
    (&["仓库", "代码", "repo"], "repo"),
    (&["论文", "paper", "arxiv"], "paper"),
    (&["网页", "搜索", "web"], "web"),
    (&["知识库", "kb"], "kb"),
    (&["colab"], "colab"),
    (&["浏览器", "browser"], "browser"),
    (&["命令", "shell", "terminal"], "shell"),
    (&["验收集", "eval", "gaia", "测试集"], "eval"),
    (&["日志", "logs", "log"], "logs"),
    (&["sidecar", "8149"], "sidecar"),
];

fn hit_table(text: &str, table: &[(&[&str], &str)], out: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for (keys, name) in table {
        if keys.iter().any(|k| text.contains(k)) && seen.insert(*name) {
            out.push((*name).to_owned());
        }
    }
}

/// 纯函数抽取（小写归一，中英混排可）。
pub fn extract_deep_features(query: &str) -> DeepFeatures {
    let text = query.to_lowercase();
    let mut feats = DeepFeatures::default();
    hit_table(&text, VERB_TABLE, &mut feats.verbs);
    hit_table(&text, CONSTRAINT_TABLE, &mut feats.constraints);
    hit_table(&text, DOMAIN_TABLE, &mut feats.domains);
    feats
}

/// 特征匹配分（供路由排序）：动词 +30，域 +20（与字面分同量级，不一刀切）。
pub fn score_by_features(candidate_tags: &[String], feats: &DeepFeatures) -> f64 {
    let lower: Vec<String> = candidate_tags.iter().map(|t| t.to_lowercase()).collect();
    let verb_hit = feats
        .verbs
        .iter()
        .filter(|v| lower.iter().any(|t| t.contains(v.as_str())))
        .count();
    let domain_hit = feats
        .domains
        .iter()
        .filter(|d| lower.iter().any(|t| t.contains(d.as_str())))
        .count();
    verb_hit as f64 * 30.0 + domain_hit as f64 * 20.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cn_debug_repo() {
        let f = extract_deep_features("修 flaky 单测");
        assert!(f.verbs.contains(&"debug".to_owned()));
        assert!(!f.is_empty());
    }

    #[test]
    fn test_extract_en_same_features() {
        // 同义不同字面 → 同核心动词（D1 对照组第 1 组；中文"单测"含"测"会多带 measure，
        // 路由以动词交集为准，debug 主导 beiden）
        let a = extract_deep_features("修 flaky 单测");
        let b = extract_deep_features("fix flaky test");
        assert!(a.verbs.contains(&"debug".to_owned()));
        assert_eq!(b.verbs, vec!["debug".to_owned()]);
    }

    #[test]
    fn test_empty_falls_back() {
        let f = extract_deep_features("zxqw kjrblp vapour");
        assert!(f.is_empty());
    }

    #[test]
    fn test_score_weights() {
        let f = extract_deep_features("跑 D5 验收集");
        // 动词分只计候选 tags 命中的（execute 需候选含该词），域分同理
        let s = score_by_features(&["execute".to_owned(), "eval".to_owned()], &f);
        assert!(s >= 50.0, "verb execute + domain eval = 30+20, got {s}");
        let s2 = score_by_features(&["eval".to_owned(), "test".to_owned()], &f);
        assert_eq!(s2, 20.0, "仅域命中，got {s2}");
    }

    #[test]
    fn test_constraint_lang() {
        let f = extract_deep_features("翻译成中文");
        assert!(f.constraints.contains(&"lang_zh".to_owned()));
    }

    /// D1 验收：设计文档附表 10 组同义 query，中英两侧核心动词必须有交集
    ///（路由以动词交集为准；任一侧空特征即 fail-open 判负）。
    #[test]
    fn test_synonym_pairs_share_verb() {
        let pairs: &[(&str, &str)] = &[
            ("修 flaky 单测", "fix flaky test"),
            (
                "找 chromiumoxide 的 user_data_dir 用法",
                "how to set user data dir in chromiumoxide",
            ),
            (
                "对比 LoRA 与全量微调显存",
                "compare LoRA vs full finetune memory",
            ),
            ("跑 D5 验收集", "run gaia mini eval set"),
            ("总结 ftv4 训练曲线", "summarize ftv4 training loss curve"),
            ("打开 colab 探针报告", "open the colab probe log"),
            ("评估 sidecar 延迟", "measure sidecar latency"),
            (
                "提取 smelt 的 800 条新卡主题",
                "extract topics of the 800 new smelt cards",
            ),
            (
                "写 D1 深特征路由设计",
                "write the d1 deep-feature routing design",
            ),
            (
                "查训练 42994 步之后有没有掉线",
                "check whether training dropped after step 42994",
            ),
        ];
        for (i, (a, b)) in pairs.iter().enumerate() {
            let fa = extract_deep_features(a);
            let fb = extract_deep_features(b);
            assert!(!fa.verbs.is_empty(), "组{i} A 侧空特征: {a}");
            assert!(!fb.verbs.is_empty(), "组{i} B 侧空特征: {b}");
            let shared = fa.verbs.iter().any(|v| fb.verbs.contains(v));
            assert!(shared, "组{i} 动词无交集: {a:?}->{fa:?} vs {b:?}->{fb:?}");
        }
    }

    /// D1 验收（域级）：第 6/7 组两侧应同域（colab / sidecar）。
    #[test]
    fn test_synonym_pairs_share_domain() {
        let fa = extract_deep_features("打开 colab 探针报告");
        let fb = extract_deep_features("open the colab probe log");
        assert!(fa.domains.contains(&"colab".to_owned()));
        assert!(fb.domains.contains(&"colab".to_owned()));
        let fa = extract_deep_features("评估 sidecar 延迟");
        let fb = extract_deep_features("measure sidecar latency");
        assert!(fa.domains.contains(&"sidecar".to_owned()));
        assert!(fb.domains.contains(&"sidecar".to_owned()));
    }
}
