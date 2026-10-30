use super::*;
use soroban_sdk::testutils::Ledger as _;

#[test]
fn only_buyer_can_claim_positive_bounded_amount_once() {
    let (env, contract, _, buyer, _, _, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    let other = Address::generate(&env);
    let evidence = BytesN::from_array(&env, &[8_u8; 32]);
    assert!(client
        .try_submit_claim(&id, &other, &100, &evidence)
        .is_err());
    assert!(client.try_submit_claim(&id, &buyer, &0, &evidence).is_err());
    assert!(client
        .try_submit_claim(&id, &buyer, &251, &evidence)
        .is_err());
    client.submit_claim(&id, &buyer, &150, &evidence);
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Claimed);
    let claim = client.get_claim(&id).unwrap();
    assert_eq!(claim.warranty_id, id);
    assert_eq!(claim.amount, 150);
    assert_eq!(claim.evidence_hash, evidence);
    assert!(client
        .try_submit_claim(&id, &buyer, &150, &evidence)
        .is_err());
}

#[test]
fn claim_deadline_is_inclusive_and_late_claims_fail() {
    let (env, contract, _, buyer, _, _, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    let deadline = client.get(&id).unwrap().expires_at;
    env.ledger().with_mut(|info| info.timestamp = deadline);
    client.submit_claim(&id, &buyer, &1, &BytesN::from_array(&env, &[9_u8; 32]));
    assert_eq!(client.get(&id).unwrap().status, WarrantyStatus::Claimed);

    let (env, contract, _, buyer, _, _, id, _) = active();
    let client = SoroBondClient::new(&env, &contract);
    let deadline = client.get(&id).unwrap().expires_at;
    env.ledger().with_mut(|info| info.timestamp = deadline + 1);
    assert!(client
        .try_submit_claim(&id, &buyer, &1, &BytesN::from_array(&env, &[9_u8; 32]))
        .is_err());
    assert!(client.get_claim(&id).is_none());
}

#[test]
fn cannot_claim_an_unactivated_offer_or_make_an_unauthorized_withdrawal() {
    let (env, contract, _, buyer, _, asset, id, _) = pending();
    let client = SoroBondClient::new(&env, &contract);
    assert!(client
        .try_submit_claim(&id, &buyer, &25, &BytesN::from_array(&env, &[9_u8; 32]))
        .is_err());
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 250);
    assert_eq!(client.locked_total(), 250);
}
