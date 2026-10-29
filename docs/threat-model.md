# SoroBond — custody and adversarial threat model

**Not production ready.** A deposit-only smart contract can trap funds. Never use valuable assets until claim, expiry, settlement and recovery paths have been implemented, externally reviewed and validated on testnet.

## Trust boundaries

| Boundary | Threat | Mitigation and outstanding work |
| --- | --- | --- |
| Seller auth | Buyer or attacker creates a deposit from victim | Require seller address authorization in `create`; negative authorization test. |
| Token contract | Malicious/upgradable asset implementation or issuer clawback | Constructor pins a single token address, asserts actual contract balance delta; allow-list governance, issuer-risk analysis and adversarial token tests still needed. |
| Reentrancy | Token transfer invokes nested contract call | Storage reentrancy guard set before external invocation; cleared on success; panic rolls back transaction. |
| Storage | Unexpected duplicate IDs or overflow | Checked arithmetic, monotonic IDs and separate `Warranty(id)` keys; TTL and restoration still unresolved. |
| Arbitrator | Conflicted third party or key compromise | Require arbitrator distinct from buyer/seller. Actual authorization, replacement and dispute escalation not yet implemented. |
| Time | User supplies misleading expiry | Use ledger timestamp with 1h–90d checked duration; timestamp and ledger drift assumptions still need deployment review. |
| Sensitive goods details | On-chain irreversible PII exposure | Only store a fixed 32-byte terms hash; never place the private evidence itself in event fields. |
| Locked bonds | Inability to withdraw | **Present in this milestone.** No release or refund functions; do not deploy for actual user deposits. |
| Asset reserve/accounting | Repeated request or inconsistent token balance | Verify the exact contract balance increase before storing agreement and increment locked total. |

## Adversarial cases to test

Unknown signer; a buyer impersonating the seller; zero/negative amount; seller = buyer; arbitrator = counterparty; insufficient token balance; expired duration; overflow; duplicate invocation; wrong asset; token rollback; storage expiration; malicious reentrancy. The currently implemented tests cover the first subset. Remaining risks require later engineering and independent review.

## Reporting

See `SECURITY.md`. Do not post vulnerabilities, seed phrases or proofs containing identifying details as public issues.
