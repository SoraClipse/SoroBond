use soroban_sdk::Env;

use crate::types::{Claim, DataKey, Warranty, WarrantyStatus};
use crate::vault;
use crate::WarrantySettled;

/// Both parties authorize this *same* contract invocation and exact payout.
/// No arbitrator or unilateral caller may finalize a disputed claim.
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
        buyer_payout >= 0 && buyer_payout <= claim.amount,
        "payout exceeds claim"
    );
    let seller_return = warranty
        .bond_amount
        .checked_sub(buyer_payout)
        .expect("invalid payout");
    warranty.buyer.require_auth();
    warranty.seller.require_auth();

    let total: i128 = env
        .storage()
        .instance()
        .get(&DataKey::LockedTotal)
        .unwrap_or(0);
    let new_total = total
        .checked_sub(warranty.bond_amount)
        .expect("locked balance underflow");
    assert!(new_total >= 0, "locked balance underflow");

    // Persist terminal state before external calls, with the transfer guard set.
    // Soroban rolls back ALL storage and token writes on any downstream failure.
    env.storage().instance().set(&DataKey::Depositing, &true);
    warranty.status = WarrantyStatus::Settled;
    env.storage()
        .persistent()
        .set(&DataKey::Warranty(id), &warranty);
    env.storage()
        .instance()
        .set(&DataKey::LockedTotal, &new_total);
    vault::distribute_exact(
        env,
        &warranty.token,
        &warranty.seller,
        &warranty.buyer,
        warranty.bond_amount,
        buyer_payout,
        seller_return,
        new_total,
    );
    env.storage().instance().set(&DataKey::Depositing, &false);
    WarrantySettled {
        id,
        buyer_payout,
        seller_return,
    }
    .publish(env);
}
