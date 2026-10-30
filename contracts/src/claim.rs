use soroban_sdk::{Address, BytesN, Env};

use crate::types::{Claim, DataKey, Warranty, WarrantyStatus};
use crate::vault;
use crate::WarrantyClaimed;

/// The buyer may submit exactly one on-time claim; expiry is inclusive.
pub fn submit(env: &Env, id: u64, buyer: Address, amount: i128, evidence_hash: BytesN<32>) {
    vault::assert_no_transfer(env);
    let mut warranty: Warranty = env
        .storage()
        .persistent()
        .get(&DataKey::Warranty(id))
        .expect("unknown warranty");
    assert_eq!(warranty.status, WarrantyStatus::Active, "not active");
    assert_eq!(buyer, warranty.buyer, "not the buyer");
    buyer.require_auth();
    let now = env.ledger().timestamp();
    assert!(now <= warranty.expires_at, "claim deadline passed");
    assert!(
        amount > 0 && amount <= warranty.bond_amount,
        "claim amount out of bounds"
    );
    assert!(
        !env.storage().persistent().has(&DataKey::Claim(id)),
        "claim already submitted"
    );
    let claim = Claim {
        warranty_id: id,
        amount,
        evidence_hash,
        filed_at: now,
    };
    warranty.status = WarrantyStatus::Claimed;
    env.storage().persistent().set(&DataKey::Claim(id), &claim);
    env.storage()
        .persistent()
        .set(&DataKey::Warranty(id), &warranty);
    WarrantyClaimed { id, buyer, amount }.publish(env);
}
