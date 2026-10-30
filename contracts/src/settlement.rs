use soroban_sdk::Env;

use crate::security;
use crate::types::{Claim, DataKey, Warranty, WarrantyStatus};
use crate::vault;
use crate::WarrantySettled;

/// Mutual settlement retains its existing public contract signature and auth rules.
/// It shares exact final payout accounting with arbitration and expiry withdrawal.
pub fn settle(env: &Env, id: u64, buyer_payout: i128) {
    vault::assert_no_transfer(env);
    let mut warranty: Warranty = env
        .storage()
        .persistent()
        .get(&DataKey::Warranty(id))
        .expect("unknown warranty");
    assert_eq!(warranty.status, WarrantyStatus::Claimed, "no open claim");
    let claim: Claim = env
        .storage()
        .persistent()
        .get(&DataKey::Claim(id))
        .expect("claim missing");
    assert!(
        (0..=claim.amount).contains(&buyer_payout),
        "payout exceeds claim"
    );
    warranty.buyer.require_auth();
    warranty.seller.require_auth();

    let seller_return =
        security::finalize(env, &mut warranty, buyer_payout, WarrantyStatus::Settled);
    WarrantySettled {
        id,
        buyer_payout,
        seller_return,
    }
    .publish(env);
}
