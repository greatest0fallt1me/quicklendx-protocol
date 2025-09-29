const DEFAULT_PLATFORM_FEE_BPS: i128 = 200; // 2%

/// Calculate investor payout and platform fee from a payment amount.
/// Returns a tuple of (investor_return, platform_fee).
pub fn calculate_profit(investment_amount: i128, payment_amount: i128) -> (i128, i128) {
    let profit = payment_amount.saturating_sub(investment_amount);
    if profit <= 0 {
        return (payment_amount.max(0), 0);
    }

    let platform_fee = profit.saturating_mul(DEFAULT_PLATFORM_FEE_BPS) / 10_000;
    let investor_return = payment_amount.saturating_sub(platform_fee);
    (investor_return, platform_fee)
}
