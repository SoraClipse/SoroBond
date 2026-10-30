use super::*;
use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    IntoVal,
};

#[test]
fn independently_reserved_bonds_cannot_be_spent_by_another_dispute() {
    let (env, contract, seller, buyer, resolver, asset, first, hash) = active();
    let client = SoroBondClient::new(&env, &contract);
    let second = client.create(&seller, &buyer, &resolver, &150, &3_600, &hash);
    assert!(approve_party(&env, &contract, second, &seller, &hash));
    assert!(approve_party(&env, &contract, second, &buyer, &hash));
    assert!(client.activate(&second));
    assert_eq!(client.locked_total(), 400);
    client.submit_claim(&first, &buyer, &100, &hash);
    client.arbitrate(&first, &30);
    let token = TokenClient::new(&env, &asset);
    assert_eq!(client.locked_total(), 150);
    assert_eq!(token.balance(&contract), 150);
    client.submit_claim(&second, &buyer, &150, &hash);
    client.settle(&second, &150);
    assert_eq!(client.locked_total(), 0);
    assert_eq!(token.balance(&contract), 0);
    assert_eq!(token.balance(&seller) + token.balance(&buyer), 1_000);
    assert!(client.try_arbitrate(&first, &1).is_err());
    assert!(client.try_settle(&second, &1).is_err());
}

#[test]
fn caller_without_seller_authorization_cannot_withdraw_expired_bond() {
    use soroban_sdk::testutils::Ledger as _;
    let (env, contract, seller, _, _, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    let deadline = client.get(&id).unwrap().expires_at + types::EXPIRY_GRACE;
    env.ledger().with_mut(|ledger| ledger.timestamp = deadline + 1);
    let outsider = Address::generate(&env);
    let auth = [MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &contract,
            fn_name: "withdraw_expired",
            args: (id,).into_val(&env),
            sub_invokes: &[],
        },
    }];
    assert!(client
        .mock_auths(&auth)
        .try_withdraw_expired(&id)
        .is_err());
    assert_eq!(client.get(&id).unwrap().seller, seller);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Active);
    assert_eq!(client.locked_total(), 250);
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 250);
}

#[test]
fn malicious_and_overflow_awards_revert_without_balance_or_state_changes() {
    let (env, contract, _, buyer, _, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    client.submit_claim(
        &id,
        &buyer,
        &20,
        &BytesN::from_array(&env, &[22_u8; 32]),
    );
    for wrong in [-1_i128, 21_i128, i128::MAX] {
        assert!(client.try_arbitrate(&id, &wrong).is_err());
        assert!(client.try_settle(&id, &wrong).is_err());
    }
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Claimed);
    assert_eq!(client.locked_total(), 250);
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 250);
}
