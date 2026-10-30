use soroban_sdk::{Address, BytesN, Env};

use crate::types::{DataKey, Warranty, WarrantyStatus};
use crate::vault;

/// Approve an immutable versioned offer using Soroban's native authorization tree.
///
/// A real signed auth entry covers the invocation's contract address, method and
/// every argument, and the network ID is part of the Soroban signature payload.
/// The expected fields make changed terms, amount, arbitration and deadlines fail.
pub fn approve(
    env: &Env,
    id: u64,
    signer: Address,
    expected_terms_hash: BytesN<32>,
    expected_bond_amount: i128,
    expected_duration_seconds: u64,
    expected_offer_expires_at: u64,
    expected_arbitrator: Address,
) -> bool {
    vault::assert_no_transfer(env);
    let mut warranty: Warranty = env
        .storage()
        .persistent()
        .get(&DataKey::Warranty(id))
        .expect("unknown warranty");
    assert_eq!(warranty.status, WarrantyStatus::Pending, "not pending");
    assert!(
        env.ledger().timestamp() <= warranty.offer_expires_at,
        "offer expired"
    );
    assert_eq!(warranty.terms_hash, expected_terms_hash, "terms mismatch");
    assert_eq!(warranty.bond_amount, expected_bond_amount, "bond mismatch");
    assert_eq!(
        warranty.duration_seconds, expected_duration_seconds,
        "duration mismatch"
    );
    assert_eq!(
        warranty.offer_expires_at, expected_offer_expires_at,
        "offer deadline mismatch"
    );
    assert_eq!(
        warranty.arbitrator, expected_arbitrator,
        "arbitrator mismatch"
    );
    assert!(
        signer == warranty.seller || signer == warranty.buyer,
        "not a participant"
    );

    // require_auth() must not be replaced with mock authorization in production.
    signer.require_auth();
    if signer == warranty.seller {
        if warranty.seller_approved {
            return false;
        }
        warranty.seller_approved = true;
    } else {
        if warranty.buyer_approved {
            return false;
        }
        warranty.buyer_approved = true;
    }
    env.storage()
        .persistent()
        .set(&DataKey::Warranty(id), &warranty);
    true
}
