use crate::errors::QuickLendXError;
use crate::events::{
    emit_dispute_created, emit_dispute_resolved, emit_dispute_under_review, emit_insurance_claimed,
    emit_invoice_defaulted, emit_invoice_expired,
};
use crate::investment::{InvestmentStatus, InvestmentStorage};
use crate::invoice::{Dispute, DisputeStatus, InvoiceStatus, InvoiceStorage};
use crate::notifications::NotificationSystem;
use soroban_sdk::{Address, BytesN, Env, String, Vec};

pub fn handle_default(env: &Env, invoice_id: &BytesN<32>) -> Result<(), QuickLendXError> {
    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;
    if invoice.status != InvoiceStatus::Funded {
        return Err(QuickLendXError::InvalidStatus);
    }
    if !invoice.is_overdue(env.ledger().timestamp()) {
        return Err(QuickLendXError::OperationNotAllowed);
    }
    InvoiceStorage::remove_from_status_invoices(env, &InvoiceStatus::Funded, invoice_id);
    invoice.mark_as_defaulted();
    InvoiceStorage::update_invoice(env, &invoice);
    InvoiceStorage::add_to_status_invoices(env, &InvoiceStatus::Defaulted, invoice_id);
    emit_invoice_expired(env, &invoice);
    if let Some(mut investment) = InvestmentStorage::get_investment_by_invoice(env, invoice_id) {
        investment.status = InvestmentStatus::Defaulted;

        let claim_details = investment
            .process_insurance_claim()
            .and_then(|(provider, amount)| {
                if amount > 0 {
                    Some((provider, amount))
                } else {
                    None
                }
            });

        InvestmentStorage::update_investment(env, &investment);

        if let Some((provider, coverage_amount)) = claim_details {
            emit_insurance_claimed(
                env,
                &investment.investment_id,
                &investment.invoice_id,
                &provider,
                coverage_amount,
            );
        }
    }
    emit_invoice_defaulted(env, &invoice);
    let _ = NotificationSystem::notify_invoice_defaulted(env, &invoice);
    Ok(())
}

/// Create a dispute for an invoice
pub fn create_dispute(
    env: &Env,
    invoice_id: &BytesN<32>,
    creator: &Address,
    reason: String,
    evidence: String,
) -> Result<(), QuickLendXError> {
    creator.require_auth();

    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    // Check if dispute already exists
    if invoice.dispute_status != DisputeStatus::None {
        return Err(QuickLendXError::DisputeAlreadyExists);
    }

    // Validate creator has stake in invoice (business or investor)
    if creator != &invoice.business {
        if let Some(investor) = &invoice.investor {
            if creator != investor {
                return Err(QuickLendXError::DisputeNotAuthorized);
            }
        } else {
            return Err(QuickLendXError::DisputeNotAuthorized);
        }
    }

    // Validate reason and evidence
    if reason.len() == 0 || reason.len() > 500 {
        return Err(QuickLendXError::InvalidDisputeReason);
    }

    if evidence.len() == 0 || evidence.len() > 1000 {
        return Err(QuickLendXError::InvalidDisputeEvidence);
    }

    // Create dispute
    let dispute = Dispute {
        created_by: creator.clone(),
        created_at: env.ledger().timestamp(),
        reason: reason.clone(),
        evidence,
        resolution: String::from_str(env, ""),
        resolved_by: Address::from_str(
            env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        ),
        resolved_at: 0,
    };

    // Update invoice with dispute
    invoice.dispute_status = DisputeStatus::Disputed;
    invoice.dispute = dispute;

    // Update invoice in storage
    InvoiceStorage::update_invoice(env, &invoice);

    // Emit dispute created event
    emit_dispute_created(env, invoice_id, creator, &reason);

    Ok(())
}

/// Put a dispute under review (admin function)
pub fn put_dispute_under_review(
    env: &Env,
    invoice_id: &BytesN<32>,
    reviewer: &Address,
) -> Result<(), QuickLendXError> {
    reviewer.require_auth();

    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    // Check if dispute exists and is in disputed status
    if invoice.dispute_status != DisputeStatus::Disputed {
        return Err(QuickLendXError::DisputeNotFound);
    }

    // Update dispute status
    invoice.dispute_status = DisputeStatus::UnderReview;

    // Update invoice in storage
    InvoiceStorage::update_invoice(env, &invoice);

    // Emit dispute under review event
    emit_dispute_under_review(env, invoice_id, reviewer);

    Ok(())
}

/// Resolve a dispute (admin function)
pub fn resolve_dispute(
    env: &Env,
    invoice_id: &BytesN<32>,
    resolver: &Address,
    resolution: String,
) -> Result<(), QuickLendXError> {
    resolver.require_auth();

    let mut invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    // Check if dispute exists and is under review
    if invoice.dispute_status != DisputeStatus::UnderReview {
        return Err(QuickLendXError::DisputeNotUnderReview);
    }

    // Validate resolution
    if resolution.len() == 0 || resolution.len() > 500 {
        return Err(QuickLendXError::InvalidDisputeReason);
    }

    // Update dispute with resolution
    if invoice.dispute_status != DisputeStatus::None {
        invoice.dispute.resolution = resolution.clone();
        invoice.dispute.resolved_by = resolver.clone();
        invoice.dispute.resolved_at = env.ledger().timestamp();
    }

    // Update dispute status
    invoice.dispute_status = DisputeStatus::Resolved;

    // Update invoice in storage
    InvoiceStorage::update_invoice(env, &invoice);

    // Emit dispute resolved event
    emit_dispute_resolved(env, invoice_id, resolver, &resolution);

    Ok(())
}

/// Get dispute details for an invoice
pub fn get_dispute_details(
    env: &Env,
    invoice_id: &BytesN<32>,
) -> Result<Option<Dispute>, QuickLendXError> {
    let invoice =
        InvoiceStorage::get_invoice(env, invoice_id).ok_or(QuickLendXError::InvoiceNotFound)?;

    if invoice.dispute_status != DisputeStatus::None {
        Ok(Some(invoice.dispute))
    } else {
        Ok(None)
    }
}

/// Get all invoices with disputes
pub fn get_invoices_with_disputes(env: &Env) -> Vec<BytesN<32>> {
    let mut disputed_invoices = Vec::new(env);

    // Check all invoice statuses for disputes
    let all_statuses = [
        InvoiceStatus::Pending,
        InvoiceStatus::Verified,
        InvoiceStatus::Funded,
        InvoiceStatus::Paid,
        InvoiceStatus::Defaulted,
    ];

    for status in all_statuses.iter() {
        let invoices = InvoiceStorage::get_invoices_by_status(env, status);
        for invoice_id in invoices.iter() {
            if let Some(invoice) = InvoiceStorage::get_invoice(env, &invoice_id) {
                if invoice.dispute_status != DisputeStatus::None {
                    disputed_invoices.push_back(invoice_id);
                }
            }
        }
    }

    disputed_invoices
}

/// Get invoices by dispute status
pub fn get_invoices_by_dispute_status(env: &Env, dispute_status: DisputeStatus) -> Vec<BytesN<32>> {
    let mut filtered_invoices = Vec::new(env);

    // Check all invoice statuses for specific dispute status
    let all_statuses = [
        InvoiceStatus::Pending,
        InvoiceStatus::Verified,
        InvoiceStatus::Funded,
        InvoiceStatus::Paid,
        InvoiceStatus::Defaulted,
    ];

    for status in all_statuses.iter() {
        let invoices = InvoiceStorage::get_invoices_by_status(env, status);
        for invoice_id in invoices.iter() {
            if let Some(invoice) = InvoiceStorage::get_invoice(env, &invoice_id) {
                if invoice.dispute_status == dispute_status {
                    filtered_invoices.push_back(invoice_id);
                }
            }
        }
    }

    filtered_invoices
}
