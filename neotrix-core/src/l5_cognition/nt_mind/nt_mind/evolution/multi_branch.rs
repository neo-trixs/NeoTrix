#[derive(Debug, Clone)]
pub struct Branch {
    pub id: String,
    pub name: String,
    pub score: f64,
    pub archived: bool,
}

pub struct MultiBranchArchive {
    branches: Vec<Branch>,
    next_id: u32,
}

impl MultiBranchArchive {
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            next_id: 0,
        }
    }

    pub fn create(&mut self, name: &str) -> String {
        let id = format!("b{}", self.next_id);
        self.next_id += 1;
        self.branches.push(Branch {
            id: id.clone(),
            name: name.to_string(),
            score: 0.0,
            archived: false,
        });
        id
    }

    pub fn update_score(&mut self, id: &str, score: f64) -> bool {
        if let Some(b) = self.branches.iter_mut().find(|b| b.id == id) {
            b.score = score;
            true
        } else {
            false
        }
    }

    pub fn best(&self) -> Option<&Branch> {
        self.branches.iter().filter(|b| !b.archived).max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn archive(&mut self, id: &str) -> bool {
        if let Some(b) = self.branches.iter_mut().find(|b| b.id == id) {
            b.archived = true;
            true
        } else {
            false
        }
    }

    pub fn active_count(&self) -> usize {
        self.branches.iter().filter(|b| !b.archived).count()
    }
}

impl Default for MultiBranchArchive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_best() {
        let mut a = MultiBranchArchive::new();
        let id = a.create("exp1");
        a.update_score(&id, 0.8);
        assert_eq!(a.best().unwrap().id, id);
    }

    #[test]
    fn test_archive() {
        let mut a = MultiBranchArchive::new();
        let id = a.create("b");
        a.archive(&id);
        assert_eq!(a.active_count(), 0);
    }
}
