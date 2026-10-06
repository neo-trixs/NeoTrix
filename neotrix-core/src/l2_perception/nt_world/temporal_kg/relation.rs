use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct TemporalRelation {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub valid_from: NaiveDateTime,
    pub valid_to: Option<NaiveDateTime>,
}

impl TemporalRelation {
    pub fn new(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relation_type: impl Into<String>,
        valid_from: NaiveDateTime,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            relation_type: relation_type.into(),
            valid_from,
            valid_to: None,
        }
    }

    pub fn with_valid_to(mut self, t: NaiveDateTime) -> Self {
        self.valid_to = Some(t);
        self
    }

    pub fn is_valid_at(&self, t: NaiveDateTime) -> bool {
        t >= self.valid_from && self.valid_to.map_or(true, |to| t <= to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(year: i32, month: u32, day: u32, hour: u32, min: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year,month,day)
            .and_then(|d| d.and_hms_opt(hour,min,0))
            .expect("测试时间戳构造：日期/时间应合法")
    }

    #[test]
    fn test_relation_creation() {
        let t0 = dt(2024, 1, 1, 0, 0);
        let r = TemporalRelation::new("e1", "e2", "works_at", t0);
        assert_eq!(r.source_id, "e1");
        assert_eq!(r.target_id, "e2");
        assert!(r.valid_to.is_none());
        assert!(r.is_valid_at(t0));
    }

    #[test]
    fn test_relation_temporal_validity() {
        let t0 = dt(2024, 1, 1, 0, 0);
        let t1 = dt(2024, 6, 15, 12, 0);
        let t2 = dt(2024, 12, 31, 23, 59);

        let r = TemporalRelation::new("e1", "e2", "employed_at", t0).with_valid_to(t1);

        assert!(r.is_valid_at(t0));
        assert!(r.is_valid_at(t1));
        assert!(!r.is_valid_at(t2));
    }

    #[test]
    fn test_open_ended_relation() {
        let t0 = dt(2024, 1, 1, 0, 0);
        let far_future = dt(2099, 12, 31, 23, 59);
        let r = TemporalRelation::new("e1", "e2", "knows", t0);
        assert!(r.is_valid_at(far_future));
    }

    #[test]
    fn test_valid_from_equals_valid_to() {
        let t0 = dt(2024, 6, 15, 12, 0);
        let r = TemporalRelation::new("a", "b", "rel", t0).with_valid_to(t0);
        assert!(r.is_valid_at(t0));
    }
}
