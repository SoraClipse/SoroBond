use super::*;
use soroban_sdk::{
    testutils::{Events as _, Ledger as _},
    Event as _,
};

#[test]
fn activation_requires_both_approvals_and_publishes_once() {
    let (env, contract, seller, buyer, _, _, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    assert!(client.try_activate(&id).is_err());
    assert!(approve_party(&env, &contract, id, &seller, &hash));
    assert!(client.try_activate(&id).is_err());
    assert!(approve_party(&env, &contract, id, &buyer, &hash));
    assert!(client.activate(&id));
    let activation = WarrantyActivated { id }.to_xdr(&env, &contract);
    assert!(env.events().all().events().contains(&activation));
    assert!(!client.activate(&id));
    let duplicates = env
        .events()
        .all()
        .events()
        .iter()
        .filter(|event| **event == activation)
        .count();
    assert!(duplicates <= 1);
    let rec = client.get(&id).unwrap();
    assert_eq!(rec.status, WarrantyStatus::Active);
    assert_eq!(rec.expires_at - rec.activated_at, 86_400);
}

#[test]
fn expired_offer_cannot_be_approved_or_activated() {
    let (env, contract, seller, buyer, _, _, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    let expiry = client.get(&id).unwrap().offer_expires_at;
    assert!(approve_party(&env, &contract, id, &seller, &hash));
    assert!(approve_party(&env, &contract, id, &buyer, &hash));
    env.ledger().with_mut(|info| info.timestamp = expiry + 1);
    assert!(client.try_activate(&id).is_err());
    assert!(client.get(&id).unwrap().status == WarrantyStatus::Pending);
    assert!(client
        .try_approve(
            &id,
            &seller,
            &hash,
            &250,
            &86_400,
            &expiry,
            &client.get(&id).unwrap().arbitrator,
        )
        .is_err());
}

#[test]
fn activation_requires_real_vault_balance_not_only_storage_accounting() {
    let (env, contract, seller, buyer, _, asset, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    assert!(approve_party(&env, &contract, id, &seller, &hash));
    assert!(approve_party(&env, &contract, id, &buyer, &hash));
    let attacker = Address::generate(&env);
    let token = TokenClient::new(&env, &asset);
    token.transfer(&contract, &attacker, &250);
    assert!(client.try_activate(&id).is_err());
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Pending);
    token.transfer(&seller, &contract, &250);
    assert!(client.activate(&id));
}
