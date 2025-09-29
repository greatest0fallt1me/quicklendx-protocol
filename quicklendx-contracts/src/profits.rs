use crate::errors::QuickLendXError;
use crate::events::emit_platform_fee_updated;
use soroban_sdk::{contracttype, symbol_short, Address, Env};

const DEFAULT_PLATFORM_FEE_BPS: i128 = 200; // 2%
const MAX_PLATFORM_FEE_BPS: i128 = 1_000; // 10%

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlatformFeeConfig {
    pub fee_bps: i128,
    pub updated_at: u64,
    pub updated_by: Address,
}

pub struct PlatformFee;

impl PlatformFee {
    const STORAGE_KEY: soroban_sdk::Symbol = symbol_short!("fee_cfg");

    fn default_config(env: &Env) -> PlatformFeeConfig {
        PlatformFeeConfig {
            fee_bps: DEFAULT_PLATFORM_FEE_BPS,
            updated_at: 0,
            updated_by: env.current_contract_address(),
        }
    }

    pub fn get_config(env: &Env) -> PlatformFeeConfig {
        env.storage()
            .instance()
            .get(&Self::STORAGE_KEY)
            .unwrap_or_else(|| Self::default_config(env))
    }

    pub fn set_config(
        env: &Env,
        admin: &Address,
        new_fee_bps: i128,
    ) -> Result<PlatformFeeConfig, QuickLendXError> {
        admin.require_auth();

        if new_fee_bps < 0 || new_fee_bps > MAX_PLATFORM_FEE_BPS {
            return Err(QuickLendXError::InvalidAmount);
        }

        let config = PlatformFeeConfig {
            fee_bps: new_fee_bps,
            updated_at: env.ledger().timestamp(),
            updated_by: admin.clone(),
        };

        env.storage().instance().set(&Self::STORAGE_KEY, &config);
        emit_platform_fee_updated(env, &config);
        Ok(config)
    }

    pub fn calculate(env: &Env, investment_amount: i128, payment_amount: i128) -> (i128, i128) {
        let config = Self::get_config(env);
        let profit = payment_amount.saturating_sub(investment_amount);
        if profit <= 0 {
            return (payment_amount.max(0), 0);
        }

        let platform_fee = profit.saturating_mul(config.fee_bps) / 10_000;
        let investor_return = payment_amount.saturating_sub(platform_fee);
        (investor_return, platform_fee)
    }
}

pub fn calculate_profit(env: &Env, investment_amount: i128, payment_amount: i128) -> (i128, i128) {
    PlatformFee::calculate(env, investment_amount, payment_amount)
}
