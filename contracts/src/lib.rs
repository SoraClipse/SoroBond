#![no_std]
mod agreement;
mod claim;
mod lifecycle;
mod settlement;
mod types;
mod vault;

use soroban_sdk::{contract, contractevent, contractimpl, Address, BytesN, Env};
use types::{validate_terms, Claim, DataKey, Warranty, WarrantyStatus};

#[contract]
pub struct SoroBond;

#[contractevent]
#[derive(Clone)]
pub struct WarrantyCreated {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct WarrantyActivated {
    pub id: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct WarrantyClaimed {
    pub id: u64,
    pub buyer: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct WarrantySettled {
    pub id: u64,
    pub buyer_payout: i128,
    pub seller_return: i128,
}

#[contractimpl]
impl SoroBond {
    /// Install with the exact, reviewed token contract address.
    pub fn __constructor(env: Env, allowed_asset: Address) {
        env.storage()
            .instance()
            .set(&DataKey::Token, &allowed_asset);
        env.storage().instance().set(&DataKey::NextId, &0_u64);
        env.storage().instance().set(&DataKey::LockedTotal, &0_i128);
    }

    pub fn token(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Token)
            .expect("uninitialized")
    }

    pub fn locked_total(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::LockedTotal)
            .unwrap_or(0)
    }

    pub fn get(env: Env, id: u64) -> Option<Warranty> {
        env.storage().persistent().get(&DataKey::Warranty(id))
    }

    pub fn get_claim(env: Env, id: u64) -> Option<Claim> {
        env.storage().persistent().get(&DataKey::Claim(id))
    }

    /// Seller funds an immutable pending offer; it does NOT activate the warranty.
    /// The existing create signature is retained, but its status semantics change.
    pub fn create(
        env: Env,
        seller: Address,
        buyer: Address,
        arbitrator: Address,
        bond_amount: i128,
        duration_seconds: u64,
        terms_hash: BytesN<32>,
    ) -> u64 {
        vault::assert_no_transfer(&env);
        validate_terms(&seller, &buyer, &arbitrator, bond_amount, duration_seconds);
        assert_ne!(
            terms_hash,
            BytesN::from_array(&env, &[0_u8; 32]),
            "terms hash is required"
        );
        let timestamp = env.ledger().timestamp();
        let offer_expires_at = timestamp
            .checked_add(types::OFFER_VALIDITY)
            .expect("offer deadline overflow");
        seller.require_auth();
        let token = Self::token(env.clone());
        let last_id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(0);
        let id = last_id.checked_add(1).expect("warranty id overflow");
        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::LockedTotal)
            .unwrap_or(0);
        let new_total = total
            .checked_add(bond_amount)
            .expect("locked total overflow");

        env.storage().instance().set(&DataKey::Depositing, &true);
        vault::deposit_exact(&env, &token, &seller, bond_amount);
        let warranty = Warranty {
            version: 1,
            id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            arbitrator,
            token,
            bond_amount,
            created_at: timestamp,
            offer_expires_at,
            duration_seconds,
            activated_at: 0,
            expires_at: 0,
            terms_hash,
            seller_approved: false,
            buyer_approved: false,
            status: WarrantyStatus::Pending,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Warranty(id), &warranty);
        env.storage()
            .instance()
            .set(&DataKey::LockedTotal, &new_total);
        env.storage().instance().set(&DataKey::NextId, &id);
        env.storage().instance().set(&DataKey::Depositing, &false);
        WarrantyCreated {
            id,
            seller,
            buyer,
            amount: bond_amount,
        }
        .publish(&env);
        id
    }

    /// An on-chain signature is scoped to contract, network, method and arguments.
    pub fn approve(
        env: Env,
        id: u64,
        signer: Address,
        expected_terms_hash: BytesN<32>,
        expected_bond_amount: i128,
        expected_duration_seconds: u64,
        expected_offer_expires_at: u64,
        expected_arbitrator: Address,
    ) -> bool {
        agreement::approve(
            &env,
            id,
            signer,
            expected_terms_hash,
            expected_bond_amount,
            expected_duration_seconds,
            expected_offer_expires_at,
            expected_arbitrator,
        )
    }

    /// Only after exact funding and both authorizations; repeat activation is a no-op.
    pub fn activate(env: Env, id: u64) -> bool {
        lifecycle::activate(&env, id)
    }

    /// Only the buyer may file a claim; the exact deadline is inclusive.
    pub fn submit_claim(
        env: Env,
        id: u64,
        buyer: Address,
        amount: i128,
        evidence_hash: BytesN<32>,
    ) {
        claim::submit(&env, id, buyer, amount, evidence_hash);
    }

    /// Both buyer and seller must authorize the identical payout in one invocation.
    pub fn settle(env: Env, id: u64, buyer_payout: i128) {
        settlement::settle(&env, id, buyer_payout);
    }
}

#[cfg(test)]
mod test;
