#[derive(Debug, Clone)]
pub struct IronLaw {
    pub id: u32,
    pub description: String,
    pub enforced: bool,
}

#[derive(Debug, Clone)]
pub struct IronLaws {
    pub laws: Vec<IronLaw>,
}

impl IronLaws {
    pub fn new() -> Self {
        Self {
            laws: vec![
                IronLaw {
                    id: 1,
                    description: "Database preservation".into(),
                    enforced: true,
                },
                IronLaw {
                    id: 2,
                    description: "Parent process protection".into(),
                    enforced: true,
                },
                IronLaw {
                    id: 3,
                    description: "Read-only runtime config".into(),
                    enforced: true,
                },
                IronLaw {
                    id: 4,
                    description: "Port isolation".into(),
                    enforced: true,
                },
            ],
        }
    }
    pub fn check(&self, law_id: u32) -> bool {
        self.laws.iter().any(|l| l.id == law_id && l.enforced)
    }
}
