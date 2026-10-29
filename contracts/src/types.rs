use soroban_sdk::{contracttype, Address, BytesN};

/// Persistent V1 agreement. No personal item details are stored on the ledger.
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
    pub expires_at: u64,
    pub terms_hash: BytesN<32>,
    pub status: WarrantyStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WarrantyStatus {
    Active,
    // Claims and settlement are intentionally not implemented in this milestone.
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Token,
    NextId,
    LockedTotal,
    Depositing,
    Warranty(u64),
}

pub const MIN_DURATION: u64 = 60 * 60;
pub const MAX_DURATION: u64 = 90 * 24 * 60 * 60;

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
