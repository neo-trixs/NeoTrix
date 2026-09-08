pub mod etl;
pub mod quality;
pub mod lineage;

pub struct MediaETL;
impl MediaETL {
    pub fn new() -> Self {
        Self
    }
}

pub struct DataQualityChecker;
impl DataQualityChecker {
    pub fn new() -> Self {
        Self
    }
}

pub struct DataLineage;
impl DataLineage {
    pub fn new() -> Self {
        Self
    }
}
