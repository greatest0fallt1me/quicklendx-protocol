use soroban_sdk::{contracttype, symbol_short, vec, Address, BytesN, Env, String, Vec};

/// Collateral type enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CollateralType {
    Cash,        // Cash collateral (XLM or other tokens)
    Assets,      // Physical or digital assets
    Guarantees,  // Bank guarantees or insurance
}

/// Collateral status enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CollateralStatus {
    Pending,     // Collateral submitted, awaiting validation
    Validated,   // Collateral validated and accepted
    Released,    // Collateral released back to business
    Forfeited,   // Collateral forfeited due to default
    Expired,     // Collateral expired
}

/// Collateral structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct Collateral {
    pub id: BytesN<32>,              // Unique collateral identifier
    pub invoice_id: BytesN<32>,      // Associated invoice ID
    pub business: Address,           // Business providing collateral
    pub collateral_type: CollateralType, // Type of collateral
    pub amount: i128,                // Collateral amount
    pub currency: Address,           // Currency of collateral
    pub description: String,         // Description of collateral
    pub status: CollateralStatus,    // Current status
    pub created_at: u64,             // Creation timestamp
    pub validated_at: Option<u64>,   // Validation timestamp
    pub released_at: Option<u64>,    // Release timestamp
    pub forfeited_at: Option<u64>,   // Forfeiture timestamp
    pub expires_at: Option<u64>,     // Expiration timestamp
    pub risk_score: u32,             // Risk score (1-100, lower is better)
    pub validation_notes: String,    // Notes from validation process
}

/// Collateral transfer structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct CollateralTransfer {
    pub id: BytesN<32>,              // Unique transfer identifier
    pub collateral_id: BytesN<32>,   // Collateral being transferred
    pub from_address: Address,       // Address transferring from
    pub to_address: Address,         // Address transferring to
    pub amount: i128,                // Amount being transferred
    pub reason: String,              // Reason for transfer
    pub timestamp: u64,              // Transfer timestamp
    pub status: CollateralTransferStatus, // Transfer status
}

/// Collateral transfer status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CollateralTransferStatus {
    Pending,     // Transfer initiated
    Completed,   // Transfer completed
    Failed,      // Transfer failed
    Cancelled,   // Transfer cancelled
}

/// Risk assessment structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct RiskAssessment {
    pub invoice_id: BytesN<32>,      // Associated invoice ID
    pub base_risk_score: u32,        // Base risk score (1-100)
    pub collateral_risk_score: u32,  // Collateral-adjusted risk score
    pub business_risk_score: u32,    // Business risk score
    pub final_risk_score: u32,       // Final calculated risk score
    pub risk_factors: Vec<String>,   // Risk factors identified
    pub assessment_date: u64,        // Assessment timestamp
    pub assessed_by: Address,        // Address of assessor
}

use crate::errors::QuickLendXError;

impl Collateral {
    /// Create a new collateral entry
    pub fn new(
        env: &Env,
        invoice_id: BytesN<32>,
        business: Address,
        collateral_type: CollateralType,
        amount: i128,
        currency: Address,
        description: String,
        expires_at: Option<u64>,
    ) -> Result<Self, QuickLendXError> {
        // Validate input parameters
        if amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        if description.len() == 0 {
            return Err(QuickLendXError::InvalidDescription);
        }

        // Check expiration date if provided
        let current_timestamp = env.ledger().timestamp();
        if let Some(expiry) = expires_at {
            if expiry <= current_timestamp {
                return Err(QuickLendXError::InvalidExpirationDate);
            }
        }

        let id = Self::generate_unique_collateral_id(env);
        let created_at = current_timestamp;

        // Calculate initial risk score based on collateral type
        let risk_score = Self::calculate_initial_risk_score(&collateral_type);

        Ok(Self {
            id,
            invoice_id,
            business,
            collateral_type,
            amount,
            currency,
            description,
            status: CollateralStatus::Pending,
            created_at,
            validated_at: None,
            released_at: None,
            forfeited_at: None,
            expires_at,
            risk_score,
            validation_notes: String::from_str(env, ""),
        })
    }

    /// Generate a unique collateral ID
    fn generate_unique_collateral_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let counter_key = symbol_short!("coll_cnt");
        let counter: u32 = env.storage().instance().get(&counter_key).unwrap_or(0);
        env.storage().instance().set(&counter_key, &(counter + 1));

        // Create a unique ID from timestamp, sequence, and counter
        let mut id_bytes = [0u8; 32];
        id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        id_bytes[8..12].copy_from_slice(&sequence.to_be_bytes());
        id_bytes[12..16].copy_from_slice(&counter.to_be_bytes());

        BytesN::from_array(env, &id_bytes)
    }

    /// Calculate initial risk score based on collateral type
    fn calculate_initial_risk_score(collateral_type: &CollateralType) -> u32 {
        match collateral_type {
            CollateralType::Cash => 20,      // Cash is lowest risk
            CollateralType::Assets => 50,    // Assets are medium risk
            CollateralType::Guarantees => 30, // Guarantees are low-medium risk
        }
    }

    /// Validate collateral
    pub fn validate(&mut self, env: &Env, validator: Address, notes: String) -> Result<(), QuickLendXError> {
        if self.status != CollateralStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = CollateralStatus::Validated;
        self.validated_at = Some(env.ledger().timestamp());
        self.validation_notes = notes;

        Ok(())
    }

    /// Release collateral back to business
    pub fn release(&mut self, env: &Env) -> Result<(), QuickLendXError> {
        if self.status != CollateralStatus::Validated {
            return Err(QuickLendXError::InvalidStatus);
        }

        // Check if collateral has expired
        if let Some(expiry) = self.expires_at {
            if env.ledger().timestamp() > expiry {
                return Err(QuickLendXError::CollateralExpired);
            }
        }

        self.status = CollateralStatus::Released;
        self.released_at = Some(env.ledger().timestamp());

        Ok(())
    }

    /// Forfeit collateral due to default
    pub fn forfeit(&mut self, env: &Env) -> Result<(), QuickLendXError> {
        if self.status != CollateralStatus::Validated {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = CollateralStatus::Forfeited;
        self.forfeited_at = Some(env.ledger().timestamp());

        Ok(())
    }

    /// Check if collateral is expired
    pub fn is_expired(&self, current_timestamp: u64) -> bool {
        if let Some(expiry) = self.expires_at {
            current_timestamp > expiry
        } else {
            false
        }
    }

    /// Update risk score
    pub fn update_risk_score(&mut self, new_score: u32) -> Result<(), QuickLendXError> {
        if new_score < 1 || new_score > 100 {
            return Err(QuickLendXError::InvalidRiskScore);
        }

        self.risk_score = new_score;
        Ok(())
    }
}

impl CollateralTransfer {
    /// Create a new collateral transfer
    pub fn new(
        env: &Env,
        collateral_id: BytesN<32>,
        from_address: Address,
        to_address: Address,
        amount: i128,
        reason: String,
    ) -> Result<Self, QuickLendXError> {
        if amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        if reason.len() == 0 {
            return Err(QuickLendXError::InvalidDescription);
        }

        let id = Self::generate_unique_transfer_id(env);
        let timestamp = env.ledger().timestamp();

        Ok(Self {
            id,
            collateral_id,
            from_address,
            to_address,
            amount,
            reason,
            timestamp,
            status: CollateralTransferStatus::Pending,
        })
    }

    /// Generate a unique transfer ID
    fn generate_unique_transfer_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let counter_key = symbol_short!("trans_cnt");
        let counter: u32 = env.storage().instance().get(&counter_key).unwrap_or(0);
        env.storage().instance().set(&counter_key, &(counter + 1));

        // Create a unique ID from timestamp, sequence, and counter
        let mut id_bytes = [0u8; 32];
        id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        id_bytes[8..12].copy_from_slice(&sequence.to_be_bytes());
        id_bytes[12..16].copy_from_slice(&counter.to_be_bytes());

        BytesN::from_array(env, &id_bytes)
    }

    /// Complete the transfer
    pub fn complete(&mut self) -> Result<(), QuickLendXError> {
        if self.status != CollateralTransferStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = CollateralTransferStatus::Completed;
        Ok(())
    }

    /// Cancel the transfer
    pub fn cancel(&mut self) -> Result<(), QuickLendXError> {
        if self.status != CollateralTransferStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = CollateralTransferStatus::Cancelled;
        Ok(())
    }

    /// Mark transfer as failed
    pub fn mark_failed(&mut self) -> Result<(), QuickLendXError> {
        if self.status != CollateralTransferStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = CollateralTransferStatus::Failed;
        Ok(())
    }
}

impl RiskAssessment {
    /// Create a new risk assessment
    pub fn new(
        env: &Env,
        invoice_id: BytesN<32>,
        base_risk_score: u32,
        business_risk_score: u32,
        risk_factors: Vec<String>,
        assessed_by: Address,
    ) -> Result<Self, QuickLendXError> {
        if base_risk_score < 1 || base_risk_score > 100 {
            return Err(QuickLendXError::InvalidRiskScore);
        }

        if business_risk_score < 1 || business_risk_score > 100 {
            return Err(QuickLendXError::InvalidRiskScore);
        }

        // Calculate collateral-adjusted risk score (weighted average)
        let collateral_risk_score = (base_risk_score + business_risk_score) / 2;
        
        // Calculate final risk score with additional factors
        let final_risk_score = Self::calculate_final_risk_score(
            base_risk_score,
            collateral_risk_score,
            business_risk_score,
            &risk_factors,
        );

        Ok(Self {
            invoice_id,
            base_risk_score,
            collateral_risk_score,
            business_risk_score,
            final_risk_score,
            risk_factors,
            assessment_date: env.ledger().timestamp(),
            assessed_by,
        })
    }

    /// Calculate final risk score based on all factors
    fn calculate_final_risk_score(
        base_risk: u32,
        collateral_risk: u32,
        business_risk: u32,
        risk_factors: &Vec<String>,
    ) -> u32 {
        // Weighted average: 40% base, 30% collateral, 30% business
        let weighted_score = (base_risk * 4 + collateral_risk * 3 + business_risk * 3) / 10;
        
        // Adjust for additional risk factors
        let factor_adjustment = risk_factors.len() as u32 * 5; // 5 points per risk factor
        
        let final_score = weighted_score + factor_adjustment;
        
        // Cap at 100
        if final_score > 100 {
            100
        } else {
            final_score
        }
    }

    /// Update risk assessment
    pub fn update_assessment(
        &mut self,
        new_base_risk: u32,
        new_business_risk: u32,
        new_risk_factors: Vec<String>,
        assessed_by: Address,
    ) -> Result<(), QuickLendXError> {
        if new_base_risk < 1 || new_base_risk > 100 {
            return Err(QuickLendXError::InvalidRiskScore);
        }

        if new_business_risk < 1 || new_business_risk > 100 {
            return Err(QuickLendXError::InvalidRiskScore);
        }

        self.base_risk_score = new_base_risk;
        self.business_risk_score = new_business_risk;
        self.risk_factors = new_risk_factors;
        self.assessed_by = assessed_by;
        self.assessment_date = self.assessment_date; // Keep original date

        // Recalculate scores
        self.collateral_risk_score = (new_base_risk + new_business_risk) / 2;
        self.final_risk_score = Self::calculate_final_risk_score(
            new_base_risk,
            self.collateral_risk_score,
            new_business_risk,
            &self.risk_factors,
        );

        Ok(())
    }
}

/// Storage keys for collateral data
pub struct CollateralStorage;

impl CollateralStorage {
    /// Store a collateral entry
    pub fn store_collateral(env: &Env, collateral: &Collateral) {
        env.storage().instance().set(&collateral.id, collateral);

        // Add to invoice collaterals list
        Self::add_to_invoice_collaterals(env, &collateral.invoice_id, &collateral.id);

        // Add to business collaterals list
        Self::add_to_business_collaterals(env, &collateral.business, &collateral.id);

        // Add to status collaterals list
        Self::add_to_status_collaterals(env, &collateral.status, &collateral.id);
    }

    /// Get a collateral by ID
    pub fn get_collateral(env: &Env, collateral_id: &BytesN<32>) -> Option<Collateral> {
        env.storage().instance().get(collateral_id)
    }

    /// Update a collateral entry
    pub fn update_collateral(env: &Env, collateral: &Collateral) {
        env.storage().instance().set(&collateral.id, collateral);
    }

    /// Get all collaterals for an invoice
    pub fn get_invoice_collaterals(env: &Env, invoice_id: &BytesN<32>) -> Vec<BytesN<32>> {
        let key = (symbol_short!("inv_coll"), invoice_id.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get all collaterals for a business
    pub fn get_business_collaterals(env: &Env, business: &Address) -> Vec<BytesN<32>> {
        let key = (symbol_short!("bus_coll"), business.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get all collaterals by status
    pub fn get_collaterals_by_status(env: &Env, status: &CollateralStatus) -> Vec<BytesN<32>> {
        let key = match status {
            CollateralStatus::Pending => symbol_short!("col_pend"),
            CollateralStatus::Validated => symbol_short!("col_val"),
            CollateralStatus::Released => symbol_short!("col_rel"),
            CollateralStatus::Forfeited => symbol_short!("col_forf"),
            CollateralStatus::Expired => symbol_short!("col_exp"),
        };
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Add collateral to invoice collaterals list
    fn add_to_invoice_collaterals(env: &Env, invoice_id: &BytesN<32>, collateral_id: &BytesN<32>) {
        let key = (symbol_short!("inv_coll"), invoice_id.clone());
        let mut collaterals = Self::get_invoice_collaterals(env, invoice_id);
        collaterals.push_back(collateral_id.clone());
        env.storage().instance().set(&key, &collaterals);
    }

    /// Add collateral to business collaterals list
    fn add_to_business_collaterals(env: &Env, business: &Address, collateral_id: &BytesN<32>) {
        let key = (symbol_short!("bus_coll"), business.clone());
        let mut collaterals = Self::get_business_collaterals(env, business);
        collaterals.push_back(collateral_id.clone());
        env.storage().instance().set(&key, &collaterals);
    }

    /// Add collateral to status collaterals list
    pub fn add_to_status_collaterals(env: &Env, status: &CollateralStatus, collateral_id: &BytesN<32>) {
        let key = match status {
            CollateralStatus::Pending => symbol_short!("col_pend"),
            CollateralStatus::Validated => symbol_short!("col_val"),
            CollateralStatus::Released => symbol_short!("col_rel"),
            CollateralStatus::Forfeited => symbol_short!("col_forf"),
            CollateralStatus::Expired => symbol_short!("col_exp"),
        };
        let mut collaterals = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        collaterals.push_back(collateral_id.clone());
        env.storage().instance().set(&key, &collaterals);
    }

    /// Remove collateral from status collaterals list
    pub fn remove_from_status_collaterals(env: &Env, status: &CollateralStatus, collateral_id: &BytesN<32>) {
        let key = match status {
            CollateralStatus::Pending => symbol_short!("col_pend"),
            CollateralStatus::Validated => symbol_short!("col_val"),
            CollateralStatus::Released => symbol_short!("col_rel"),
            CollateralStatus::Forfeited => symbol_short!("col_forf"),
            CollateralStatus::Expired => symbol_short!("col_exp"),
        };
        let collaterals = Self::get_collaterals_by_status(env, status);

        // Find and remove the collateral ID
        let mut new_collaterals = Vec::new(env);
        for id in collaterals.iter() {
            if id != *collateral_id {
                new_collaterals.push_back(id);
            }
        }

        env.storage().instance().set(&key, &new_collaterals);
    }

    /// Store a collateral transfer
    pub fn store_transfer(env: &Env, transfer: &CollateralTransfer) {
        env.storage().instance().set(&transfer.id, transfer);
    }

    /// Get a collateral transfer by ID
    pub fn get_transfer(env: &Env, transfer_id: &BytesN<32>) -> Option<CollateralTransfer> {
        env.storage().instance().get(transfer_id)
    }

    /// Store a risk assessment
    pub fn store_risk_assessment(env: &Env, assessment: &RiskAssessment) {
        let key = (symbol_short!("risk"), assessment.invoice_id.clone());
        env.storage().instance().set(&key, assessment);
    }

    /// Get a risk assessment by invoice ID
    pub fn get_risk_assessment(env: &Env, invoice_id: &BytesN<32>) -> Option<RiskAssessment> {
        let key = (symbol_short!("risk"), invoice_id.clone());
        env.storage().instance().get(&key)
    }

    /// Get collaterals by type
    pub fn get_collaterals_by_type(env: &Env, collateral_type: &CollateralType) -> Vec<BytesN<32>> {
        let mut type_collaterals = vec![env];
        let all_statuses = [
            CollateralStatus::Pending,
            CollateralStatus::Validated,
            CollateralStatus::Released,
            CollateralStatus::Forfeited,
            CollateralStatus::Expired,
        ];

        for status in all_statuses.iter() {
            let collaterals = Self::get_collaterals_by_status(env, status);
            for collateral_id in collaterals.iter() {
                if let Some(collateral) = Self::get_collateral(env, &collateral_id) {
                    if collateral.collateral_type == *collateral_type {
                        type_collaterals.push_back(collateral_id);
                    }
                }
            }
        }
        type_collaterals
    }

    /// Get collaterals with risk score below threshold
    pub fn get_low_risk_collaterals(env: &Env, threshold: u32) -> Vec<BytesN<32>> {
        let mut low_risk_collaterals = vec![env];
        let validated_collaterals = Self::get_collaterals_by_status(env, &CollateralStatus::Validated);
        
        for collateral_id in validated_collaterals.iter() {
            if let Some(collateral) = Self::get_collateral(env, &collateral_id) {
                if collateral.risk_score <= threshold {
                    low_risk_collaterals.push_back(collateral_id);
                }
            }
        }
        low_risk_collaterals
    }
}