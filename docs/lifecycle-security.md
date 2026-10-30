# SoroBond: mutual agreement, activation, claim and settlement (Issues #5–#8)

This repository implements a **local-contract-tested mechanism only**. No live deployment, professional security review, physical defect oracle, adjudication or refund guarantee is implied.

## Authentication and domain binding

All actor signatures use Soroban's built-in `Address::require_auth()`, **not raw off-chain signatures or a reusable generic hash**. Soroban signs an authorization tree bound to the executing contract address, function name, arguments and Stellar network ID. Every signed approval includes the warranty id, signer, expected off-chain terms hash, bond amount, duration, acceptance deadline and arbitrator. Both party identities and the pinned token contract are immutable fields of the referenced warranty. Altered parameters fail even when the caller is otherwise authorized. A given authorization cannot authorize a different warranty id, contract instance, function or network by replaying the signature. Use properly configured live wallet signature providers; `mock_all_auths` is strictly for local unit tests and proves no production wallet integration.

## Timelines and status

- `create` requires the seller's authorization and transfers the exact bond into the contract. A deposit produces a **Pending** offer, NOT an Active warranty. `offer_expires_at = created_at + 86400` (24 hours). The warranty expiry is uninitialized (0) until activation.
- Seller and buyer must each call `approve` at or before `offer_expires_at` and use the exact same stored parameters. Approval is monotonic and repeated identical authorization is an idempotent no-op (`false`). There are no editable terms.
- `activate` is permissionless after both approvals and only on/before the offer deadline; it requires the vault's actual token balance to cover **all** outstanding locked balances. It records `activated_at`, sets `expires_at = activated_at + duration_seconds`, changes state to Active and emits one `WarrantyActivated` event. Retrying while already active returns `false` and does not publish another event.
- `submit_claim` requires the on-chain buyer's authorization, `0 < amount <= bond_amount`, Active status and ledger timestamp `<= expires_at`. **The boundary timestamp is included.** One claim is stored per warranty under a unique key, with an opaque 32-byte evidence hash; a claim moves the warranty to Claimed. The seller cannot settle or withdraw unilaterally.
- `settle` requires **both seller and buyer** to authorize the **same function invocation** with the identical `buyer_payout`; Claimed state is required. Allowed `0 <= buyer_payout <= claim.amount`. `seller_return = bond_amount - buyer_payout`. Both payouts are transferred in the immutable pinned token, with observed token balance deltas verified. There are no protocol fees. The full bond is deducted from `locked_total`; both transfers and the terminal Settled record are atomic. All future settlement attempts fail.

All state transitions are enforced on-chain. Unauthorized, late, reentrant and incorrectly scoped calls panic/revert. If either asset transfer fails, Soroban rolls back **the entire invocation** (state, event and token movements).

## Risks and missing pathways

**Important:** The original deposit-only contract is no longer safe to treat as backward-compatible: `create` now records Pending and adds fields to serialized `Warranty`. Do not upgrade an existing deployed instance without a reviewed migration (and no production deployment is claimed).

- There is **no recovery path for an abandoned Pending offer**, an Active warranty without a claim, or a Claimed warranty when one participant refuses settlement. Expiry-based withdrawal, dispute adjudication, cancellation, dispute timeout and emergency recovery are not included in #5–#8. Deposits in these states can be permanently locked.
- Persistent records and instance storage do not yet extend or recover ledger TTL. Long-lived warranties may become inaccessible after archival, potentially locking assets.
- No physical goods verification or arbitrator enforcement exists; arbitrator identity is agreed and immutable but not yet authorized to resolve a dispute.
- Only the constructor-pinned asset is supported. Its administrator, clawback, upgrades, bad token behavior, token event semantics and exchange rate remain externally trusted.
- The per-invocation reentrancy guard protects deposit/settlement and all state transitions. The global `locked_total` is checked with checked integer arithmetic. A deliberately drained vault is rejected, but arbitrary token clawback or admin intervention can still break solvency.
- Claim evidence is a hash, NOT a validated proof. The parties must retain private evidence off-chain.
- The local test environment uses a Stellar Asset Contract and mock wallet authorizations; no genuine production signatures or mainnet funds were used.

## Verification

```sh
cd contracts
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

The test suite covers missing and mismatched approvals, offer expiry, underfunding, idempotent activation, buyer-only claims, exact deadline behavior, duplicate/over-limit claims, partial/zero payout, payout caps, repeated settlement and token transfer failure rollback. Check the **actual GitHub Actions CI result** after opening a PR; do not infer a pass before it runs.

Do not deploy or deposit valuable assets before expiry and dispute exit paths, TTL restoration, independent audit, adversarial token testing and real testnet verification.

## Additional exits and arbitration (#9–#12)

A Claimed warranty can end either through the existing jointly authorized `settle(id, payout)` call or through `arbitrate(id, payout)` authenticated by the *immutable arbitrator* selected at offer creation. An arbitration payout is bound to `0 <= payout <= stored claim amount`; the remainder of the same bond returns to the seller. Neither arbitrator nor user can choose another token or destination.

The seller may call `withdraw_expired(id)` only for an unclaimed Active warranty when ledger timestamp is strictly later than `expires_at + 86400`. At equality it fails, and a claim filed exactly at `expires_at` permanently closes this seller-only exit. The claimant and original seller cannot double-spend across any terminal paths. Shared accounting checks token identity, checked liabilities, vault balance and exact transfers atomically. Pending offers still have no cancellation path; archival/TTL and lost arbitrator failure remain significant blockers. See [state invariants](testing/state-invariants.md).
