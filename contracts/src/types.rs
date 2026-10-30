use soroban_sdk::{contracttype, Address, BytesN};

/// Versioned immutable agreement fields plus monotonic on-chain lifecycle state.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Warranty {
    pub version: u32,
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub arbitrator: Address,
    pub token: Address,
    pub bond_amount: i128,
    pub created_at: u64,
    pub offer_expires_at: u64,
    pub duration_seconds: u64,
    pub activated_at: u64,
    pub expires_at: u64,
    pub terms_hash: BytesN<32>,
    pub seller_approved: bool,
    pub buyer_approved: bool,
    pub status: WarrantyStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    pub warranty_id: u64,
    pub amount: i128,
    pub evidence_hash: BytesN<32>,
    pub filed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WarrantyStatus {
    Pending,
    Active,
    Claimed,
    Settled,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Token,
    NextId,
    LockedTotal,
    Depositing,
    Warranty(u64),
    Claim(u64),
}

pub const MIN_DURATION: u64 = 60 * 60;
pub const MAX_DURATION: u64 = 90 * 24 * 60 * 60;
pub const OFFER_VALIDITY: u64 = 24 * 60 * 60;

pub fn validate_terms(
    seller: &Address,
    buyer: &Address,
    arbitrator: &Address,
    amount: i128,
    duration: u64,
) {
    assert!(seller != buyer, "seller and buyer must be distinct");
    assert!(
        arbitrator != seller && arbitrator != buyer,
        "arbitrator must be independent"
    );
    assert!(amount > 0, "bond must be positive");
    assert!(
        (MIN_DURATION..=MAX_DURATION).contains(&duration),
        "duration outside permitted window"
    );
}
