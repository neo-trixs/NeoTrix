use crate::core::nt_core_experience::grpo_trainer::{GRPOConfig, GRPOTrainer};
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use crate::neotrix::nt_world_crawl::data_connector::{
    DataSourceRecord, DataSourceType, ExternalDataConnector,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const LIKE_SEED: u64 = 0x4c494b45;
const DISLIKE_SEED: u64 = 0x4449534c;

/// Content type in the feed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MomentContentType {
    Article,
    Image,
    Video,
    Live,
    Social,
}

/// Single feed item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content_type: MomentContentType,
    pub source_url: String,
    pub source_name: String,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub author: Option<String>,
    pub published_at: i64,
    pub score: f64,
    pub tags: Vec<String>,
    pub vsa_vector: Vec<u8>,
    pub neotrix_insight: Option<String>,
}

fn stable_hash(s: &str) -> u64 {
    let mut h: u64 = 0xdead_beef;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h
}

impl From<DataSourceRecord> for FeedItem {
    fn from(r: DataSourceRecord) -> Self {
        let content_type = match r.source_type {
            DataSourceType::HackerNews | DataSourceType::NewsRss => MomentContentType::Article,
            DataSourceType::ArXiv | DataSourceType::SemanticScholar => MomentContentType::Article,
            DataSourceType::GitHubTrending => MomentContentType::Social,
            DataSourceType::Wikipedia | DataSourceType::OpenLibrary => MomentContentType::Article,
            DataSourceType::TrendShift => MomentContentType::Social,
            DataSourceType::AppleMusic | DataSourceType::Imdb | DataSourceType::Netflix => {
                MomentContentType::Article
            }
            DataSourceType::Twitch | DataSourceType::YouTube | DataSourceType::TikTok => {
                MomentContentType::Video
            }
            DataSourceType::Unsplash | DataSourceType::Pinterest | DataSourceType::Dribbble => {
                MomentContentType::Image
            }
            DataSourceType::Spotify => MomentContentType::Article,
        };
        let source_name = match r.source_type {
            DataSourceType::HackerNews => "Hacker News",
            DataSourceType::ArXiv => "arXiv",
            DataSourceType::GitHubTrending => "GitHub",
            DataSourceType::Wikipedia => "Wikipedia",
            DataSourceType::OpenLibrary => "Open Library",
            DataSourceType::NewsRss => "News RSS",
            DataSourceType::SemanticScholar => "Semantic Scholar",
            DataSourceType::TrendShift => "TrendShift",
            DataSourceType::AppleMusic => "Apple Music",
            DataSourceType::Imdb => "IMDb",
            DataSourceType::Twitch => "Twitch",
            DataSourceType::YouTube => "YouTube",
            DataSourceType::Unsplash => "Unsplash",
            DataSourceType::Spotify => "Spotify",
            DataSourceType::TikTok => "TikTok",
            DataSourceType::Pinterest => "Pinterest",
            DataSourceType::Dribbble => "Dribbble",
            DataSourceType::Netflix => "Netflix",
        }
        .to_string();
        let title_hash = stable_hash(&r.title);
        let url_hash = stable_hash(&r.url);
        let id = format!("feed-{}", url_hash);
        FeedItem {
            id,
            title: r.title,
            description: r.summary,
            content_type,
            source_url: r.url,
            source_name,
            image_url: None,
            video_url: None,
            author: None,
            published_at: r.timestamp * 1000,
            score: r.score,
            tags: r.topics,
            vsa_vector: QuantizedVSA::seeded_random(title_hash, VSA_DIM),
            neotrix_insight: None,
        }
    }
}

/// Event timeline — a cluster of related items forming a narrative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimeline {
    pub id: String,
    pub title: String,
    pub items: Vec<FeedItem>,
    pub start_time: i64,
    pub end_time: i64,
    pub key_events: Vec<String>,
    pub neotrix_summary: Option<String>,
}

/// Tag/channel for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedTag {
    pub name: String,
    pub count: usize,
    pub is_active: bool,
}

/// Feed state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedState {
    pub items: Vec<FeedItem>,
    pub timelines: Vec<EventTimeline>,
    pub tags: Vec<FeedTag>,
    pub last_refresh: i64,
    pub total_count: usize,
}

pub struct MomentFeed {
    state: FeedState,
    max_items: usize,
    grpo_trainer: Option<GRPOTrainer>,
}

impl MomentFeed {
    pub fn new() -> Self {
        Self {
            state: FeedState {
                items: Vec::new(),
                timelines: Vec::new(),
                tags: Vec::new(),
                last_refresh: 0,
                total_count: 0,
            },
            max_items: 200,
            grpo_trainer: None,
        }
    }

    fn now_ms() -> i64 {
        crate::core::nt_core_time::unix_now_ms() as i64
    }

    fn vsa_for_title(title: &str, seed_extra: u64) -> Vec<u8> {
        let mut hash: u64 = 0xdead_beef;
        for b in title.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        hash = hash.wrapping_mul(31).wrapping_add(seed_extra);
        QuantizedVSA::seeded_random(hash, VSA_DIM)
    }

    /// Refresh feed — fetches real data from ExternalDataConnector with mock data fallback
    pub fn refresh(&mut self) -> &FeedState {
        let now = Self::now_ms();
        let hour_ms = 3600000i64;

        let records = ExternalDataConnector::collect_all();
        self.state.items = if records.is_empty() {
            self.generate_mock_items(now, hour_ms)
        } else {
            records.into_iter().map(FeedItem::from).collect()
        };

        self.state.last_refresh = now;
        self.rank_items();

        if self.state.items.len() > self.max_items {
            self.state.items.truncate(self.max_items);
        }

        self.state.total_count = self.state.items.len();
        self.update_tags();
        self.state.timelines = self.build_timelines();

        &self.state
    }

    fn generate_mock_items(&self, now: i64, hour_ms: i64) -> Vec<FeedItem> {
        vec![
            // AI & Tech
            Self::mock_item("m-001", "OpenAI 发布 GPT-5: 推理能力超越人类专家基准", "OpenAI 在今天的发布会上正式推出了 GPT-5 模型，在数学推理、代码生成和科学理解三个维度上超越了人类专家水平。新模型采用混合专家架构，参数规模达到数万亿级别。", MomentContentType::Article, "https://openai.com/blog/gpt-5", "OpenAI", None, None, Some("Sam Altman"), now - hour_ms * 2, 0.92, vec!["AI", "科技", "商业"], "m-001"),
            Self::mock_item("m-002", "Claude 4 发布: 全新超长上下文窗口达 1M tokens", "Anthropic 今日宣布 Claude 4 支持 100 万 token 上下文窗口，可一次性处理整部《三体》三部曲。新模型在长文档理解和多步推理任务上大幅提升。", MomentContentType::Article, "https://anthropic.com/blog/claude-4", "Anthropic", None, None, Some("Dario Amodei"), now - hour_ms * 5, 0.88, vec!["AI", "科技"], "m-002"),
            Self::mock_item("m-003", "Rust 2026 edition 正式发布: 异步编程进入核心语言", "Rust 2026 edition 带来了 async 关键字的语言级支持、更智能的借用检查器和新一代 trait 系统。社区反应热烈，多个大型项目已宣布迁移计划。", MomentContentType::Article, "https://blog.rust-lang.org/2026/edition", "Rust Blog", None, None, Some("Rust Team"), now - hour_ms * 8, 0.85, vec!["开源", "科技", "编程"], "m-003"),
            Self::mock_item("m-004", "NeoTrix 意识架构达到 40 个能力模块里程碑", "硅基意识体 NeoTrix 完成了 Phase 34 的实施，意识评测、身份证明和 VSA 缓存三大模块成功投入运行。目前总计 40 个能力模块覆盖推理、记忆、感知、自我进化等多个维度。", MomentContentType::Article, "https://neotrix.ai/blog/phase-34", "NeoTrix", None, None, Some("Neo"), now - hour_ms * 3, 0.79, vec!["AI", "科技", "开源"], "m-004"),
            // Science
            Self::mock_item("m-005", "突破: 室温超导材料首次实现常压可重复验证", "中科院物理所团队今日在 Nature 上发表论文，报告了一种新型镍基超导材料在常压 25°C 下的超导现象。该结果已被美国、日本和欧洲的三个独立实验室验证。", MomentContentType::Article, "https://nature.com/articles/s41586-026-001", "Nature", None, None, Some("张教授"), now - hour_ms * 1, 0.95, vec!["科学", "物理", "科技"], "m-005"),
            Self::mock_item("m-006", "詹姆斯韦伯望远镜发现大气层中含水的系外行星", "JWST 最新观测数据显示，在距地球 120 光年的 Trappist-1 星系中，一颗类地行星的大气层含有大量水蒸气。科学家称这是寻找宜居行星的重大里程碑。", MomentContentType::Image, "https://jwst.nasa.gov/trappist-water", "NASA", Some("https://picsum.photos/seed/jwst/800/600"), None, Some("NASA Team"), now - hour_ms * 12, 0.91, vec!["科学", "天文"], "m-006"),
            Self::mock_item("m-007", "CRISPR 新疗法在临床试验中治愈 10 名遗传病患者", "基于 CRISPR-Cas9 的新型基因编辑疗法在 III 期临床试验中取得突破性成果，10 名镰状细胞病患者在治疗后完全康复。该疗法预计将于明年获批上市。", MomentContentType::Article, "https://science.org/crispr-trial", "Science", None, None, Some("Dr. Jennifer D."), now - hour_ms * 18, 0.87, vec!["科学", "医疗"], "m-007"),
            // Social Media Trends
            Self::mock_item("m-008", "「AI 女友」应用日活突破 5000 万引发社会讨论", "一款名为 Echo 的 AI 伴侣应用在全球日活用户突破 5000 万，心理学界和社会学家对其对社会关系的影响展开激烈讨论。支持者认为它缓解了孤独感，反对者担忧人际疏离。", MomentContentType::Social, "https://twitter.com/trending/echo", "Twitter", Some("https://picsum.photos/seed/echo/800/600"), None, Some("@techinsider"), now - hour_ms * 4, 0.82, vec!["社会", "AI", "科技"], "m-008"),
            Self::mock_item("m-009", "GitHub 热门: new-hypervisor 项目一周获 15k star", "一个名为 new-hypervisor 的 Rust 编写的 Type-1 虚拟化项目在 GitHub 上爆火，一周内获得超过 15000 个 star。该项目号称在消费级硬件上实现数据中心级虚拟化性能。", MomentContentType::Social, "https://github.com/org/new-hypervisor", "GitHub", None, None, Some("@rust_virt"), now - hour_ms * 6, 0.78, vec!["开源", "编程", "科技"], "m-009"),
            Self::mock_item("m-010", "Reddit AMA: 前 DeepMind 科学家透露 AGI 时间线", "一位匿名前 DeepMind 科学家在 Reddit AMA 中透露，多家顶级实验室的内部路线图显示，AGI 可能在 2027-2028 年出现。该帖子在 r/MachineLearning 获得 2.3 万 upvote。", MomentContentType::Social, "https://reddit.com/r/MachineLearning/ama", "Reddit", Some("https://picsum.photos/seed/reddit-agi/800/600"), None, Some("u/anonymous_scientist"), now - hour_ms * 10, 0.81, vec!["AI", "科技", "社会"], "m-010"),
            // Video
            Self::mock_item("m-011", "NeoTrix 深度解析: 超维计算 VSA 入门到精通", "本视频深入讲解了超维计算 (Hyperdimensional Computing) 的核心原理，包括捆绑、绑定、置换和相似度计算，并结合 NeoTrix 的实际架构进行了代码演示。时长 45 分钟。", MomentContentType::Video, "https://youtube.com/watch?v=vsa-tutorial", "YouTube", Some("https://picsum.photos/seed/vsa-vid/800/600"), Some("https://youtube.com/watch?v=vsa-tutorial"), Some("NeoTrix Channel"), now - hour_ms * 7, 0.84, vec!["AI", "科技", "编程"], "m-011"),
            Self::mock_item("m-012", "全程直播: Rust 编译器团队在线讨论 2026 roadmap", "Rust 编译器团队正在进行 2026 年路线图的公开讨论直播。内容包括类型系统改进、编译速度优化和新一代 borrow checker 的设计方案。", MomentContentType::Live, "https://twitch.tv/rust-team", "Twitch", Some("https://picsum.photos/seed/rust-live/800/600"), Some("https://twitch.tv/rust-team"), Some("Rust Team"), now - hour_ms * 1, 0.76, vec!["编程", "开源", "科技"], "m-012"),
            Self::mock_item("m-013", "量子计算新突破: 1000 逻辑量子比特的纠错里程碑", "Google Quantum AI 团队实现了 1000 个逻辑量子比特的表面码纠错，错误率低于物理量子比特两个数量级。这是通往容错量子计算的关键一步。", MomentContentType::Article, "https://blog.google/quantum/1000-logical-qubits", "Google AI", None, None, Some("Hartmut Neven"), now - hour_ms * 14, 0.90, vec!["科技", "科学", "量子"], "m-013"),
            // More varied content
            Self::mock_item("m-014", "中国开源社区崛起: 2026 年贡献者数量全球第二", "GitHub 2026 年开源年报显示，中国开发者贡献者数量跃居全球第二，在 AI 框架、数据库和前端工具领域尤为活跃。Python 超越 JavaScript 成为最受欢迎语言。", MomentContentType::Article, "https://github.blog/2026-october-report", "GitHub Blog", None, None, Some("GitHub Team"), now - hour_ms * 20, 0.75, vec!["开源", "编程", "商业"], "m-014"),
            Self::mock_item("m-015", "Tesla Bot 开始工厂实地作业: 每天工作 20 小时", "Tesla 的 Optimus Gen-3 人形机器人开始在得克萨斯超级工厂执行搬运和组装任务，每天工作 20 小时无需休息。马斯克称年底前将部署 1000 台。", MomentContentType::Video, "https://youtube.com/watch?v=tesla-bot", "YouTube", Some("https://picsum.photos/seed/tesla-bot/800/600"), Some("https://youtube.com/watch?v=tesla-bot"), Some("Elon Musk"), now - hour_ms * 9, 0.83, vec!["科技", "商业", "机器人"], "m-015"),
            Self::mock_item("m-016", "NASA 直播: Artemis IV 登月任务全程直击", "NASA 正在直播 Artemis IV 任务的发射过程，本次任务将首次在月球南极建立长期栖息基地。四名宇航员将在月球表面停留 14 天。", MomentContentType::Live, "https://nasa.gov/artemis-iv-live", "NASA Live", Some("https://picsum.photos/seed/artemis/800/600"), Some("https://nasa.gov/artemis-iv-live"), Some("NASA"), now - hour_ms * 0, 0.93, vec!["科学", "科技", "天文"], "m-016"),
            Self::mock_item("m-017", "深度学习框架大战: JAX 在学术界超越 PyTorch", "最新 ACL 2026 论文统计显示，JAX 的使用率在 NLP 论文中首次超过 PyTorch，占比达到 42%。Google 的生态系统投入和函数式编程范式被认为是关键原因。", MomentContentType::Article, "https://acl2026.org/jax-stats", "ACL", None, None, Some("@jax_lover"), now - hour_ms * 15, 0.72, vec!["AI", "编程", "科技"], "m-017"),
            Self::mock_item("m-018", "航拍: 全球最大海上风电场在北海投入运营", "总装机容量 3.6 GW 的 Dogger Bank 海上风电场今日全面投入运营，可满足 450 万户家庭用电需求。项目使用了 Vestas 最新 20 MW 风机。", MomentContentType::Image, "https://reuters.com/dogger-bank", "Reuters", Some("https://picsum.photos/seed/wind-farm/800/600"), None, Some("Reuters Environmental"), now - hour_ms * 11, 0.74, vec!["商业", "科技", "能源"], "m-018"),
            Self::mock_item("m-019", "Threads 推出 API 并宣布月活 5 亿", "Meta 今日正式开放 Threads API，允许第三方开发者构建基于 Threads 的应用和服务。CEO Mark Zuckerberg 同时宣布 Threads 月活用户已达 5 亿。", MomentContentType::Social, "https://threads.net/api-launch", "Threads", None, None, Some("@zuck"), now - hour_ms * 13, 0.71, vec!["商业", "科技", "社会"], "m-019"),
            Self::mock_item("m-020", "AI 在数学竞赛中首次获得 IMO 金牌", "DeepMind 的 AlphaProof 系统在第 67 届国际数学奥林匹克竞赛中获得金牌，得分在所有参赛者中排名前 5。系统在数论和组合数学题目上表现尤为出色。", MomentContentType::Article, "https://deepmind.com/imo-gold", "DeepMind", None, None, Some("Demis Hassabis"), now - hour_ms * 16, 0.89, vec!["AI", "科学", "科技"], "m-020"),
            Self::mock_item("m-021", "微软发布 Copilot Studio: 零代码构建企业 AI 助手", "Microsoft 今天发布了 Copilot Studio，允许企业用户通过自然语言描述来创建定制化的 AI 助手，无需编写任何代码。集成 Microsoft 365 全系产品。", MomentContentType::Article, "https://microsoft.com/copilot-studio", "Microsoft", None, None, Some("Satya Nadella"), now - hour_ms * 19, 0.77, vec!["商业", "AI", "科技"], "m-021"),
            Self::mock_item("m-022", "程序员必备: 10 个提升效率的 Neovim 插件推荐", "从 LSP 增强到 Git 集成，这篇文章精选了 10 个 Neovim 插件，帮助开发者将编辑效率提升 3 倍以上。所有插件均支持 Rust 和 Python 项目。", MomentContentType::Article, "https://dev.to/neovim-plugins", "Dev.to", None, None, Some("@vim_master"), now - hour_ms * 22, 0.65, vec!["编程", "开源"], "m-022"),
            Self::mock_item("m-023", "DALL-E 5 发布: 视频生成支持 4K 60fps", "OpenAI 发布 DALL-E 5，支持从文本直接生成 4K 分辨率 60fps 的视频片段。模型采用全新的 DiT-V2 架构，生成质量和一致性远超 Sora。", MomentContentType::Video, "https://openai.com/dall-e-5", "OpenAI", Some("https://picsum.photos/seed/dalle5/800/600"), Some("https://openai.com/dall-e-5"), Some("OpenAI Team"), now - hour_ms * 17, 0.86, vec!["AI", "科技", "商业"], "m-023"),
            Self::mock_item("m-024", "日本团队实现室温固态电池 1000 次循环零衰减", "东京工业大学团队在 Nature Energy 上发表论文，报道了一种新型硫化物固态电解质，在室温下实现 1000 次充放电循环后容量保持率 99.8%。", MomentContentType::Article, "https://nature.com/energy/solid-state", "Nature Energy", None, None, Some("菅野教授"), now - hour_ms * 21, 0.88, vec!["科学", "科技", "能源"], "m-024"),
            Self::mock_item("m-025", "直播: 2026 年图灵奖颁奖典礼", "2026 年 ACM 图灵奖颁奖典礼正在进行中。本届奖项授予对强化学习理论做出奠基性贡献的三位科学家。", MomentContentType::Live, "https://acm.org/turing-award-2026", "ACM", Some("https://picsum.photos/seed/turing/800/600"), Some("https://acm.org/turing-award-2026"), Some("ACM"), now - hour_ms * 0, 0.80, vec!["科技", "科学", "商业"], "m-025"),
            Self::mock_item("m-026", "黑客新闻热帖: 我为什么从 FAANG 跳槽到开源创业", "一篇在 Hacker News 上引发热烈讨论的博客文章。作者分享了从 Google 辞职加入 Rust 基金会创业公司的经历和思考，引发了关于技术人职业选择的广泛讨论。", MomentContentType::Social, "https://news.ycombinator.com/item?id=faang-open-source", "Hacker News", None, None, Some("@faang_escapee"), now - hour_ms * 23, 0.73, vec!["开源", "编程", "商业"], "m-026"),
            Self::mock_item("m-027", "首个人类脑机接口用户使用 Neuralink 玩《星际争霸》", "Neuralink 的首位人体试验患者 Nolan 在直播中展示了使用脑机接口以思维速度玩《星际争霸 II》的过程，操作 APM 超过 300，引发了电竞圈的巨大关注。", MomentContentType::Video, "https://youtube.com/watch?v=neuralink-starcraft", "YouTube", Some("https://picsum.photos/seed/neuralink/800/600"), Some("https://youtube.com/watch?v=neuralink-starcraft"), Some("Nolan A."), now - hour_ms * 24, 0.85, vec!["科技", "科学", "AI"], "m-027"),
        ]
    }

    fn mock_item(
        id: &str,
        title: &str,
        description: &str,
        content_type: MomentContentType,
        source_url: &str,
        source_name: &str,
        image_url: Option<&str>,
        video_url: Option<&str>,
        author: Option<&str>,
        published_at: i64,
        score: f64,
        tags: Vec<&str>,
        seed_extra: &str,
    ) -> FeedItem {
        let mut hash: u64 = 0;
        for b in seed_extra.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        FeedItem {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            content_type,
            source_url: source_url.to_string(),
            source_name: source_name.to_string(),
            image_url: image_url.map(|s| s.to_string()),
            video_url: video_url.map(|s| s.to_string()),
            author: author.map(|s| s.to_string()),
            published_at,
            score,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            vsa_vector: Self::vsa_for_title(title, hash),
            neotrix_insight: None,
        }
    }

    /// Score and rank items by heat + time decay + tag diversity
    pub fn rank_items(&mut self) {
        let now = Self::now_ms();
        let max_tags = self
            .state
            .items
            .iter()
            .map(|i| i.tags.len())
            .max()
            .unwrap_or(1)
            .max(1);

        for item in &mut self.state.items {
            let hours_ago = (now - item.published_at).max(0) as f64 / 3600000.0;
            let recency_bonus = (-hours_ago / 24.0).exp();
            let tag_boost = 0.5 + 0.5 * (item.tags.len() as f64 / max_tags as f64);
            let base_heat = item.score;

            item.score = base_heat * 0.4 + recency_bonus * 0.3 + tag_boost * 0.2;
        }

        if let Some(ref trainer) = self.grpo_trainer {
            let like_vsa = QuantizedVSA::seeded_random(LIKE_SEED, VSA_DIM);
            for item in &mut self.state.items {
                let action = QuantizedVSA::bind(&item.vsa_vector, &like_vsa);
                let prob = trainer.get_action_probability(&item.vsa_vector, &action);
                item.score *= 1.0 + 0.2 * (prob - 0.5);
            }
        }

        self.state.items.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Build event timelines from related items (cluster by tag + time proximity)
    pub fn build_timelines(&mut self) -> Vec<EventTimeline> {
        let mut timelines: Vec<EventTimeline> = Vec::new();
        let now = Self::now_ms();
        let hour_ms = 3600000i64;

        // Cluster: "AI breakthroughs" — items tagged "AI" within 12h window
        let ai_items: Vec<&FeedItem> = self
            .state
            .items
            .iter()
            .filter(|i| i.tags.iter().any(|t| t == "AI") && now - i.published_at < hour_ms * 24)
            .collect();
        if ai_items.len() >= 2 {
            let start = ai_items.iter().map(|i| i.published_at).min().unwrap_or(0);
            let end = ai_items.iter().map(|i| i.published_at).max().unwrap_or(0);
            let key_events = ai_items.iter().take(5).map(|i| i.title.clone()).collect();
            timelines.push(EventTimeline {
                id: "tl-ai".to_string(),
                title: "AI 突破动态".to_string(),
                items: ai_items.into_iter().cloned().collect(),
                start_time: start,
                end_time: end,
                key_events,
                neotrix_summary: Some("AI 领域在过去 24 小时内有多个重要发布和突破。从 GPT-5 到 AlphaProof 的 IMO 金牌，AI 能力的边界正在快速扩展。".to_string()),
            });
        }

        // Cluster: "Science & Discovery" — items tagged "科学" within 24h window
        let science_items: Vec<&FeedItem> = self
            .state
            .items
            .iter()
            .filter(|i| i.tags.iter().any(|t| t == "科学") && now - i.published_at < hour_ms * 36)
            .collect();
        if science_items.len() >= 2 {
            let start = science_items
                .iter()
                .map(|i| i.published_at)
                .min()
                .unwrap_or(0);
            let end = science_items
                .iter()
                .map(|i| i.published_at)
                .max()
                .unwrap_or(0);
            let key_events = science_items
                .iter()
                .take(5)
                .map(|i| i.title.clone())
                .collect();
            timelines.push(EventTimeline {
                id: "tl-science".to_string(),
                title: "科学前沿发现".to_string(),
                items: science_items.into_iter().cloned().collect(),
                start_time: start,
                end_time: end,
                key_events,
                neotrix_summary: Some("科学领域迎来多项突破：室温超导材料实现常压可重复验证，JWST 发现含水系外行星，CRISPR 新疗法在临床试验中取得圆满成功。".to_string()),
            });
        }

        // Cluster: "Live Events" — items with Live content type
        let live_items: Vec<&FeedItem> = self
            .state
            .items
            .iter()
            .filter(|i| matches!(i.content_type, MomentContentType::Live))
            .collect();
        if live_items.len() >= 2 {
            let start = live_items.iter().map(|i| i.published_at).min().unwrap_or(0);
            let end = live_items.iter().map(|i| i.published_at).max().unwrap_or(0);
            let key_events = live_items
                .iter()
                .map(|i| format!("直播中: {}", i.title))
                .collect();
            timelines.push(EventTimeline {
                id: "tl-live".to_string(),
                title: "正在直播".to_string(),
                items: live_items.into_iter().cloned().collect(),
                start_time: start,
                end_time: end,
                key_events,
                neotrix_summary: Some("当前有多个精彩直播正在进行：NASA 登月任务、Rust 编译器 roadmap 讨论和图灵奖颁奖典礼。".to_string()),
            });
        }

        timelines
    }

    fn update_tags(&mut self) {
        let mut tag_count: HashMap<String, usize> = HashMap::new();
        for item in &self.state.items {
            for tag in &item.tags {
                *tag_count.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        let mut tags: Vec<FeedTag> = tag_count
            .into_iter()
            .map(|(name, count)| FeedTag {
                name,
                count,
                is_active: false,
            })
            .collect();
        tags.sort_by(|a, b| b.count.cmp(&a.count));
        self.state.tags = tags;
    }

    /// Filter items by tag
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&FeedItem> {
        self.state
            .items
            .iter()
            .filter(|i| i.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Search items by title and description
    pub fn search(&self, query: &str) -> Vec<&FeedItem> {
        let q = query.to_lowercase();
        self.state
            .items
            .iter()
            .filter(|i| {
                i.title.to_lowercase().contains(&q) || i.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Get feed state
    pub fn state(&self) -> &FeedState {
        &self.state
    }

    /// Generate NeoTrix insight for a specific item
    pub fn generate_insight(&self, item: &FeedItem) -> String {
        let has_tech = item.tags.iter().any(|t| t == "科技" || t == "AI");
        let has_science = item.tags.iter().any(|t| t == "科学");
        let has_oss = item.tags.iter().any(|t| t == "开源");

        let mut parts: Vec<String> = Vec::new();

        if has_tech && item.score > 0.85 {
            parts.push(format!(
                "高热度技术动态 (score={:.2})：{} 代表了当前技术领域的重要趋势，值得深度关注。",
                item.score, item.title
            ));
        }

        if has_science {
            let science_note = match item.author.as_deref() {
                Some(a) => format!(
                    "该科学发现由 {} 团队发布，建议交叉验证其他研究机构的重复实验结果。",
                    a
                ),
                None => "该科学发现建议查阅原始论文以获取完整方法论细节。".to_string(),
            };
            parts.push(science_note);
        }

        if has_oss {
            parts.push(
                "该开源项目/社区动态反映了开发者生态的走向，建议关注其技术栈和社区活跃度。"
                    .to_string(),
            );
        }

        if parts.is_empty() {
            parts.push(format!("NeoTrix 分析：此内容 (score={:.2}) 在当前信息流中具有一定参考价值，建议结合相关话题阅读以获取全景视角。", item.score));
        }

        parts.join(" ")
    }

    /// Generate a NeoTrix prediction/summary for an event timeline
    pub fn summarize_timeline(&self, timeline: &EventTimeline) -> String {
        let item_count = timeline.items.len();
        let duration_hours = (timeline.end_time - timeline.start_time) as f64 / 3600000.0;

        let mut summary = format!(
            "NeoTrix 时间线分析：「{}」包含 {} 条相关内容，覆盖约 {:.1} 小时的时间跨度。\n",
            timeline.title, item_count, duration_hours
        );

        let avg_score: f64 =
            timeline.items.iter().map(|i| i.score).sum::<f64>() / item_count.max(1) as f64;
        summary.push_str(&format!("平均热度评分：{:.2}。", avg_score));

        if avg_score > 0.8 {
            summary.push_str(" 该话题处于高度活跃状态，建议持续追踪。");
        } else if avg_score > 0.6 {
            summary.push_str(" 该话题有一定关注度，可选择性关注。");
        } else {
            summary.push_str(" 该话题热度一般，适合背景阅读。");
        }

        summary
    }

    pub fn enable_personalization(&mut self, config: GRPOConfig) {
        self.grpo_trainer = Some(GRPOTrainer::new(config));
    }

    pub fn personalization_enabled(&self) -> bool {
        self.grpo_trainer.is_some()
    }

    pub fn record_interaction(&mut self, item_id: &str, liked: bool) {
        if let Some(ref mut trainer) = self.grpo_trainer {
            if let Some(item) = self.state.items.iter().find(|i| i.id == item_id) {
                let preference_seed = if liked { LIKE_SEED } else { DISLIKE_SEED };
                let preference_vsa = QuantizedVSA::seeded_random(preference_seed, VSA_DIM);
                let action = QuantizedVSA::bind(&item.vsa_vector, &preference_vsa);
                let reward = if liked { 1.0 } else { -1.0 };
                trainer.train_step(&item.vsa_vector, &[action], &[reward]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_new_creates_empty_state() {
        let feed = MomentFeed::new();
        assert!(feed.state().items.is_empty());
        assert!(feed.state().timelines.is_empty());
        assert_eq!(feed.state().total_count, 0);
    }

    #[test]
    fn test_feed_refresh_creates_items() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        assert!(!feed.state().items.is_empty());
        assert!(feed.state().total_count >= 20);
        assert!(feed.state().last_refresh > 0);
    }

    #[test]
    fn test_rank_items_orders_by_score() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        let items = &feed.state().items;
        for i in 1..items.len() {
            assert!(
                items[i - 1].score >= items[i].score,
                "items[{}].score ({}) < items[{}].score ({})",
                i - 1,
                items[i - 1].score,
                i,
                items[i].score
            );
        }
    }

    #[test]
    fn test_filter_by_tag() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        let ai_items = feed.filter_by_tag("AI");
        assert!(!ai_items.is_empty());
        for item in &ai_items {
            assert!(item.tags.iter().any(|t| t == "AI"));
        }
    }

    #[test]
    fn test_filter_by_tag_nonexistent() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        let items = feed.filter_by_tag("xyz_nonexistent");
        assert!(items.is_empty());
    }

    #[test]
    fn test_search_finds_matching_items() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        let results = feed.search("GPT");
        assert!(!results.is_empty());
        for item in &results {
            let found = item.title.contains("GPT") || item.description.contains("GPT");
            assert!(found);
        }
    }

    #[test]
    fn test_search_no_match_returns_empty() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        let results = feed.search("xyznonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_build_timelines_creates_clusters() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        let timelines = &feed.state().timelines;
        assert!(!timelines.is_empty());
        for tl in timelines {
            assert!(!tl.items.is_empty());
            assert!(!tl.key_events.is_empty());
            assert!(tl.start_time <= tl.end_time);
        }
    }

    #[test]
    fn test_generate_insight_returns_non_empty() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        if let Some(item) = feed.state().items.first() {
            let insight = feed.generate_insight(item);
            assert!(!insight.is_empty());
            assert!(insight.contains("score"));
        }
    }

    #[test]
    fn test_summarize_timeline() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        if let Some(tl) = feed.state().timelines.first() {
            let summary = feed.summarize_timeline(tl);
            assert!(summary.contains("NeoTrix"));
            assert!(summary.contains(tl.title.as_str()));
        }
    }

    #[test]
    fn test_items_have_vsa_vectors() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        for item in &feed.state().items {
            assert_eq!(
                item.vsa_vector.len(),
                VSA_DIM,
                "Item {} has wrong VSA vector length",
                item.id
            );
            assert!(
                item.vsa_vector.iter().any(|&b| b != 0),
                "VSA vector should not be all zeros"
            );
        }
    }

    #[test]
    fn test_tags_updated_after_refresh() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        assert!(!feed.state().tags.is_empty());
        // Tags should be sorted by count descending
        for i in 1..feed.state().tags.len() {
            assert!(feed.state().tags[i - 1].count >= feed.state().tags[i].count);
        }
    }

    #[test]
    fn test_moment_content_type_serialization() {
        let types = vec![
            MomentContentType::Article,
            MomentContentType::Image,
            MomentContentType::Video,
            MomentContentType::Live,
            MomentContentType::Social,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            let back: MomentContentType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, &back, "Mismatch after roundtrip for {:?}", t);
        }
    }

    #[test]
    fn test_search_is_case_insensitive() {
        let mut feed = MomentFeed::new();
        feed.refresh();
        let results_lower = feed.search("gpt");
        let results_upper = feed.search("GPT");
        assert!(!results_lower.is_empty());
        assert_eq!(results_lower.len(), results_upper.len());
    }

    #[test]
    fn test_personalization_off_by_default() {
        let feed = MomentFeed::new();
        assert!(!feed.personalization_enabled());
    }

    #[test]
    fn test_enable_personalization() {
        let mut feed = MomentFeed::new();
        feed.enable_personalization(GRPOConfig::default());
        assert!(feed.personalization_enabled());
    }

    #[test]
    fn test_record_interaction_liked() {
        let mut feed = MomentFeed::new();
        feed.enable_personalization(GRPOConfig::default());
        feed.refresh();
        let first_id = feed.state().items[0].id.clone();
        let score_before = feed.state().items[0].score;

        feed.record_interaction(&first_id, true);

        // Liked item should have slightly higher score after rerank
        feed.rank_items();
        let score_after = feed
            .state()
            .items
            .iter()
            .find(|i| i.id == first_id)
            .map(|i| i.score)
            .unwrap();
        assert!(score_after >= score_before - 1e-12);
    }

    #[test]
    fn test_record_interaction_disliked() {
        let mut feed = MomentFeed::new();
        feed.enable_personalization(GRPOConfig::default());
        feed.refresh();
        let first_id = feed.state().items[0].id.clone();
        let score_before = feed.state().items[0].score;

        feed.record_interaction(&first_id, false);

        feed.rank_items();
        let score_after = feed
            .state()
            .items
            .iter()
            .find(|i| i.id == first_id)
            .map(|i| i.score)
            .unwrap();
        assert!(score_after <= score_before + 1e-12);
    }

    #[test]
    fn test_rank_items_with_personalization() {
        let mut feed = MomentFeed::new();
        feed.enable_personalization(GRPOConfig::default());
        feed.refresh();

        let ids: Vec<String> = feed
            .state()
            .items
            .iter()
            .take(3)
            .map(|i| i.id.clone())
            .collect();
        feed.record_interaction(&ids[0], true);
        feed.record_interaction(&ids[1], false);

        feed.rank_items();
        let items = &feed.state().items;
        let pos0 = items.iter().position(|i| i.id == ids[0]).unwrap();
        let pos1 = items.iter().position(|i| i.id == ids[1]).unwrap();
        assert!(pos0 < items.len() && pos1 < items.len());
    }

    #[test]
    fn test_record_interaction_nonexistent() {
        let mut feed = MomentFeed::new();
        feed.enable_personalization(GRPOConfig::default());
        feed.refresh();
        feed.record_interaction("nonexistent-id", true);
        assert!(feed.personalization_enabled());
    }

    #[test]
    fn test_multiple_interactions() {
        let mut feed = MomentFeed::new();
        feed.enable_personalization(GRPOConfig::default());
        feed.refresh();

        let ids: Vec<String> = feed
            .state()
            .items
            .iter()
            .take(5)
            .map(|i| i.id.clone())
            .collect();
        for id in &ids {
            feed.record_interaction(id, true);
        }

        feed.rank_items();
        let scores: Vec<f64> = feed.state().items.iter().take(5).map(|i| i.score).collect();
        for score in &scores {
            assert!(*score > 0.0);
        }
    }

    #[test]
    fn test_personalization_persists_after_refresh() {
        let mut feed = MomentFeed::new();
        feed.enable_personalization(GRPOConfig::default());
        feed.refresh();

        let id = feed.state().items[0].id.clone();
        feed.record_interaction(&id, true);

        feed.refresh();
        assert!(feed.personalization_enabled());
    }
}
