//! NT-WORLD NLP: 中文分词
//!
//! 基于最大匹配法的中文分词器

/// 中文分词器
pub struct _ChineseTokenizer {
    /// 词典 (简化版)
    dictionary: std::collections::HashSet<String>,
    /// 最大词长
    max_word_len: usize,
}

impl _ChineseTokenizer {
    /// 创建新的分词器
    pub fn new() -> Self {
        let mut dictionary = std::collections::HashSet::new();

        // 添加常见词汇
        let common_words = [
            "自然", "语言", "处理", "人工智能", "机器", "学习", "深度", "神经", "网络",
            "数据", "分析", "文本", "分类", "聚类", "提取", "摘要", "生成", "翻译",
            "理解", "识别", "检测", "预测", "模型", "算法", "特征", "训练", "优化",
            "中文", "英文", "分词", "词性", "标注", "命名", "实体", "关系", "抽取",
            "知识", "图谱", "推理", "搜索", "推荐", "系统", "平台", "工具", "框架",
            "你好", "世界", "测试", "示例", "演示", "实验", "研究", "开发", "应用",
        ];

        for word in common_words {
            dictionary.insert(word.to_string());
        }

        Self {
            dictionary,
            max_word_len: 4,
        }
    }

    /// 最大匹配法分词
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // 非中文字符直接输出
            if !Self::is_chinese(c) {
                let mut token = String::new();
                while i < chars.len() && !Self::is_chinese(chars[i]) {
                    token.push(chars[i]);
                    i += 1;
                }
                if !token.is_empty() {
                    tokens.push(token);
                }
                continue;
            }

            // 中文最大匹配
            let mut matched = false;
            let end = (i + self.max_word_len).min(chars.len());

            for len in (1..=end - i).rev() {
                let word: String = chars[i..i + len].iter().collect();
                if self.dictionary.contains(&word) || len == 1 {
                    tokens.push(word);
                    i += len;
                    matched = true;
                    break;
                }
            }

            if !matched {
                // 单字输出
                tokens.push(c.to_string());
                i += 1;
            }
        }

        tokens
    }

    /// 检查是否为中文字符
    fn is_chinese(c: char) -> bool {
        let code = c as u32;
        matches!(code, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF)
    }

    /// 添加词汇到词典
    pub fn _add_word(&mut self, word: String) {
        self.dictionary.insert(word);
    }

    /// 获取词典大小
    pub fn _dictionary_size(&self) -> usize {
        self.dictionary.len()
    }
}

impl Default for _ChineseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 词性标注
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _PosTag {
    /// 名词
    Noun,
    /// 动词
    Verb,
    /// 形容词
    Adjective,
    /// 副词
    Adverb,
    /// 介词
    Preposition,
    /// 连词
    Conjunction,
    /// 代词
    Pronoun,
    /// 数词
    Numeral,
    /// 量词
    MeasureWord,
    /// 助词
    Particle,
    /// 标点
    Punctuation,
    /// 未知
    Unknown,
}

impl _PosTag {
    pub fn _from_char(c: char) -> Self {
        match c {
            'n' | 'N' => _PosTag::Noun,
            'v' | 'V' => _PosTag::Verb,
            'a' | 'A' => _PosTag::Adjective,
            'd' | 'D' => _PosTag::Adverb,
            'p' | 'P' => _PosTag::Preposition,
            'c' | 'C' => _PosTag::Conjunction,
            'r' | 'R' => _PosTag::Pronoun,
            'm' | 'M' => _PosTag::Numeral,
            'q' | 'Q' => _PosTag::MeasureWord,
            'u' | 'U' => _PosTag::Particle,
            _ => _PosTag::Unknown,
        }
    }
}

/// 带词性的分词结果
#[derive(Debug, Clone)]
pub struct _TaggedToken {
    pub word: String,
    pub pos: _PosTag,
}

/// 词性标注器
pub struct _PosTagger;

impl _PosTagger {
    /// 简单规则词性标注
    pub fn tag(tokens: &[String]) -> Vec<_TaggedToken> {
        tokens.iter().map(|word| {
            let pos = if word.len() <= 2 && word.chars().all(|c| Self::is_chinese_char(c)) {
                // 短词可能是名词或动词
                _PosTag::Noun
            } else if word.ends_with("了") || word.ends_with("过") || word.ends_with("着") {
                _PosTag::Verb
            } else if word.ends_with("的") || word.ends_with("地") || word.ends_with("得") {
                _PosTag::Particle
            } else if word.chars().all(|c| c.is_ascii_digit()) {
                _PosTag::Numeral
            } else {
                _PosTag::Unknown
            };

            _TaggedToken { word: word.clone(), pos }
        }).collect()
    }

    fn is_chinese_char(c: char) -> bool {
        let code = c as u32;
        matches!(code, 0x4E00..=0x9FFF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_chinese() {
        let tokenizer = _ChineseTokenizer::new();
        let tokens = tokenizer.tokenize("自然语言处理是人工智能");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn tokenize_mixed() {
        let tokenizer = _ChineseTokenizer::new();
        let tokens = tokenizer.tokenize("Hello你好World世界");
        assert!(tokens.contains(&"Hello".to_string()));
        assert!(tokens.contains(&"World".to_string()));
    }

    #[test]
    fn pos_tagging() {
        let tokens = vec!["自然".to_string(), "语言".to_string(), "处理".to_string()];
        let tagged = _PosTagger::tag(&tokens);
        assert_eq!(tagged.len(), 3);
    }
}
