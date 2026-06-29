use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EconomicVariable {
    pub name: String,
    pub current_value: f64,
    pub previous_value: f64,
    pub growth_rate: f64,
    pub volatility: f64,
    pub history: VecDeque<f64>,
}

impl EconomicVariable {
    pub fn new(name: &str, initial: f64) -> Self {
        let mut history = VecDeque::with_capacity(100);
        history.push_back(initial);
        Self {
            name: name.into(),
            current_value: initial,
            previous_value: initial,
            growth_rate: 0.0,
            volatility: 0.0,
            history,
        }
    }

    pub fn update(&mut self, new_value: f64) {
        self.previous_value = self.current_value;
        self.current_value = new_value;
        if self.previous_value.abs() > 1e-9 {
            self.growth_rate =
                (self.current_value - self.previous_value) / self.previous_value.abs();
        }
        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back(new_value);
        if self.history.len() >= 10 {
            let recent: Vec<f64> = self.history.iter().rev().take(10).copied().collect();
            let mean = recent.iter().sum::<f64>() / recent.len() as f64;
            let variance =
                recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
            self.volatility = variance.sqrt();
        }
    }

    pub fn momentum(&self, n: usize) -> f64 {
        let n = n.min(self.history.len());
        if n < 2 {
            return 0.0;
        }
        let recent: Vec<&f64> = self.history.iter().rev().take(n).collect();
        let first = recent[recent.len() - 1];
        let last = recent[0];
        if first.abs() < 1e-9 {
            return 0.0;
        }
        (last - first) / first.abs()
    }
}

#[derive(Debug, Clone)]
pub struct EconomicWorldModel {
    pub gdp: EconomicVariable,
    pub inflation: EconomicVariable,
    pub interest_rate: EconomicVariable,
    pub unemployment: EconomicVariable,
    pub market_sentiment: EconomicVariable,
    pub crypto_dominance: EconomicVariable,
    pub vix: EconomicVariable,
    cycle: u64,
}

impl EconomicWorldModel {
    pub fn new() -> Self {
        Self {
            gdp: EconomicVariable::new("gdp", 100.0),
            inflation: EconomicVariable::new("inflation", 2.5),
            interest_rate: EconomicVariable::new("interest_rate", 5.0),
            unemployment: EconomicVariable::new("unemployment", 4.0),
            market_sentiment: EconomicVariable::new("market_sentiment", 0.5),
            crypto_dominance: EconomicVariable::new("crypto_dominance", 50.0),
            vix: EconomicVariable::new("vix", 15.0),
            cycle: 0,
        }
    }

    pub fn predict_regime(&self) -> &'static str {
        if self.vix.current_value > 30.0 {
            return "crisis";
        }
        if self.vix.current_value > 20.0 {
            return "volatile";
        }
        if self.market_sentiment.current_value > 0.7 && self.gdp.growth_rate > 0.02 {
            return "bull";
        }
        if self.market_sentiment.current_value < 0.3 || self.gdp.growth_rate < -0.01 {
            return "bear";
        }
        "neutral"
    }

    pub fn risk_appetite(&self) -> f64 {
        let base = self.market_sentiment.current_value;
        let vix_penalty = ((self.vix.current_value - 10.0) / 40.0).clamp(0.0, 1.0);
        let rate_penalty = (self.interest_rate.current_value / 10.0).clamp(0.0, 1.0);
        (base * (1.0 - vix_penalty) * (1.0 - rate_penalty * 0.3)).clamp(0.0, 1.0)
    }

    pub fn tick(&mut self) {
        self.cycle += 1;
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle
    }
}

impl Default for EconomicWorldModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economic_variable_update() {
        let mut var = EconomicVariable::new("test", 100.0);
        var.update(110.0);
        assert!((var.growth_rate - 0.1).abs() < 1e-6);
        assert_eq!(var.previous_value, 100.0);
        assert_eq!(var.current_value, 110.0);
    }

    #[test]
    fn test_economic_variable_momentum() {
        let mut var = EconomicVariable::new("test", 100.0);
        var.update(105.0);
        var.update(110.0);
        var.update(115.0);
        let m = var.momentum(3);
        assert!(m > 0.0);
    }

    #[test]
    fn test_world_model_regime_prediction() {
        let mut model = EconomicWorldModel::new();
        assert_eq!(model.predict_regime(), "neutral");
        model.vix.update(35.0);
        assert_eq!(model.predict_regime(), "crisis");
    }

    #[test]
    fn test_risk_appetite() {
        let model = EconomicWorldModel::new();
        let appetite = model.risk_appetite();
        assert!(appetite >= 0.0 && appetite <= 1.0);
    }

    #[test]
    fn test_world_model_tick() {
        let mut model = EconomicWorldModel::new();
        assert_eq!(model.cycle_count(), 0);
        model.tick();
        assert_eq!(model.cycle_count(), 1);
    }
}
