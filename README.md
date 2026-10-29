# SoroBond

SoroBond is an experimental **seller-funded, post-sale warranty bond** protocol for peer-to-peer purchases on Soroban. A person-to-person warranty cannot verify physical faults on-chain. The eventual warranty protects only the terms and bond actually agreed by the parties.

> **Security warning: DO NOT DEPOSIT REAL MONEY.** This first contract milestone implements **deposit only**. Claims, arbitrator decisions, refunds, expiry-based withdrawals, TTL extension and independent contract security review are not implemented. Funding this draft contract could permanently lock funds. No public-network readiness is claimed.

## Initial implementation (Issues #1–#4)

- Rust contract using `soroban-sdk =29.0.1` with initialized allow-listed asset address.
- Versioned `Warranty` record: seller, buyer, independent arbitrator, asset contract, exact bond, ledger-backed bounded expiry, fixed-size off-chain terms hash, `Active` status.
- Seller `require_auth`; `token::Client` transfer to contract; strict observed balance delta check; overflow-checked accounting and reentrancy guard.
- `WarrantyCreated` event emitted on success, plus read-only queries and mock authorization/real in-memory Stellar Asset Contract tests.

## Developer commands

Prerequisites: Rust 1.84+, Soroban-compatible toolchain; install Stellar CLI >=29 for `soroban-sdk` 29 according to the Stellar CLI/SDK compatibility guidance. Then:

```sh
cd contracts
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
# For future reviewed deployment builds only:
stellar contract build
```

The source tree includes a GitHub Actions CI workflow running formatting, tests and Clippy. No local Rust toolchain was available in the initial authoring environment; CI results should be checked before merging any PR.

## Documentation

- [Detailed product and agreement model](docs/product-spec.md)
- [Threat model and deployment blockers](docs/threat-model.md)
- [Roadmap](docs/ROADMAP.md)
- [Issues](https://github.com/SoraClipse/SoroBond/issues)
- [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)

## License

Apache-2.0. Independent work, not endorsed by the SDF or Drips.
