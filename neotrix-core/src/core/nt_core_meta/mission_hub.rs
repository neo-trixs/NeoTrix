use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Mission {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub prd: String,
}

#[derive(Debug, Clone)]
pub struct MissionHub {
    pub missions: HashMap<u64, Mission>,
    pub next_id: u64,
}

impl MissionHub {
    pub fn new() -> Self {
        Self {
            missions: HashMap::new(),
            next_id: 1,
        }
    }
    pub fn create(&mut self, name: &str, prd: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.missions.insert(
            id,
            Mission {
                id,
                name: name.into(),
                status: "active".into(),
                prd: prd.into(),
            },
        );
        id
    }
    pub fn audit(&self, id: u64) -> Option<&Mission> {
        self.missions.get(&id)
    }
}
