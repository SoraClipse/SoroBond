# Issue #19 acceptance and evidence ledger

**Live Testnet lifecycle status: NOT EXECUTED or independently verified.**
This repository does not contain a contract deployment address, genuine
transaction hashes or a proven full testnet lifecycle. No signatures or
transactions have been submitted by this change.

The existing `testnet-warranty.test.mjs` runs only when both
`SOROBOND_TESTNET_MANIFEST` and `SOROBOND_TESTNET_ADAPTER` are specified.
Otherwise it reports SKIP; this is intentional and cannot be reported as
an end-to-end pass.

### Evidence that must be attached before closing #19

- Real Testnet contract and token IDs with constructor-pinned token checks
- Three warranties with real transaction hashes: clean expiry, mutual claim
  settlement, and resolver arbitration
- Real failed unauthorized-settlement and repeated-withdraw transactions
- Exact payout and collateral balance reconciliation; unmodified RPC events
- Operator date, ledger heights, source commit, passphrase and verifiable
  explorer/RPC links, with all private signing material omitted
- A genuinely passing opt-in proof verification run with a reviewed adapter

**Timing limitation:** the earliest clean expiry requires an activated
warranty with minimum one-hour term plus the 24-hour grace interval on actual
Stellar ledger time. Local clock manipulation or mock tests do not count.

Node fixtures deliberately cover rejection paths, but cannot attest to
network deployment or signed transactions. CI tests independently cover
malformed proof, wrong network, payout mismatch, unauthorized settlement
not rejected, and duplicate withdrawal not rejected.
