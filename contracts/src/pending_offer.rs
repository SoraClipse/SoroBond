use soroban_sdk::{contractevent, Env};

use crate::security;
use crate::types::{DataKey, Warranty, WarrantyStatus};
use crate::vault;

#[contractevent]
#[derive(Clone)]
pub struct PendingOfferCancelled {
    pub id: u64,
    pub seller_refund: i128,
}

/// A funded offer can be cancelled by its seller before buyer approval, or
/// after its acceptance deadline. A buyer-approved offer stays protected
/// through the entire acceptance window to avoid a seller consent race.
pub fn cancel_offer(env: &Env, id: u64) {
    vault::assert_no_transfer(env);
    let mut warranty: Warranty = env
        .storage()
        .persistent()
        .get(&DataKey::Warranty(id))
        .expect("unknown warranty");
    warranty.seller.require_auth();
    assert_eq!(
        warranty.status,
        WarrantyStatus::Pending,
        "offer already activated or finalized"
    );
    assert!(
        !warranty.buyer_approved || env.ledger().timestamp() > warranty.offer_expires_at,
        "buyer-approved offer remains binding until deadline"
    );
    security::finalize(env, &mut warranty, 0, WarrantyStatus::Expired);
    PendingOfferCancelled {
        id,
        seller_refund: warranty.bond_amount,
    }
    .publish(env);
}
