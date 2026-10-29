use super::*;
use soroban_sdk::{testutils::Address as _, token::{Client as TokenClient, StellarAssetClient}};

fn fixture() -> (Env, Address, Address, Address, Address, Address) {
    let env = Env::default();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let resolver = Address::generate(&env);
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let token_id = sac.address();
    StellarAssetClient::new(&env, &token_id).mock_all_auths().mint(&seller, &1_000_i128);
    let contract_id = env.register(SoroBond, (&token_id,));
    (env, contract_id, seller, buyer, resolver, token_id)
}

#[test]
fn exact_deposit_and_versioned_record() {
    let (env, contract, seller, buyer, resolver, asset) = fixture();
    env.mock_all_auths();
    let client = SoroBondClient::new(&env, &contract);
    let token = TokenClient::new(&env, &asset);
    let hash = BytesN::from_array(&env, &[42_u8; 32]);
    let id = client.create(&seller, &buyer, &resolver, &250_i128, &86_400_u64, &hash);
    assert_eq!(id, 1);
    assert_eq!(token.balance(&seller), 750);
    assert_eq!(token.balance(&contract), 250);
    assert_eq!(client.locked_total(), 250);
    let rec = client.get(&id).unwrap();
    assert_eq!(rec.version, 1);
    assert_eq!(rec.status, WarrantyStatus::Active);
    assert_eq!(rec.bond_amount, 250);
    assert_eq!(rec.token, asset);
    assert_eq!(rec.terms_hash, hash);
    assert_eq!(rec.expires_at - rec.created_at, 86_400);
    // The second independent bond is separately accounted, not reused.
    assert_eq!(client.create(&seller, &buyer, &resolver, &100_i128, &3_600_u64, &hash), 2);
    assert_eq!(client.locked_total(), 350);
    assert_eq!(token.balance(&contract), 350);
}

#[test]
fn rejects_bad_deposit_terms_without_moving_tokens() {
    let (env, contract, seller, buyer, resolver, asset) = fixture();
    env.mock_all_auths();
    let client = SoroBondClient::new(&env, &contract);
    let hash = BytesN::from_array(&env, &[0;32]);
    assert!(client.try_create(&seller, &buyer, &resolver, &0, &3600, &hash).is_err());
    assert!(client.try_create(&seller, &seller, &resolver, &1, &3600, &hash).is_err());
    assert!(client.try_create(&seller, &buyer, &seller, &1, &3600, &hash).is_err());
    assert!(client.try_create(&seller, &buyer, &resolver, &1, &1, &hash).is_err());
    assert!(client.try_create(&seller, &buyer, &resolver, &1, &(90*86400+1), &hash).is_err());
    assert!(client.try_create(&seller, &buyer, &resolver, &2000, &3600, &hash).is_err());
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract),0);
    assert_eq!(client.locked_total(),0);
    assert!(client.get(&1).is_none());
}

#[test]
fn requires_seller_auth_instead_of_implicit_auto_approval() {
    let (env, contract, seller, buyer, resolver, asset) = fixture();
    // Do not enable mock_all_auths: a missing seller authorization MUST fail.
    let client = SoroBondClient::new(&env, &contract);
    let hash = BytesN::from_array(&env, &[5;32]);
    assert!(client.try_create(&seller, &buyer, &resolver, &100, &3600, &hash).is_err());
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 0);
}
