use super::*;
use soroban_sdk::{
    testutils::{Events as _, MockAuth, MockAuthInvoke},
    Event as _, IntoVal,
};

fn claimed() -> (Env, Address, Address, Address, Address, u64) {
    let (env, contract, seller, buyer, _, asset, id, _) = active();
    SoroBondClient::new(&env, &contract).submit_claim(
        &id,
        &buyer,
        &180,
        &BytesN::from_array(&env, &[9_u8; 32]),
    );
    (env, contract, seller, buyer, asset, id)
}

#[test]
fn partial_payout_and_remaining_collateral_are_exact_and_irreversible() {
    let (env, contract, seller, buyer, asset, id) = claimed();
    let client = SoroBondClient::new(&env, &contract);
    let token = TokenClient::new(&env, &asset);
    client.settle(&id, &100);
    let settlement = WarrantySettled {
        id,
        buyer_payout: 100,
        seller_return: 150,
    }
    .to_xdr(&env, &contract);
    assert!(env.events().all().events().contains(&settlement));
    assert_eq!(token.balance(&buyer), 100);
    assert_eq!(token.balance(&seller), 900);
    assert_eq!(token.balance(&contract), 0);
    assert_eq!(client.locked_total(), 0);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Settled);
    assert!(client.try_settle(&id, &100).is_err());
    assert_eq!(token.balance(&buyer), 100);
}

#[test]
fn zero_payout_returns_all_collateral_and_invalid_payout_is_rejected() {
    let (env, contract, seller, buyer, asset, id) = claimed();
    let client = SoroBondClient::new(&env, &contract);
    assert!(client.try_settle(&id, &181).is_err());
    assert!(client.try_settle(&id, &(-1)).is_err());
    assert_eq!(client.locked_total(), 250);
    client.settle(&id, &0);
    assert_eq!(TokenClient::new(&env, &asset).balance(&seller), 1_000);
    assert_eq!(TokenClient::new(&env, &asset).balance(&buyer), 0);
    assert_eq!(client.locked_total(), 0);
}

#[test]
fn failed_token_transfer_rolls_back_status_and_accounting() {
    let (env, contract, seller, buyer, asset, id) = claimed();
    let client = SoroBondClient::new(&env, &contract);
    let token = TokenClient::new(&env, &asset);
    let outsider = Address::generate(&env);
    token.transfer(&contract, &outsider, &250);
    assert!(client.try_settle(&id, &100).is_err());
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Claimed);
    assert_eq!(client.locked_total(), 250);
    assert_eq!(token.balance(&buyer), 0);
    token.transfer(&seller, &contract, &250);
    client.settle(&id, &100);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Settled);
}

#[test]
fn settlement_rejects_single_party_authorization() {
    let (env, contract, seller, buyer, asset, id) = claimed();
    let client = SoroBondClient::new(&env, &contract);
    let auth = [MockAuth {
        address: &seller,
        invoke: &MockAuthInvoke {
            contract: &contract,
            fn_name: "settle",
            args: (id, 100_i128).into_val(&env),
            sub_invokes: &[],
        },
    }];
    assert!(client.mock_auths(&auth).try_settle(&id, &100).is_err());
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Claimed);
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 250);
    assert_eq!(TokenClient::new(&env, &asset).balance(&buyer), 0);
}
