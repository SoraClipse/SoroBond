# SoroBond — custody and adversarial threat model

**Not production ready.** Funds can be permanently locked if mutual settlement is never reached, the offer expires without agreement, the preselected resolver becomes unavailable, or contract state is archived. Use only local test tokens.

## Trust boundaries

| Boundary | Threat | Mitigation and remaining risk |
| --- | --- | --- |
| Seller and buyer auth | Unilateral changes or forged terms acceptance | Each separately calls `require_auth` for same immutable terms, amount, duration, arbitrator and offer expiry, scoped to contract and network. Wallet UI and actual signature integration need real-network review. |
| Deposit custody | Fake or insufficient collateral | Seller signs token deposit, observed balance delta checked, total reserved collateral tracked with checked arithmetic. |
| Activation | Unfunded warranty or buyer never agreeing | Pending until both consent, actual token balance checked, idempotent activation publishes once. |
| Deadline | Late claim or expired offer | Explicit ledger timestamps; activation at/before offer expiry, buyer claim at/before warranty expiry (inclusive). |
| Claim | Seller manipulation, repeat/over-limit claim | Only buyer-authenticated Active claim; single Claimed transition; amount at most bond. Physical defect not independently established. |
| Disputed bond | Seller withdrawing during a claim | Claimed state blocks seller-only expiry return and requires mutual settlement or an authorized verdict from the immutable resolver. **Deadlock/locked funds remain possible.** |
| Payout | Duplicate spend, partial token loss | Mutual payout requires both parties, or resolver verdict requires the preselected arbitrator with a claim-bounded award. Terminal Settled/Expired states, all token transfers verified and atomic. No fees. |
| Reentrancy | Malicious token calls back during transfer | Reentrancy guard blocks all state mutations during deposit/settlement. Requires adversarial token review. |
| Token/issuer risk | Clawback, upgrade, malicious token implementation | Asset contract fixed at constructor, exact balance checks. Cannot defend against malicious token admin or non-standard asset. |
| Storage | TTL archival, lost records | **Not solved.** Persistent and instance key TTL policies, restore flows and retention cost need implementation before actual deployment. |
| Item evidence | Leaked private purchase details | Only a 32-byte evidence hash and terms hash stored on-chain, not private documents. |
| Arbitration | Parties disagree, one refuses to sign | The original preselected arbitrator may finalize only a recorded claim; unavailable or dishonest arbitration remains a custody and fairness risk. |

## Attack tests

Unit tests exercise invalid amounts, distinct-role validation, missing or altered approvals, late offer, no-funding activation, repeated activation, forged caller identity, late/duplicate/oversized claims, payout over the claim limit, repeated payout and token failure rollback. Local `mock_all_auths` only simulates valid wallet authorization and cannot prove actual signatures or domain restrictions on a live network.

## Deployment warning

Unclaimed Active collateral can be withdrawn by its seller strictly after expiry plus 24 hours. Pending offer cancellation is still missing. No deployed product or testnet transfer is proven here. The old `create` semantics are changed from immediately Active to Pending; the expanded `Warranty` serialized schema is not a safe in-place upgrade. Review and redeploy from a clean instance for local testing. Do not deposit valuable assets until expiry/cancellation/refund, TTL restoration and independent security review exist.
