use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolContract {
    pub name: String,
    pub version: String,
    pub input_schema: HashMap<String, String>,
    pub output_schema: HashMap<String, String>,
    pub required_permissions: Vec<String>,
    pub max_execution_ms: u64,
    pub cost_per_call: f64,
}

impl ToolContract {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            input_schema: HashMap::new(),
            output_schema: HashMap::new(),
            required_permissions: Vec::new(),
            max_execution_ms: 30000,
            cost_per_call: 0.0,
        }
    }

    pub fn with_input(mut self, key: &str, ty: &str) -> Self {
        self.input_schema.insert(key.to_string(), ty.to_string());
        self
    }

    pub fn with_output(mut self, key: &str, ty: &str) -> Self {
        self.output_schema.insert(key.to_string(), ty.to_string());
        self
    }

    pub fn with_permission(mut self, perm: &str) -> Self {
        self.required_permissions.push(perm.to_string());
        self
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.max_execution_ms = ms;
        self
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost_per_call = cost;
        self
    }

    pub fn validate_input(&self, input: &HashMap<String, String>) -> Result<(), String> {
        for key in self.input_schema.keys() {
            if !input.contains_key(key) {
                return Err(format!("Missing input: {}", key));
            }
        }
        Ok(())
    }

    pub fn validate_output(&self, output: &HashMap<String, String>) -> Result<(), String> {
        for key in self.output_schema.keys() {
            if !output.contains_key(key) {
                return Err(format!("Missing output: {}", key));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input() {
        let c = ToolContract::new("search", "1.0").with_input("query", "string");
        assert!(c
            .validate_input(&[("query".into(), "test".into())].into())
            .is_ok());
        assert!(c.validate_input(&HashMap::new()).is_err());
    }

    #[test]
    fn test_builder() {
        let c = ToolContract::new("t", "1.0")
            .with_input("a", "string")
            .with_output("b", "int")
            .with_permission("read")
            .with_timeout(5000)
            .with_cost(0.01);
        assert_eq!(c.name, "t");
        assert_eq!(c.required_permissions.len(), 1);
    }
}
