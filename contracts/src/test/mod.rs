use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
};

mod agreement;
mod claim;
mod lifecycle;
mod settlement;

fn fixture() -> (Env, Address, Address, Address, Address, Address) {
    let env = Env::default();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let resolver = Address::generate(&env);
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let token_id = sac.address();
    StellarAssetClient::new(&env, &token_id)
        .mock_all_auths()
        .mint(&seller, &1_000_i128);
    let contract_id = env.register(SoroBond, (&token_id,));
    (env, contract_id, seller, buyer, resolver, token_id)
}

fn pending() -> (Env, Address, Address, Address, Address, Address, u64, BytesN<32>) {
    let (env, contract, seller, buyer, resolver, asset) = fixture();
    env.mock_all_auths();
    let hash = BytesN::from_array(&env, &[42_u8; 32]);
    let id = SoroBondClient::new(&env, &contract)
        .create(&seller, &buyer, &resolver, &250, &86_400, &hash);
    (env, contract, seller, buyer, resolver, asset, id, hash)
}

fn approve_party(
    env: &Env,
    contract: &Address,
    id: u64,
    party: &Address,
    hash: &BytesN<32>,
) -> bool {
    let warranty = SoroBondClient::new(env, contract).get(&id).unwrap();
    SoroBondClient::new(env, contract).approve(
        &id,
        party,
        hash,
        &warranty.bond_amount,
        &warranty.duration_seconds,
        &warranty.offer_expires_at,
        &warranty.arbitrator,
    )
}

fn active() -> (Env, Address, Address, Address, Address, Address, u64, BytesN<32>) {
    let (env, contract, seller, buyer, resolver, asset, id, hash) = pending();
    assert!(approve_party(&env, &contract, id, &seller, &hash));
    assert!(approve_party(&env, &contract, id, &buyer, &hash));
    assert!(SoroBondClient::new(&env, &contract).activate(&id));
    (env, contract, seller, buyer, resolver, asset, id, hash)
}

#[test]
fn exact_deposit_remains_pending_until_both_parties_consent() {
    let (env, contract, seller, buyer, resolver, asset, id, hash) = pending();
    let client = SoroBondClient::new(&env, &contract);
    let token = TokenClient::new(&env, &asset);
    assert_eq!(id, 1);
    assert_eq!(token.balance(&seller), 750);
    assert_eq!(token.balance(&contract), 250);
    assert_eq!(client.locked_total(), 250);
    let rec = client.get(&id).unwrap();
    assert_eq!(rec.version, 1);
    assert_eq!(rec.status, WarrantyStatus::Pending);
    assert_eq!(rec.bond_amount, 250);
    assert_eq!(rec.token, asset);
    assert_eq!(rec.terms_hash, hash);
    assert_eq!(rec.expires_at, 0);
    assert_eq!(rec.offer_expires_at - rec.created_at, types::OFFER_VALIDITY);
    assert_eq!(
        client.create(&seller, &buyer, &resolver, &100, &3_600, &hash),
        2
    );
    assert_eq!(client.locked_total(), 350);
    assert_eq!(token.balance(&contract), 350);
}

#[test]
fn invalid_deposit_does_not_move_tokens() {
    let (env, contract, seller, buyer, resolver, asset) = fixture();
    env.mock_all_auths();
    let hash = BytesN::from_array(&env, &[17_u8; 32]);
    let zero = BytesN::from_array(&env, &[0_u8; 32]);
    let client = SoroBondClient::new(&env, &contract);
    assert!(client.try_create(&seller, &buyer, &resolver, &0, &3600, &hash).is_err());
    assert!(client.try_create(&seller, &seller, &resolver, &1, &3600, &hash).is_err());
    assert!(client.try_create(&seller, &buyer, &seller, &1, &3600, &hash).is_err());
    assert!(client.try_create(&seller, &buyer, &resolver, &1, &1, &hash).is_err());
    assert!(client
        .try_create(&seller, &buyer, &resolver, &1, &(90 * 86400 + 1), &hash)
        .is_err());
    assert!(client.try_create(&seller, &buyer, &resolver, &2_000, &3600, &hash).is_err());
    assert!(client.try_create(&seller, &buyer, &resolver, &1, &3600, &zero).is_err());
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 0);
    assert_eq!(client.locked_total(), 0);
    assert!(client.get(&1).is_none());
}

#[test]
fn seller_must_authorize_funding() {
    let (env, contract, seller, buyer, resolver, asset) = fixture();
    let hash = BytesN::from_array(&env, &[5_u8; 32]);
    let client = SoroBondClient::new(&env, &contract);
    assert!(client
        .try_create(&seller, &buyer, &resolver, &100, &3600, &hash)
        .is_err());
    assert_eq!(TokenClient::new(&env, &asset).balance(&contract), 0);
}
