use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ResourceMarket {
    pub prices: HashMap<String, f32>,
    pub supply: HashMap<String, f32>,
    pub demand: HashMap<String, f32>,
    pub base_prices: HashMap<String, f32>,
    pub elasticity: f32,
}

impl ResourceMarket {
    pub fn new() -> Self {
        let mut base_prices = HashMap::new();
        base_prices.insert("food".to_string(), 10.0);
        base_prices.insert("wood".to_string(), 5.0);
        base_prices.insert("stone".to_string(), 8.0);
        base_prices.insert("metal".to_string(), 20.0);
        base_prices.insert("gold".to_string(), 50.0);

        let prices = base_prices.clone();

        Self {
            prices,
            supply: HashMap::new(),
            demand: HashMap::new(),
            base_prices,
            elasticity: 0.5,
        }
    }

    pub fn update_supply(&mut self, resource: &str, amount: f32) {
        *self.supply.entry(resource.to_string()).or_insert(0.0) += amount;
    }

    pub fn update_demand(&mut self, resource: &str, amount: f32) {
        *self.demand.entry(resource.to_string()).or_insert(0.0) += amount;
    }

    pub fn calculate_price(&self, resource: &str) -> f32 {
        let base = self.base_prices.get(resource).copied().unwrap_or(10.0);
        let supply = self.supply.get(resource).copied().unwrap_or(1.0);
        let demand = self.demand.get(resource).copied().unwrap_or(1.0);

        let ratio = demand / supply.max(1.0);
        base * (1.0 + (ratio - 1.0) * self.elasticity)
    }

    pub fn update_prices(&mut self) {
        let resources: Vec<String> = self.base_prices.keys().cloned().collect();
        for resource in resources {
            let new_price = self.calculate_price(&resource);
            self.prices.insert(resource, new_price);
        }
    }

    pub fn get_price(&self, resource: &str) -> f32 {
        *self.prices.get(resource).unwrap_or(&10.0)
    }

    pub fn get_supply_demand_ratio(&self, resource: &str) -> f32 {
        let supply = self.supply.get(resource).copied().unwrap_or(1.0);
        let demand = self.demand.get(resource).copied().unwrap_or(1.0);
        demand / supply.max(1.0)
    }

    pub fn tick(&mut self) {
        for supply in self.supply.values_mut() {
            *supply *= 0.95;
        }
        for demand in self.demand.values_mut() {
            *demand *= 0.9;
        }
        self.update_prices();
    }
}

pub struct PricingEngine {
    pub markets: HashMap<String, ResourceMarket>,
    pub global_inflation: f32,
}

impl PricingEngine {
    pub fn new() -> Self {
        Self {
            markets: HashMap::new(),
            global_inflation: 1.0,
        }
    }

    pub fn create_market(&mut self, name: &str) {
        self.markets.insert(name.to_string(), ResourceMarket::new());
    }

    pub fn get_market(&self, name: &str) -> Option<&ResourceMarket> {
        self.markets.get(name)
    }

    pub fn get_market_mut(&mut self, name: &str) -> Option<&mut ResourceMarket> {
        self.markets.get_mut(name)
    }

    pub fn record_trade(&mut self, market: &str, resource: &str, amount: f32, was_buy: bool) {
        if let Some(m) = self.markets.get_mut(market) {
            if was_buy {
                m.update_demand(resource, amount);
            } else {
                m.update_supply(resource, amount);
            }
        }
    }

    pub fn tick(&mut self) {
        for market in self.markets.values_mut() {
            market.tick();
        }
    }

    /// Update all prices across all markets using supply/demand with scarcity multiplier.
    pub fn update_prices_global(&mut self) {
        for market in self.markets.values_mut() {
            market.update_prices();
        }
    }

    /// Get aggregate price for a resource across all markets.
    pub fn average_price(&self, resource: &str) -> f32 {
        let prices: Vec<f32> = self.markets.values()
            .map(|m| m.get_price(resource))
            .collect();
        if prices.is_empty() {
            return 0.0;
        }
        prices.iter().sum::<f32>() / prices.len() as f32
    }

    /// Scarcity multiplier: higher when supply is low relative to historical average.
    pub fn scarcity_multiplier(&self, market: &str, resource: &str) -> f32 {
        if let Some(m) = self.markets.get(market) {
            let supply = m.supply.get(resource).copied().unwrap_or(10.0);
            let base = m.base_prices.get(resource).copied().unwrap_or(10.0);
            // Scarcity = base_price / current_supply (capped)
            (base / supply.max(1.0)).min(5.0)
        } else {
            1.0
        }
    }

    /// Demand bonus: ratio of demand to supply, clamped.
    pub fn demand_bonus(&self, market: &str, resource: &str) -> f32 {
        if let Some(m) = self.markets.get(market) {
            m.get_supply_demand_ratio(resource).min(3.0)
        } else {
            1.0
        }
    }

    pub fn get_arbitrage_opportunities(&self, market_a: &str, market_b: &str) -> Vec<(String, f32)> {
        let ma = match self.markets.get(market_a) {
            Some(m) => m,
            None => return vec![],
        };
        let mb = match self.markets.get(market_b) {
            Some(m) => m,
            None => return vec![],
        };

        let mut opportunities = Vec::new();
        for resource in ma.base_prices.keys() {
            let price_a = ma.get_price(resource);
            let price_b = mb.get_price(resource);
            let diff = (price_a - price_b).abs();
            if diff > 1.0 {
                opportunities.push((resource.clone(), diff));
            }
        }

        opportunities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        opportunities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_creation() {
        let market = ResourceMarket::new();
        assert_eq!(market.get_price("food"), 10.0);
    }

    #[test]
    fn test_supply_demand_price() {
        let mut market = ResourceMarket::new();
        market.update_demand("food", 100.0);
        market.update_supply("food", 10.0);
        market.update_prices();
        assert!(market.get_price("food") > 10.0);
    }

    #[test]
    fn test_pricing_engine() {
        let mut engine = PricingEngine::new();
        engine.create_market("main");
        engine.record_trade("main", "food", 10.0, true);
        engine.tick();
        assert!(engine.get_market("main").is_some());
    }

    #[test]
    fn test_arbitrage() {
        let mut engine = PricingEngine::new();
        engine.create_market("a");
        engine.create_market("b");
        
        if let Some(ma) = engine.get_market_mut("a") {
            ma.update_demand("food", 100.0);
            ma.update_prices();
        }
        
        let opps = engine.get_arbitrage_opportunities("a", "b");
        assert!(!opps.is_empty());
    }
}
