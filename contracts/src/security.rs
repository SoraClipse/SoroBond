use soroban_sdk::{Address, Env};

use crate::types::{DataKey, Warranty, WarrantyStatus};
use crate::vault;

/// One shared, atomic exit accounting routine for mutual settlement,
/// preselected arbitration and seller-only expiry release.
pub fn finalize(
    env: &Env,
    warranty: &mut Warranty,
    buyer_payout: i128,
    terminal: WarrantyStatus,
) -> i128 {
    vault::assert_no_transfer(env);
    assert!(
        terminal == WarrantyStatus::Settled || terminal == WarrantyStatus::Expired,
        "invalid terminal status"
    );
    assert!(
        (0..=warranty.bond_amount).contains(&buyer_payout),
        "invalid buyer payout"
    );
    let pinned: Address = env
        .storage()
        .instance()
        .get(&DataKey::Token)
        .expect("missing allow-listed token");
    assert_eq!(warranty.token, pinned, "wrong collateral token");
    let seller_return = warranty
        .bond_amount
        .checked_sub(buyer_payout)
        .expect("payout underflow");
    let locked: i128 = env
        .storage()
        .instance()
        .get(&DataKey::LockedTotal)
        .expect("missing collateral liabilities");
    let remaining = locked
        .checked_sub(warranty.bond_amount)
        .expect("liability underflow");
    assert!(remaining >= 0, "negative liabilities");

    // Persist the terminal state before calling external token code. Any
    // failed token transfer reverts all writes and events in this invocation.
    env.storage().instance().set(&DataKey::Depositing, &true);
    warranty.status = terminal;
    env.storage()
        .persistent()
        .set(&DataKey::Warranty(warranty.id), &*warranty);
    env.storage()
        .instance()
        .set(&DataKey::LockedTotal, &remaining);
    vault::distribute_exact(
        env,
        &warranty.token,
        &warranty.seller,
        &warranty.buyer,
        warranty.bond_amount,
        buyer_payout,
        seller_return,
        remaining,
    );
    env.storage().instance().set(&DataKey::Depositing, &false);
    seller_return
}
