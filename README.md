# SoroBond

**A proposed Soroban-based warranty-bond protocol for person-to-person secondhand sales.**

> **Project status: early development.** A repo and detailed [engineering backlog](https://github.com/SoraClipse/SoroBond/issues) exist, but the collateral contract and end-to-end purchase workflow are not currently implemented, security reviewed or deployed here. Do not lock real-value tokens into unreviewed contracts.

## What problem are we addressing?

A buyer of a secondhand phone, laptop, or other item may discover an agreed warranty fault after paying a seller. Ordinary payment escrow ends when the sale closes. SoroBond aims to let a seller voluntarily lock a limited, clearly defined token bond **after purchase** for a bounded period, with transparent claim and settlement rules.

## Planned user workflow

1. Buyer and seller review and sign precise warranty terms: item description, covered faults, maximum bond, expiration, accepted token and dispute resolver.
2. The seller funds a Soroban warranty bond with a supported Stellar asset. Funding confirmation is verified on-chain.
3. During the stated window, the buyer can file an eligible claim with off-chain, access-controlled evidence; the contract preserves the bond under clearly specified rules.
4. Both parties may agree on a settlement or follow a pre-agreed resolver process with auditable decisions and bounded authority.
5. Once settled or expired, the Soroban contract releases the bond according to the agreed state machine; double-spending, replay and invalid transitions must be rejected.

## Core engineering principles

- **Limited protection, not insurance:** The protocol cannot independently verify a real-world defect or guarantee full reimbursement beyond the agreed bond.
- **Custody safety:** Contract authorization, token handling, dispute resolution and deadline rules must have malicious-party and edge-case tests.
- **Clear risk disclosure:** Buyer and seller must understand what a bond covers, the resolver's power, expiry conditions and cases where funds may be locked or lost.
- **Testnet first:** Fund movement and claim flows need reproducible Soroban testnet evidence and rigorous review before public-network deployment.
- **Open development:** Reviewable Rust contract code, explicit storage transitions, real tests and usable frontend/backend integration.

## Project planning and contribution

- [Engineering roadmap and issue breakdown](docs/ROADMAP.md)
- [Open issues](https://github.com/SoraClipse/SoroBond/issues)
- [Contributor guide](CONTRIBUTING.md)
- [Security reporting](SECURITY.md)

The P0 backlog defines a credible testnet submission baseline; the P1 backlog holds future, independently scoped work. Neither backlog implies official Stellar or Drips endorsement, guaranteed approval or an active reward program.

## License

See [LICENSE](LICENSE).
