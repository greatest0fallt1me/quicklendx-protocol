use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvestmentStatus {
    Active,
    Withdrawn,
    Completed,
    Defaulted,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Investment {
    pub investment_id: BytesN<32>,
    pub invoice_id: BytesN<32>,
    pub investor: Address,
    pub amount: i128,
    pub funded_at: u64,
    pub status: InvestmentStatus,
}

pub struct InvestmentStorage;

impl InvestmentStorage {
    fn invoice_index_key(invoice_id: &BytesN<32>) -> (Symbol, BytesN<32>) {
        (symbol_short!("inv_map"), invoice_id.clone())
    }

    /// Generate a unique investment ID using timestamp and counter
    pub fn generate_unique_investment_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let counter_key = symbol_short!("invst_cnt");
        let counter = env.storage().instance().get(&counter_key).unwrap_or(0u64);
        env.storage().instance().set(&counter_key, &(counter + 1));

        let mut id_bytes = [0u8; 32];
        // Add investment prefix to distinguish from other entity types
        id_bytes[0] = 0x1A; // 'I' for Investment
        id_bytes[1] = 0x4E; // 'N' for iNvestment
                            // Embed timestamp in next 8 bytes
        id_bytes[2..10].copy_from_slice(&timestamp.to_be_bytes());
        // Embed counter in next 8 bytes
        id_bytes[10..18].copy_from_slice(&counter.to_be_bytes());
        // Fill remaining bytes with a pattern to ensure uniqueness
        for i in 18..32 {
            id_bytes[i] = ((timestamp + counter as u64 + 0x1A4E) % 256) as u8;
        }

        BytesN::from_array(env, &id_bytes)
    }

    pub fn store_investment(env: &Env, investment: &Investment) {
        env.storage()
            .instance()
            .set(&investment.investment_id, investment);

        env.storage().instance().set(
            &Self::invoice_index_key(&investment.invoice_id),
            &investment.investment_id,
        );
    }
    pub fn get_investment(env: &Env, investment_id: &BytesN<32>) -> Option<Investment> {
        env.storage().instance().get(investment_id)
    }
    pub fn get_investment_by_invoice(env: &Env, invoice_id: &BytesN<32>) -> Option<Investment> {
        let index_key = Self::invoice_index_key(invoice_id);
        let investment_id: Option<BytesN<32>> = env.storage().instance().get(&index_key);
        investment_id.and_then(|id| Self::get_investment(env, &id))
    }
    pub fn update_investment(env: &Env, investment: &Investment) {
        env.storage()
            .instance()
            .set(&investment.investment_id, investment);

        env.storage().instance().set(
            &Self::invoice_index_key(&investment.invoice_id),
            &investment.investment_id,
        );
    }
}
