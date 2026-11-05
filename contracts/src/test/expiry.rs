use super::*;
use soroban_sdk::testutils::Ledger as _;

#[test]
fn expiry_release_is_strictly_after_grace_and_single_use() {
    let (env, contract, seller, _, _, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    let deadline = client.get(&id).unwrap().expires_at + types::EXPIRY_GRACE;
    env.ledger()
        .with_mut(|ledger| ledger.timestamp = deadline - 1);
    assert!(client.try_withdraw_expired(&id).is_err());
    env.ledger().with_mut(|ledger| ledger.timestamp = deadline);
    assert!(client.try_withdraw_expired(&id).is_err());
    env.ledger()
        .with_mut(|ledger| ledger.timestamp = deadline + 1);
    client.withdraw_expired(&id);
    let token = TokenClient::new(&env, &asset);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Expired);
    assert_eq!(token.balance(&seller), 1_000);
    assert_eq!(token.balance(&contract), 0);
    assert_eq!(client.locked_total(), 0);
    assert!(client.try_withdraw_expired(&id).is_err());
}

#[test]
fn timely_claim_at_expiry_blocks_seller_even_after_grace() {
    let (env, contract, _, buyer, _, asset, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    let expiry = client.get(&id).unwrap().expires_at;
    env.ledger().with_mut(|ledger| ledger.timestamp = expiry);
    client.submit_claim(&id, &buyer, &80, &BytesN::from_array(&env, &[21_u8; 32]));
    env.ledger()
        .with_mut(|ledger| ledger.timestamp = expiry + types::EXPIRY_GRACE + 1);
    assert!(client.try_withdraw_expired(&id).is_err());
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 250);
    assert_eq!(client.locked_total(), 250);
    client.arbitrate(&id, &80);
    assert_eq!(client.locked_total(), 0);
}

#[test]
fn expired_offer_is_not_an_activated_warranty_and_cannot_use_active_exit() {
    let (env, contract, _, _, _, asset, id, _) = pending();
    let client = SoroBondClient::new(&env, &contract);
    let deadline = client.get(&id).unwrap().offer_expires_at;
    env.ledger()
        .with_mut(|ledger| ledger.timestamp = deadline + 100_000);
    assert!(client.try_withdraw_expired(&id).is_err());
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Pending);
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 250);
}

#[test]
fn unfunded_activation_is_not_needed_to_refund_unaccepted_offer() {
    let (env, contract, seller, _, _, asset, id, _) = pending();
    let client = SoroBondClient::new(&env, &contract);
    assert!(!client.get(&id).unwrap().buyer_approved);
    client.cancel_offer(&id);
    let token = TokenClient::new(&env, &asset);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Expired);
    assert_eq!(client.locked_total(), 0);
    assert_eq!(token.balance(&seller), 1_000);
    assert_eq!(token.balance(&contract), 0);
    assert!(client.try_cancel_offer(&id).is_err());
    assert!(client.try_activate(&id).is_err());
}

#[test]
fn buyer_approved_offer_cannot_be_cancelled_until_after_deadline() {
    let (env, contract, seller, buyer, _, asset, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    assert!(approve_party(&env, &contract, id, &buyer, &hash));
    assert!(client.try_cancel_offer(&id).is_err());
    let deadline = client.get(&id).unwrap().offer_expires_at;
    env.ledger().with_mut(|ledger| ledger.timestamp = deadline);
    assert!(client.try_cancel_offer(&id).is_err());
    env.ledger()
        .with_mut(|ledger| ledger.timestamp = deadline + 1);
    client.cancel_offer(&id);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Expired);
    assert_eq!(TokenClient::new(&env, &asset).balance(&seller), 1_000);
    assert_eq!(client.locked_total(), 0);
}

#[test]
fn cancelled_pending_offer_cannot_release_other_bonds_collateral() {
    let (env, contract, seller, buyer, resolver, asset, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    let second = client.create(&seller, &buyer, &resolver, &100, &3_600, &hash);
    assert_eq!(client.locked_total(), 350);
    client.cancel_offer(&id);
    assert_eq!(client.locked_total(), 100);
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 100);
    assert_eq!(client.get(&second).unwrap().status, WarrantyStatus::Pending);
    assert_eq!(TokenClient::new(&env, &asset).balance(&seller), 900);
}
