use super::*;
use soroban_sdk::testutils::Ledger as _;

/// Deterministic pseudo-random scenario generator makes every failing case reproducible.
/// Multiple legal settlement routes and illegal replay actions are checked in isolation.
#[test]
fn seeded_state_sequences_preserve_global_conservation_and_terminal_finality() {
    let mut seed = 0x6c62_7dd1_9c58_44e3_u64;
    for _case in 0..18 {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let (env, contract, seller, buyer, _, asset, id, _) = active();
        let client = SoroBondClient::new(&env, &contract);
        let token = TokenClient::new(&env, &asset);
        let evidence = BytesN::from_array(&env, &[23_u8; 32]);
        let route = seed % 3;

        if route == 0 {
            let claim_amount = ((seed >> 12) % 250 + 1) as i128;
            client.submit_claim(&id, &buyer, &claim_amount, &evidence);
            assert!(client.try_withdraw_expired(&id).is_err());
            client.arbitrate(&id, &(claim_amount / 2));
        } else if route == 1 {
            client.submit_claim(&id, &buyer, &250, &evidence);
            client.settle(&id, &(((seed >> 10) % 251) as i128));
        } else {
            let expiry = client.get(&id).unwrap().expires_at + types::EXPIRY_GRACE;
            env.ledger().with_mut(|ledger| ledger.timestamp = expiry);
            assert!(client.try_withdraw_expired(&id).is_err());
            env.ledger()
                .with_mut(|ledger| ledger.timestamp = expiry + 1);
            client.withdraw_expired(&id);
        }

        let status = client.get(&id).unwrap().status;
        assert!(status == WarrantyStatus::Settled || status == WarrantyStatus::Expired);
        assert_eq!(client.locked_total(), 0);
        assert_eq!(token.balance(&contract), 0);
        assert_eq!(token.balance(&seller) + token.balance(&buyer), 1_000);
        assert!(client.try_submit_claim(&id, &buyer, &1, &evidence).is_err());
        assert!(client.try_arbitrate(&id, &0).is_err());
        assert!(client.try_settle(&id, &0).is_err());
        assert!(client.try_withdraw_expired(&id).is_err());
        assert_eq!(client.get(&id).unwrap().status, status);
    }
}
