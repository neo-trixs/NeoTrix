#![forbid(unsafe_code)]

pub trait Configurable: Send + Sync + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn config_name(&self) -> &str;

    fn validate(&self) -> Result<(), String>;

    fn merge_defaults(&mut self, defaults: &Self);

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn from_json(json: &str) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_str(json)
    }
}
