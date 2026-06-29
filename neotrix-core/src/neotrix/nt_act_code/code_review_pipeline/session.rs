use super::types::ReviewComment;

pub struct ReviewSession {
    pub id: String,
    pub comments: Vec<ReviewComment>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ReviewSession {
    pub fn new(comments: Vec<ReviewComment>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            comments,
            timestamp: chrono::Utc::now(),
        }
    }
}
