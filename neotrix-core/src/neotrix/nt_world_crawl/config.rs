use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrawlTopic {
    LawAndGovernance,
    PolicyAndRegulation,
    ScienceAndTechnology,
    HumanitiesAndCulture,
    SocietyAndEconomics,
    HealthAndMedicine,
    EducationAndAcademia,
    NewsAndMedia,
    PhilosophyAndEthics,
    HistoryAndArcheology,
    ArtsAndLiterature,
    General,
}

impl CrawlTopic {
    pub fn name(&self) -> &'static str {
        match self {
            CrawlTopic::LawAndGovernance => "law_and_governance",
            CrawlTopic::PolicyAndRegulation => "policy_and_regulation",
            CrawlTopic::ScienceAndTechnology => "science_and_technology",
            CrawlTopic::HumanitiesAndCulture => "humanities_and_culture",
            CrawlTopic::SocietyAndEconomics => "society_and_economics",
            CrawlTopic::HealthAndMedicine => "health_and_medicine",
            CrawlTopic::EducationAndAcademia => "education_and_academia",
            CrawlTopic::NewsAndMedia => "news_and_media",
            CrawlTopic::PhilosophyAndEthics => "philosophy_and_ethics",
            CrawlTopic::HistoryAndArcheology => "history_and_archeology",
            CrawlTopic::ArtsAndLiterature => "arts_and_literature",
            CrawlTopic::General => "general",
        }
    }

    pub fn all() -> Vec<CrawlTopic> {
        vec![
            CrawlTopic::LawAndGovernance,
            CrawlTopic::PolicyAndRegulation,
            CrawlTopic::ScienceAndTechnology,
            CrawlTopic::HumanitiesAndCulture,
            CrawlTopic::SocietyAndEconomics,
            CrawlTopic::HealthAndMedicine,
            CrawlTopic::EducationAndAcademia,
            CrawlTopic::NewsAndMedia,
            CrawlTopic::PhilosophyAndEthics,
            CrawlTopic::HistoryAndArcheology,
            CrawlTopic::ArtsAndLiterature,
            CrawlTopic::General,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrawlFormat {
    LegalDocument,
    AcademicPaper,
    GovernmentPortal,
    NewsArticle,
    Encyclopedia,
    BlogPost,
    OfficialDocument,
    DiscussionForum,
    CodeRepository,
    ReferenceWork,
    Multimedia,
    Other,
}

impl CrawlFormat {
    pub fn name(&self) -> &'static str {
        match self {
            CrawlFormat::LegalDocument => "legal_document",
            CrawlFormat::AcademicPaper => "academic_paper",
            CrawlFormat::GovernmentPortal => "government_portal",
            CrawlFormat::NewsArticle => "news_article",
            CrawlFormat::Encyclopedia => "encyclopedia",
            CrawlFormat::BlogPost => "blog_post",
            CrawlFormat::OfficialDocument => "official_document",
            CrawlFormat::DiscussionForum => "discussion_forum",
            CrawlFormat::CodeRepository => "code_repository",
            CrawlFormat::ReferenceWork => "reference_work",
            CrawlFormat::Multimedia => "multimedia",
            CrawlFormat::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrawlStrategy {
    Polite,
    Balanced,
    Aggressive,
}

impl CrawlStrategy {
    pub fn delay_ms(&self) -> u64 {
        match self {
            CrawlStrategy::Polite => 2000,
            CrawlStrategy::Balanced => 500,
            CrawlStrategy::Aggressive => 100,
        }
    }

    pub fn max_concurrent(&self) -> usize {
        match self {
            CrawlStrategy::Polite => 2,
            CrawlStrategy::Balanced => 5,
            CrawlStrategy::Aggressive => 10,
        }
    }
}

pub struct CrawlerConfig {
    pub seed_urls: Vec<SeedEntry>,
    pub strategy: CrawlStrategy,
    pub max_pages_per_domain: usize,
    pub max_depth: u32,
    pub respect_robots_txt: bool,
    pub proxy_pool: Vec<String>,
    pub user_agent_rotation: bool,
    pub store_raw_content: bool,
    pub cycle_interval_secs: u64,
    pub self_heal_interval: u32,
    pub fetch_timeout_secs: u64,
    pub max_retries: u32,
    pub topic_weights: HashMap<CrawlTopic, f64>,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        let mut topic_weights = HashMap::new();
        topic_weights.insert(CrawlTopic::LawAndGovernance, 1.0);
        topic_weights.insert(CrawlTopic::PolicyAndRegulation, 0.9);
        topic_weights.insert(CrawlTopic::ScienceAndTechnology, 0.95);
        topic_weights.insert(CrawlTopic::PhilosophyAndEthics, 0.85);
        topic_weights.insert(CrawlTopic::HumanitiesAndCulture, 0.7);
        topic_weights.insert(CrawlTopic::SocietyAndEconomics, 0.7);
        topic_weights.insert(CrawlTopic::HealthAndMedicine, 0.6);
        topic_weights.insert(CrawlTopic::EducationAndAcademia, 0.6);
        topic_weights.insert(CrawlTopic::NewsAndMedia, 0.5);
        topic_weights.insert(CrawlTopic::HistoryAndArcheology, 0.5);
        topic_weights.insert(CrawlTopic::ArtsAndLiterature, 0.4);
        topic_weights.insert(CrawlTopic::General, 0.3);

        CrawlerConfig {
            seed_urls: default_seed_urls(),
            strategy: CrawlStrategy::Balanced,
            max_pages_per_domain: 100,
            max_depth: 3,
            respect_robots_txt: true,
            proxy_pool: vec![],
            user_agent_rotation: true,
            store_raw_content: false,
            cycle_interval_secs: 43200,
            self_heal_interval: 50,
            fetch_timeout_secs: 30,
            max_retries: 3,
            topic_weights,
        }
    }
}

pub struct SeedEntry {
    pub url: String,
    pub topic: CrawlTopic,
    pub depth: u32,
    pub enabled: bool,
}

pub fn default_seed_urls() -> Vec<SeedEntry> {
    vec![
        // === 现有种子 (保留)— 法律/治理 ===
        SeedEntry {
            url: "https://www.un.org/en/about-us/un-charter".into(),
            topic: CrawlTopic::LawAndGovernance,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.icj-cij.org/statute".into(),
            topic: CrawlTopic::LawAndGovernance,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.ohchr.org/en/instruments-listings".into(),
            topic: CrawlTopic::LawAndGovernance,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://european-union.europa.eu/principles-countries-history_en".into(),
            topic: CrawlTopic::PolicyAndRegulation,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.gov.uk/government/publications".into(),
            topic: CrawlTopic::PolicyAndRegulation,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.whitehouse.gov/briefing-room".into(),
            topic: CrawlTopic::PolicyAndRegulation,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.archives.gov/founding-docs".into(),
            topic: CrawlTopic::LawAndGovernance,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.wto.org/english/res_e/res_e.htm".into(),
            topic: CrawlTopic::PolicyAndRegulation,
            depth: 2,
            enabled: true,
        },
        // === 现有 — 哲学/科学 ===
        SeedEntry {
            url: "https://plato.stanford.edu".into(),
            topic: CrawlTopic::PhilosophyAndEthics,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Philosophy_of_law".into(),
            topic: CrawlTopic::PhilosophyAndEthics,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.nature.com".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.sciencedirect.com".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.britannica.com".into(),
            topic: CrawlTopic::General,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.worldhistory.org".into(),
            topic: CrawlTopic::HistoryAndArcheology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.oecd.org/publications".into(),
            topic: CrawlTopic::PolicyAndRegulation,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.imf.org/en/Publications".into(),
            topic: CrawlTopic::SocietyAndEconomics,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.who.int/publications".into(),
            topic: CrawlTopic::HealthAndMedicine,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.unesco.org/en/documents".into(),
            topic: CrawlTopic::EducationAndAcademia,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.constituteproject.org".into(),
            topic: CrawlTopic::LawAndGovernance,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.loc.gov/law/help/guide.php".into(),
            topic: CrawlTopic::LawAndGovernance,
            depth: 2,
            enabled: true,
        },
        // ========== 🆕 AI / ML / 意识科学 ==========
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Artificial_intelligence".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Machine_learning".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Deep_learning".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Integrated_information_theory".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Global_workspace_theory".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Free_energy_principle".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Active_inference".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Predictive_coding".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Hyperdimensional_computing".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 3,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Consciousness".into(),
            topic: CrawlTopic::PhilosophyAndEthics,
            depth: 3,
            enabled: true,
        },
        // ========== 🆕 安全 ==========
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Computer_nt_shield".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Vulnerability_(computing)".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://www.owasp.org".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://nvd.nist.gov".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        // ========== 🆕 数学/物理 ==========
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Category_theory".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Information_theory".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Quantum_mechanics".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Statistical_mechanics".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Complex_system".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Chaos_theory".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        // ========== 🆕 系统编程 / Rust ==========
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Rust_(programming_language)".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://doc.rust-lang.org/reference".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Compiler".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Concurrency_(computer_science)".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        // ========== 🆕 多维数据 ==========
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Graph_database".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Vector_database".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
        SeedEntry {
            url: "https://en.wikipedia.org/wiki/Knowledge_representation_and_reasoning".into(),
            topic: CrawlTopic::ScienceAndTechnology,
            depth: 2,
            enabled: true,
        },
    ]
}
