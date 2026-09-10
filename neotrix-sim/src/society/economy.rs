// Economy - Resource-based economy for agent society
// Agents trade, harvest, consume, and compete for resources

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Wood,
    Stone,
    Water,
    Energy,
    Knowledge,
    Tools,
    Luxury,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: HashMap<ResourceType, f32>,
}

impl Inventory {
    pub fn new() -> Self { Self { items: HashMap::new() } }

    pub fn add(&mut self, resource: ResourceType, amount: f32) {
        *self.items.entry(resource).or_insert(0.0) += amount;
    }

    pub fn remove(&mut self, resource: ResourceType, amount: f32) -> bool {
        if let Some(current) = self.items.get_mut(&resource) {
            if *current >= amount {
                *current -= amount;
                return true;
            }
        }
        false
    }

    pub fn get(&self, resource: &ResourceType) -> f32 {
        self.items.get(resource).copied().unwrap_or(0.0)
    }

    pub fn total_value(&self) -> f32 {
        self.items.iter().map(|(r, &amount)| {
            amount * Self::unit_price(r)
        }).sum()
    }

    fn unit_price(resource: &ResourceType) -> f32 {
        match resource {
            ResourceType::Food => 1.0,
            ResourceType::Water => 1.2,
            ResourceType::Wood => 1.5,
            ResourceType::Stone => 2.0,
            ResourceType::Energy => 3.0,
            ResourceType::Knowledge => 5.0,
            ResourceType::Tools => 4.0,
            ResourceType::Luxury => 8.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOffer {
    pub from: String,
    pub to: String,
    pub offer: ResourceType,
    pub offer_amount: f32,
    pub want: ResourceType,
    pub want_amount: f32,
}

pub struct Economy {
    pub market_prices: HashMap<ResourceType, f32>,
    pub trade_log: Vec<TradeOffer>,
    pub total_trades: u64,
}

impl Economy {
    pub fn new() -> Self {
        let mut market_prices = HashMap::new();
        market_prices.insert(ResourceType::Food, 1.0);
        market_prices.insert(ResourceType::Water, 1.2);
        market_prices.insert(ResourceType::Wood, 1.5);
        market_prices.insert(ResourceType::Stone, 2.0);
        market_prices.insert(ResourceType::Energy, 3.0);
        market_prices.insert(ResourceType::Knowledge, 5.0);
        market_prices.insert(ResourceType::Tools, 4.0);
        market_prices.insert(ResourceType::Luxury, 8.0);

        Self { market_prices, trade_log: Vec::new(), total_trades: 0 }
    }

    pub fn execute_trade(&mut self, offer: TradeOffer, buyer_inv: &mut Inventory, seller_inv: &mut Inventory) -> bool {
        if seller_inv.get(&offer.offer) >= offer.offer_amount
            && buyer_inv.get(&offer.want) >= offer.want_amount
        {
            let offer_type = offer.offer.clone();
            let want_type = offer.want.clone();
            seller_inv.remove(offer_type.clone(), offer.offer_amount);
            buyer_inv.add(offer_type, offer.offer_amount);
            buyer_inv.remove(want_type.clone(), offer.want_amount);
            seller_inv.add(want_type, offer.want_amount);

            // Update market prices based on supply/demand
            let price = self.market_prices.entry(offer.offer.clone()).or_insert(1.0);
            *price = (*price * 1.01).min(20.0);

            self.trade_log.push(offer);
            if self.trade_log.len() > 100 { self.trade_log.remove(0); }
            self.total_trades += 1;
            true
        } else {
            false
        }
    }

    pub fn update_prices(&mut self, scarcity_modifier: f32) {
        for price in self.market_prices.values_mut() {
            *price = (*price * scarcity_modifier).clamp(0.5, 20.0);
        }
    }
}

impl Default for Economy {
    fn default() -> Self { Self::new() }
}
