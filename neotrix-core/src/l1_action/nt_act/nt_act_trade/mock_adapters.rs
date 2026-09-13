//! Mock adapters for C2-C4 integration testing of foreign trade systems.
//! Simulates ERP, Bank, Customs, and Shipping system interfaces.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Order status in mock ERP system
#[derive(Debug, Clone, PartialEq)]
pub enum MockOrderStatus {
    Created,
    Confirmed,
    InProduction,
    QualityCheck,
    Packed,
    ReadyToShip,
    Shipped,
    Completed,
    Cancelled,
}

/// Mock ERP system adapter
pub struct MockErpSystem {
    orders: HashMap<String, MockOrder>,
    inventory: HashMap<String, i64>,
    #[allow(dead_code)]
    production_schedules: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MockOrder {
    pub order_id: String,
    pub product: String,
    pub quantity: i64,
    pub status: MockOrderStatus,
    pub milestones: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Default for MockErpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockErpSystem {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            inventory: HashMap::new(),
            production_schedules: HashMap::new(),
        }
    }

    pub fn create_order(&mut self, order_id: &str, product: &str, quantity: i64) -> Result<&MockOrder, String> {
        if self.orders.contains_key(order_id) {
            return Err(format!("Order {} already exists", order_id));
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let order = MockOrder {
            order_id: order_id.to_string(),
            product: product.to_string(),
            quantity,
            status: MockOrderStatus::Created,
            milestones: vec![
                "Order Created".into(),
                "Raw Material Sourcing".into(),
                "Production Start".into(),
                "Quality Check".into(),
                "Packaging".into(),
                "Ready to Ship".into(),
            ],
            created_at: now,
            updated_at: now,
        };

        self.orders.insert(order_id.to_string(), order);
        Ok(self.orders.get(order_id).unwrap())
    }

    pub fn update_status(&mut self, order_id: &str, new_status: MockOrderStatus) -> Result<&MockOrder, String> {
        let order = self.orders.get_mut(order_id).ok_or_else(|| format!("Order {} not found", order_id))?;
        order.status = new_status;
        order.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        Ok(self.orders.get(order_id).unwrap())
    }

    pub fn get_order(&self, order_id: &str) -> Result<&MockOrder, String> {
        self.orders.get(order_id).ok_or_else(|| format!("Order {} not found", order_id))
    }

    pub(crate) fn _list_orders(&self) -> Vec<&MockOrder> {
        self.orders.values().collect()
    }

    pub(crate) fn _get_production_milestones(&self, order_id: &str) -> Result<Vec<String>, String> {
        let order = self.get_order(order_id)?;
        Ok(order.milestones.clone())
    }

    pub(crate) fn _update_inventory(&mut self, product: &str, quantity: i64) {
        *self.inventory.entry(product.to_string()).or_insert(0) += quantity;
    }

    pub(crate) fn _check_inventory(&self, product: &str) -> i64 {
        self.inventory.get(product).copied().unwrap_or(0)
    }
}

/// Letter of Credit status in mock Bank system
#[derive(Debug, Clone, PartialEq)]
pub enum MockLcStatus {
    Issued,
    Confirmed,
    Amended,
    DocumentsPresented,
    UnderReview,
    Accepted,
    Rejected,
    Paid,
    Expired,
}

/// Mock Bank system adapter
pub struct MockBankSystem {
    lcs: HashMap<String, MockLc>,
    payments: HashMap<String, MockPayment>,
}

#[derive(Debug, Clone)]
pub struct MockLc {
    pub lc_number: String,
    pub issuer_bank: String,
    pub beneficiary: String,
    pub amount: f64,
    pub currency: String,
    pub status: MockLcStatus,
    pub issued_at: u64,
    pub expiry: u64,
    pub documents: Vec<String>,
    pub soft_clauses: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MockPayment {
    pub payment_id: String,
    pub lc_number: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub payer: String,
    pub payee: String,
    pub status: String,
    pub timestamp: u64,
}

impl Default for MockBankSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBankSystem {
    pub fn new() -> Self {
        Self {
            lcs: HashMap::new(),
            payments: HashMap::new(),
        }
    }

    pub fn issue_lc(
        &mut self,
        lc_number: &str,
        issuer_bank: &str,
        beneficiary: &str,
        amount: f64,
        currency: &str,
        expiry_days: u64,
    ) -> Result<&MockLc, String> {
        if self.lcs.contains_key(lc_number) {
            return Err(format!("LC {} already exists", lc_number));
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let lc = MockLc {
            lc_number: lc_number.to_string(),
            issuer_bank: issuer_bank.to_string(),
            beneficiary: beneficiary.to_string(),
            amount,
            currency: currency.to_string(),
            status: MockLcStatus::Issued,
            issued_at: now,
            expiry: now + expiry_days * 86400,
            documents: Vec::new(),
            soft_clauses: Vec::new(),
        };

        self.lcs.insert(lc_number.to_string(), lc);
        Ok(self.lcs.get(lc_number).unwrap())
    }

    pub fn confirm_lc(&mut self, lc_number: &str) -> Result<&MockLc, String> {
        let lc = self.lcs.get_mut(lc_number).ok_or_else(|| format!("LC {} not found", lc_number))?;
        if lc.status != MockLcStatus::Issued {
            return Err(format!("LC {} cannot be confirmed in status {:?}", lc_number, lc.status));
        }
        lc.status = MockLcStatus::Confirmed;
        Ok(self.lcs.get(lc_number).unwrap())
    }

    pub(crate) fn _amend_lc(&mut self, lc_number: &str, new_amount: Option<f64>, new_expiry: Option<u64>) -> Result<&MockLc, String> {
        let lc = self.lcs.get_mut(lc_number).ok_or_else(|| format!("LC {} not found", lc_number))?;
        if let Some(amount) = new_amount {
            lc.amount = amount;
        }
        if let Some(expiry) = new_expiry {
            lc.expiry = expiry;
        }
        lc.status = MockLcStatus::Amended;
        Ok(self.lcs.get(lc_number).unwrap())
    }

    pub fn add_soft_clause(&mut self, lc_number: &str, clause: &str) -> Result<(), String> {
        let lc = self.lcs.get_mut(lc_number).ok_or_else(|| format!("LC {} not found", lc_number))?;
        lc.soft_clauses.push(clause.to_string());
        Ok(())
    }

    pub fn get_soft_clauses(&self, lc_number: &str) -> Result<Vec<String>, String> {
        let lc = self.get_lc(lc_number)?;
        Ok(lc.soft_clauses.clone())
    }

    pub fn present_documents(&mut self, lc_number: &str, documents: Vec<String>) -> Result<&MockLc, String> {
        let lc = self.lcs.get_mut(lc_number).ok_or_else(|| format!("LC {} not found", lc_number))?;
        lc.documents = documents;
        lc.status = MockLcStatus::DocumentsPresented;
        Ok(self.lcs.get(lc_number).unwrap())
    }

    pub fn review_documents(&mut self, lc_number: &str, accept: bool) -> Result<&MockLc, String> {
        let lc = self.lcs.get_mut(lc_number).ok_or_else(|| format!("LC {} not found", lc_number))?;
        if accept {
            lc.status = MockLcStatus::Accepted;
        } else {
            lc.status = MockLcStatus::Rejected;
        }
        Ok(self.lcs.get(lc_number).unwrap())
    }

    pub fn process_payment(&mut self, lc_number: &str, payer: &str, payee: &str) -> Result<&MockPayment, String> {
        let lc = self.get_lc(lc_number)?;
        if lc.status != MockLcStatus::Accepted {
            return Err(format!("LC {} not accepted for payment", lc_number));
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let payment_id = format!("PAY-{}-{}", lc_number, now);
        let payment = MockPayment {
            payment_id: payment_id.clone(),
            lc_number: Some(lc_number.to_string()),
            amount: lc.amount,
            currency: lc.currency.clone(),
            payer: payer.to_string(),
            payee: payee.to_string(),
            status: "Completed".into(),
            timestamp: now,
        };

        self.payments.insert(payment_id.clone(), payment);
        Ok(self.payments.get(&payment_id).unwrap())
    }

    pub fn get_lc(&self, lc_number: &str) -> Result<&MockLc, String> {
        self.lcs.get(lc_number).ok_or_else(|| format!("LC {} not found", lc_number))
    }

    pub(crate) fn _list_lcs(&self) -> Vec<&MockLc> {
        self.lcs.values().collect()
    }

    pub fn get_fx_rate(&self, base: &str, quote: &str) -> f64 {
        match (base, quote) {
            ("USD", "CNY") => 7.25,
            ("EUR", "CNY") => 7.85,
            ("GBP", "CNY") => 9.15,
            ("USD", "EUR") => 0.92,
            ("USD", "GBP") => 0.79,
            _ => 1.0,
        }
    }
}

/// Customs declaration status
#[derive(Debug, Clone, PartialEq)]
pub enum MockCustomsStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Cleared,
    Held,
}

/// Mock Customs system adapter
pub struct MockCustomsSystem {
    declarations: HashMap<String, MockDeclaration>,
}

#[derive(Debug, Clone)]
pub struct MockDeclaration {
    pub declaration_id: String,
    pub order_id: String,
    pub status: MockCustomsStatus,
    pub hs_code: String,
    pub declared_value: f64,
    pub duty_amount: f64,
    pub submitted_at: u64,
    pub cleared_at: Option<u64>,
    pub documents: Vec<String>,
}

impl Default for MockCustomsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockCustomsSystem {
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
        }
    }

    pub fn submit_declaration(
        &mut self,
        declaration_id: &str,
        order_id: &str,
        hs_code: &str,
        declared_value: f64,
        duty_rate: f64,
    ) -> Result<&MockDeclaration, String> {
        if self.declarations.contains_key(declaration_id) {
            return Err(format!("Declaration {} already exists", declaration_id));
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let declaration = MockDeclaration {
            declaration_id: declaration_id.to_string(),
            order_id: order_id.to_string(),
            status: MockCustomsStatus::Submitted,
            hs_code: hs_code.to_string(),
            declared_value,
            duty_amount: declared_value * duty_rate,
            submitted_at: now,
            cleared_at: None,
            documents: Vec::new(),
        };

        self.declarations.insert(declaration_id.to_string(), declaration);
        Ok(self.declarations.get(declaration_id).unwrap())
    }

    pub fn update_status(&mut self, declaration_id: &str, new_status: MockCustomsStatus) -> Result<&MockDeclaration, String> {
        let decl = self.declarations.get_mut(declaration_id)
            .ok_or_else(|| format!("Declaration {} not found", declaration_id))?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        decl.status = new_status.clone();

        if new_status == MockCustomsStatus::Cleared {
            decl.cleared_at = Some(now);
        }

        Ok(self.declarations.get(declaration_id).unwrap())
    }

    pub fn add_documents(&mut self, declaration_id: &str, documents: Vec<String>) -> Result<(), String> {
        let decl = self.declarations.get_mut(declaration_id)
            .ok_or_else(|| format!("Declaration {} not found", declaration_id))?;
        decl.documents.extend(documents);
        Ok(())
    }

    pub(crate) fn _get_declaration(&self, declaration_id: &str) -> Result<&MockDeclaration, String> {
        self.declarations.get(declaration_id)
            .ok_or_else(|| format!("Declaration {} not found", declaration_id))
    }

    pub(crate) fn _list_declarations(&self) -> Vec<&MockDeclaration> {
        self.declarations.values().collect()
    }

    pub fn calculate_duty(&self, declared_value: f64, duty_rate: f64) -> f64 {
        declared_value * duty_rate
    }
}

/// Shipment tracking status
#[derive(Debug, Clone, PartialEq)]
pub enum MockShipmentStatus {
    Booked,
    ContainerReceived,
    Loaded,
    Departed,
    InTransit,
    Arrived,
    CustomsClearance,
    Delivered,
    Exception,
}

/// Mock Shipping system adapter
pub struct MockShippingSystem {
    shipments: HashMap<String, MockShipment>,
    containers: HashMap<String, MockContainer>,
}

#[derive(Debug, Clone)]
pub struct MockShipment {
    pub shipment_id: String,
    pub order_id: String,
    pub status: MockShipmentStatus,
    pub carrier: String,
    pub vessel: String,
    pub voyage: String,
    pub port_of_loading: String,
    pub port_of_discharge: String,
    pub etd: u64,
    pub eta: u64,
    pub containers: Vec<String>,
    pub bl_number: String,
}

#[derive(Debug, Clone)]
pub struct MockContainer {
    pub container_id: String,
    pub container_type: String,
    pub seal_number: String,
    pub weight: f64,
    pub volume: f64,
    pub loaded: bool,
}

impl Default for MockShippingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockShippingSystem {
    pub fn new() -> Self {
        Self {
            shipments: HashMap::new(),
            containers: HashMap::new(),
        }
    }

    pub fn book_shipment(
        &mut self,
        shipment_id: &str,
        order_id: &str,
        carrier: &str,
        vessel: &str,
        voyage: &str,
        pol: &str,
        pod: &str,
    ) -> Result<&MockShipment, String> {
        if self.shipments.contains_key(shipment_id) {
            return Err(format!("Shipment {} already exists", shipment_id));
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let shipment = MockShipment {
            shipment_id: shipment_id.to_string(),
            order_id: order_id.to_string(),
            status: MockShipmentStatus::Booked,
            carrier: carrier.to_string(),
            vessel: vessel.to_string(),
            voyage: voyage.to_string(),
            port_of_loading: pol.to_string(),
            port_of_discharge: pod.to_string(),
            etd: now + 7 * 86400, // ETD in 7 days
            eta: now + 21 * 86400, // ETA in 21 days
            containers: Vec::new(),
            bl_number: format!("BL-{}", shipment_id),
        };

        self.shipments.insert(shipment_id.to_string(), shipment);
        Ok(self.shipments.get(shipment_id).unwrap())
    }

    pub fn add_container(&mut self, shipment_id: &str, container: MockContainer) -> Result<(), String> {
        let shipment = self.shipments.get_mut(shipment_id)
            .ok_or_else(|| format!("Shipment {} not found", shipment_id))?;

        self.containers.insert(container.container_id.clone(), container.clone());
        shipment.containers.push(container.container_id);
        Ok(())
    }

    pub fn update_status(&mut self, shipment_id: &str, new_status: MockShipmentStatus) -> Result<&MockShipment, String> {
        let shipment = self.shipments.get_mut(shipment_id)
            .ok_or_else(|| format!("Shipment {} not found", shipment_id))?;
        shipment.status = new_status;
        Ok(self.shipments.get(shipment_id).unwrap())
    }

    pub fn get_tracking(&self, shipment_id: &str) -> Result<&MockShipment, String> {
        self.shipments.get(shipment_id)
            .ok_or_else(|| format!("Shipment {} not found", shipment_id))
    }

    pub(crate) fn _list_shipments(&self) -> Vec<&MockShipment> {
        self.shipments.values().collect()
    }

    pub(crate) fn _generate_bl(&self, shipment_id: &str) -> Result<String, String> {
        let shipment = self.get_tracking(shipment_id)?;
        Ok(format!(
            "BILL OF LADING\n\nShipper: Exporter Ltd\nConsignee: Importer Co\n\nVessel: {} {}\nVoyage: {}\n\nPort of Loading: {}\nPort of Discharge: {}\n\nContainer: {}\n\nBL Number: {}\n\nDate: {}",
            shipment.carrier, shipment.vessel,
            shipment.voyage,
            shipment.port_of_loading,
            shipment.port_of_discharge,
            shipment.containers.first().unwrap_or(&"N/A".to_string()),
            shipment.bl_number,
            shipment.etd
        ))
    }
}

/// Integration test harness combining all mock systems
pub struct TradeIntegrationHarness {
    pub erp: MockErpSystem,
    pub bank: MockBankSystem,
    pub customs: MockCustomsSystem,
    pub shipping: MockShippingSystem,
}

impl Default for TradeIntegrationHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeIntegrationHarness {
    pub fn new() -> Self {
        Self {
            erp: MockErpSystem::new(),
            bank: MockBankSystem::new(),
            customs: MockCustomsSystem::new(),
            shipping: MockShippingSystem::new(),
        }
    }

    /// Simulate a complete trade flow: Order → LC → Production → Shipping → Customs → Payment
    pub fn simulate_full_trade(&mut self) -> Result<String, String> {
        // 1. Create order in ERP
        self.erp.create_order("ORD-001", "Industrial Machinery", 100)?;
        self.erp.update_status("ORD-001", MockOrderStatus::Confirmed)?;

        // 2. Issue and confirm LC
        self.bank.issue_lc("LC-001", "HSBC", "Exporter Ltd", 150000.0, "USD", 90)?;
        self.bank.confirm_lc("LC-001")?;
        self.bank.add_soft_clause("LC-001", "Beneficiary must provide inspection certificate issued by SGS")?;

        // 3. Check for soft clauses
        let soft_clauses = self.bank.get_soft_clauses("LC-001")?;
        if !soft_clauses.is_empty() {
            return Err(format!("LC has {} soft clauses that need to be addressed", soft_clauses.len()));
        }

        // 4. Update production in ERP
        self.erp.update_status("ORD-001", MockOrderStatus::InProduction)?;
        self.erp.update_status("ORD-001", MockOrderStatus::QualityCheck)?;
        self.erp.update_status("ORD-001", MockOrderStatus::Packed)?;
        self.erp.update_status("ORD-001", MockOrderStatus::ReadyToShip)?;

        // 5. Book shipment
        self.shipping.book_shipment("SHP-001", "ORD-001", "Maersk", "Maersk SEALAND", "AE123", "Shanghai", "Hamburg")?;
        self.shipping.add_container("SHP-001", MockContainer {
            container_id: "MSKU1234567".into(),
            container_type: "40HQ".into(),
            seal_number: "SL12345".into(),
            weight: 5000.0,
            volume: 45.0,
            loaded: true,
        })?;
        self.shipping.update_status("SHP-001", MockShipmentStatus::Loaded)?;
        self.shipping.update_status("SHP-001", MockShipmentStatus::Departed)?;

        // 6. Submit customs declaration
        self.customs.submit_declaration("CD-001", "ORD-001", "8479.89", 150000.0, 0.0)?;
        self.customs.add_documents("CD-001", vec![
            "Commercial Invoice".into(),
            "Packing List".into(),
            "Bill of Lading".into(),
            "Certificate of Origin".into(),
        ])?;
        self.customs.update_status("CD-001", MockCustomsStatus::Approved)?;
        self.customs.update_status("CD-001", MockCustomsStatus::Cleared)?;

        // 7. Present documents and process payment
        self.bank.present_documents("LC-001", vec![
            "Commercial Invoice".into(),
            "Bill of Lading".into(),
            "Packing List".into(),
            "Certificate of Origin".into(),
        ])?;
        self.bank.review_documents("LC-001", true)?;
        self.bank.process_payment("LC-001", "HSBC", "Exporter Ltd")?;

        // 8. Final status update
        self.erp.update_status("ORD-001", MockOrderStatus::Completed)?;
        self.shipping.update_status("SHP-001", MockShipmentStatus::Delivered)?;

        Ok("Trade flow completed successfully".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erp_order_flow() {
        let mut erp = MockErpSystem::new();
        let order = erp.create_order("ORD-001", "Test Product", 50).unwrap();
        assert_eq!(order.status, MockOrderStatus::Created);

        erp.update_status("ORD-001", MockOrderStatus::Confirmed).unwrap();
        let order = erp.get_order("ORD-001").unwrap();
        assert_eq!(order.status, MockOrderStatus::Confirmed);
    }

    #[test]
    fn test_bank_lc_flow() {
        let mut bank = MockBankSystem::new();
        let lc = bank.issue_lc("LC-001", "HSBC", "Test Co", 100000.0, "USD", 90).unwrap();
        assert_eq!(lc.status, MockLcStatus::Issued);

        bank.confirm_lc("LC-001").unwrap();
        let lc = bank.get_lc("LC-001").unwrap();
        assert_eq!(lc.status, MockLcStatus::Confirmed);
    }

    #[test]
    fn test_customs_declaration() {
        let mut customs = MockCustomsSystem::new();
        let decl = customs.submit_declaration("CD-001", "ORD-001", "8479.89", 50000.0, 0.0).unwrap();
        assert_eq!(decl.status, MockCustomsStatus::Submitted);
        assert_eq!(decl.duty_amount, 0.0);
    }

    #[test]
    fn test_shipping_tracking() {
        let mut shipping = MockShippingSystem::new();
        shipping.book_shipment("SHP-001", "ORD-001", "Maersk", "Sealand", "V123", "Shanghai", "Rotterdam").unwrap();
        let shipment = shipping.get_tracking("SHP-001").unwrap();
        assert_eq!(shipment.status, MockShipmentStatus::Booked);
    }

    #[test]
    fn test_soft_clause_detection() {
        let mut bank = MockBankSystem::new();
        bank.issue_lc("LC-001", "HSBC", "Test Co", 100000.0, "USD", 90).unwrap();
        bank.add_soft_clause("LC-001", "Inspection required by buyer's agent").unwrap();

        let clauses = bank.get_soft_clauses("LC-001").unwrap();
        assert_eq!(clauses.len(), 1);
        assert!(clauses[0].contains("inspection"));
    }

    #[test]
    fn test_integration_harness() {
        let mut harness = TradeIntegrationHarness::new();
        let result = harness.simulate_full_trade();
        assert!(result.is_ok(), "Full trade simulation should succeed: {:?}", result.err());
    }

    #[test]
    fn test_fx_rate() {
        let bank = MockBankSystem::new();
        assert_eq!(bank.get_fx_rate("USD", "CNY"), 7.25);
        assert_eq!(bank.get_fx_rate("EUR", "CNY"), 7.85);
    }

    #[test]
    fn test_duty_calculation() {
        let customs = MockCustomsSystem::new();
        let duty = customs.calculate_duty(100000.0, 0.05);
        assert_eq!(duty, 5000.0);
    }
}
