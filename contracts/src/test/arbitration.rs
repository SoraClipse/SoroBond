use super::*;
use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    IntoVal,
};

#[test]
fn preselected_arbitrator_awards_only_claimed_amount_once() {
    let (env, contract, seller, buyer, resolver, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    let evidence = BytesN::from_array(&env, &[7_u8; 32]);
    client.submit_claim(&id, &buyer, &80, &evidence);
    assert!(client.try_arbitrate(&id, &81).is_err());
    assert_eq!(client.get(&id).unwrap().arbitrator, resolver);
    client.arbitrate(&id, &60);
    let token = TokenClient::new(&env, &asset);
    assert_eq!(token.balance(&buyer), 60);
    assert_eq!(token.balance(&seller), 940);
    assert_eq!(token.balance(&contract), 0);
    assert_eq!(client.locked_total(), 0);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Settled);
    assert!(client.try_arbitrate(&id, &60).is_err());
    assert!(client.try_settle(&id, &0).is_err());
    assert!(client.try_withdraw_expired(&id).is_err());
}

#[test]
fn fake_arbitrator_cannot_release_any_tokens() {
    let (env, contract, _, buyer, resolver, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    client.submit_claim(&id, &buyer, &80, &BytesN::from_array(&env, &[8_u8; 32]));
    let outsider = Address::generate(&env);
    let auth = [MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &contract,
            fn_name: "arbitrate",
            args: (id, 20_i128).into_val(&env),
            sub_invokes: &[],
        },
    }];
    assert!(client.mock_auths(&auth).try_arbitrate(&id, &20).is_err());
    assert_eq!(client.get(&id).unwrap().arbitrator, resolver);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Claimed);
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 250);
    assert_eq!(client.locked_total(), 250);
}

#[test]
fn failed_arbitration_transfer_preserves_open_claim_and_liability() {
    let (env, contract, seller, buyer, _, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    client.submit_claim(&id, &buyer, &100, &BytesN::from_array(&env, &[16_u8; 32]));
    let token = TokenClient::new(&env, &asset);
    let outsider = Address::generate(&env);
    token.transfer(&contract, &outsider, &250);
    assert!(client.try_arbitrate(&id, &50).is_err());
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Claimed);
    assert_eq!(client.locked_total(), 250);
    assert_eq!(token.balance(&buyer), 0);
    token.transfer(&seller, &contract, &250);
    client.arbitrate(&id, &50);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Settled);
    assert_eq!(client.locked_total(), 0);
}

#[test]
fn zero_award_and_arbitration_after_mutual_settlement_are_terminal() {
    let (env, contract, seller, buyer, _, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    client.submit_claim(&id, &buyer, &10, &BytesN::from_array(&env, &[10_u8; 32]));
    client.arbitrate(&id, &0);
    assert_eq!(TokenClient::new(&env, &asset).balance(&seller), 1_000);
    assert_eq!(client.locked_total(), 0);
    assert!(client.try_arbitrate(&id, &0).is_err());

    let (env, contract, _, buyer, _, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    client.submit_claim(&id, &buyer, &50, &BytesN::from_array(&env, &[11_u8; 32]));
    client.settle(&id, &10);
    assert!(client.try_arbitrate(&id, &10).is_err());
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 0);
}
