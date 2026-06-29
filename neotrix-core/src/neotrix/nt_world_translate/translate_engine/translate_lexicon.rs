use crate::core::nt_core_translate::bilingual::BilingualLexicon;
use crate::core::nt_core_translate::hypergraph_integration::store_translation_as_hyperedge;
use crate::core::nt_core_translate::language::Language;
use crate::core::nt_core_translate::translate_engine::translate_types::TranslationStrategy;
use crate::core::nt_core_translate::translate_engine::VsaTranslationEngine;

impl VsaTranslationEngine {
    pub fn try_multiword_match(
        &mut self,
        text: &str,
        source_lang: Language,
        target_lang: Language,
    ) -> Option<crate::core::nt_core_translate::translate_engine::translate_types::TranslationResult>
    {
        if let Some((target, conf)) = self.multiword_match_for_text(text, source_lang, target_lang)
        {
            return Some(crate::core::nt_core_translate::translate_engine::translate_types::TranslationResult {
                source_text: text.to_string(),
                target_text: target,
                source_lang,
                target_lang,
                strategy: TranslationStrategy::DirectLookup,
                confidence: conf,
                vsa_similarity: conf,
                entry_id: None,
            });
        }

        if let Some(ref mut mem) = self.spreading_memory {
            let query_vsa = BilingualLexicon::text_to_vsa_deterministic(text);
            let activated = mem.retrieve_with_hops(&query_vsa, 5, 2);
            for (_, label, activation) in &activated {
                if *activation > 0.6 && label.to_lowercase() != text.to_lowercase() {
                    let entries = self.lexicon.entries_for_pair(source_lang, target_lang);
                    if let Some(e) = entries
                        .iter()
                        .find(|e| e.source_text.to_lowercase() == label.to_lowercase())
                    {
                        return Some(crate::core::nt_core_translate::translate_engine::translate_types::TranslationResult {
                            source_text: text.to_string(),
                            target_text: e.target_text.clone(),
                            source_lang,
                            target_lang,
                            strategy: TranslationStrategy::DirectLookup,
                            confidence: *activation * 0.85,
                            vsa_similarity: *activation,
                            entry_id: Some(e.id),
                        });
                    }
                }
            }
        }

        None
    }

    fn multiword_match_for_text(
        &self,
        text: &str,
        source_lang: Language,
        target_lang: Language,
    ) -> Option<(String, f64)> {
        let entries = self.lexicon.entries_for_pair(source_lang, target_lang);
        let text_lower = text.to_lowercase();

        if let Some(e) = entries
            .iter()
            .find(|e| e.source_text.to_lowercase() == text_lower)
        {
            return Some((e.target_text.clone(), 0.95));
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        for len in (2..=words.len().min(4)).rev() {
            for chunk in words.windows(len) {
                let phrase = chunk.join(" ");
                if let Some(e) = entries
                    .iter()
                    .find(|e| e.source_text.to_lowercase() == phrase.to_lowercase())
                {
                    return Some((e.target_text.clone(), 0.9));
                }
            }
        }

        None
    }

    pub fn learn_from_translation(
        &mut self,
        result: &crate::core::nt_core_translate::translate_engine::translate_types::TranslationResult,
    ) {
        if result.confidence > 0.6
            && result.source_text != result.target_text
            && result.source_lang != Language::Unknown
            && result.target_lang != Language::Unknown
        {
            let source_lower = result.source_text.to_lowercase().trim().to_string();
            let target_lower = result.target_text.to_lowercase().trim().to_string();
            if !source_lower.is_empty()
                && !target_lower.is_empty()
                && source_lower != target_lower
                && !self.lexicon.has_entry(
                    &source_lower,
                    &target_lower,
                    result.source_lang,
                    result.target_lang,
                )
            {
                let id = self.lexicon.add_entry(
                    &source_lower,
                    &target_lower,
                    result.source_lang,
                    result.target_lang,
                );
                if let Some(store) = self.hypergraph_store.as_mut() {
                    if let Some(entry) = self.lexicon.entries.iter().find(|e| e.id == id) {
                        store_translation_as_hyperedge(store, entry);
                    }
                }
                if self.lexicon.len() >= self.lexicon.max_entries() * 3 / 4 {
                    self.lexicon.prune();
                }
            }
        }
    }

    pub fn seed_common_pairs(&mut self) {
        let pairs: &[(&str, &str, Language, Language)] = &[
            // English ↔ Spanish
            ("hello", "hola", Language::English, Language::Spanish),
            ("goodbye", "adiós", Language::English, Language::Spanish),
            ("thank you", "gracias", Language::English, Language::Spanish),
            ("please", "por favor", Language::English, Language::Spanish),
            ("yes", "sí", Language::English, Language::Spanish),
            ("no", "no", Language::English, Language::Spanish),
            ("water", "agua", Language::English, Language::Spanish),
            ("food", "comida", Language::English, Language::Spanish),
            ("house", "casa", Language::English, Language::Spanish),
            ("friend", "amigo", Language::English, Language::Spanish),
            ("love", "amor", Language::English, Language::Spanish),
            ("sun", "sol", Language::English, Language::Spanish),
            ("world", "mundo", Language::English, Language::Spanish),
            ("time", "tiempo", Language::English, Language::Spanish),
            ("day", "día", Language::English, Language::Spanish),
            ("night", "noche", Language::English, Language::Spanish),
            ("big", "grande", Language::English, Language::Spanish),
            ("small", "pequeño", Language::English, Language::Spanish),
            // English ↔ French
            ("hello", "bonjour", Language::English, Language::French),
            ("goodbye", "au revoir", Language::English, Language::French),
            ("thank you", "merci", Language::English, Language::French),
            (
                "please",
                "s'il vous plaît",
                Language::English,
                Language::French,
            ),
            ("yes", "oui", Language::English, Language::French),
            ("no", "non", Language::English, Language::French),
            ("water", "eau", Language::English, Language::French),
            ("house", "maison", Language::English, Language::French),
            ("love", "amour", Language::English, Language::French),
            ("friend", "ami", Language::English, Language::French),
            ("world", "monde", Language::English, Language::French),
            ("time", "temps", Language::English, Language::French),
            ("day", "jour", Language::English, Language::French),
            ("night", "nuit", Language::English, Language::French),
            ("bread", "pain", Language::English, Language::French),
            ("sun", "soleil", Language::English, Language::French),
            // English ↔ German
            ("hello", "hallo", Language::English, Language::German),
            (
                "goodbye",
                "auf Wiedersehen",
                Language::English,
                Language::German,
            ),
            ("thank you", "danke", Language::English, Language::German),
            ("yes", "ja", Language::English, Language::German),
            ("no", "nein", Language::English, Language::German),
            ("water", "Wasser", Language::English, Language::German),
            ("house", "Haus", Language::English, Language::German),
            ("love", "Liebe", Language::English, Language::German),
            ("friend", "Freund", Language::English, Language::German),
            ("sun", "Sonne", Language::English, Language::German),
            ("world", "Welt", Language::English, Language::German),
            ("time", "Zeit", Language::English, Language::German),
            ("day", "Tag", Language::English, Language::German),
            ("night", "Nacht", Language::English, Language::German),
            ("bread", "Brot", Language::English, Language::German),
            ("good", "gut", Language::English, Language::German),
            // English ↔ Japanese
            ("hello", "こんにちは", Language::English, Language::Japanese),
            (
                "goodbye",
                "さようなら",
                Language::English,
                Language::Japanese,
            ),
            (
                "thank you",
                "ありがとう",
                Language::English,
                Language::Japanese,
            ),
            ("yes", "はい", Language::English, Language::Japanese),
            ("no", "いいえ", Language::English, Language::Japanese),
            ("water", "水", Language::English, Language::Japanese),
            ("love", "愛", Language::English, Language::Japanese),
            ("friend", "友達", Language::English, Language::Japanese),
            ("sun", "太陽", Language::English, Language::Japanese),
            ("world", "世界", Language::English, Language::Japanese),
            ("time", "時間", Language::English, Language::Japanese),
            ("day", "日", Language::English, Language::Japanese),
            ("night", "夜", Language::English, Language::Japanese),
            ("good", "良い", Language::English, Language::Japanese),
            ("beautiful", "美しい", Language::English, Language::Japanese),
            // English ↔ Korean
            ("hello", "안녕하세요", Language::English, Language::Korean),
            (
                "goodbye",
                "안녕히 가세요",
                Language::English,
                Language::Korean,
            ),
            (
                "thank you",
                "감사합니다",
                Language::English,
                Language::Korean,
            ),
            ("yes", "네", Language::English, Language::Korean),
            ("no", "아니요", Language::English, Language::Korean),
            ("water", "물", Language::English, Language::Korean),
            ("love", "사랑", Language::English, Language::Korean),
            ("friend", "친구", Language::English, Language::Korean),
            ("world", "세계", Language::English, Language::Korean),
            ("time", "시간", Language::English, Language::Korean),
            ("day", "날", Language::English, Language::Korean),
            ("night", "밤", Language::English, Language::Korean),
            ("beautiful", "아름다운", Language::English, Language::Korean),
            // English ↔ Russian
            (
                "hello",
                "здравствуйте",
                Language::English,
                Language::Russian,
            ),
            (
                "goodbye",
                "до свидания",
                Language::English,
                Language::Russian,
            ),
            ("thank you", "спасибо", Language::English, Language::Russian),
            ("yes", "да", Language::English, Language::Russian),
            ("no", "нет", Language::English, Language::Russian),
            ("water", "вода", Language::English, Language::Russian),
            ("love", "любовь", Language::English, Language::Russian),
            ("friend", "друг", Language::English, Language::Russian),
            ("world", "мир", Language::English, Language::Russian),
            ("time", "время", Language::English, Language::Russian),
            ("day", "день", Language::English, Language::Russian),
            ("night", "ночь", Language::English, Language::Russian),
            ("bread", "хлеб", Language::English, Language::Russian),
            // English ↔ Arabic
            ("hello", "مرحبا", Language::English, Language::Arabic),
            ("goodbye", "وداعا", Language::English, Language::Arabic),
            ("thank you", "شكرا", Language::English, Language::Arabic),
            ("yes", "نعم", Language::English, Language::Arabic),
            ("no", "لا", Language::English, Language::Arabic),
            ("water", "ماء", Language::English, Language::Arabic),
            ("love", "حب", Language::English, Language::Arabic),
            ("friend", "صديق", Language::English, Language::Arabic),
            ("world", "عالم", Language::English, Language::Arabic),
            ("time", "وقت", Language::English, Language::Arabic),
            ("day", "يوم", Language::English, Language::Arabic),
            ("night", "ليل", Language::English, Language::Arabic),
            ("beautiful", "جميل", Language::English, Language::Arabic),
            // English ↔ Hindi
            ("hello", "नमस्ते", Language::English, Language::Hindi),
            ("goodbye", "अलविदा", Language::English, Language::Hindi),
            ("thank you", "धन्यवाद", Language::English, Language::Hindi),
            ("yes", "हाँ", Language::English, Language::Hindi),
            ("no", "नहीं", Language::English, Language::Hindi),
            ("water", "पानी", Language::English, Language::Hindi),
            ("love", "प्यार", Language::English, Language::Hindi),
            ("friend", "दोस्त", Language::English, Language::Hindi),
            ("world", "दुनिया", Language::English, Language::Hindi),
            ("time", "समय", Language::English, Language::Hindi),
            ("day", "दिन", Language::English, Language::Hindi),
            ("night", "रात", Language::English, Language::Hindi),
            ("good", "अच्छा", Language::English, Language::Hindi),
            // English ↔ Thai
            ("hello", "สวัสดี", Language::English, Language::Thai),
            ("goodbye", "ลาก่อน", Language::English, Language::Thai),
            ("thank you", "ขอบคุณ", Language::English, Language::Thai),
            ("yes", "ใช่", Language::English, Language::Thai),
            ("no", "ไม่", Language::English, Language::Thai),
            ("water", "น้ำ", Language::English, Language::Thai),
            ("love", "รัก", Language::English, Language::Thai),
            ("friend", "เพื่อน", Language::English, Language::Thai),
            ("world", "โลก", Language::English, Language::Thai),
            ("time", "เวลา", Language::English, Language::Thai),
            ("day", "วัน", Language::English, Language::Thai),
            ("night", "กลางคืน", Language::English, Language::Thai),
            // English ↔ Portuguese
            ("hello", "olá", Language::English, Language::Portuguese),
            ("goodbye", "tchau", Language::English, Language::Portuguese),
            (
                "thank you",
                "obrigado",
                Language::English,
                Language::Portuguese,
            ),
            ("yes", "sim", Language::English, Language::Portuguese),
            ("no", "não", Language::English, Language::Portuguese),
            ("water", "água", Language::English, Language::Portuguese),
            ("love", "amor", Language::English, Language::Portuguese),
            ("friend", "amigo", Language::English, Language::Portuguese),
            ("world", "mundo", Language::English, Language::Portuguese),
            ("time", "tempo", Language::English, Language::Portuguese),
            ("day", "dia", Language::English, Language::Portuguese),
            ("night", "noite", Language::English, Language::Portuguese),
            ("house", "casa", Language::English, Language::Portuguese),
            ("sun", "sol", Language::English, Language::Portuguese),
            ("bread", "pão", Language::English, Language::Portuguese),
            ("good", "bom", Language::English, Language::Portuguese),
            ("big", "grande", Language::English, Language::Portuguese),
            ("small", "pequeno", Language::English, Language::Portuguese),
            // English ↔ Italian
            ("hello", "ciao", Language::English, Language::Italian),
            (
                "goodbye",
                "arrivederci",
                Language::English,
                Language::Italian,
            ),
            ("thank you", "grazie", Language::English, Language::Italian),
            ("please", "per favore", Language::English, Language::Italian),
            ("yes", "sì", Language::English, Language::Italian),
            ("no", "no", Language::English, Language::Italian),
            ("water", "acqua", Language::English, Language::Italian),
            ("love", "amore", Language::English, Language::Italian),
            ("friend", "amico", Language::English, Language::Italian),
            ("world", "mondo", Language::English, Language::Italian),
            ("time", "tempo", Language::English, Language::Italian),
            ("day", "giorno", Language::English, Language::Italian),
            ("night", "notte", Language::English, Language::Italian),
            ("house", "casa", Language::English, Language::Italian),
            ("sun", "sole", Language::English, Language::Italian),
            ("bread", "pane", Language::English, Language::Italian),
            ("good", "buono", Language::English, Language::Italian),
            ("beautiful", "bello", Language::English, Language::Italian),
            // Chinese ↔ English (bidirectional)
            ("你好", "hello", Language::Chinese, Language::English),
            ("谢谢", "thank you", Language::Chinese, Language::English),
            ("是", "yes", Language::Chinese, Language::English),
            ("不", "no", Language::Chinese, Language::English),
            ("水", "water", Language::Chinese, Language::English),
            ("爱", "love", Language::Chinese, Language::English),
            ("朋友", "friend", Language::Chinese, Language::English),
            ("世界", "world", Language::Chinese, Language::English),
            ("时间", "time", Language::Chinese, Language::English),
            ("日", "day", Language::Chinese, Language::English),
            ("夜", "night", Language::Chinese, Language::English),
            ("大", "big", Language::Chinese, Language::English),
            ("小", "small", Language::Chinese, Language::English),
            ("好", "good", Language::Chinese, Language::English),
            ("美丽", "beautiful", Language::Chinese, Language::English),
            ("hello", "你好", Language::English, Language::Chinese),
            ("thank you", "谢谢", Language::English, Language::Chinese),
            ("water", "水", Language::English, Language::Chinese),
            ("house", "房子", Language::English, Language::Chinese),
            ("love", "爱", Language::English, Language::Chinese),
            ("friend", "朋友", Language::English, Language::Chinese),
            ("world", "世界", Language::English, Language::Chinese),
            ("time", "时间", Language::English, Language::Chinese),
            ("day", "天", Language::English, Language::Chinese),
            ("night", "夜", Language::English, Language::Chinese),
            ("big", "大", Language::English, Language::Chinese),
            ("small", "小", Language::English, Language::Chinese),
            ("good", "好", Language::English, Language::Chinese),
            ("beautiful", "美丽", Language::English, Language::Chinese),
        ];

        for (src, tgt, src_lang, tgt_lang) in pairs {
            self.lexicon.add_entry(src, tgt, *src_lang, *tgt_lang);
        }
        self.init_spreading_from_lexicon();
        self.sync_to_hypergraph();
    }
}
