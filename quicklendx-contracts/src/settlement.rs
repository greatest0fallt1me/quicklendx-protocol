use crate::errors::QuickLendXError;
use crate::events::emit_invoice_settled;
use crate::investment::{InvestmentStatus, InvestmentStorage};
use crate::invoice::{InvoiceStatus, InvoiceStorage};
use crate::notifications::NotificationSystem;
use crate::payments::transfer_funds;
use crate::profits::calculate_profit;
use soroban_sdk::{BytesN, Env};

pub fn settle_invoice(
    env: &Env,
    invoice_id: &BytesN<32>,
    payment_amount: i128,
) -> Result<(), QuickLendXError> {
    if payment_amount <= 0 {
        return Err(QuickLendXError::InvalidAmount);
    }

    // Get and validate invoice
    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    if invoice.status != InvoiceStatus::Funded {
        return Err(QuickLendXError::InvalidStatus);
    }

    // Get investor from invoice
    let investor = invoice
        .investor
        .as_ref()
        .ok_or(QuickLendXError::NotInvestor)?;

    // Get investment details
    let investment = InvestmentStorage::get_investment_by_invoice(env, invoice_id)
        .ok_or(QuickLendXError::StorageKeyNotFound)?;

    if payment_amount < investment.amount {
        return Err(QuickLendXError::PaymentTooLow);
    }

    // Calculate profit and platform fee
    let (investor_return, platform_fee) = calculate_profit(investment.amount, payment_amount);

    // Transfer funds to investor and platform
    let business_address = invoice.business.clone();
    let investor_address = investor.clone();
    let investor_paid = transfer_funds(env, &business_address, &investor_address, investor_return);
    if !investor_paid {
        return Err(QuickLendXError::InsufficientFunds);
    }

    if platform_fee > 0 {
        let platform_account = env.current_contract_address();
        let platform_paid = transfer_funds(env, &business_address, &platform_account, platform_fee);
        if !platform_paid {
            return Err(QuickLendXError::InsufficientFunds);
        }
    }

    // Update invoice status
    invoice.mark_as_paid(env, business_address.clone(), env.ledger().timestamp());
    InvoiceStorage::update_invoice(env, &invoice);

    // Update investment status
    let mut updated_investment = investment;
    updated_investment.status = InvestmentStatus::Completed;
    InvestmentStorage::update_investment(env, &updated_investment);

    // Emit settlement event
    emit_invoice_settled(env, &invoice, investor_return, platform_fee);

    // Send notification about payment received
    let _ = NotificationSystem::notify_payment_received(env, &invoice, payment_amount);

    Ok(())
}
