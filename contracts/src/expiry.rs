use soroban_sdk::{contractevent, Env};

use crate::security;
use crate::types::{DataKey, Warranty, WarrantyStatus, EXPIRY_GRACE};
use crate::vault;

#[contractevent]
#[derive(Clone)]
pub struct BondExpired {
    pub id: u64,
    pub seller_refund: i128,
}

/// At exactly expiry + grace, the seller still may not withdraw.
/// A timely claim permanently freezes this route, regardless of later time.
pub fn withdraw_expired(env: &Env, id: u64) {
    vault::assert_no_transfer(env);
    let mut warranty: Warranty = env
        .storage()
        .persistent()
        .get(&DataKey::Warranty(id))
        .expect("unknown warranty");
    warranty.seller.require_auth();
    assert_eq!(
        warranty.status,
        WarrantyStatus::Active,
        "bond disputed, pending, or finalized"
    );
    assert!(
        !env.storage().persistent().has(&DataKey::Claim(id)),
        "claim blocks expiry"
    );
    let deadline = warranty
        .expires_at
        .checked_add(EXPIRY_GRACE)
        .expect("grace deadline overflow");
    assert!(
        env.ledger().timestamp() > deadline,
        "expiry grace not elapsed"
    );
    security::finalize(env, &mut warranty, 0, WarrantyStatus::Expired);
    BondExpired {
        id,
        seller_refund: warranty.bond_amount,
    }
    .publish(env);
}
