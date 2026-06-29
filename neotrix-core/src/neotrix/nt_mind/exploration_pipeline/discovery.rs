use super::super::core::FIELD_NAMES;
use super::super::self_iterating::ReasoningBrain;
use super::super::web_miner::WebMinedKnowledge;
use super::*;

impl ExplorationPipeline {
    /// 从已抓取的 Wikipedia 内容中提取新链接，自动发现相关页面
    pub(super) fn discover_from_content(
        &mut self,
        mined: &[WebMinedKnowledge],
        domain: ExploreDomain,
    ) -> usize {
        let mut discovered = 0usize;
        for kn in mined {
            let text = &kn.summary;
            let lower = text.to_lowercase();
            let domain_terms: &[&str] = match domain {
                ExploreDomain::Parapsychology => &[
                    "psi",
                    "esp",
                    "telepathy",
                    "precognition",
                    "psychokinesis",
                    "medium",
                    "haunting",
                    "poltergeist",
                    "clairvoyance",
                    "intuition",
                    "anomaly",
                    "supernatural",
                    "quantum consciousness",
                    "orchestrated reduction",
                ],
                ExploreDomain::Theology => &[
                    "theology",
                    "religion",
                    "god",
                    "divine",
                    "sacred",
                    "faith",
                    "prayer",
                    "worship",
                    "scripture",
                    "revelation",
                    "salvation",
                    "grace",
                    "bible",
                    "quran",
                    "torah",
                    "vedas",
                    "sutra",
                ],
                ExploreDomain::EsotericStudies => &[
                    "occult",
                    "mystery",
                    "hermetic",
                    "alchemical",
                    "astral",
                    "chakra",
                    "morphic",
                    "akashic",
                    "subtle body",
                    "etheric",
                    "initiation",
                    "correspondence",
                    "synchronicity",
                    "archetype",
                ],
                ExploreDomain::Consciousness => &[
                    "consciousness",
                    "qualia",
                    "phenomenology",
                    "awareness",
                    "sentience",
                    "integrated information",
                    "phi",
                    "global workspace",
                    "binding",
                    "attention",
                    "metacognition",
                    "self-awareness",
                    "theory of mind",
                    "predictive processing",
                    "active inference",
                    "free energy",
                    "neural correlate",
                    "hard problem",
                    "panpsychism",
                    "iit",
                ],
                ExploreDomain::RustML => &[
                    "rust",
                    "machine learning",
                    "neural",
                    "tensor",
                    "deep learning",
                    "transformer",
                    "reinforcement",
                    "differentiable",
                    "gradient",
                    "candle",
                    "burn",
                    "dfdx",
                    "linfa",
                    "tract",
                ],
                ExploreDomain::Security => &[
                    "nt_shield",
                    "vulnerability",
                    "exploit",
                    "penetration",
                    "injection",
                    "xss",
                    "csrf",
                    "buffer",
                    "overflow",
                    "mitigation",
                    "owasp",
                    "cve",
                    "zero-day",
                    "authentication",
                    "authorization",
                ],
                ExploreDomain::MathPhysics => &[
                    "category",
                    "topology",
                    "group theory",
                    "representation",
                    "gauge",
                    "symmetry",
                    "entropy",
                    "information",
                    "complexity",
                    "fractal",
                    "chaos",
                    "quantum",
                    "field theory",
                    "renormalization",
                    "gravity",
                ],
                _ => &["philosophy", "theory", "concept", "history", "research"],
            };
            let mut matched = false;
            for term in domain_terms {
                if lower.contains(term) {
                    matched = true;
                    break;
                }
            }
            if !matched {
                continue;
            }

            let title_lower = kn.title.to_lowercase().replace(' ', "_");
            let candidate = format!("https://en.wikipedia.org/wiki/{}", title_lower);
            if !self.processed.contains(&candidate)
                && !self.auto_discovered.contains(&candidate)
                && candidate != kn.source_url
            {
                self.auto_discovered.insert(candidate.clone());
                self.seed_queue.push_back((domain, vec![candidate]));
                discovered += 1;
                if discovered >= 10 {
                    break;
                }
            }
        }
        discovered
    }

    /// 自动构建探索目标：根据能力缺口生成新的探索任务
    pub(super) fn auto_generate_goals(&mut self, brain: &ReasoningBrain) -> usize {
        let mut goals = 0usize;
        let cap = &brain.capability;
        let arr = cap.arr();
        let weak_dims: Vec<(usize, f64)> = arr
            .iter()
            .enumerate()
            .filter(|(_, &v)| v < 0.3)
            .map(|(i, &v)| (i, v))
            .collect();

        for (idx, _) in &weak_dims {
            let name = FIELD_NAMES.get(*idx).copied().unwrap_or("unknown");
            let new_urls: &[&str] = match name {
                "inference_depth" | "analysis" => &[
                    "https://en.wikipedia.org/wiki/Reasoning",
                    "https://en.wikipedia.org/wiki/Critical_thinking",
                    "https://en.wikipedia.org/wiki/Problem_solving",
                ],
                "synthesis" | "creativity" => &[
                    "https://en.wikipedia.org/wiki/Creativity",
                    "https://en.wikipedia.org/wiki/Innovation",
                    "https://en.wikipedia.org/wiki/Design_thinking",
                ],
                "domain_specificity" => &[
                    "https://en.wikipedia.org/wiki/Expert",
                    "https://en.wikipedia.org/wiki/Specialization",
                ],
                "experimental" => &[
                    "https://en.wikipedia.org/wiki/Scientific_method",
                    "https://en.wikipedia.org/wiki/Experiment",
                ],
                _ => continue,
            };
            for url in new_urls {
                let url_str = url.to_string();
                if !self.processed.contains(&url_str) && !self.auto_discovered.contains(&url_str) {
                    self.seed_queue
                        .push_back((ExploreDomain::General, vec![url_str.clone()]));
                    self.auto_discovered.insert(url_str);
                    goals += 1;
                }
            }
            if goals >= 6 {
                break;
            }
        }
        goals
    }
}
