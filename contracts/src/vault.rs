use soroban_sdk::{token, Address, Env};

/// A single allow-listed token contract is installed in __constructor and never
/// supplied by an untrusted depositor. The effect of transfer is atomic on-chain.
/// This is a deposit-only milestone: do not fund with real value.
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
