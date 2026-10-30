use soroban_sdk::{contractevent, Env};

use crate::security;
use crate::types::{Claim, DataKey, Warranty, WarrantyStatus};
use crate::vault;
use crate::WarrantySettled;

#[contractevent]
#[derive(Clone)]
pub struct ArbitrationVerdict {
    pub id: u64,
    pub buyer_payout: i128,
}

/// The immutable arbitrator is authorized only to dispose of a recorded claim.
/// It cannot change participants, token, terms, or move unrelated collateral.
pub fn arbitrate(env: &Env, id: u64, buyer_payout: i128) {
    vault::assert_no_transfer(env);
    let mut warranty: Warranty = env
        .storage()
        .persistent()
        .get(&DataKey::Warranty(id))
        .expect("unknown warranty");
    assert_eq!(
        warranty.status,
        WarrantyStatus::Claimed,
        "no disputed claim"
    );
    warranty.arbitrator.require_auth();
    let claim: Claim = env
        .storage()
        .persistent()
        .get(&DataKey::Claim(id))
        .expect("claim missing");
    assert!(
        (0..=claim.amount).contains(&buyer_payout),
        "award exceeds claim"
    );
    let seller_return =
        security::finalize(env, &mut warranty, buyer_payout, WarrantyStatus::Settled);
    WarrantySettled {
        id,
        buyer_payout,
        seller_return,
    }
    .publish(env);
    ArbitrationVerdict { id, buyer_payout }.publish(env);
}
