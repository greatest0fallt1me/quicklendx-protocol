use soroban_sdk::{contracterror, symbol_short, Symbol};

/// Custom error types for the QuickLendX contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum QuickLendXError {
    // Invoice errors (1000-1099)
    InvoiceNotFound = 1000,
    InvoiceAlreadyExists = 1001,
    InvoiceNotAvailableForFunding = 1002,
    InvoiceAlreadyFunded = 1003,
    InvoiceAmountInvalid = 1004,
    InvoiceDueDateInvalid = 1005,
    InvoiceNotVerified = 1006,
    InvoiceNotFunded = 1007,
    InvoiceAlreadyPaid = 1008,
    InvoiceAlreadyDefaulted = 1009,

    // Authorization errors (1100-1199)
    Unauthorized = 1100,
    NotBusinessOwner = 1101,
    NotInvestor = 1102,
    NotAdmin = 1103,

    // Validation errors (1200-1299)
    InvalidAmount = 1200,
    InvalidAddress = 1201,
    InvalidCurrency = 1202,
    InvalidTimestamp = 1203,
    InvalidDescription = 1204,

    // Storage errors (1300-1399)
    StorageError = 1300,
    StorageKeyNotFound = 1301,

    // Business logic errors (1400-1499)
    InsufficientFunds = 1400,
    InvalidStatus = 1401,
    OperationNotAllowed = 1402,

    // Rating errors (1500-1599)
    InvalidRating = 1500,
    NotFunded = 1501,
    AlreadyRated = 1502,
    NotRater = 1503,

    // KYC/Verification errors (1600-1699)
    BusinessNotVerified = 1600,
    KYCAlreadyPending = 1601,
    KYCAlreadyVerified = 1602,
    KYCNotFound = 1603,
    InvalidKYCStatus = 1604,

    // Audit errors (1700-1799)
    AuditLogNotFound = 1700,
    AuditValidationFailed = 1701,
    AuditIntegrityError = 1702,
    AuditQueryError = 1703,

    // Category and Tag errors (1800-1899)
    InvalidTag = 1802,
    TagLimitExceeded = 1803,

    // Dispute errors (1900-1999)
    DisputeNotFound = 1900,
    DisputeAlreadyExists = 1901,
    DisputeNotAuthorized = 1902,
    DisputeAlreadyResolved = 1903,
    DisputeNotUnderReview = 1904,
    InvalidDisputeReason = 1905,
    InvalidDisputeEvidence = 1906,

    // Notification errors
    NotificationNotFound = 2000,
    NotificationBlocked = 2001,

    // Priority errors (2100-2199)
    InvalidPriorityLevel = 2100,
    InvalidUrgencyLevel = 2101,
    InvalidPriorityChange = 2102,
    PriorityChangeNotFound = 2103,
    InvalidFeeStructure = 2104,
    PriorityChangeNotAllowed = 2105,
}

impl From<QuickLendXError> for Symbol {
    fn from(error: QuickLendXError) -> Self {
        match error {
            QuickLendXError::InvoiceNotFound => symbol_short!("INV_NF"),
            QuickLendXError::InvoiceAlreadyExists => symbol_short!("INV_EX"),
            QuickLendXError::InvoiceNotAvailableForFunding => symbol_short!("INV_NA"),
            QuickLendXError::InvoiceAlreadyFunded => symbol_short!("INV_FD"),
            QuickLendXError::InvoiceAmountInvalid => symbol_short!("INV_AI"),
            QuickLendXError::InvoiceDueDateInvalid => symbol_short!("INV_DI"),
            QuickLendXError::InvoiceNotVerified => symbol_short!("INV_NV"),
            QuickLendXError::InvoiceNotFunded => symbol_short!("INV_NF"),
            QuickLendXError::InvoiceAlreadyPaid => symbol_short!("INV_PD"),
            QuickLendXError::InvoiceAlreadyDefaulted => symbol_short!("INV_DF"),
            QuickLendXError::Unauthorized => symbol_short!("UNAUTH"),
            QuickLendXError::NotBusinessOwner => symbol_short!("NOT_OWN"),
            QuickLendXError::NotInvestor => symbol_short!("NOT_INV"),
            QuickLendXError::NotAdmin => symbol_short!("NOT_ADM"),
            QuickLendXError::InvalidAmount => symbol_short!("INV_AMT"),
            QuickLendXError::InvalidAddress => symbol_short!("INV_ADR"),
            QuickLendXError::InvalidCurrency => symbol_short!("INV_CR"),
            QuickLendXError::InvalidTimestamp => symbol_short!("INV_TM"),
            QuickLendXError::InvalidDescription => symbol_short!("INV_DS"),
            QuickLendXError::StorageError => symbol_short!("STORE"),
            QuickLendXError::StorageKeyNotFound => symbol_short!("KEY_NF"),
            QuickLendXError::InsufficientFunds => symbol_short!("INSUF"),
            QuickLendXError::InvalidStatus => symbol_short!("INV_ST"),
            QuickLendXError::OperationNotAllowed => symbol_short!("OP_NA"),
            QuickLendXError::InvalidRating => symbol_short!("INV_RT"),
            QuickLendXError::NotFunded => symbol_short!("NOT_FD"),
            QuickLendXError::AlreadyRated => symbol_short!("ALR_RT"),
            QuickLendXError::NotRater => symbol_short!("NOT_RT"),
            QuickLendXError::BusinessNotVerified => symbol_short!("BUS_NV"),
            QuickLendXError::KYCAlreadyPending => symbol_short!("KYC_PD"),
            QuickLendXError::KYCAlreadyVerified => symbol_short!("KYC_VF"),
            QuickLendXError::KYCNotFound => symbol_short!("KYC_NF"),
            QuickLendXError::InvalidKYCStatus => symbol_short!("KYC_IS"),
            QuickLendXError::AuditLogNotFound => symbol_short!("AUD_NF"),
            QuickLendXError::AuditValidationFailed => symbol_short!("AUD_VF"),
            QuickLendXError::AuditIntegrityError => symbol_short!("AUD_IE"),
            QuickLendXError::AuditQueryError => symbol_short!("AUD_QE"),
            QuickLendXError::InvalidTag => symbol_short!("INV_TAG"),
            QuickLendXError::TagLimitExceeded => symbol_short!("TAG_LIM"),
            // Dispute errors
            QuickLendXError::DisputeNotFound => symbol_short!("DSP_NF"),
            QuickLendXError::DisputeAlreadyExists => symbol_short!("DSP_EX"),
            QuickLendXError::DisputeNotAuthorized => symbol_short!("DSP_NA"),
            QuickLendXError::DisputeAlreadyResolved => symbol_short!("DSP_RS"),
            QuickLendXError::DisputeNotUnderReview => symbol_short!("DSP_UR"),
            QuickLendXError::InvalidDisputeReason => symbol_short!("DSP_RN"),
            QuickLendXError::InvalidDisputeEvidence => symbol_short!("DSP_EV"),
            // Notification errors
            QuickLendXError::NotificationNotFound => symbol_short!("NOT_NF"),
            QuickLendXError::NotificationBlocked => symbol_short!("NOT_BL"),
            // Priority errors
            QuickLendXError::InvalidPriorityLevel => symbol_short!("PRI_IL"),
            QuickLendXError::InvalidUrgencyLevel => symbol_short!("URG_IL"),
            QuickLendXError::InvalidPriorityChange => symbol_short!("PRI_IC"),
            QuickLendXError::PriorityChangeNotFound => symbol_short!("PRI_NF"),
            QuickLendXError::InvalidFeeStructure => symbol_short!("FEE_IS"),
            QuickLendXError::PriorityChangeNotAllowed => symbol_short!("PRI_NA"),
        }
    }
}
