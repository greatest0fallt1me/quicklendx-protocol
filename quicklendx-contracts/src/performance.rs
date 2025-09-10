use soroban_sdk::{contracttype, symbol_short, vec, Address, BytesN, Env, String, Vec};

/// Business credit score structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct BusinessCreditScore {
    pub business: Address,           // Business address
    pub score: u32,                 // Credit score (300-850)
    pub score_date: u64,            // Date when score was calculated
    pub factors: Vec<String>,       // Factors affecting the score
    pub payment_history: PaymentHistory, // Payment history data
    pub debt_to_income_ratio: i128, // Debt to income ratio (in basis points)
    pub credit_utilization: i128,   // Credit utilization (in basis points)
    pub last_updated: u64,          // Last update timestamp
}

/// Payment history structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct PaymentHistory {
    pub total_payments: u32,        // Total number of payments
    pub on_time_payments: u32,      // Number of on-time payments
    pub late_payments: u32,         // Number of late payments
    pub defaulted_payments: u32,    // Number of defaulted payments
    pub average_delay_days: u32,    // Average delay in days
    pub payment_consistency: i128,  // Payment consistency score (0-10000)
}

/// Invoice performance metrics
#[contracttype]
#[derive(Clone, Debug)]
pub struct InvoicePerformanceMetrics {
    pub invoice_id: BytesN<32>,     // Invoice ID
    pub business: Address,          // Business address
    pub performance_score: u32,     // Performance score (0-100)
    pub payment_timeliness: u32,    // Payment timeliness score (0-100)
    pub amount_accuracy: u32,       // Amount accuracy score (0-100)
    pub communication_quality: u32, // Communication quality score (0-100)
    pub dispute_frequency: u32,     // Dispute frequency (number of disputes)
    pub resolution_time: u32,       // Average dispute resolution time in days
    pub customer_satisfaction: u32, // Customer satisfaction score (0-100)
    pub calculated_at: u64,         // Calculation timestamp
    pub metrics_period: u64,        // Period covered by metrics (in days)
}

/// Risk assessment structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct RiskAssessment {
    pub business: Address,          // Business address
    pub risk_score: u32,            // Risk score (0-100, higher is riskier)
    pub risk_level: RiskLevel,      // Risk level classification
    pub assessment_date: u64,       // Assessment date
    pub risk_factors: Vec<String>,  // Risk factors identified
    pub mitigation_strategies: Vec<String>, // Suggested mitigation strategies
    pub next_assessment_due: u64,   // Next assessment due date
    pub assessed_by: Address,       // Address of assessor
}

/// Risk level enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low = 1,        // Low risk
    Medium = 2,     // Medium risk
    High = 3,       // High risk
    Critical = 4,   // Critical risk
}

/// Performance-based fee adjustment
#[contracttype]
#[derive(Clone, Debug)]
pub struct PerformanceFeeAdjustment {
    pub business: Address,          // Business address
    pub base_fee_bps: i128,         // Base fee in basis points
    pub performance_multiplier: i128, // Performance-based multiplier
    pub risk_adjustment: i128,      // Risk-based adjustment
    pub final_fee_bps: i128,        // Final fee in basis points
    pub effective_date: u64,        // When adjustment becomes effective
    pub review_date: u64,           // Next review date
}

/// Historical performance data
#[contracttype]
#[derive(Clone, Debug)]
pub struct HistoricalPerformance {
    pub business: Address,          // Business address
    pub period_start: u64,          // Period start date
    pub period_end: u64,            // Period end date
    pub total_invoices: u32,        // Total invoices in period
    pub paid_invoices: u32,         // Paid invoices in period
    pub defaulted_invoices: u32,    // Defaulted invoices in period
    pub average_payment_delay: u32, // Average payment delay in days
    pub total_volume: i128,         // Total invoice volume
    pub total_profits: i128,        // Total profits generated
    pub performance_trend: PerformanceTrend, // Performance trend
}

/// Performance trend enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PerformanceTrend {
    Improving,      // Performance is improving
    Stable,         // Performance is stable
    Declining,      // Performance is declining
    Volatile,       // Performance is volatile
}

use crate::errors::QuickLendXError;

impl BusinessCreditScore {
    /// Create a new business credit score
    pub fn new(
        env: &Env,
        business: Address,
        score: u32,
        factors: Vec<String>,
        payment_history: PaymentHistory,
        debt_to_income_ratio: i128,
        credit_utilization: i128,
    ) -> Result<Self, QuickLendXError> {
        if score < 300 || score > 850 {
            return Err(QuickLendXError::InvalidCreditScore);
        }

        if debt_to_income_ratio < 0 || debt_to_income_ratio > 10000 {
            return Err(QuickLendXError::InvalidRatio);
        }

        if credit_utilization < 0 || credit_utilization > 10000 {
            return Err(QuickLendXError::InvalidRatio);
        }

        let current_timestamp = env.ledger().timestamp();

        Ok(Self {
            business,
            score,
            score_date: current_timestamp,
            factors,
            payment_history,
            debt_to_income_ratio,
            credit_utilization,
            last_updated: current_timestamp,
        })
    }

    /// Update credit score
    pub fn update_score(
        &mut self,
        env: &Env,
        new_score: u32,
        new_factors: Vec<String>,
        new_payment_history: PaymentHistory,
        new_debt_to_income: i128,
        new_credit_utilization: i128,
    ) -> Result<(), QuickLendXError> {
        if new_score < 300 || new_score > 850 {
            return Err(QuickLendXError::InvalidCreditScore);
        }

        if new_debt_to_income < 0 || new_debt_to_income > 10000 {
            return Err(QuickLendXError::InvalidRatio);
        }

        if new_credit_utilization < 0 || new_credit_utilization > 10000 {
            return Err(QuickLendXError::InvalidRatio);
        }

        self.score = new_score;
        self.factors = new_factors;
        self.payment_history = new_payment_history;
        self.debt_to_income_ratio = new_debt_to_income;
        self.credit_utilization = new_credit_utilization;
        self.last_updated = env.ledger().timestamp();

        Ok(())
    }

    /// Calculate credit score based on payment history
    pub fn calculate_score_from_history(&self) -> u32 {
        let payment_ratio = if self.payment_history.total_payments > 0 {
            (self.payment_history.on_time_payments as f64 / self.payment_history.total_payments as f64) * 100.0
        } else {
            0.0
        };

        let base_score = (payment_ratio * 3.5) as u32; // Payment history is 35% of score
        let debt_penalty = (self.debt_to_income_ratio / 100) as u32; // Debt penalty
        let utilization_penalty = (self.credit_utilization / 200) as u32; // Utilization penalty

        let calculated_score = 300 + base_score - debt_penalty - utilization_penalty;

        // Ensure score is within valid range
        if calculated_score > 850 {
            850
        } else if calculated_score < 300 {
            300
        } else {
            calculated_score
        }
    }

    /// Get credit score category
    pub fn get_score_category(&self) -> String {
        match self.score {
            750..=850 => "Excellent",
            700..=749 => "Good",
            650..=699 => "Fair",
            600..=649 => "Poor",
            _ => "Very Poor",
        }
    }
}

impl PaymentHistory {
    /// Create new payment history
    pub fn new() -> Self {
        Self {
            total_payments: 0,
            on_time_payments: 0,
            late_payments: 0,
            defaulted_payments: 0,
            average_delay_days: 0,
            payment_consistency: 10000, // Start with perfect consistency
        }
    }

    /// Add a payment record
    pub fn add_payment(&mut self, is_on_time: bool, delay_days: u32, is_defaulted: bool) {
        self.total_payments += 1;

        if is_defaulted {
            self.defaulted_payments += 1;
        } else if is_on_time {
            self.on_time_payments += 1;
        } else {
            self.late_payments += 1;
            self.average_delay_days = ((self.average_delay_days * (self.late_payments - 1)) + delay_days) / self.late_payments;
        }

        // Recalculate payment consistency
        self.payment_consistency = if self.total_payments > 0 {
            (self.on_time_payments * 10000) / self.total_payments
        } else {
            10000
        };
    }

    /// Get payment success rate
    pub fn get_success_rate(&self) -> i128 {
        if self.total_payments == 0 {
            return 10000; // 100% if no payments yet
        }

        let successful_payments = self.on_time_payments + self.late_payments; // Late payments are still successful
        (successful_payments * 10000) / self.total_payments
    }
}

impl InvoicePerformanceMetrics {
    /// Create new invoice performance metrics
    pub fn new(
        env: &Env,
        invoice_id: BytesN<32>,
        business: Address,
        payment_timeliness: u32,
        amount_accuracy: u32,
        communication_quality: u32,
        dispute_frequency: u32,
        resolution_time: u32,
        customer_satisfaction: u32,
        metrics_period: u64,
    ) -> Result<Self, QuickLendXError> {
        // Validate scores are within 0-100 range
        if payment_timeliness > 100 || amount_accuracy > 100 || communication_quality > 100 || customer_satisfaction > 100 {
            return Err(QuickLendXError::InvalidScore);
        }

        // Calculate overall performance score (weighted average)
        let performance_score = (
            payment_timeliness * 30 +      // 30% weight
            amount_accuracy * 25 +         // 25% weight
            communication_quality * 20 +   // 20% weight
            customer_satisfaction * 25     // 25% weight
        ) / 100;

        Ok(Self {
            invoice_id,
            business,
            performance_score,
            payment_timeliness,
            amount_accuracy,
            communication_quality,
            dispute_frequency,
            resolution_time,
            customer_satisfaction,
            calculated_at: env.ledger().timestamp(),
            metrics_period,
        })
    }

    /// Update performance metrics
    pub fn update_metrics(
        &mut self,
        env: &Env,
        payment_timeliness: u32,
        amount_accuracy: u32,
        communication_quality: u32,
        dispute_frequency: u32,
        resolution_time: u32,
        customer_satisfaction: u32,
    ) -> Result<(), QuickLendXError> {
        if payment_timeliness > 100 || amount_accuracy > 100 || communication_quality > 100 || customer_satisfaction > 100 {
            return Err(QuickLendXError::InvalidScore);
        }

        self.payment_timeliness = payment_timeliness;
        self.amount_accuracy = amount_accuracy;
        self.communication_quality = communication_quality;
        self.dispute_frequency = dispute_frequency;
        self.resolution_time = resolution_time;
        self.customer_satisfaction = customer_satisfaction;

        // Recalculate overall performance score
        self.performance_score = (
            payment_timeliness * 30 +
            amount_accuracy * 25 +
            communication_quality * 20 +
            customer_satisfaction * 25
        ) / 100;

        self.calculated_at = env.ledger().timestamp();

        Ok(())
    }

    /// Get performance grade
    pub fn get_performance_grade(&self) -> String {
        match self.performance_score {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        }
    }
}

impl RiskAssessment {
    /// Create new risk assessment
    pub fn new(
        env: &Env,
        business: Address,
        risk_score: u32,
        risk_factors: Vec<String>,
        mitigation_strategies: Vec<String>,
        assessed_by: Address,
    ) -> Result<Self, QuickLendXError> {
        if risk_score > 100 {
            return Err(QuickLendXError::InvalidRiskScore);
        }

        let risk_level = match risk_score {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        let current_timestamp = env.ledger().timestamp();
        let next_assessment = current_timestamp + (90 * 24 * 60 * 60); // 90 days from now

        Ok(Self {
            business,
            risk_score,
            risk_level,
            assessment_date: current_timestamp,
            risk_factors,
            mitigation_strategies,
            next_assessment_due: next_assessment,
            assessed_by,
        })
    }

    /// Update risk assessment
    pub fn update_assessment(
        &mut self,
        env: &Env,
        new_risk_score: u32,
        new_risk_factors: Vec<String>,
        new_mitigation_strategies: Vec<String>,
        assessed_by: Address,
    ) -> Result<(), QuickLendXError> {
        if new_risk_score > 100 {
            return Err(QuickLendXError::InvalidRiskScore);
        }

        self.risk_score = new_risk_score;
        self.risk_level = match new_risk_score {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
        self.risk_factors = new_risk_factors;
        self.mitigation_strategies = new_mitigation_strategies;
        self.assessed_by = assessed_by;
        self.assessment_date = env.ledger().timestamp();
        self.next_assessment_due = env.ledger().timestamp() + (90 * 24 * 60 * 60);

        Ok(())
    }

    /// Check if assessment is due for renewal
    pub fn is_assessment_due(&self, current_timestamp: u64) -> bool {
        current_timestamp >= self.next_assessment_due
    }
}

impl PerformanceFeeAdjustment {
    /// Create new performance fee adjustment
    pub fn new(
        env: &Env,
        business: Address,
        base_fee_bps: i128,
        performance_multiplier: i128,
        risk_adjustment: i128,
    ) -> Result<Self, QuickLendXError> {
        if base_fee_bps < 0 || base_fee_bps > 10000 {
            return Err(QuickLendXError::InvalidFeeStructure);
        }

        if performance_multiplier < 0 {
            return Err(QuickLendXError::InvalidMultiplier);
        }

        // Calculate final fee
        let final_fee_bps = (base_fee_bps * performance_multiplier / 10000) + risk_adjustment;

        // Ensure final fee is within reasonable bounds
        let final_fee_bps = if final_fee_bps < 0 {
            0
        } else if final_fee_bps > 20000 { // Cap at 200%
            20000
        } else {
            final_fee_bps
        };

        let current_timestamp = env.ledger().timestamp();
        let review_date = current_timestamp + (30 * 24 * 60 * 60); // Review in 30 days

        Ok(Self {
            business,
            base_fee_bps,
            performance_multiplier,
            risk_adjustment,
            final_fee_bps,
            effective_date: current_timestamp,
            review_date,
        })
    }

    /// Update fee adjustment
    pub fn update_adjustment(
        &mut self,
        env: &Env,
        new_performance_multiplier: i128,
        new_risk_adjustment: i128,
    ) -> Result<(), QuickLendXError> {
        if new_performance_multiplier < 0 {
            return Err(QuickLendXError::InvalidMultiplier);
        }

        self.performance_multiplier = new_performance_multiplier;
        self.risk_adjustment = new_risk_adjustment;

        // Recalculate final fee
        let final_fee_bps = (self.base_fee_bps * new_performance_multiplier / 10000) + new_risk_adjustment;

        self.final_fee_bps = if final_fee_bps < 0 {
            0
        } else if final_fee_bps > 20000 {
            20000
        } else {
            final_fee_bps
        };

        self.effective_date = env.ledger().timestamp();
        self.review_date = env.ledger().timestamp() + (30 * 24 * 60 * 60);

        Ok(())
    }

    /// Check if adjustment is due for review
    pub fn is_review_due(&self, current_timestamp: u64) -> bool {
        current_timestamp >= self.review_date
    }
}

impl HistoricalPerformance {
    /// Create new historical performance record
    pub fn new(
        env: &Env,
        business: Address,
        period_start: u64,
        period_end: u64,
        total_invoices: u32,
        paid_invoices: u32,
        defaulted_invoices: u32,
        average_payment_delay: u32,
        total_volume: i128,
        total_profits: i128,
    ) -> Result<Self, QuickLendXError> {
        if period_start >= period_end {
            return Err(QuickLendXError::InvalidTimestamp);
        }

        if total_invoices < paid_invoices + defaulted_invoices {
            return Err(QuickLendXError::InvalidAmount);
        }

        // Calculate performance trend based on payment success rate
        let success_rate = if total_invoices > 0 {
            (paid_invoices * 100) / total_invoices
        } else {
            100
        };

        let performance_trend = match success_rate {
            90..=100 => PerformanceTrend::Improving,
            70..=89 => PerformanceTrend::Stable,
            50..=69 => PerformanceTrend::Declining,
            _ => PerformanceTrend::Volatile,
        };

        Ok(Self {
            business,
            period_start,
            period_end,
            total_invoices,
            paid_invoices,
            defaulted_invoices,
            average_payment_delay,
            total_volume,
            total_profits,
            performance_trend,
        })
    }

    /// Get payment success rate
    pub fn get_success_rate(&self) -> u32 {
        if self.total_invoices == 0 {
            return 100;
        }
        (self.paid_invoices * 100) / self.total_invoices
    }

    /// Get default rate
    pub fn get_default_rate(&self) -> u32 {
        if self.total_invoices == 0 {
            return 0;
        }
        (self.defaulted_invoices * 100) / self.total_invoices
    }

    /// Get profit margin
    pub fn get_profit_margin(&self) -> i128 {
        if self.total_volume == 0 {
            return 0;
        }
        (self.total_profits * 10000) / self.total_volume
    }
}

/// Storage keys for performance data
pub struct PerformanceStorage;

impl PerformanceStorage {
    /// Store business credit score
    pub fn store_credit_score(env: &Env, credit_score: &BusinessCreditScore) {
        let key = (symbol_short!("credit"), credit_score.business.clone());
        env.storage().instance().set(&key, credit_score);
    }

    /// Get business credit score
    pub fn get_credit_score(env: &Env, business: &Address) -> Option<BusinessCreditScore> {
        let key = (symbol_short!("credit"), business.clone());
        env.storage().instance().get(&key)
    }

    /// Store invoice performance metrics
    pub fn store_performance_metrics(env: &Env, metrics: &InvoicePerformanceMetrics) {
        let key = (symbol_short!("perf"), metrics.invoice_id.clone());
        env.storage().instance().set(&key, metrics);
    }

    /// Get invoice performance metrics
    pub fn get_performance_metrics(env: &Env, invoice_id: &BytesN<32>) -> Option<InvoicePerformanceMetrics> {
        let key = (symbol_short!("perf"), invoice_id.clone());
        env.storage().instance().get(&key)
    }

    /// Store risk assessment
    pub fn store_risk_assessment(env: &Env, assessment: &RiskAssessment) {
        let key = (symbol_short!("risk"), assessment.business.clone());
        env.storage().instance().set(&key, assessment);
    }

    /// Get risk assessment
    pub fn get_risk_assessment(env: &Env, business: &Address) -> Option<RiskAssessment> {
        let key = (symbol_short!("risk"), business.clone());
        env.storage().instance().get(&key)
    }

    /// Store performance fee adjustment
    pub fn store_fee_adjustment(env: &Env, adjustment: &PerformanceFeeAdjustment) {
        let key = (symbol_short!("fee_adj"), adjustment.business.clone());
        env.storage().instance().set(&key, adjustment);
    }

    /// Get performance fee adjustment
    pub fn get_fee_adjustment(env: &Env, business: &Address) -> Option<PerformanceFeeAdjustment> {
        let key = (symbol_short!("fee_adj"), business.clone());
        env.storage().instance().get(&key)
    }

    /// Store historical performance
    pub fn store_historical_performance(env: &Env, performance: &HistoricalPerformance) {
        let key = (symbol_short!("hist"), performance.business.clone());
        env.storage().instance().set(&key, performance);
    }

    /// Get historical performance
    pub fn get_historical_performance(env: &Env, business: &Address) -> Option<HistoricalPerformance> {
        let key = (symbol_short!("hist"), business.clone());
        env.storage().instance().get(&key)
    }

    /// Get businesses with high credit scores
    pub fn get_high_credit_businesses(env: &Env, threshold: u32) -> Vec<Address> {
        let mut high_credit_businesses = vec![env];
        // This would typically iterate through all businesses, but for simplicity
        // we'll return an empty vector. In a real implementation, you'd maintain
        // a list of all businesses and check their credit scores.
        high_credit_businesses
    }

    /// Get businesses with low risk scores
    pub fn get_low_risk_businesses(env: &Env, threshold: u32) -> Vec<Address> {
        let mut low_risk_businesses = vec![env];
        // Similar to above, this would iterate through all businesses
        low_risk_businesses
    }

    /// Get businesses with high performance scores
    pub fn get_high_performance_businesses(env: &Env, threshold: u32) -> Vec<Address> {
        let mut high_performance_businesses = vec![env];
        // Similar to above, this would iterate through all businesses
        high_performance_businesses
    }
}