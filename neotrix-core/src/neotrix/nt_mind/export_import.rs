use super::ReasoningMemory;

pub struct ReasoningBankExporter;

impl ReasoningBankExporter {
    pub fn export_to_toml(_bank: &Vec<ReasoningMemory>) -> String {
        "# NeoTrix ReasoningBank Export\n".into()
    }
    pub fn import_from_toml(_data: &str) -> Vec<ReasoningMemory> {
        Vec::new()
    }
}
