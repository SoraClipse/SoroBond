use soroban_sdk::{token, Address, Env};

use crate::types::DataKey;

pub fn assert_no_transfer(env: &Env) {
    assert!(
        !env.storage()
            .instance()
            .get::<_, bool>(&DataKey::Depositing)
            .unwrap_or(false),
        "reentrant token transfer"
    );
}

/// Allow-listed token deposit. Reverts the complete create transaction on failure.
pub fn deposit_exact(env: &Env, asset: &Address, seller: &Address, amount: i128) {
    assert!(amount > 0, "amount must be positive");
    let client = token::Client::new(env, asset);
    let contract = env.current_contract_address();
    let before = client.balance(&contract);
    client.transfer(seller, &contract, &amount);
    let after = client.balance(&contract);
    assert_eq!(
        after.checked_sub(before),
        Some(amount),
        "token deposit mismatch"
    );
}

/// Fee-free exact bond accounting; failure of either transfer rolls back both.
pub fn distribute_exact(
    env: &Env,
    asset: &Address,
    seller: &Address,
    buyer: &Address,
    bond_amount: i128,
    buyer_payout: i128,
    seller_return: i128,
    remaining_locked: i128,
) {
    assert!(bond_amount > 0 && buyer_payout >= 0 && seller_return >= 0);
    assert_eq!(
        buyer_payout.checked_add(seller_return),
        Some(bond_amount),
        "payout does not equal bond"
    );
    let client = token::Client::new(env, asset);
    let contract = env.current_contract_address();
    let before = client.balance(&contract);
    let buyer_before = client.balance(buyer);
    let seller_before = client.balance(seller);
    assert!(
        before >= bond_amount && before >= remaining_locked,
        "insufficient vault balance"
    );
    if buyer_payout > 0 {
        client.transfer(&contract, buyer, &buyer_payout);
    }
    if seller_return > 0 {
        client.transfer(&contract, seller, &seller_return);
    }
    let after = client.balance(&contract);
    assert_eq!(
        before.checked_sub(after),
        Some(bond_amount),
        "token payout mismatch"
    );
    assert!(after >= remaining_locked, "other bonds undercollateralized");
    assert_eq!(
        client.balance(buyer).checked_sub(buyer_before),
        Some(buyer_payout),
        "buyer did not receive exact payout"
    );
    assert_eq!(
        client.balance(seller).checked_sub(seller_before),
        Some(seller_return),
        "seller did not receive exact return"
    );
}
