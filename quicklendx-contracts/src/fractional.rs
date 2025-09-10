use soroban_sdk::{contracttype, symbol_short, vec, Address, BytesN, Env, String, Vec};

/// Fractional investment status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FractionalInvestmentStatus {
    Pending,    // Investment pending
    Active,     // Investment active
    Completed,  // Investment completed
    Withdrawn,  // Investment withdrawn
    Defaulted,  // Investment defaulted
}

/// Fractional investment structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct FractionalInvestment {
    pub id: BytesN<32>,              // Unique investment ID
    pub invoice_id: BytesN<32>,      // Associated invoice ID
    pub investor: Address,           // Investor address
    pub amount: i128,                // Investment amount
    pub percentage: i128,            // Percentage of total invoice (in basis points)
    pub expected_return: i128,       // Expected return amount
    pub funded_at: u64,              // When the investment was funded
    pub status: FractionalInvestmentStatus, // Investment status
    pub withdrawal_requested_at: Option<u64>, // When withdrawal was requested
    pub withdrawal_completed_at: Option<u64>, // When withdrawal was completed
    pub profit_share: i128,          // Actual profit share received
    pub profit_share_paid_at: Option<u64>, // When profit share was paid
}

/// Funding deadline structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct FundingDeadline {
    pub invoice_id: BytesN<32>,      // Associated invoice ID
    pub deadline: u64,               // Funding deadline timestamp
    pub minimum_funding_amount: i128, // Minimum amount needed to proceed
    pub current_funding_amount: i128, // Current total funding amount
    pub is_extended: bool,           // Whether deadline has been extended
    pub extension_count: u32,        // Number of extensions
    pub max_extensions: u32,         // Maximum allowed extensions
}

/// Investment withdrawal request
#[contracttype]
#[derive(Clone, Debug)]
pub struct WithdrawalRequest {
    pub id: BytesN<32>,              // Unique request ID
    pub investment_id: BytesN<32>,   // Associated investment ID
    pub investor: Address,           // Investor requesting withdrawal
    pub amount: i128,                // Amount to withdraw
    pub reason: String,              // Reason for withdrawal
    pub requested_at: u64,           // Request timestamp
    pub status: WithdrawalStatus,    // Request status
    pub processed_at: Option<u64>,   // When request was processed
    pub processed_by: Option<Address>, // Who processed the request
}

/// Withdrawal request status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WithdrawalStatus {
    Pending,     // Request pending
    Approved,    // Request approved
    Rejected,    // Request rejected
    Cancelled,   // Request cancelled
}

/// Partial funding status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PartialFundingStatus {
    NotStarted,      // No funding started
    InProgress,      // Funding in progress
    FullyFunded,     // Fully funded
    PartiallyFunded, // Partially funded but deadline passed
    Cancelled,       // Funding cancelled
}

use crate::errors::QuickLendXError;

impl FractionalInvestment {
    /// Create a new fractional investment
    pub fn new(
        env: &Env,
        invoice_id: BytesN<32>,
        investor: Address,
        amount: i128,
        total_invoice_amount: i128,
        expected_return: i128,
    ) -> Result<Self, QuickLendXError> {
        if amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        if total_invoice_amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        if expected_return <= amount {
            return Err(QuickLendXError::InvalidAmount);
        }

        let id = Self::generate_unique_investment_id(env);
        let funded_at = env.ledger().timestamp();

        // Calculate percentage (in basis points, 10000 = 100%)
        let percentage = (amount * 10000) / total_invoice_amount;

        Ok(Self {
            id,
            invoice_id,
            investor,
            amount,
            percentage,
            expected_return,
            funded_at,
            status: FractionalInvestmentStatus::Active,
            withdrawal_requested_at: None,
            withdrawal_completed_at: None,
            profit_share: 0,
            profit_share_paid_at: None,
        })
    }

    /// Generate a unique investment ID
    fn generate_unique_investment_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let counter_key = symbol_short!("frac_cnt");
        let counter: u32 = env.storage().instance().get(&counter_key).unwrap_or(0);
        env.storage().instance().set(&counter_key, &(counter + 1));

        // Create a unique ID from timestamp, sequence, and counter
        let mut id_bytes = [0u8; 32];
        id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        id_bytes[8..12].copy_from_slice(&sequence.to_be_bytes());
        id_bytes[12..16].copy_from_slice(&counter.to_be_bytes());

        BytesN::from_array(env, &id_bytes)
    }

    /// Request withdrawal
    pub fn request_withdrawal(&mut self, env: &Env) -> Result<(), QuickLendXError> {
        if self.status != FractionalInvestmentStatus::Active {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.withdrawal_requested_at = Some(env.ledger().timestamp());
        Ok(())
    }

    /// Complete withdrawal
    pub fn complete_withdrawal(&mut self, env: &Env) -> Result<(), QuickLendXError> {
        if self.status != FractionalInvestmentStatus::Active {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = FractionalInvestmentStatus::Withdrawn;
        self.withdrawal_completed_at = Some(env.ledger().timestamp());
        Ok(())
    }

    /// Mark as defaulted
    pub fn mark_defaulted(&mut self) {
        self.status = FractionalInvestmentStatus::Defaulted;
    }

    /// Complete investment
    pub fn complete_investment(&mut self, env: &Env, profit_share: i128) -> Result<(), QuickLendXError> {
        if self.status != FractionalInvestmentStatus::Active {
            return Err(QuickLendXError::InvalidStatus);
        }

        if profit_share < 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        self.status = FractionalInvestmentStatus::Completed;
        self.profit_share = profit_share;
        self.profit_share_paid_at = Some(env.ledger().timestamp());
        Ok(())
    }

    /// Calculate profit share based on total profit
    pub fn calculate_profit_share(&self, total_profit: i128) -> i128 {
        (total_profit * self.percentage) / 10000
    }
}

impl FundingDeadline {
    /// Create a new funding deadline
    pub fn new(
        env: &Env,
        invoice_id: BytesN<32>,
        deadline: u64,
        minimum_funding_amount: i128,
        max_extensions: u32,
    ) -> Result<Self, QuickLendXError> {
        let current_timestamp = env.ledger().timestamp();
        if deadline <= current_timestamp {
            return Err(QuickLendXError::InvalidTimestamp);
        }

        if minimum_funding_amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        Ok(Self {
            invoice_id,
            deadline,
            minimum_funding_amount,
            current_funding_amount: 0,
            is_extended: false,
            extension_count: 0,
            max_extensions,
        })
    }

    /// Add funding to the deadline
    pub fn add_funding(&mut self, amount: i128) -> Result<(), QuickLendXError> {
        if amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        self.current_funding_amount += amount;
        Ok(())
    }

    /// Check if funding deadline has passed
    pub fn is_deadline_passed(&self, current_timestamp: u64) -> bool {
        current_timestamp > self.deadline
    }

    /// Check if minimum funding is met
    pub fn is_minimum_funding_met(&self) -> bool {
        self.current_funding_amount >= self.minimum_funding_amount
    }

    /// Check if fully funded
    pub fn is_fully_funded(&self, total_invoice_amount: i128) -> bool {
        self.current_funding_amount >= total_invoice_amount
    }

    /// Extend deadline
    pub fn extend_deadline(&mut self, new_deadline: u64, current_timestamp: u64) -> Result<(), QuickLendXError> {
        if self.extension_count >= self.max_extensions {
            return Err(QuickLendXError::OperationNotAllowed);
        }

        if new_deadline <= current_timestamp {
            return Err(QuickLendXError::InvalidTimestamp);
        }

        self.deadline = new_deadline;
        self.is_extended = true;
        self.extension_count += 1;
        Ok(())
    }

    /// Get funding status
    pub fn get_funding_status(&self, total_invoice_amount: i128, current_timestamp: u64) -> PartialFundingStatus {
        if self.current_funding_amount == 0 {
            return PartialFundingStatus::NotStarted;
        }

        if self.is_fully_funded(total_invoice_amount) {
            return PartialFundingStatus::FullyFunded;
        }

        if self.is_deadline_passed(current_timestamp) {
            if self.is_minimum_funding_met() {
                return PartialFundingStatus::PartiallyFunded;
            } else {
                return PartialFundingStatus::Cancelled;
            }
        }

        PartialFundingStatus::InProgress
    }
}

impl WithdrawalRequest {
    /// Create a new withdrawal request
    pub fn new(
        env: &Env,
        investment_id: BytesN<32>,
        investor: Address,
        amount: i128,
        reason: String,
    ) -> Result<Self, QuickLendXError> {
        if amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        if reason.len() == 0 {
            return Err(QuickLendXError::InvalidDescription);
        }

        let id = Self::generate_unique_request_id(env);
        let requested_at = env.ledger().timestamp();

        Ok(Self {
            id,
            investment_id,
            investor,
            amount,
            reason,
            requested_at,
            status: WithdrawalStatus::Pending,
            processed_at: None,
            processed_by: None,
        })
    }

    /// Generate a unique request ID
    fn generate_unique_request_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let counter_key = symbol_short!("with_cnt");
        let counter: u32 = env.storage().instance().get(&counter_key).unwrap_or(0);
        env.storage().instance().set(&counter_key, &(counter + 1));

        // Create a unique ID from timestamp, sequence, and counter
        let mut id_bytes = [0u8; 32];
        id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        id_bytes[8..12].copy_from_slice(&sequence.to_be_bytes());
        id_bytes[12..16].copy_from_slice(&counter.to_be_bytes());

        BytesN::from_array(env, &id_bytes)
    }

    /// Approve withdrawal request
    pub fn approve(&mut self, env: &Env, approver: Address) -> Result<(), QuickLendXError> {
        if self.status != WithdrawalStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = WithdrawalStatus::Approved;
        self.processed_at = Some(env.ledger().timestamp());
        self.processed_by = Some(approver);
        Ok(())
    }

    /// Reject withdrawal request
    pub fn reject(&mut self, env: &Env, rejector: Address) -> Result<(), QuickLendXError> {
        if self.status != WithdrawalStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        self.status = WithdrawalStatus::Rejected;
        self.processed_at = Some(env.ledger().timestamp());
        self.processed_by = Some(rejector);
        Ok(())
    }

    /// Cancel withdrawal request
    pub fn cancel(&mut self, env: &Env, canceller: Address) -> Result<(), QuickLendXError> {
        if self.status != WithdrawalStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }

        // Only the investor can cancel their own request
        if self.investor != canceller {
            return Err(QuickLendXError::Unauthorized);
        }

        self.status = WithdrawalStatus::Cancelled;
        self.processed_at = Some(env.ledger().timestamp());
        self.processed_by = Some(canceller);
        Ok(())
    }
}

/// Storage keys for fractional investment data
pub struct FractionalStorage;

impl FractionalStorage {
    /// Store a fractional investment
    pub fn store_investment(env: &Env, investment: &FractionalInvestment) {
        env.storage().instance().set(&investment.id, investment);

        // Add to invoice investments list
        Self::add_to_invoice_investments(env, &investment.invoice_id, &investment.id);

        // Add to investor investments list
        Self::add_to_investor_investments(env, &investment.investor, &investment.id);

        // Add to status investments list
        Self::add_to_status_investments(env, &investment.status, &investment.id);
    }

    /// Get a fractional investment by ID
    pub fn get_investment(env: &Env, investment_id: &BytesN<32>) -> Option<FractionalInvestment> {
        env.storage().instance().get(investment_id)
    }

    /// Update a fractional investment
    pub fn update_investment(env: &Env, investment: &FractionalInvestment) {
        env.storage().instance().set(&investment.id, investment);
    }

    /// Get all investments for an invoice
    pub fn get_invoice_investments(env: &Env, invoice_id: &BytesN<32>) -> Vec<BytesN<32>> {
        let key = (symbol_short!("inv_inv"), invoice_id.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get all investments for an investor
    pub fn get_investor_investments(env: &Env, investor: &Address) -> Vec<BytesN<32>> {
        let key = (symbol_short!("inv_inv"), investor.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get all investments by status
    pub fn get_investments_by_status(env: &Env, status: &FractionalInvestmentStatus) -> Vec<BytesN<32>> {
        let key = match status {
            FractionalInvestmentStatus::Pending => symbol_short!("inv_pend"),
            FractionalInvestmentStatus::Active => symbol_short!("inv_act"),
            FractionalInvestmentStatus::Completed => symbol_short!("inv_comp"),
            FractionalInvestmentStatus::Withdrawn => symbol_short!("inv_with"),
            FractionalInvestmentStatus::Defaulted => symbol_short!("inv_def"),
        };
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Add investment to invoice investments list
    fn add_to_invoice_investments(env: &Env, invoice_id: &BytesN<32>, investment_id: &BytesN<32>) {
        let key = (symbol_short!("inv_inv"), invoice_id.clone());
        let mut investments = Self::get_invoice_investments(env, invoice_id);
        investments.push_back(investment_id.clone());
        env.storage().instance().set(&key, &investments);
    }

    /// Add investment to investor investments list
    fn add_to_investor_investments(env: &Env, investor: &Address, investment_id: &BytesN<32>) {
        let key = (symbol_short!("inv_inv"), investor.clone());
        let mut investments = Self::get_investor_investments(env, investor);
        investments.push_back(investment_id.clone());
        env.storage().instance().set(&key, &investments);
    }

    /// Add investment to status investments list
    fn add_to_status_investments(env: &Env, status: &FractionalInvestmentStatus, investment_id: &BytesN<32>) {
        let key = match status {
            FractionalInvestmentStatus::Pending => symbol_short!("inv_pend"),
            FractionalInvestmentStatus::Active => symbol_short!("inv_act"),
            FractionalInvestmentStatus::Completed => symbol_short!("inv_comp"),
            FractionalInvestmentStatus::Withdrawn => symbol_short!("inv_with"),
            FractionalInvestmentStatus::Defaulted => symbol_short!("inv_def"),
        };
        let mut investments = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        investments.push_back(investment_id.clone());
        env.storage().instance().set(&key, &investments);
    }

    /// Remove investment from status investments list
    pub fn remove_from_status_investments(env: &Env, status: &FractionalInvestmentStatus, investment_id: &BytesN<32>) {
        let key = match status {
            FractionalInvestmentStatus::Pending => symbol_short!("inv_pend"),
            FractionalInvestmentStatus::Active => symbol_short!("inv_act"),
            FractionalInvestmentStatus::Completed => symbol_short!("inv_comp"),
            FractionalInvestmentStatus::Withdrawn => symbol_short!("inv_with"),
            FractionalInvestmentStatus::Defaulted => symbol_short!("inv_def"),
        };
        let investments = Self::get_investments_by_status(env, status);

        // Find and remove the investment ID
        let mut new_investments = Vec::new(env);
        for id in investments.iter() {
            if id != *investment_id {
                new_investments.push_back(id);
            }
        }

        env.storage().instance().set(&key, &new_investments);
    }

    /// Store funding deadline
    pub fn store_funding_deadline(env: &Env, deadline: &FundingDeadline) {
        let key = (symbol_short!("fund_dead"), deadline.invoice_id.clone());
        env.storage().instance().set(&key, deadline);
    }

    /// Get funding deadline for an invoice
    pub fn get_funding_deadline(env: &Env, invoice_id: &BytesN<32>) -> Option<FundingDeadline> {
        let key = (symbol_short!("fund_dead"), invoice_id.clone());
        env.storage().instance().get(&key)
    }

    /// Update funding deadline
    pub fn update_funding_deadline(env: &Env, deadline: &FundingDeadline) {
        let key = (symbol_short!("fund_dead"), deadline.invoice_id.clone());
        env.storage().instance().set(&key, deadline);
    }

    /// Store withdrawal request
    pub fn store_withdrawal_request(env: &Env, request: &WithdrawalRequest) {
        env.storage().instance().set(&request.id, request);

        // Add to pending requests list
        Self::add_to_pending_withdrawals(env, &request.id);
    }

    /// Get withdrawal request by ID
    pub fn get_withdrawal_request(env: &Env, request_id: &BytesN<32>) -> Option<WithdrawalRequest> {
        env.storage().instance().get(request_id)
    }

    /// Update withdrawal request
    pub fn update_withdrawal_request(env: &Env, request: &WithdrawalRequest) {
        env.storage().instance().set(&request.id, request);
    }

    /// Get all pending withdrawal requests
    pub fn get_pending_withdrawals(env: &Env) -> Vec<BytesN<32>> {
        let key = symbol_short!("pend_with");
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Add request to pending withdrawals list
    fn add_to_pending_withdrawals(env: &Env, request_id: &BytesN<32>) {
        let key = symbol_short!("pend_with");
        let mut requests = Self::get_pending_withdrawals(env);
        requests.push_back(request_id.clone());
        env.storage().instance().set(&key, &requests);
    }

    /// Remove request from pending withdrawals list
    pub fn remove_from_pending_withdrawals(env: &Env, request_id: &BytesN<32>) {
        let key = symbol_short!("pend_with");
        let requests = Self::get_pending_withdrawals(env);

        // Find and remove the request ID
        let mut new_requests = Vec::new(env);
        for id in requests.iter() {
            if id != *request_id {
                new_requests.push_back(id);
            }
        }

        env.storage().instance().set(&key, &new_requests);
    }

    /// Get total funding amount for an invoice
    pub fn get_total_funding_amount(env: &Env, invoice_id: &BytesN<32>) -> i128 {
        let investments = Self::get_invoice_investments(env, invoice_id);
        let mut total = 0i128;

        for investment_id in investments.iter() {
            if let Some(investment) = Self::get_investment(env, &investment_id) {
                if investment.status == FractionalInvestmentStatus::Active {
                    total += investment.amount;
                }
            }
        }

        total
    }

    /// Get investor count for an invoice
    pub fn get_investor_count(env: &Env, invoice_id: &BytesN<32>) -> u32 {
        let investments = Self::get_invoice_investments(env, invoice_id);
        let mut count = 0u32;

        for investment_id in investments.iter() {
            if let Some(investment) = Self::get_investment(env, &investment_id) {
                if investment.status == FractionalInvestmentStatus::Active {
                    count += 1;
                }
            }
        }

        count
    }

    /// Check if minimum investment amount is met
    pub fn is_minimum_investment_met(env: &Env, invoice_id: &BytesN<32>, minimum_amount: i128) -> bool {
        let total_funding = Self::get_total_funding_amount(env, invoice_id);
        total_funding >= minimum_amount
    }

    /// Get investments above minimum amount
    pub fn get_investments_above_minimum(env: &Env, minimum_amount: i128) -> Vec<BytesN<32>> {
        let mut filtered_investments = vec![env];
        let active_investments = Self::get_investments_by_status(env, &FractionalInvestmentStatus::Active);

        for investment_id in active_investments.iter() {
            if let Some(investment) = Self::get_investment(env, &investment_id) {
                if investment.amount >= minimum_amount {
                    filtered_investments.push_back(investment_id);
                }
            }
        }

        filtered_investments
    }
}