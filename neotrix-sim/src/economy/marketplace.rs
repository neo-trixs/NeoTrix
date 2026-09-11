use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::currency::Wallet;
use super::crafting::Resource;

/// A listing on the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: u64,
    pub seller_id: String,
    pub resource: Resource,
    pub quantity: u32,
    pub price_per_unit: f32,
    pub currency: String,
    pub created_tick: u64,
    pub active: bool,
}

/// A completed trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub buyer_id: String,
    pub seller_id: String,
    pub resource: Resource,
    pub quantity: u32,
    pub price_per_unit: f32,
    pub total_price: f32,
    pub currency: String,
    pub tick: u64,
}

/// Marketplace: manages listings and completed trades.
pub struct Marketplace {
    pub listings: Vec<Listing>,
    pub completed_trades: Vec<TradeRecord>,
    pub next_listing_id: u64,
    pub max_history: usize,
    /// Price history by resource name.
    pub price_history: HashMap<String, Vec<f32>>,
}

impl Marketplace {
    pub fn new() -> Self {
        Self {
            listings: Vec::new(),
            completed_trades: Vec::new(),
            next_listing_id: 1,
            max_history: 200,
            price_history: HashMap::new(),
        }
    }

    /// List an item for sale. Returns the listing ID.
    pub fn list_item(
        &mut self,
        seller_id: &str,
        resource: Resource,
        quantity: u32,
        price_per_unit: f32,
        currency: &str,
        tick: u64,
    ) -> u64 {
        let id = self.next_listing_id;
        self.next_listing_id += 1;

        self.listings.push(Listing {
            id,
            seller_id: seller_id.to_string(),
            resource,
            quantity,
            price_per_unit,
            currency: currency.to_string(),
            created_tick: tick,
            active: true,
        });

        id
    }

    /// Buy an item from a listing. Transfers currency and resource.
    pub fn buy_item(
        &mut self,
        buyer_id: &str,
        listing_id: u64,
        buyer_wallet: &mut Wallet,
        seller_wallet: &mut Wallet,
        buyer_inventory: &mut super::crafting::Inventory,
        tick: u64,
    ) -> Result<TradeRecord, MarketplaceError> {
        let listing = self.listings.iter_mut()
            .find(|l| l.id == listing_id && l.active)
            .ok_or(MarketplaceError::ListingNotFound)?;

        let total_price = listing.price_per_unit * listing.quantity as f32;
        let currency = listing.currency.clone();
        let resource = listing.resource.clone();
        let quantity = listing.quantity;
        let seller_id = listing.seller_id.clone();
        let price_per_unit = listing.price_per_unit;

        // Check buyer can afford
        if !buyer_wallet.can_afford(&currency, total_price) {
            return Err(MarketplaceError::InsufficientFunds);
        }

        // Transfer currency
        Wallet::transfer(buyer_wallet, seller_wallet, &currency, total_price, tick)
            .map_err(|_| MarketplaceError::TransferFailed)?;

        // Transfer resource to buyer
        buyer_inventory.add(&resource.name, quantity);

        // Mark listing as sold
        listing.active = false;

        let record = TradeRecord {
            buyer_id: buyer_id.to_string(),
            seller_id,
            resource: resource.clone(),
            quantity,
            price_per_unit,
            total_price,
            currency,
            tick,
        };

        // Record price in history
        self.price_history
            .entry(resource.name.clone())
            .or_default()
            .push(price_per_unit);

        // Trim history
        if let Some(history) = self.price_history.get_mut(&resource.name) {
            if history.len() > self.max_history {
                history.remove(0);
            }
        }

        self.completed_trades.push(record.clone());

        // Trim trade history
        if self.completed_trades.len() > self.max_history {
            self.completed_trades.remove(0);
        }

        Ok(record)
    }

    /// Get price history for a resource.
    pub fn get_price_history(&self, resource_name: &str) -> Vec<f32> {
        self.price_history.get(resource_name).cloned().unwrap_or_default()
    }

    /// Get average price for a resource from trade history.
    pub fn average_price(&self, resource_name: &str) -> f32 {
        let history = self.get_price_history(resource_name);
        if history.is_empty() {
            return 0.0;
        }
        history.iter().sum::<f32>() / history.len() as f32
    }

    /// Get all active listings for a resource.
    pub fn active_listings_for(&self, resource_name: &str) -> Vec<&Listing> {
        self.listings.iter()
            .filter(|l| l.active && l.resource.name == resource_name)
            .collect()
    }

    /// Get the cheapest active listing for a resource.
    pub fn cheapest_listing(&self, resource_name: &str) -> Option<&Listing> {
        self.active_listings_for(resource_name)
            .into_iter()
            .min_by(|a, b| a.price_per_unit.partial_cmp(&b.price_per_unit).unwrap())
    }

    /// Cancel a listing (seller only).
    pub fn cancel_listing(&mut self, listing_id: u64, seller_id: &str) -> bool {
        if let Some(listing) = self.listings.iter_mut().find(|l| l.id == listing_id) {
            if listing.seller_id == seller_id && listing.active {
                listing.active = false;
                return true;
            }
        }
        false
    }

    /// Prune expired listings (older than max_age ticks).
    pub fn prune_expired(&mut self, current_tick: u64, max_age: u64) {
        for listing in &mut self.listings {
            if listing.active && current_tick.saturating_sub(listing.created_tick) > max_age {
                listing.active = false;
            }
        }
    }

    /// Total number of active listings.
    pub fn active_count(&self) -> usize {
        self.listings.iter().filter(|l| l.active).count()
    }

    /// Total completed trades.
    pub fn trade_count(&self) -> usize {
        self.completed_trades.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarketplaceError {
    ListingNotFound,
    InsufficientFunds,
    TransferFailed,
    AlreadySold,
}

impl std::fmt::Display for MarketplaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ListingNotFound => write!(f, "listing not found"),
            Self::InsufficientFunds => write!(f, "insufficient funds"),
            Self::TransferFailed => write!(f, "transfer failed"),
            Self::AlreadySold => write!(f, "already sold"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::currency::Wallet;
    use super::super::crafting::{Resource, Inventory};

    #[test]
    fn marketplace_creation() {
        let mp = Marketplace::new();
        assert_eq!(mp.active_count(), 0);
        assert_eq!(mp.trade_count(), 0);
    }

    #[test]
    fn list_item() {
        let mut mp = Marketplace::new();
        let id = mp.list_item("seller", Resource::wood(), 10, 5.0, "NeoCoin", 0);
        assert_eq!(id, 1);
        assert_eq!(mp.active_count(), 1);
    }

    #[test]
    fn buy_item_success() {
        let mut mp = Marketplace::new();
        let id = mp.list_item("seller", Resource::wood(), 10, 5.0, "NeoCoin", 0);

        let mut buyer = Wallet::with_balance("buyer", 200.0);
        let mut seller = Wallet::with_balance("seller", 0.0);
        let mut buyer_inv = Inventory::new();

        let result = mp.buy_item("buyer", id, &mut buyer, &mut seller, &mut buyer_inv, 1);
        assert!(result.is_ok());
        assert_eq!(buyer.neo_balance(), 150.0); // 200 - 50
        assert_eq!(seller.neo_balance(), 50.0);
        assert_eq!(buyer_inv.count("Wood"), 10);
        assert_eq!(mp.trade_count(), 1);
    }

    #[test]
    fn buy_item_insufficient_funds() {
        let mut mp = Marketplace::new();
        let id = mp.list_item("seller", Resource::sword(), 1, 100.0, "NeoCoin", 0);

        let mut buyer = Wallet::with_balance("buyer", 50.0);
        let mut seller = Wallet::with_balance("seller", 0.0);
        let mut buyer_inv = Inventory::new();

        let result = mp.buy_item("buyer", id, &mut buyer, &mut seller, &mut buyer_inv, 1);
        assert!(result.is_err());
        assert_eq!(buyer.neo_balance(), 50.0); // unchanged
    }

    #[test]
    fn buy_item_listing_not_found() {
        let mut mp = Marketplace::new();
        let mut buyer = Wallet::with_balance("buyer", 200.0);
        let mut seller = Wallet::with_balance("seller", 0.0);
        let mut buyer_inv = Inventory::new();

        let result = mp.buy_item("buyer", 999, &mut buyer, &mut seller, &mut buyer_inv, 1);
        assert!(result.is_err());
    }

    #[test]
    fn price_history_tracking() {
        let mut mp = Marketplace::new();
        let id1 = mp.list_item("s1", Resource::wood(), 5, 3.0, "NeoCoin", 0);
        let id2 = mp.list_item("s2", Resource::wood(), 5, 5.0, "NeoCoin", 1);

        let mut buyer = Wallet::with_balance("b", 200.0);
        let mut seller1 = Wallet::with_balance("s1", 0.0);
        let mut seller2 = Wallet::with_balance("s2", 0.0);
        let mut inv = Inventory::new();

        mp.buy_item("b", id1, &mut buyer, &mut seller1, &mut inv, 2).unwrap();
        mp.buy_item("b", id2, &mut buyer, &mut seller2, &mut inv, 3).unwrap();

        let history = mp.get_price_history("Wood");
        assert_eq!(history, vec![3.0, 5.0]);
        assert!((mp.average_price("Wood") - 4.0).abs() < 0.01);
    }

    #[test]
    fn cancel_listing() {
        let mut mp = Marketplace::new();
        let id = mp.list_item("seller", Resource::wood(), 10, 5.0, "NeoCoin", 0);
        assert!(mp.cancel_listing(id, "seller"));
        assert!(!mp.cancel_listing(id, "seller")); // already cancelled
        assert_eq!(mp.active_count(), 0);
    }

    #[test]
    fn cheapest_listing() {
        let mut mp = Marketplace::new();
        mp.list_item("s1", Resource::wood(), 5, 10.0, "NeoCoin", 0);
        mp.list_item("s2", Resource::wood(), 5, 3.0, "NeoCoin", 1);
        mp.list_item("s3", Resource::wood(), 5, 7.0, "NeoCoin", 2);

        let cheapest = mp.cheapest_listing("Wood").unwrap();
        assert_eq!(cheapest.price_per_unit, 3.0);
        assert_eq!(cheapest.seller_id, "s2");
    }

    #[test]
    fn prune_expired() {
        let mut mp = Marketplace::new();
        mp.list_item("s", Resource::wood(), 5, 3.0, "NeoCoin", 100);
        mp.list_item("s", Resource::wood(), 5, 3.0, "NeoCoin", 200);

        mp.prune_expired(260, 100); // max_age=100, current=260
        assert_eq!(mp.active_count(), 1); // only the 200 tick listing survives
    }
}
