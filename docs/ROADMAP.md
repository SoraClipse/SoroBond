# SoroBond engineering roadmap

> **Status:** Early engineering backlog. Planned features below are **not** implemented or verified merely because these issues exist. Maintainers should not advertise public-network readiness before independent review and reproducible testnet evidence.

## Product boundary

Soroban collateral-backed, explicitly limited warranties for peer-to-peer secondhand purchases; smart contracts cannot establish that a physical product is defective.

## Submission baseline — P0 (21 issues)

These must be implemented, reviewed, and demonstrated before presenting the project as a functioning testnet product. Work is sequenced by actual dependencies; do not infer parallel independence from issue numbering.

- [#1 — Specify collateral warranty threat model, actors and settlement rules](https://github.com/SoraClipse/SoroBond/issues/1)
- [#2 — Bootstrap Soroban Rust contract workspace and unit test harness](https://github.com/SoraClipse/SoroBond/issues/2)
- [#3 — Define versioned on-chain warranty agreement data model](https://github.com/SoraClipse/SoroBond/issues/3)
- [#4 — Enforce seller token authorization and exact bond deposits](https://github.com/SoraClipse/SoroBond/issues/4)
- [#5 — Require buyer and seller agreement to identical terms](https://github.com/SoraClipse/SoroBond/issues/5)
- [#6 — Activate warranties only after funding and mutual consent](https://github.com/SoraClipse/SoroBond/issues/6)
- [#7 — Implement buyer claim submission and freeze disputed bonds](https://github.com/SoraClipse/SoroBond/issues/7)
- [#8 — Implement mutually approved claim payout and closure](https://github.com/SoraClipse/SoroBond/issues/8)
- [#9 — Add explicitly preselected arbitration and dispute verdict enforcement](https://github.com/SoraClipse/SoroBond/issues/9)
- [#10 — Release unclaimed collateral after expiry with race-safe deadlines](https://github.com/SoraClipse/SoroBond/issues/10)
- [#11 — Enforce authorization, token whitelist and conservation invariants](https://github.com/SoraClipse/SoroBond/issues/11)
- [#12 — Build property-based tests for complete warranty state machine](https://github.com/SoraClipse/SoroBond/issues/12)
- [#13 — Implement typed Soroban RPC client and transaction tracking](https://github.com/SoraClipse/SoroBond/issues/13)
- [#14 — Build idempotent warranty event indexer and query API](https://github.com/SoraClipse/SoroBond/issues/14)
- [#15 — Protect private warranty evidence and claim metadata](https://github.com/SoraClipse/SoroBond/issues/15)
- [#16 — Build buyer/seller warranty negotiation and signing screens](https://github.com/SoraClipse/SoroBond/issues/16)
- [#17 — Build seller collateral deposit and bond status dashboard](https://github.com/SoraClipse/SoroBond/issues/17)
- [#18 — Build buyer claims and arbitrator decision workflow](https://github.com/SoraClipse/SoroBond/issues/18)
- [#19 — Prove successful and disputed Testnet collateral lifecycle end to end](https://github.com/SoraClipse/SoroBond/issues/19)
- [#20 — Set up Rust, TypeScript and Wasm GitHub Actions checks](https://github.com/SoraClipse/SoroBond/issues/20)
- [#21 — Publish real setup guide, arbitration disclosures and deploy playbook](https://github.com/SoraClipse/SoroBond/issues/21)

## Contributor extensions — P1 (6 issues)

These are real follow-up technical opportunities, not claims of present functionality. Reassess scopes as the architecture matures.

- [#22 — Release integration SDK for external secondhand marketplaces](https://github.com/SoraClipse/SoroBond/issues/22)
- [#23 — Provide contract upgrade and migration safety planning tools](https://github.com/SoraClipse/SoroBond/issues/23)
- [#24 — Export transparent warranty and settlement receipts](https://github.com/SoraClipse/SoroBond/issues/24)
- [#25 — Add opt-in dispute deadlines and status notifications](https://github.com/SoraClipse/SoroBond/issues/25)
- [#26 — Improve warranty purchase and claim flow accessibility](https://github.com/SoraClipse/SoroBond/issues/26)
- [#27 — Benchmark contract storage cost and transaction resource budget](https://github.com/SoraClipse/SoroBond/issues/27)

## Launch evidence checklist

- [ ] Fresh-clone install/build instructions work.
- [ ] Functional product behavior is demonstrated end to end.
- [ ] Positive, negative, and adversarial tests run in CI.
- [ ] Audited custody boundaries, immutable deal terms, token transfer safety, bounded warranty deadlines, buyer claims, explicit arbitration trust, deterministic settlement and reproducible testnet proof.
- [ ] Actual Stellar testnet transactions or Soroban invocations are recorded where relevant, with source links and reproducible instructions.
- [ ] Security and trust limits are documented; no public-funds guarantee is made.
- [ ] Deployment and operations are repeatable and secret-safe.
- [ ] Maintainers independently review claims before any Drips Stellar Wave application or issue nomination.

## Contributor workflow

Read [CONTRIBUTING.md](../CONTRIBUTING.md), select an open issue, discuss API and security assumptions with maintainers, and submit one focused PR with tests. Do not apply Stellar Wave reward labels unless the program approves the repository and selects the issue.
