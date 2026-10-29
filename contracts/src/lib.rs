#![no_std]
mod types;
mod vault;

use soroban_sdk::{contract, contractevent, contractimpl, Address, BytesN, Env, Event};
use types::{validate_terms, DataKey, Warranty, WarrantyStatus};

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

#[contractimpl]
impl SoroBond {
    /// Install with the exact, reviewed token contract address (issuer/asset identity).
    pub fn __constructor(env: Env, allowed_asset: Address) {
        env.storage().instance().set(&DataKey::Token, &allowed_asset);
        env.storage().instance().set(&DataKey::NextId, &0_u64);
        env.storage().instance().set(&DataKey::LockedTotal, &0_i128);
    }

    pub fn token(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Token).expect("uninitialized")
    }

    pub fn locked_total(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::LockedTotal).unwrap_or(0)
    }

    pub fn get(env: Env, id: u64) -> Option<Warranty> {
        env.storage().persistent().get(&DataKey::Warranty(id))
    }

    /// Authenticated exact token transfer; emits event only on successful transfer.
    /// NO claim, resolution, expiry withdrawal or settlement function exists yet.
    pub fn create(
        env: Env,
        seller: Address,
        buyer: Address,
        arbitrator: Address,
        bond_amount: i128,
        duration_seconds: u64,
        terms_hash: BytesN<32>,
    ) -> u64 {
        assert!(!env.storage().instance().get::<_, bool>(&DataKey::Depositing).unwrap_or(false), "reentrant deposit");
        validate_terms(&seller, &buyer, &arbitrator, bond_amount, duration_seconds);
        let timestamp = env.ledger().timestamp();
        let expires_at = timestamp.checked_add(duration_seconds).expect("deadline overflow");
        let token = Self::token(env.clone());
        // This authorization is independent of the underlying asset transfer authorization.
        seller.require_auth();
        let next_id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(0);
        let id = next_id.checked_add(1).expect("warranty id overflow");
        let total: i128 = env.storage().instance().get(&DataKey::LockedTotal).unwrap_or(0);
        let new_total = total.checked_add(bond_amount).expect("locked total overflow");

        // Set before external token call to guard recursive callbacks; rollback occurs on panic.
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
            expires_at,
            terms_hash,
            status: WarrantyStatus::Active,
        };
        env.storage().persistent().set(&DataKey::Warranty(id), &warranty);
        env.storage().instance().set(&DataKey::LockedTotal, &new_total);
        env.storage().instance().set(&DataKey::NextId, &id);
        env.storage().instance().set(&DataKey::Depositing, &false);
        WarrantyCreated { id, seller, buyer, amount: bond_amount }.publish(&env);
        id
    }
}

#[cfg(test)]
mod test;
