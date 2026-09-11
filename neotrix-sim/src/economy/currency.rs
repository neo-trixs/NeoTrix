use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A currency type with exchange rate relative to the base currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub name: String,
    pub symbol: String,
    pub exchange_rate: f32, // rate relative to base (NeoCoin = 1.0)
}

impl Currency {
    pub fn new(name: &str, symbol: &str, exchange_rate: f32) -> Self {
        Self {
            name: name.to_string(),
            symbol: symbol.to_string(),
            exchange_rate,
        }
    }

    pub fn neo_coin() -> Self {
        Self::new("NeoCoin", "NC", 1.0)
    }

    /// Convert amount in this currency to NeoCoin.
    pub fn to_neo_coin(&self, amount: f32) -> f32 {
        amount * self.exchange_rate
    }

    /// Convert NeoCoin amount to this currency.
    pub fn from_neo_coin(&self, amount: f32) -> f32 {
        amount / self.exchange_rate.max(0.001)
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::neo_coin()
    }
}

/// A record of a single transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f32,
    pub currency: String,
    pub tick: u64,
    pub description: String,
}

/// Wallet: holds balance and transaction history for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub owner_id: String,
    pub balances: HashMap<String, f32>, // currency_name → amount
    pub history: Vec<Transaction>,
    pub max_history: usize,
}

impl Wallet {
    pub fn new(owner_id: &str) -> Self {
        let mut balances = HashMap::new();
        balances.insert("NeoCoin".to_string(), 100.0); // Starting balance
        Self {
            owner_id: owner_id.to_string(),
            balances,
            history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn with_balance(owner_id: &str, neo_coins: f32) -> Self {
        let mut balances = HashMap::new();
        balances.insert("NeoCoin".to_string(), neo_coins);
        Self {
            owner_id: owner_id.to_string(),
            balances,
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Get balance for a currency (default: NeoCoin).
    pub fn balance(&self, currency: &str) -> f32 {
        self.balances.get(currency).copied().unwrap_or(0.0)
    }

    /// Get NeoCoin balance (convenience).
    pub fn neo_balance(&self) -> f32 {
        self.balance("NeoCoin")
    }

    /// Deposit currency.
    pub fn deposit(&mut self, currency: &str, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        *self.balances.entry(currency.to_string()).or_insert(0.0) += amount;
    }

    /// Withdraw currency. Returns false if insufficient funds.
    pub fn withdraw(&mut self, currency: &str, amount: f32) -> bool {
        if amount <= 0.0 {
            return false;
        }
        let bal = self.balance(currency);
        if bal < amount {
            return false;
        }
        *self.balances.entry(currency.to_string()).or_insert(0.0) -= amount;
        true
    }

    /// Check if wallet can afford `amount` of `currency`.
    pub fn can_afford(&self, currency: &str, amount: f32) -> bool {
        self.balance(currency) >= amount
    }

    /// Record a transaction in history.
    fn record(&mut self, tx: Transaction) {
        self.history.push(tx);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Transfer currency between two wallets.
    pub fn transfer(
        from: &mut Wallet,
        to: &mut Wallet,
        currency: &str,
        amount: f32,
        tick: u64,
    ) -> Result<(), TransferError> {
        if amount <= 0.0 {
            return Err(TransferError::InvalidAmount);
        }
        if from.owner_id == to.owner_id {
            return Err(TransferError::SelfTransfer);
        }
        if !from.can_afford(currency, amount) {
            return Err(TransferError::InsufficientFunds {
                available: from.balance(currency),
                requested: amount,
            });
        }

        from.withdraw(currency, amount);
        to.deposit(currency, amount);

        let tx = Transaction {
            from: from.owner_id.clone(),
            to: to.owner_id.clone(),
            amount,
            currency: currency.to_string(),
            tick,
            description: format!("transfer {} {} from {} to {}", amount, currency, from.owner_id, to.owner_id),
        };

        from.record(Transaction {
            from: from.owner_id.clone(),
            to: to.owner_id.clone(),
            amount,
            currency: currency.to_string(),
            tick,
            description: tx.description.clone(),
        });

        to.record(tx);

        Ok(())
    }

    /// Total value in NeoCoin (sum of all currencies converted).
    pub fn total_value(&self, currencies: &HashMap<String, Currency>) -> f32 {
        self.balances.iter().map(|(name, &amount)| {
            currencies.get(name)
                .map(|c| c.to_neo_coin(amount))
                .unwrap_or(amount)
        }).sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferError {
    InsufficientFunds { available: f32, requested: f32 },
    InvalidAmount,
    SelfTransfer,
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientFunds { available, requested } => {
                write!(f, "insufficient funds: {} available, {} requested", available, requested)
            }
            Self::InvalidAmount => write!(f, "invalid amount"),
            Self::SelfTransfer => write!(f, "cannot transfer to self"),
        }
    }
}

/// Currency registry: manages all currencies and their exchange rates.
pub struct CurrencyRegistry {
    pub currencies: HashMap<String, Currency>,
}

impl CurrencyRegistry {
    pub fn new() -> Self {
        let mut currencies = HashMap::new();
        currencies.insert("NeoCoin".to_string(), Currency::neo_coin());
        currencies.insert("IronShard".to_string(), Currency::new("IronShard", "IS", 0.5));
        currencies.insert("Crystal".to_string(), Currency::new("Crystal", "CR", 2.0));
        Self { currencies }
    }

    pub fn get(&self, name: &str) -> Option<&Currency> {
        self.currencies.get(name)
    }

    pub fn register(&mut self, currency: Currency) {
        self.currencies.insert(currency.name.clone(), currency);
    }

    pub fn convert(&self, amount: f32, from: &str, to: &str) -> Option<f32> {
        let from_c = self.currencies.get(from)?;
        let to_c = self.currencies.get(to)?;
        Some(from_c.to_neo_coin(amount) / to_c.exchange_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_conversion() {
        let nc = Currency::neo_coin();
        let is = Currency::new("IronShard", "IS", 0.5);
        assert_eq!(nc.to_neo_coin(10.0), 10.0);
        assert_eq!(is.to_neo_coin(10.0), 5.0);
        assert_eq!(is.from_neo_coin(10.0), 20.0);
    }

    #[test]
    fn wallet_creation() {
        let w = Wallet::new("agent_1");
        assert_eq!(w.neo_balance(), 100.0);
        assert_eq!(w.balance("NeoCoin"), 100.0);
    }

    #[test]
    fn wallet_deposit_withdraw() {
        let mut w = Wallet::new("agent_1");
        w.deposit("NeoCoin", 50.0);
        assert_eq!(w.neo_balance(), 150.0);
        assert!(w.withdraw("NeoCoin", 30.0));
        assert_eq!(w.neo_balance(), 120.0);
        assert!(!w.withdraw("NeoCoin", 200.0));
    }

    #[test]
    fn wallet_transfer() {
        let mut a = Wallet::new("alice");
        let mut b = Wallet::new("bob");
        let result = Wallet::transfer(&mut a, &mut b, "NeoCoin", 30.0, 0);
        assert!(result.is_ok());
        assert_eq!(a.neo_balance(), 70.0);
        assert_eq!(b.neo_balance(), 130.0);
    }

    #[test]
    fn wallet_transfer_errors() {
        let mut a = Wallet::new("alice");
        let mut b = Wallet::new("bob");
        // Insufficient funds
        assert!(Wallet::transfer(&mut a, &mut b, "NeoCoin", 200.0, 0).is_err());
        // Invalid amount
        assert!(Wallet::transfer(&mut a, &mut b, "NeoCoin", -1.0, 0).is_err());
        // Self transfer
        let mut a2 = Wallet::new("alice");
        assert!(Wallet::transfer(&mut a, &mut a2, "NeoCoin", 10.0, 0).is_err());
    }

    #[test]
    fn currency_registry_conversion() {
        let reg = CurrencyRegistry::new();
        let result = reg.convert(10.0, "IronShard", "NeoCoin");
        assert_eq!(result, Some(5.0));
    }

    #[test]
    fn wallet_history() {
        let mut a = Wallet::new("alice");
        let mut b = Wallet::new("bob");
        let _ = Wallet::transfer(&mut a, &mut b, "NeoCoin", 10.0, 1);
        let _ = Wallet::transfer(&mut a, &mut b, "NeoCoin", 20.0, 2);
        assert_eq!(a.history.len(), 2);
        assert_eq!(b.history.len(), 2);
    }
}
