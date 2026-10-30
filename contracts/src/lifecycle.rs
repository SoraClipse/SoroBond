use soroban_sdk::{token, Env};

use crate::types::{DataKey, Warranty, WarrantyStatus};
use crate::vault;
use crate::WarrantyActivated;

/// Permissionless, atomic transition after both independently authenticated
/// approvals and the original seller's exact locked deposit.
pub fn activate(env: &Env, id: u64) -> bool {
    vault::assert_no_transfer(env);
    let mut warranty: Warranty = env
        .storage()
        .persistent()
        .get(&DataKey::Warranty(id))
        .expect("unknown warranty");
    if warranty.status == WarrantyStatus::Active {
        return false;
    }
    assert_eq!(warranty.status, WarrantyStatus::Pending, "not pending");
    let now = env.ledger().timestamp();
    assert!(now <= warranty.offer_expires_at, "offer expired");
    assert!(
        warranty.buyer_approved && warranty.seller_approved,
        "both approvals required"
    );
    let locked: i128 = env
        .storage()
        .instance()
        .get(&DataKey::LockedTotal)
        .unwrap_or(0);
    assert!(locked >= warranty.bond_amount, "bond not reserved");
    let vault_balance = token::Client::new(env, &warranty.token)
        .balance(&env.current_contract_address());
    assert!(vault_balance >= locked, "bond not fully funded");
    warranty.activated_at = now;
    warranty.expires_at = now
        .checked_add(warranty.duration_seconds)
        .expect("warranty deadline overflow");
    warranty.status = WarrantyStatus::Active;
    env.storage()
        .persistent()
        .set(&DataKey::Warranty(id), &warranty);
    WarrantyActivated { id }.publish(env);
    true
}
