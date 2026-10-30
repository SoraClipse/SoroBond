use super::*;
use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    IntoVal,
};

#[test]
fn approvals_require_distinct_matching_signers_and_are_idempotent() {
    let (env, contract, seller, buyer, resolver, _, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    assert!(approve_party(&env, &contract, id, &seller, &hash));
    assert!(!approve_party(&env, &contract, id, &seller, &hash));
    assert!(!client.get(&id).unwrap().buyer_approved);
    assert!(approve_party(&env, &contract, id, &buyer, &hash));
    let record = client.get(&id).unwrap();
    assert!(record.buyer_approved && record.seller_approved);
    let outsider = Address::generate(&env);
    assert!(client
        .try_approve(
            &id,
            &outsider,
            &hash,
            &250,
            &86_400,
            &record.offer_expires_at,
            &resolver,
        )
        .is_err());
}

#[test]
fn altered_amount_duration_deadline_arbitrator_or_hash_never_approves() {
    let (env, contract, seller, _, resolver, _, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    let offer = client.get(&id).unwrap().offer_expires_at;
    let other = Address::generate(&env);
    let changed = BytesN::from_array(&env, &[66_u8; 32]);
    assert!(client
        .try_approve(&id, &seller, &changed, &250, &86_400, &offer, &resolver)
        .is_err());
    assert!(client
        .try_approve(&id, &seller, &hash, &251, &86_400, &offer, &resolver)
        .is_err());
    assert!(client
        .try_approve(&id, &seller, &hash, &250, &86_401, &offer, &resolver)
        .is_err());
    assert!(client
        .try_approve(&id, &seller, &hash, &250, &86_400, &(offer + 1), &resolver)
        .is_err());
    assert!(client
        .try_approve(&id, &seller, &hash, &250, &86_400, &offer, &other)
        .is_err());
    assert!(!client.get(&id).unwrap().seller_approved);
}

#[test]
fn approvals_are_scoped_to_warranty_ids() {
    let (env, contract, seller, buyer, resolver, _, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    let second = client.create(&seller, &buyer, &resolver, &100, &3_600, &hash);
    assert!(approve_party(&env, &contract, id, &seller, &hash));
    assert!(!client.get(&second).unwrap().seller_approved);
    // A signed invocation must include each id and contract address.
    let rec = client.get(&second).unwrap();
    assert!(client
        .try_approve(
            &second,
            &seller,
            &hash,
            &250,
            &86_400,
            &rec.offer_expires_at,
            &resolver,
        )
        .is_err());
    assert!(!client.get(&second).unwrap().seller_approved);
}

#[test]
fn authorization_for_a_different_warranty_id_does_not_authorize_this_one() {
    let (env, contract, seller, _, resolver, _, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    let offer_expires_at = client.get(&id).unwrap().offer_expires_at;
    let auth = [MockAuth {
        address: &seller,
        invoke: &MockAuthInvoke {
            contract: &contract,
            fn_name: "approve",
            args: (
                id + 1,
                seller.clone(),
                hash.clone(),
                250_i128,
                86_400_u64,
                offer_expires_at,
                resolver.clone(),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }];
    assert!(client
        .mock_auths(&auth)
        .try_approve(
            &id,
            &seller,
            &hash,
            &250,
            &86_400,
            &offer_expires_at,
            &resolver,
        )
        .is_err());
    assert!(!client.get(&id).unwrap().seller_approved);
}
