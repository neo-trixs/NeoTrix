/// An action encoded for the world model.
///
/// Actions are typed (e.g. "move", "rotate", "push") and carry
/// a variable-length parameter vector for continuous control.
#[derive(Debug, Clone)]
pub struct ActionEmbedding {
    /// Action type identifier (e.g. "move", "rotate", "push")
    pub action_type: String,
    /// Continuous parameters for the action (e.g. [dx, dy, dz])
    pub parameters: Vec<f64>,
}

impl ActionEmbedding {
    /// Create a new action embedding.
    pub fn new(action_type: impl Into<String>, parameters: Vec<f64>) -> Self {
        Self {
            action_type: action_type.into(),
            parameters,
        }
    }

    /// Dimensionality of the parameter vector.
    pub fn param_dim(&self) -> usize {
        self.parameters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_creation() {
        let a = ActionEmbedding::new("move", vec![1.0, 0.0, 0.0]);
        assert_eq!(a.action_type, "move");
        assert_eq!(a.parameters, vec![1.0, 0.0, 0.0]);
        assert_eq!(a.param_dim(), 3);
    }
}
