# SoroBond

SoroBond is an experimental **seller-funded, post-sale warranty bond** protocol for peer-to-peer purchases on Soroban. An on-chain contract cannot prove that a physical item is defective; the parties independently decide how to resolve a claim.

> **Security warning: DO NOT DEPOSIT REAL MONEY.** This remains an experimental contract. Expired/unclaimed bonds, abandoned pending offers, TTL archival, arbitration, upgrade safety, malicious tokens and independent security review are not yet handled. Funding this contract could permanently lock funds. There is no public-network readiness claim.

## Implemented contract features

- A Soroban contract using `soroban-sdk =29.0.1` with a constructor-pinned asset contract.
- Seller-authenticated, exact token deposits using a protected vault and checked accounting.
- **Pending** warranties that require separate seller and buyer `approve` authorizations for the exact immutable hash, amount, duration, arbitrator and offer deadline.
- **Idempotent activation** only after both approvals and a fully backed vault, emitting exactly one activation event.
- Buyer-only, bounded, one-time claims through the inclusive warranty deadline; a claim freezes the bond for mutual settlement.
- Full or partial settlement with **both parties authorizing the same invocation**; exact buyer payout, seller return and total collateral accounting are checked atomically.
- Local Soroban test fixtures for normal and adversarial lifecycle transitions.

## Developer commands

Prerequisites: Rust 1.84+ and a compatible Soroban/Stellar toolchain. From the repository root:

```sh
cd contracts
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
# Reviewed deployment builds only:
stellar contract build
```

The GitHub Actions PR workflow runs format, tests and Clippy. The contract has **not** been validated against an actual Stellar testnet deployment. A successful local test run is not a security audit.

## Lifecycle integration

1. Call `create(seller, buyer, arbitrator, amount, duration_seconds, terms_hash)`. The seller funds exactly `amount`, but the warranty remains **Pending** with a fixed 24-hour acceptance window.
2. Both seller and buyer independently call `approve(id, signer, expected_terms_hash, expected_bond_amount, expected_duration_seconds, expected_offer_expires_at, expected_arbitrator)` using their Soroban signing wallets. The ledger's network-specific authorization tree prevents changing the called contract or signed arguments. Wallet software must show the full approval details to the signer.
3. Any caller may invoke `activate(id)` before the offer deadline. It returns `false` if already Active, never emits a second activation event, and only activates when both approvals and actual funding exist. The warranty duration starts **at activation**.
4. The buyer may invoke `submit_claim(id, buyer, amount, evidence_hash)` while Active and at or before the warranty expiry timestamp, including exactly at the deadline.
5. To settle an active claim, the seller and buyer both authorize **the same** `settle(id, buyer_payout)` call. The remaining bond goes back to the seller. The state becomes permanently Settled; fees are zero.

No `withdraw`, `refund_expired`, `arbitrate`, `cancel_offer` or emergency rescue endpoint exists yet; a never-claimed or abandoned bond may remain locked. **Do not use real assets** until these missing exits, TTL management and independent review are completed.

## Documentation

- [Product agreement model](docs/product-spec.md)
- [Lifecycle, authorization and event specification](docs/lifecycle-security.md)
- [Threat model](docs/threat-model.md)
- [Roadmap](docs/ROADMAP.md)
- [Issues](https://github.com/SoraClipse/SoroBond/issues)
- [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)

## License

Apache-2.0. Independent work, not endorsed by the SDF or Drips.
