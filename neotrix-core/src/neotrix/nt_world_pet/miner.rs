use crate::neotrix::nt_world_pet::traits::TraitSignal;

pub struct ConversationMiner;

impl ConversationMiner {
    pub fn mine(text: &str) -> TraitSignal {
        let mut signal = TraitSignal::new();
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();

        for (i, w) in words.iter().enumerate() {
            Self::match_word(w, &mut signal);

            if i + 1 < words.len() {
                let pair = format!("{} {}", w, words[i + 1]);
                let pair_lower = pair.to_lowercase();
                Self::match_bigram(&pair_lower, &mut signal);
            }

            if i + 2 < words.len() {
                let triple = format!("{} {} {}", w, words[i + 1], words[i + 2]);
                let triple_lower = triple.to_lowercase();
                Self::match_trigram(&triple_lower, &mut signal);
            }
        }

        Self::match_full_text(&lower, &mut signal);

        signal
    }

    fn match_full_text(text: &str, signal: &mut TraitSignal) {
        if Self::contains_any(text, &["你就像一只猫", "你就像一只小猫", "你像一只猫", "你就像猫", "like a cat", "like a kitten"]) {
            *signal = signal.clone().with_visual("creature", -0.35).with_visual("softness", 0.2);
        }
        if Self::contains_any(text, &["你就像一只狗", "你就像一只小狗", "你像一只狗", "like a dog", "like a puppy"]) {
            *signal = signal.clone().with_visual("creature", 0.3);
        }
        if Self::contains_any(text, &["你像一只鸟", "like a bird"]) {
            *signal = signal.clone().with_visual("creature", 0.35).with_visual("size", -0.15);
        }
        if Self::contains_any(text, &["你像一条龙", "你像龙", "like a dragon"]) {
            *signal = signal.clone().with_visual("creature", 0.5).with_visual("size", 0.15);
        }
        if Self::contains_any(text, &["你像一团光", "你像光", "like light", "like a light"]) {
            *signal = signal.clone().with_visual("brightness", 0.3).with_visual("creature", 0.5);
        }
        if Self::contains_any(text, &["你像一团", "a ball of"]) {
            *signal = signal.clone().with_visual("softness", 0.25);
        }
    }

    fn match_word(w: &str, signal: &mut TraitSignal) {
        if Self::contains_any(w, &["猫", "cat", "小猫", "kitten", "老虎", "tiger", "豹", "leopard"]) {
            *signal = signal.clone().with_visual("creature", -0.15).with_visual("softness", 0.1);
        } else if Self::contains_any(w, &["狗", "dog", "小狗", "puppy", "狼", "wolf"]) {
            *signal = signal.clone().with_visual("creature", 0.15).with_visual("softness", 0.05);
        } else if Self::contains_any(w, &["鸟", "bird", "鹰", "eagle", "owl", "猫头鹰"]) {
            *signal = signal.clone().with_visual("creature", 0.3).with_visual("size", -0.1);
        } else if Self::contains_any(w, &["龙", "dragon", "精灵", "spirit"]) {
            *signal = signal.clone().with_visual("creature", 0.45).with_visual("brightness", 0.2);
        } else if Self::contains_any(w, &["小", "small", "tiny", "迷你", "mini"]) {
            *signal = signal.clone().with_visual("size", -0.2);
        } else if Self::contains_any(w, &["大", "big", "large", "巨大", "huge", "giant"]) {
            *signal = signal.clone().with_visual("size", 0.2);
        } else if Self::contains_any(w, &["温暖", "暖", "warm", "热情", "sunny", "阳光"]) {
            *signal = signal.clone().with_visual("warmth", 0.2);
        } else if Self::contains_any(w, &["冷", "cool", "冷淡", "高冷", "icy", "冰冷"]) {
            *signal = signal.clone().with_visual("warmth", -0.2);
        } else if Self::contains_any(w, &["软", "圆润", "round", "可爱", "cute", "fluffy"]) {
            *signal = signal.clone().with_visual("softness", 0.2);
        } else if Self::contains_any(w, &["硬", "尖锐", "sharp", "angular", "棱角"]) {
            *signal = signal.clone().with_visual("softness", -0.2);
        } else if Self::contains_any(w, &["活跃", "蹦跳", "bouncy", "跑来跑去", "精力", "energetic"]) {
            *signal = signal.clone().with_visual("energy", 0.2).with_behavior("playfulness", 0.15);
        } else if Self::contains_any(w, &["安静", "quiet", "懒", "lazy", "躺着", "sleep", "calm", "平静"]) {
            *signal = signal.clone().with_visual("energy", -0.2).with_behavior("playfulness", -0.1);
        } else if Self::contains_any(w, &["闪亮", "shiny", "发光", "glow", "bright", "亮", "璀璨"]) {
            *signal = signal.clone().with_visual("brightness", 0.25);
        } else if Self::contains_any(w, &["暗", "dark", "dim", "暗淡", "shadow", "阴影"]) {
            *signal = signal.clone().with_visual("brightness", -0.2);
        } else if Self::contains_any(w, &["好奇", "curious", "探索", "explore", "到处"]) {
            *signal = signal.clone().with_behavior("curiosity", 0.2);
        } else if Self::contains_any(w, &["严肃", "serious", "沉思", "pensive"]) {
            *signal = signal.clone().with_behavior("playfulness", -0.15);
        } else if Self::contains_any(w, &["话多", "talkative", "健谈", "chatty", "唠叨"]) {
            *signal = signal.clone().with_behavior("talkativeness", 0.2);
        } else if Self::contains_any(w, &["沉默", "silent", "寡言", "少话"]) {
            *signal = signal.clone().with_behavior("talkativeness", -0.2);
        } else if Self::contains_any(w, &["敏感", "sensitive", "反应快", "quick", "alert", "警觉"]) {
            *signal = signal.clone().with_behavior("reactivity", 0.2);
        } else if Self::contains_any(w, &["迟钝", "呆", "dull"]) {
            *signal = signal.clone().with_behavior("reactivity", -0.2);
        } else if Self::contains_any(w, &["精致", "delicate", "华丽", "ornate", "复杂", "complex"]) {
            *signal = signal.clone().with_visual("complexity", 0.2);
        } else if Self::contains_any(w, &["简约", "simple", "极简", "minimal", "朴素"]) {
            *signal = signal.clone().with_visual("complexity", -0.2);
        }
    }

    fn match_bigram(pair: &str, signal: &mut TraitSignal) {
        if Self::matches_any(pair, &["像猫", "like a cat", "像小猫", "like a kitten"]) {
            *signal = signal.clone().with_visual("creature", -0.25).with_visual("softness", 0.15);
        } else if Self::matches_any(pair, &["像狗", "like a dog", "像小狗", "like a puppy"]) {
            *signal = signal.clone().with_visual("creature", 0.2);
        } else if Self::matches_any(pair, &["像鸟", "like a bird"]) {
            *signal = signal.clone().with_visual("creature", 0.35).with_visual("size", -0.15);
        } else if Self::matches_any(pair, &["像龙", "like a dragon"]) {
            *signal = signal.clone().with_visual("creature", 0.5).with_visual("size", 0.15);
        } else if Self::matches_any(pair, &["像光", "like light", "like a light"]) {
            *signal = signal.clone().with_visual("brightness", 0.3).with_visual("creature", 0.5);
        } else if Self::matches_any(pair, &["一团", "a ball"]) {
            *signal = signal.clone().with_visual("softness", 0.25);
        }
    }

    fn match_trigram(triple: &str, signal: &mut TraitSignal) {
        if Self::matches_any(triple, &["就像一只猫", "就像一只小猫"]) {
            *signal = signal.clone().with_visual("creature", -0.35).with_visual("softness", 0.2);
        } else if Self::matches_any(triple, &["就像一只狗", "就像一只小狗"]) {
            *signal = signal.clone().with_visual("creature", 0.3);
        }
    }

    fn matches_any(word: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|p| word == *p)
    }

    fn contains_any(word: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|p| word.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_reference() {
        let signal = ConversationMiner::mine("你就像一只猫");
        assert!(signal.visual_deltas.creature <= -0.35, "creature delta should be <= -0.35, got {}", signal.visual_deltas.creature);
    }

    #[test]
    fn test_dog_reference() {
        let signal = ConversationMiner::mine("你像一只小狗");
        assert!(signal.visual_deltas.creature >= 0.15);
    }

    #[test]
    fn test_warmth() {
        let signal = ConversationMiner::mine("你很温暖");
        assert!(signal.visual_deltas.warmth > 0.15);
    }

    #[test]
    fn test_energy() {
        let signal = ConversationMiner::mine("你好活跃");
        assert!(signal.visual_deltas.energy > 0.15);
    }

    #[test]
    fn test_dragon_light() {
        let signal = ConversationMiner::mine("你像一团光");
        assert!(signal.visual_deltas.brightness > 0.2);
    }

    #[test]
    fn test_no_match_leaves_default() {
        let signal = ConversationMiner::mine("今天天气不错");
        assert!((signal.visual_deltas.creature).abs() < 1e-6);
    }

    #[test]
    fn test_english_cat() {
        let signal = ConversationMiner::mine("you are like a cat");
        assert!(signal.visual_deltas.creature <= -0.15);
    }

    #[test]
    fn test_english_warm() {
        let signal = ConversationMiner::mine("you are so warm and bright");
        assert!(signal.visual_deltas.warmth > 0.15);
        assert!(signal.visual_deltas.brightness > 0.2);
    }
}
