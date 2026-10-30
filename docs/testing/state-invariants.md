# SoroBond dispute, expiry and conservation invariants (#9–#12)

This is an **experimental Soroban contract extension**, not a production warranty service, legal arbitration guarantee or proof of live testnet settlement.

## Contract states and deadlines

- `Pending`: seller funds a fixed allow-listed asset; both parties independently approve matching, versioned terms. Activation requires both signatures, adequate collateral and an unexpired offer (Issues #5–#8).
- `Active`: buyer alone may call `submit_claim` until **and including** the warranty expiry timestamp. One claim only; its amount is `1..=bond`.
- `Claimed`: seller cannot withdraw, even long after expiry. The preselected resolver may call `arbitrate(id, buyer_payout)`, with `0..=claim.amount`, or seller and buyer may jointly call `settle(id, buyer_payout)` under the same bound. Both produce `Settled`.
- `Active -> Expired`: only the seller may call `withdraw_expired(id)` when ledger timestamp is **strictly greater** than `expires_at + 86400`; equality fails. Any recorded claim blocks expiry return.
- `Settled` and `Expired` are irreversible. Failed calls revert; races are serialized by Soroban ledger execution.

## Role and accounting invariants

The arbitrator and all financial terms are fixed in the jointly approved offer. No arbitrator may edit an offer or transfer any other bond or other asset. Every release is from the constructor-pinned asset contract only; token debits are split between the exact original seller and buyer. A terminal release reduces `locked_total` by the original bond, with checked arithmetic and strict token balance-delta/recipient receipt checks. Unrelated deposits must remain fully backed. Storage updates precede external transfers behind the contract-wide reentrancy guard, and Soroban reverts the entire invocation on any transfer failure.

## Reproduction and limitations

From `contracts/`: `cargo fmt --check`; `cargo test`; `cargo clippy --all-targets -- -D warnings`. The deterministic state-sequence test uses seed `0x6c627dd19c5844e3`, covering claims, mutual settlements, arbitration, expiry boundary, replay, wrong signers, and global balance conservation with real in-memory Stellar Asset Contract instances. This is not external fuzzing nor live network evidence.

**Known risks:** unfinished Pending-offer cancellation, unavailable/corrupt arbitrator potentially freezing claimed funds indefinitely, unverified off-chain evidence, no ledger TTL/storage archival restoration, token admin/clawback/upgrades, Soroban deployment and upgrade reviews not completed. Do not deposit real assets.
