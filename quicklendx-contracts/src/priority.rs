use soroban_sdk::{contracttype, symbol_short, vec, Address, BytesN, Env, String, Vec};

/// Priority level enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PriorityLevel {
    Low = 1,      // Low priority
    Medium = 2,   // Medium priority
    High = 3,     // High priority
    Critical = 4, // Critical priority
}

/// Urgency level enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum UrgencyLevel {
    Low = 1,      // Low urgency
    Medium = 2,   // Medium urgency
    High = 3,     // High urgency
    Critical = 4, // Critical urgency
}

/// Priority change request structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct PriorityChangeRequest {
    pub id: BytesN<32>,              // Unique request ID
    pub invoice_id: BytesN<32>,      // Invoice ID
    pub requester: Address,          // Address requesting the change
    pub old_priority: PriorityLevel, // Current priority level
    pub new_priority: PriorityLevel, // Requested priority level
    pub reason: String,              // Reason for the change
    pub requested_at: u64,           // Request timestamp
    pub status: PriorityChangeStatus, // Request status
    pub reviewed_by: Option<Address>, // Address who reviewed the request
    pub reviewed_at: Option<u64>,    // Review timestamp
    pub review_notes: String,        // Review notes
}

/// Priority change request status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PriorityChangeStatus {
    Pending,   // Request pending review
    Approved,  // Request approved
    Rejected,  // Request rejected
    Cancelled, // Request cancelled
}

/// Fee structure based on priority
#[contracttype]
#[derive(Clone, Debug)]
pub struct PriorityFeeStructure {
    pub priority_level: PriorityLevel, // Priority level
    pub base_fee_bps: i128,           // Base fee in basis points
    pub urgency_multiplier: i128,     // Urgency multiplier (in basis points, 10000 = 1.0)
    pub minimum_fee: i128,            // Minimum fee amount
    pub maximum_fee: i128,            // Maximum fee amount
}

/// Priority-based bid matching criteria
#[contracttype]
#[derive(Clone, Debug)]
pub struct PriorityBidCriteria {
    pub invoice_id: BytesN<32>,       // Invoice ID
    pub priority_level: PriorityLevel, // Required priority level
    pub urgency_threshold: UrgencyLevel, // Minimum urgency threshold
    pub fee_preference: i128,         // Fee preference (lower is better)
    pub created_at: u64,              // Criteria creation timestamp
}

use crate::errors::QuickLendXError;

impl PriorityLevel {
    /// Get priority level from integer value
    pub fn from_value(value: u32) -> Result<Self, QuickLendXError> {
        match value {
            1 => Ok(PriorityLevel::Low),
            2 => Ok(PriorityLevel::Medium),
            3 => Ok(PriorityLevel::High),
            4 => Ok(PriorityLevel::Critical),
            _ => Err(QuickLendXError::InvalidPriorityLevel),
        }
    }

    /// Get integer value of priority level
    pub fn to_value(&self) -> u32 {
        match self {
            PriorityLevel::Low => 1,
            PriorityLevel::Medium => 2,
            PriorityLevel::High => 3,
            PriorityLevel::Critical => 4,
        }
    }

    /// Get priority level name
    pub fn to_string(&self, env: &Env) -> String {
        match self {
            PriorityLevel::Low => String::from_str(env, "Low"),
            PriorityLevel::Medium => String::from_str(env, "Medium"),
            PriorityLevel::High => String::from_str(env, "High"),
            PriorityLevel::Critical => String::from_str(env, "Critical"),
        }
    }
}

impl UrgencyLevel {
    /// Get urgency level from integer value
    pub fn from_value(value: u32) -> Result<Self, QuickLendXError> {
        match value {
            1 => Ok(UrgencyLevel::Low),
            2 => Ok(UrgencyLevel::Medium),
            3 => Ok(UrgencyLevel::High),
            4 => Ok(UrgencyLevel::Critical),
            _ => Err(QuickLendXError::InvalidUrgencyLevel),
        }
    }

    /// Get integer value of urgency level
    pub fn to_value(&self) -> u32 {
        match self {
            UrgencyLevel::Low => 1,
            UrgencyLevel::Medium => 2,
            UrgencyLevel::High => 3,
            UrgencyLevel::Critical => 4,
        }
    }

    /// Get urgency level name
    pub fn to_string(&self, env: &Env) -> String {
        match self {
            UrgencyLevel::Low => String::from_str(env, "Low"),
            UrgencyLevel::Medium => String::from_str(env, "Medium"),
            UrgencyLevel::High => String::from_str(env, "High"),
            UrgencyLevel::Critical => String::from_str(env, "Critical"),
        }
    }
}

impl PriorityChangeRequest {
    /// Create a new priority change request
    pub fn new(
        env: &Env,
        invoice_id: BytesN<32>,
        requester: Address,
        old_priority: PriorityLevel,
        new_priority: PriorityLevel,
        reason: String,
    ) -> Result<Self, QuickLendXError> {
        if reason.len() == 0 {
            return Err(QuickLendXError::InvalidDescription);
        }

        // Validate priority change is meaningful
        if old_priority == new_priority {
            return Err(QuickLendXError::InvalidPriorityChange);
        }

        let id = Self::generate_unique_request_id(env);
        let requested_at = env.ledger().timestamp();

        Ok(Self {
            id,
            invoice_id,
            requester,
            old_priority,
            new_priority,
            reason,
            requested_at,
            status: PriorityChangeStatus::Pending,
            reviewed_by: None,
            reviewed_at: None,
            review_notes: String::from_str(env, ""),
        })
    }

    /// Generate a unique request ID
    fn generate_unique_request_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let counter_key = symbol_short!("pri_cnt");
        let counter: u32 = env.storage().instance().get(&counter_key).unwrap_or(0);
        env.storage().instance().set(&counter_key, &(counter + 1));

        // Create a unique ID from timestamp, sequence, and counter
        let mut id_bytes = [0u8; 32];
        id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        id_bytes[8..12].copy_from_slice(&sequence.to_be_bytes());
        id_bytes[12..16].copy_from_slice(&counter.to_be_bytes());

        BytesN::from_array(env, &id_bytes)
    }

    /// Approve the priority change request
    pub fn approve(&mut self, env: &Env, reviewer: Address, notes: String) -> Result<(), QuickLendXError> {
        if self.status != PriorityChangeStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = PriorityChangeStatus::Approved;
        self.reviewed_by = Some(reviewer);
        self.reviewed_at = Some(env.ledger().timestamp());
        self.review_notes = notes;

        Ok(())
    }

    /// Reject the priority change request
    pub fn reject(&mut self, env: &Env, reviewer: Address, notes: String) -> Result<(), QuickLendXError> {
        if self.status != PriorityChangeStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = PriorityChangeStatus::Rejected;
        self.reviewed_by = Some(reviewer);
        self.reviewed_at = Some(env.ledger().timestamp());
        self.review_notes = notes;

        Ok(())
    }

    /// Cancel the priority change request
    pub fn cancel(&mut self, env: &Env, canceller: Address) -> Result<(), QuickLendXError> {
        if self.status != PriorityChangeStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        // Only the requester can cancel their own request
        if self.requester != canceller {
            return Err(QuickLendXError::Unauthorized);
        }

        self.status = PriorityChangeStatus::Cancelled;
        self.reviewed_by = Some(canceller);
        self.reviewed_at = Some(env.ledger().timestamp());
        self.review_notes = String::from_str(env, "Cancelled by requester");

        Ok(())
    }
}

impl PriorityFeeStructure {
    /// Create a new priority fee structure
    pub fn new(
        priority_level: PriorityLevel,
        base_fee_bps: i128,
        urgency_multiplier: i128,
        minimum_fee: i128,
        maximum_fee: i128,
    ) -> Result<Self, QuickLendXError> {
        if base_fee_bps < 0 || base_fee_bps > 10000 {
            return Err(QuickLendXError::InvalidFeeStructure);
        }

        if urgency_multiplier < 0 {
            return Err(QuickLendXError::InvalidFeeStructure);
        }

        if minimum_fee < 0 || maximum_fee < 0 || minimum_fee > maximum_fee {
            return Err(QuickLendXError::InvalidFeeStructure);
        }

        Ok(Self {
            priority_level,
            base_fee_bps,
            urgency_multiplier,
            minimum_fee,
            maximum_fee,
        })
    }

    /// Calculate fee based on urgency
    pub fn calculate_fee(&self, urgency_level: UrgencyLevel, base_amount: i128) -> i128 {
        let urgency_multiplier = match urgency_level {
            UrgencyLevel::Low => 10000,      // 1.0x
            UrgencyLevel::Medium => 12000,   // 1.2x
            UrgencyLevel::High => 15000,     // 1.5x
            UrgencyLevel::Critical => 20000, // 2.0x
        };

        let adjusted_multiplier = (self.urgency_multiplier * urgency_multiplier) / 10000;
        let calculated_fee = (base_amount * self.base_fee_bps * adjusted_multiplier) / (10000 * 10000);

        // Apply minimum and maximum constraints
        if calculated_fee < self.minimum_fee {
            self.minimum_fee
        } else if calculated_fee > self.maximum_fee {
            self.maximum_fee
        } else {
            calculated_fee
        }
    }
}

impl PriorityBidCriteria {
    /// Create new priority bid criteria
    pub fn new(
        env: &Env,
        invoice_id: BytesN<32>,
        priority_level: PriorityLevel,
        urgency_threshold: UrgencyLevel,
        fee_preference: i128,
    ) -> Result<Self, QuickLendXError> {
        if fee_preference < 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        Ok(Self {
            invoice_id,
            priority_level,
            urgency_threshold,
            fee_preference,
            created_at: env.ledger().timestamp(),
        })
    }

    /// Check if a bid matches the criteria
    pub fn matches_criteria(
        &self,
        bid_priority: PriorityLevel,
        bid_urgency: UrgencyLevel,
        bid_fee: i128,
    ) -> bool {
        // Check priority level (must be equal or higher)
        if bid_priority < self.priority_level {
            return false;
        }

        // Check urgency threshold (must be equal or higher)
        if bid_urgency < self.urgency_threshold {
            return false;
        }

        // Check fee preference (lower is better, but we allow equal)
        if bid_fee > self.fee_preference {
            return false;
        }

        true
    }
}

/// Calculate urgency level based on due date proximity
pub fn calculate_urgency_level(env: &Env, due_date: u64) -> UrgencyLevel {
    let current_timestamp = env.ledger().timestamp();
    let time_remaining = if due_date > current_timestamp {
        due_date - current_timestamp
    } else {
        0
    };

    // Convert to days (assuming 86400 seconds per day)
    let days_remaining = time_remaining / 86400;

    match days_remaining {
        0..=1 => UrgencyLevel::Critical,   // 0-1 days
        2..=7 => UrgencyLevel::High,       // 2-7 days
        8..=30 => UrgencyLevel::Medium,    // 8-30 days
        _ => UrgencyLevel::Low,            // 30+ days
    }
}

/// Get default priority fee structures
pub fn get_default_fee_structures(env: &Env) -> Vec<PriorityFeeStructure> {
    vec![
        env,
        PriorityFeeStructure::new(PriorityLevel::Low, 100, 10000, 10, 1000).unwrap(),
        PriorityFeeStructure::new(PriorityLevel::Medium, 150, 12000, 15, 1500).unwrap(),
        PriorityFeeStructure::new(PriorityLevel::High, 200, 15000, 20, 2000).unwrap(),
        PriorityFeeStructure::new(PriorityLevel::Critical, 300, 20000, 30, 3000).unwrap(),
    ]
}

/// Storage keys for priority data
pub struct PriorityStorage;

impl PriorityStorage {
    /// Store a priority change request
    pub fn store_priority_request(env: &Env, request: &PriorityChangeRequest) {
        env.storage().instance().set(&request.id, request);

        // Add to pending requests list
        Self::add_to_pending_requests(env, &request.id);

        // Add to invoice requests list
        Self::add_to_invoice_requests(env, &request.invoice_id, &request.id);
    }

    /// Get a priority change request by ID
    pub fn get_priority_request(env: &Env, request_id: &BytesN<32>) -> Option<PriorityChangeRequest> {
        env.storage().instance().get(request_id)
    }

    /// Update a priority change request
    pub fn update_priority_request(env: &Env, request: &PriorityChangeRequest) {
        env.storage().instance().set(&request.id, request);
    }

    /// Get all pending priority change requests
    pub fn get_pending_requests(env: &Env) -> Vec<BytesN<32>> {
        let key = symbol_short!("pend_req");
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get all priority change requests for an invoice
    pub fn get_invoice_requests(env: &Env, invoice_id: &BytesN<32>) -> Vec<BytesN<32>> {
        let key = (symbol_short!("inv_req"), invoice_id.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Add request to pending requests list
    fn add_to_pending_requests(env: &Env, request_id: &BytesN<32>) {
        let key = symbol_short!("pend_req");
        let mut requests = Self::get_pending_requests(env);
        requests.push_back(request_id.clone());
        env.storage().instance().set(&key, &requests);
    }

    /// Add request to invoice requests list
    fn add_to_invoice_requests(env: &Env, invoice_id: &BytesN<32>, request_id: &BytesN<32>) {
        let key = (symbol_short!("inv_req"), invoice_id.clone());
        let mut requests = Self::get_invoice_requests(env, invoice_id);
        requests.push_back(request_id.clone());
        env.storage().instance().set(&key, &requests);
    }

    /// Remove request from pending requests list
    pub fn remove_from_pending_requests(env: &Env, request_id: &BytesN<32>) {
        let key = symbol_short!("pend_req");
        let requests = Self::get_pending_requests(env);

        // Find and remove the request ID
        let mut new_requests = Vec::new(env);
        for id in requests.iter() {
            if id != *request_id {
                new_requests.push_back(id);
            }
        }

        env.storage().instance().set(&key, &new_requests);
    }

    /// Store priority fee structure
    pub fn store_fee_structure(env: &Env, fee_structure: &PriorityFeeStructure) {
        let key = (symbol_short!("fee_str"), fee_structure.priority_level.clone());
        env.storage().instance().set(&key, fee_structure);
    }

    /// Get priority fee structure
    pub fn get_fee_structure(env: &Env, priority_level: &PriorityLevel) -> Option<PriorityFeeStructure> {
        let key = (symbol_short!("fee_str"), priority_level.clone());
        env.storage().instance().get(&key)
    }

    /// Store priority bid criteria
    pub fn store_bid_criteria(env: &Env, criteria: &PriorityBidCriteria) {
        let key = (symbol_short!("bid_crit"), criteria.invoice_id.clone());
        env.storage().instance().set(&key, criteria);
    }

    /// Get priority bid criteria for an invoice
    pub fn get_bid_criteria(env: &Env, invoice_id: &BytesN<32>) -> Option<PriorityBidCriteria> {
        let key = (symbol_short!("bid_crit"), invoice_id.clone());
        env.storage().instance().get(&key)
    }

    /// Get invoices by priority level
    pub fn get_invoices_by_priority(env: &Env, priority_level: &PriorityLevel) -> Vec<BytesN<32>> {
        let key = (symbol_short!("inv_pri"), priority_level.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Add invoice to priority list
    pub fn add_to_priority_list(env: &Env, priority_level: &PriorityLevel, invoice_id: &BytesN<32>) {
        let key = (symbol_short!("inv_pri"), priority_level.clone());
        let mut invoices = Self::get_invoices_by_priority(env, priority_level);
        invoices.push_back(invoice_id.clone());
        env.storage().instance().set(&key, &invoices);
    }

    /// Remove invoice from priority list
    pub fn remove_from_priority_list(env: &Env, priority_level: &PriorityLevel, invoice_id: &BytesN<32>) {
        let key = (symbol_short!("inv_pri"), priority_level.clone());
        let invoices = Self::get_invoices_by_priority(env, priority_level);

        // Find and remove the invoice ID
        let mut new_invoices = Vec::new(env);
        for id in invoices.iter() {
            if id != *invoice_id {
                new_invoices.push_back(id);
            }
        }

        env.storage().instance().set(&key, &new_invoices);
    }

    /// Get invoices by urgency level
    pub fn get_invoices_by_urgency(env: &Env, urgency_level: &UrgencyLevel) -> Vec<BytesN<32>> {
        let key = (symbol_short!("inv_urg"), urgency_level.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Add invoice to urgency list
    pub fn add_to_urgency_list(env: &Env, urgency_level: &UrgencyLevel, invoice_id: &BytesN<32>) {
        let key = (symbol_short!("inv_urg"), urgency_level.clone());
        let mut invoices = Self::get_invoices_by_urgency(env, urgency_level);
        invoices.push_back(invoice_id.clone());
        env.storage().instance().set(&key, &invoices);
    }

    /// Remove invoice from urgency list
    pub fn remove_from_urgency_list(env: &Env, urgency_level: &UrgencyLevel, invoice_id: &BytesN<32>) {
        let key = (symbol_short!("inv_urg"), urgency_level.clone());
        let invoices = Self::get_invoices_by_urgency(env, urgency_level);

        // Find and remove the invoice ID
        let mut new_invoices = Vec::new(env);
        for id in invoices.iter() {
            if id != *invoice_id {
                new_invoices.push_back(id);
            }
        }

        env.storage().instance().set(&key, &new_invoices);
    }
}