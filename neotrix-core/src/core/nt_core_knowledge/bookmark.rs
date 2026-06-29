use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// 书签分类体系 — 层级类别，用于组织对话中用户提供的分析URL
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BookmarkCategory {
    /// 论文/学术研究
    Research,
    /// 工具/库/框架
    Tool,
    /// 技术参考/文档
    Reference,
    /// 文章/博客/分析
    Article,
    /// GitHub项目/开源项目
    Project,
    /// 社交媒体（Twitter/Reddit等）
    Social,
    /// 视频/多媒体
    Video,
    /// 新闻/时事
    News,
    /// 产品/服务
    Product,
    /// 其他（用户自定义）
    Other(String),
}

impl BookmarkCategory {
    pub fn name(&self) -> String {
        match self {
            BookmarkCategory::Research => "research".into(),
            BookmarkCategory::Tool => "tool".into(),
            BookmarkCategory::Reference => "reference".into(),
            BookmarkCategory::Article => "article".into(),
            BookmarkCategory::Project => "project".into(),
            BookmarkCategory::Social => "social".into(),
            BookmarkCategory::Video => "video".into(),
            BookmarkCategory::News => "news".into(),
            BookmarkCategory::Product => "product".into(),
            BookmarkCategory::Other(s) => format!("other:{}", s),
        }
    }

    /// 根据URL域名自动推断类别
    pub fn infer_from_url(url: &str) -> Self {
        let lower = url.to_lowercase();
        if lower.contains("arxiv.org") || lower.contains("scholar.google")
            || lower.contains("semanticscholar.org") || lower.contains("researchgate.net")
            || lower.contains("aclweb.org") || lower.contains("openreview.net")
            || lower.contains("paperswithcode") || lower.contains("dblp.org")
        {
            return BookmarkCategory::Research;
        }
        if lower.contains("github.com") || lower.contains("gitlab.com") || lower.contains("crates.io") {
            return BookmarkCategory::Project;
        }
        if lower.contains("youtube.com") || lower.contains("youtu.be")
            || lower.contains("bilibili.com") || lower.contains("vimeo.com")
            || lower.contains("twitch.tv") || lower.contains("tiktok.com")
        {
            return BookmarkCategory::Video;
        }
        if lower.contains("reddit.com") || lower.contains("twitter.com")
            || lower.contains("x.com") || lower.contains("discord")
            || lower.contains("medium.com") || lower.contains("zhihu.com")
        {
            return BookmarkCategory::Social;
        }
        if lower.contains("news.") || lower.contains("reuters.com")
            || lower.contains("bloomberg.com") || lower.contains("bbc.com")
            || lower.contains("cnn.com") || lower.contains("nytimes.com")
            || lower.contains("theguardian.com") || lower.contains("ft.com")
            || lower.contains("wsj.com") || lower.contains("nikkei.com")
            || lower.contains("caixin.com") || lower.contains("36kr.com")
        {
            return BookmarkCategory::News;
        }
        if lower.contains("docs.") || lower.contains(".md")
            || lower.contains("manual") || lower.contains("wiki")
            || lower.contains("tutorial") || lower.contains("guide")
        {
            return BookmarkCategory::Reference;
        }
        if lower.contains("npmjs.com") || lower.contains("pypi.org")
            || lower.contains("hub.docker.com") || lower.contains("marketplace.")
            || lower.contains("app.") || lower.contains("product")
            || lower.contains("pricing") || lower.contains("get")
        {
            return BookmarkCategory::Product;
        }
        BookmarkCategory::Article
    }

    pub fn all_default() -> Vec<Self> {
        vec![
            BookmarkCategory::Research,
            BookmarkCategory::Tool,
            BookmarkCategory::Reference,
            BookmarkCategory::Article,
            BookmarkCategory::Project,
            BookmarkCategory::Social,
            BookmarkCategory::Video,
            BookmarkCategory::News,
            BookmarkCategory::Product,
        ]
    }
}

/// 书签条目 — 用户从对话中提供的URL及其分析上下文
#[derive(Debug, Clone)]
pub struct BookmarkEntry {
    /// 唯一ID (UUID)
    pub id: String,
    /// URL链接
    pub url: String,
    /// 页面标题（如果已知）
    pub title: String,
    /// 用户描述/为什么分析这个
    pub description: String,
    /// 类别
    pub category: BookmarkCategory,
    /// 标签
    pub tags: Vec<String>,
    /// 重要度 0.0-1.0
    pub importance: f64,
    /// 对话上下文摘要 — 用户当时为什么给这个链接
    pub context: String,
    /// 分析结果摘要
    pub analysis_summary: String,
    /// 关联的EvidenceRecord IDs（用于溯源分析结果）
    pub evidence_ids: Vec<u64>,
    /// 访问/查看次数
    pub visit_count: u64,
    /// 最后访问cycle
    pub last_visited_cycle: u64,
    /// 创建时间戳 (unix ms)
    pub created_at: i64,
    /// 最后更新时间戳
    pub updated_at: i64,
    /// 是否已归档
    pub is_archived: bool,
}

impl BookmarkEntry {
    pub fn new(url: &str, title: &str, description: &str, context: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let category = BookmarkCategory::infer_from_url(url);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category,
            tags: Vec::new(),
            importance: 0.5,
            context: context.to_string(),
            analysis_summary: String::new(),
            evidence_ids: Vec::new(),
            visit_count: 0,
            last_visited_cycle: 0,
            created_at: now,
            updated_at: now,
            is_archived: false,
        }
    }
}

/// 书签管理器 — 存储、分类、检索对话中的URL
#[derive(Debug, Clone)]
pub struct BookmarkManager {
    /// 所有书签 id → entry
    bookmarks: HashMap<String, BookmarkEntry>,
    /// 按URL去重索引 url → id
    url_index: HashMap<String, String>,
    /// 按类别的索引 category → [id]
    category_index: HashMap<String, Vec<String>>,
    /// 按标签的索引 tag → [id]
    tag_index: HashMap<String, Vec<String>>,
    /// 最大容量
    max_capacity: usize,
    /// 最近添加的顺序（用于LRU淘汰）
    access_order: VecDeque<String>,
    /// 统计
    pub stats: BookmarkStats,
}

/// 书签统计
#[derive(Debug, Clone, Default)]
pub struct BookmarkStats {
    pub total_count: usize,
    pub archived_count: usize,
    pub per_category: HashMap<String, usize>,
    pub total_visits: u64,
    pub last_added_cycle: u64,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self {
            bookmarks: HashMap::new(),
            url_index: HashMap::new(),
            category_index: HashMap::new(),
            tag_index: HashMap::new(),
            max_capacity: 500,
            access_order: VecDeque::new(),
            stats: BookmarkStats::default(),
        }
    }

    pub fn with_capacity(max: usize) -> Self {
        Self {
            max_capacity: max,
            ..Self::new()
        }
    }

    /// 添加书签 — 如果URL已存在则更新，否则新增
    pub fn add(&mut self, entry: BookmarkEntry, cycle: u64) -> String {
        // 去重：如果URL已存在，更新而非新增
        if let Some(existing_id) = self.url_index.get(&entry.url).cloned() {
            if let Some(existing) = self.bookmarks.get_mut(&existing_id) {
                existing.title = entry.title;
                existing.description = entry.description;
                existing.context = entry.context;
                existing.importance = entry.importance.max(existing.importance);
                existing.updated_at = entry.updated_at;
                existing.tags = {
                    let mut merged = existing.tags.clone();
                    for t in &entry.tags {
                        if !merged.contains(t) {
                            merged.push(t.clone());
                        }
                    }
                    merged
                };
                existing.is_archived = false;
            }
            return existing_id;
        }

        // 容量检查 + LRU淘汰
        if self.bookmarks.len() >= self.max_capacity {
            self.evict_one();
        }

        let id = entry.id.clone();
        let cat_name = entry.category.name();
        let url = entry.url.clone();
        let tags = entry.tags.clone();

        self.bookmarks.insert(id.clone(), entry);
        self.url_index.insert(url, id.clone());
        self.access_order.push_back(id.clone());

        // 类别索引
        self.category_index
            .entry(cat_name.clone())
            .or_default()
            .push(id.clone());

        // 标签索引
        for t in &tags {
            self.tag_index.entry(t.clone()).or_default().push(id.clone());
        }

        self.stats.total_count = self.bookmarks.len();
        *self.stats.per_category.entry(cat_name).or_insert(0) += 1;
        self.stats.last_added_cycle = cycle;

        id
    }

    /// 从对话文本中提取URL并创建书签
    pub fn add_from_conversation(
        &mut self,
        url: &str,
        title: &str,
        description: &str,
        context: &str,
        tags: Vec<String>,
        cycle: u64,
    ) -> Option<String> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return None;
        }
        let mut entry = BookmarkEntry::new(url, title, description, context);
        entry.tags = tags;
        let id = self.add(entry, cycle).to_string();
        Some(id)
    }

    /// 按类别获取书签
    pub fn by_category(&self, category: &BookmarkCategory) -> Vec<&BookmarkEntry> {
        let name = category.name();
        self.category_index
            .get(&name)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.bookmarks.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 按标签搜索
    pub fn by_tag(&self, tag: &str) -> Vec<&BookmarkEntry> {
        self.tag_index
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.bookmarks.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 全文搜索（URL/标题/描述/上下文）
    pub fn search(&self, query: &str) -> Vec<&BookmarkEntry> {
        let q = query.to_lowercase();
        let mut results: Vec<&BookmarkEntry> = self
            .bookmarks
            .values()
            .filter(|e| {
                e.url.to_lowercase().contains(&q)
                    || e.title.to_lowercase().contains(&q)
                    || e.description.to_lowercase().contains(&q)
                    || e.context.to_lowercase().contains(&q)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect();
        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// 重新分类书签
    pub fn recategorize(&mut self, id: &str, new_cat: BookmarkCategory) -> bool {
        if let Some(entry) = self.bookmarks.get_mut(id) {
            let old_cat = entry.category.name();
            entry.category = new_cat;
            let new_cat_name = entry.category.name();
            entry.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;

            // 更新类别索引
            if let Some(ids) = self.category_index.get_mut(&old_cat) {
                ids.retain(|i| i != id);
            }
            self.category_index
                .entry(new_cat_name.clone())
                .or_default()
                .push(id.to_string());

            *self.stats.per_category.entry(old_cat).or_insert(0) =
                self.stats.per_category.get(&old_cat).copied().unwrap_or(1).saturating_sub(1);
            *self.stats.per_category.entry(new_cat_name).or_insert(0) += 1;
            true
        } else {
            false
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, id: &str, tag: &str) -> bool {
        if let Some(entry) = self.bookmarks.get_mut(id) {
            if !entry.tags.contains(&tag.to_string()) {
                entry.tags.push(tag.to_string());
                self.tag_index
                    .entry(tag.to_string())
                    .or_default()
                    .push(id.to_string());
            }
            true
        } else {
            false
        }
    }

    /// 标记为已访问
    pub fn mark_visited(&mut self, id: &str, cycle: u64) {
        if let Some(entry) = self.bookmarks.get_mut(id) {
            entry.visit_count += 1;
            entry.last_visited_cycle = cycle;
            self.stats.total_visits += 1;
        }
    }

    /// 更新分析结果摘要
    pub fn update_analysis(&mut self, id: &str, summary: &str) -> bool {
        if let Some(entry) = self.bookmarks.get_mut(id) {
            entry.analysis_summary = summary.to_string();
            entry.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            true
        } else {
            false
        }
    }

    /// 归档书签
    pub fn archive(&mut self, id: &str) -> bool {
        if let Some(entry) = self.bookmarks.get_mut(id) {
            entry.is_archived = true;
            self.stats.archived_count += 1;
            true
        } else {
            false
        }
    }

    /// 获取所有书签（按类别分组）
    pub fn all_grouped(&self) -> HashMap<String, Vec<&BookmarkEntry>> {
        let mut grouped = HashMap::new();
        for entry in self.bookmarks.values() {
            let cat = entry.category.name();
            grouped.entry(cat).or_insert_with(Vec::new).push(entry);
        }
        grouped
    }

    /// 获取最近添加的N个书签
    pub fn recent(&self, n: usize) -> Vec<&BookmarkEntry> {
        self.access_order
            .iter()
            .rev()
            .filter_map(|id| self.bookmarks.get(id))
            .take(n)
            .collect()
    }

    /// 获取所有未归档的书签
    pub fn active_bookmarks(&self) -> Vec<&BookmarkEntry> {
        self.bookmarks
            .values()
            .filter(|e| !e.is_archived)
            .collect()
    }

    /// 通过ID获取
    pub fn get(&self, id: &str) -> Option<&BookmarkEntry> {
        self.bookmarks.get(id)
    }

    /// 通过URL获取
    pub fn get_by_url(&self, url: &str) -> Option<&BookmarkEntry> {
        self.url_index
            .get(url)
            .and_then(|id| self.bookmarks.get(id))
    }

    /// 统计快照（用于自进化任务检测）
    pub fn stale_bookmarks(&self, current_cycle: u64, threshold: u64) -> Vec<&BookmarkEntry> {
        self.bookmarks
            .values()
            .filter(|e| {
                !e.is_archived
                    && e.last_visited_cycle > 0
                    && current_cycle.saturating_sub(e.last_visited_cycle) > threshold
            })
            .collect()
    }

    /// 所有类别分布
    pub fn category_distribution(&self) -> Vec<(&str, usize)> {
        let mut dist: Vec<(&str, usize)> = self
            .stats
            .per_category
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        dist.sort_by(|a, b| b.1.cmp(&a.1));
        dist
    }

    fn evict_one(&mut self) {
        while let Some(lru_id) = self.access_order.pop_front() {
            if let Some(entry) = self.bookmarks.remove(&lru_id) {
                self.url_index.remove(&entry.url);
                let cat = entry.category.name();
                if let Some(ids) = self.category_index.get_mut(&cat) {
                    ids.retain(|i| i != &lru_id);
                }
                for t in &entry.tags {
                    if let Some(ids) = self.tag_index.get_mut(t) {
                        ids.retain(|i| i != &lru_id);
                    }
                }
                self.stats.archived_count = self.stats.archived_count.saturating_sub(1);
                break;
            }
        }
    }

    /// 提升到KnowledgeEntry
    pub fn promote_to_knowledge_entry(&self, id: &str) -> Option<crate::core::nt_core_knowledge::types::KnowledgeEntry> {
        let entry = self.bookmarks.get(id)?;
        let mut ke = crate::core::nt_core_knowledge::types::KnowledgeEntry::new(
            &entry.title,
            &format!("{}\n\n{}", entry.description, entry.analysis_summary),
            crate::core::nt_core_knowledge::types::KnowledgeSourceType::Bookmark,
            &entry.url,
        );
        ke.tags = entry.tags.clone();
        ke.evidence_ids = entry.evidence_ids.clone();
        ke.importance = entry.importance;
        Some(ke)
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut bm = BookmarkManager::new();
        let id = bm.add_from_conversation(
            "https://arxiv.org/abs/2605.22721",
            "Self-Evolving MAS Paper",
            "关于去中心化记忆的论文",
            "用户在研究自进化Agent",
            vec!["mas".into(), "memory".into()],
            1,
        );
        assert!(id.is_some());
        let entry = bm.get(&id.unwrap());
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().category, BookmarkCategory::Research);
    }

    #[test]
    fn test_auto_categorize_arxiv() {
        let mut bm = BookmarkManager::new();
        let id = bm.add_from_conversation(
            "https://arxiv.org/abs/2504.08912",
            "Multi-Head Resonator",
            "",
            "",
            vec![],
            1,
        );
        let entry = bm.get(&id.unwrap()).unwrap();
        assert_eq!(entry.category, BookmarkCategory::Research);
    }

    #[test]
    fn test_auto_categorize_github() {
        let mut bm = BookmarkManager::new();
        let id = bm.add_from_conversation(
            "https://github.com/automataIA/graphrag-rs",
            "GraphRAG-rs",
            "",
            "",
            vec![],
            1,
        );
        let entry = bm.get(&id.unwrap()).unwrap();
        assert_eq!(entry.category, BookmarkCategory::Project);
    }

    #[test]
    fn test_auto_categorize_youtube() {
        let cat = BookmarkCategory::infer_from_url("https://youtube.com/watch?v=abc123");
        assert_eq!(cat, BookmarkCategory::Video);
    }

    #[test]
    fn test_auto_categorize_news() {
        let cat = BookmarkCategory::infer_from_url("https://reuters.com/article/tech-ai");
        assert_eq!(cat, BookmarkCategory::News);
    }

    #[test]
    fn test_dedup_by_url() {
        let mut bm = BookmarkManager::new();
        let id1 = bm.add_from_conversation(
            "https://github.com/neotrix/test",
            "Test Repo",
            "original",
            "",
            vec![],
            1,
        );
        let id2 = bm.add_from_conversation(
            "https://github.com/neotrix/test",
            "Test Repo Updated",
            "updated",
            "",
            vec![],
            2,
        );
        // 相同URL应该返回相同ID
        assert_eq!(id1, id2);
        // 验证更新后的描述
        let entry = bm.get(&id1.unwrap()).unwrap();
        assert_eq!(entry.description, "updated");
    }

    #[test]
    fn test_search() {
        let mut bm = BookmarkManager::new();
        bm.add_from_conversation(
            "https://arxiv.org/abs/2605.22721",
            "Self-Evolving MAS",
            "multi-agent memory paper",
            "test",
            vec!["mas".into()],
            1,
        );
        bm.add_from_conversation(
            "https://github.com/rust-lang/rust",
            "Rust Language",
            "systems programming",
            "test",
            vec!["rust".into()],
            1,
        );
        let results = bm.search("rust");
        assert_eq!(results.len(), 1);
        let results2 = bm.search("memory");
        assert_eq!(results2.len(), 1);
    }

    #[test]
    fn test_by_category() {
        let mut bm = BookmarkManager::new();
        bm.add_from_conversation(
            "https://arxiv.org/abs/2504.08912",
            "Paper",
            "",
            "",
            vec![],
            1,
        );
        bm.add_from_conversation(
            "https://github.com/neotrix",
            "Repo",
            "",
            "",
            vec![],
            1,
        );
        assert_eq!(bm.by_category(&BookmarkCategory::Research).len(), 1);
        assert_eq!(bm.by_category(&BookmarkCategory::Project).len(), 1);
    }

    #[test]
    fn test_recategorize() {
        let mut bm = BookmarkManager::new();
        let id = bm.add_from_conversation(
            "https://example.com/paper",
            "Some Paper",
            "",
            "",
            vec![],
            1,
        );
        let id = id.unwrap();
        assert_eq!(bm.get(&id).unwrap().category, BookmarkCategory::Article);
        bm.recategorize(&id, BookmarkCategory::Research);
        assert_eq!(bm.get(&id).unwrap().category, BookmarkCategory::Research);
    }

    #[test]
    fn test_lru_eviction() {
        let mut bm = BookmarkManager::with_capacity(3);
        for i in 0..5 {
            bm.add_from_conversation(
                &format!("https://example.com/{}", i),
                &format!("Entry {}", i),
                "",
                "",
                vec![],
                i,
            );
        }
        // 5个添加，容量3，应淘汰最早2个
        assert_eq!(bm.bookmarks.len(), 3);
        assert!(bm.get_by_url("https://example.com/0").is_none());
        assert!(bm.get_by_url("https://example.com/1").is_none());
        assert!(bm.get_by_url("https://example.com/4").is_some());
    }

    #[test]
    fn test_stale_bookmarks() {
        let mut bm = BookmarkManager::new();
        let id = bm.add_from_conversation(
            "https://example.com/stale",
            "Stale",
            "",
            "",
            vec![],
            10,
        );
        let id = id.unwrap();
        bm.mark_visited(&id, 10);
        let stale = bm.stale_bookmarks(110, 50);
        assert_eq!(stale.len(), 1);
    }

    #[test]
    fn test_add_invalid_url() {
        let mut bm = BookmarkManager::new();
        let id = bm.add_from_conversation(
            "not a url",
            "Invalid",
            "",
            "",
            vec![],
            1,
        );
        assert!(id.is_none());
    }

    #[test]
    fn test_promote_to_knowledge() {
        let mut bm = BookmarkManager::new();
        let id = bm.add_from_conversation(
            "https://arxiv.org/abs/2605.22721",
            "Test Paper",
            "A test paper about MAS",
            "conversation context",
            vec!["mas".into()],
            1,
        );
        let id = id.unwrap();
        bm.update_analysis(&id, "Key finding: decentralized memory improves MAS by 15%");
        let ke = bm.promote_to_knowledge_entry(&id);
        assert!(ke.is_some());
        let ke = ke.unwrap();
        assert_eq!(ke.source_url, "https://arxiv.org/abs/2605.22721");
        assert!(ke.body.contains("Key finding"));
    }

    #[test]
    fn test_category_distribution() {
        let mut bm = BookmarkManager::new();
        bm.add_from_conversation(
            "https://arxiv.org/abs/2504.08912",
            "A",
            "",
            "",
            vec![],
            1,
        );
        bm.add_from_conversation(
            "https://arxiv.org/abs/2605.22721",
            "B",
            "",
            "",
            vec![],
            1,
        );
        bm.add_from_conversation(
            "https://github.com/neotrix",
            "C",
            "",
            "",
            vec![],
            1,
        );
        let dist = bm.category_distribution();
        assert_eq!(dist.iter().find(|(k, _)| *k == "research").unwrap().1, 2);
        assert_eq!(dist.iter().find(|(k, _)| *k == "project").unwrap().1, 1);
    }

    #[test]
    fn test_recent() {
        let mut bm = BookmarkManager::new();
        for i in 0..10 {
            bm.add_from_conversation(
                &format!("https://example.com/{}", i),
                &format!("Entry {}", i),
                "",
                "",
                vec![],
                i,
            );
        }
        let recent = bm.recent(3);
        assert_eq!(recent.len(), 3);
        assert!(recent[0].url.contains("9"));
    }
}
