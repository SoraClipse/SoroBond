# Development, Testnet deployment and operational safety

**UNAUDITED, EXPERIMENTAL, TESTNET ONLY. DO NOT DEPOSIT REAL MONEY.**
No independent audit, hosted production deployment, validated wallet integration
or consumer-protection guarantee exists. SoroBond cannot determine whether a
physical product is defective or compel an independent arbitrator to act.

## Fresh clone: reproducible local checks

Requires Node 22.16+, Rust 1.84+ (stable recommended), rustfmt, clippy,
Stellar CLI compatible with soroban-sdk 29.0.1 and the
`wasm32v1-none` target. The current workspace is **Rust plus native Node ESM**:
it has no package.json, TypeScript files or pnpm lockfile. Accordingly no pnpm
install or TypeScript compiler invocation is claimed.

```sh
git clone https://github.com/SoraClipse/SoroBond.git
cd SoroBond
rustup component add rustfmt clippy
rustup target add wasm32v1-none
bash scripts/verify.sh
stellar contract build --manifest-path contracts/Cargo.toml
```

The CI workflow runs Rust and Node checks in separate least-privilege jobs.
The Stellar CLI is installed in CI from crates.io with its locked dependency
resolution. As no project Cargo.lock is checked in, `soroban-sdk` is pinned
at 29.0.1 but its transitive dependency graph is **not reproducibly pinned**
between fresh clones. Generating and reviewing a Cargo.lock is a separate
prerequisite to strict supply-chain reproducibility.

## Testnet deploy procedure (operator required)

Use only the public Stellar Testnet network. Check that the CLI, SDK and
target network protocol versions are mutually compatible before deploying.
Follow the current official CLI reference:
https://developers.stellar.org/docs/tools/cli/stellar-cli

1. Create and fund independent **test-only** seller, buyer, resolver and
   test-token administrator identities, using the CLI's secure identity store.
   Never commit secrets, post seed phrases or add them to GitHub Actions.
2. Build the reviewed contract with `stellar contract build --manifest-path
   contracts/Cargo.toml`. Inspect the resulting Wasm and note its checksum.
3. Deploy a separate reviewed test-only token contract first, recording its
   actual contract ID and decimals. Only use tokens designed for testing.
4. Deploy the SoroBond Wasm on Testnet, passing the token contract's real
   address to the constructor `allowed_asset`. Check `token()` returns the
   exact expected address before funding any warranty.
5. Submit each signed transaction from the authorized identity and check
   real Stellar RPC confirmation and decoded events; record transaction hashes,
   warranty IDs, network passphrase, timestamp and balance deltas.
6. Execute all three scenarios and negative checks in
   [the Testnet proof guide](../verification/testnet.md). Never claim local
   fixtures prove live deployment.

Do not blindly execute sample shell commands with a real signing key. CLI
arguments and auth conventions can change; review `stellar contract deploy
--help`, `stellar contract invoke --help` and the current network options
before authorizing a transaction.

## Contract and product limitations

A pending offer locks seller collateral before buyer acceptance, with no
cancellation/rescue endpoint. An accepted claim may remain locked if the
arbitrator disappears and the parties cannot agree. Expiry recovery requires
the specified 24-hour grace period; contract ledger TTL archival is not yet
managed. Buyer and seller must consent to the **same settlement invocation**;
the preselected resolver can independently choose a payout within the claim
cap. That resolver can be mistaken, unavailable or biased. Claim evidence is
only an on-chain hash, not an independently verified item condition.

Off-chain evidence storage, signed wallet UX, indexer and web adapters are
experimental; the browser requires integrations not shipped by default.
The contract can permanently lock funds; seek independent technical, legal
and financial review before any public deployment.

## PR acceptance and review

Run `bash scripts/verify.sh`, attach a real green GitHub Actions run, and have
a second contributor check links, steps and network assertions. Issue #19
requires real contract IDs and Testnet receipts. A skipped opt-in live test
is *not* a passing live verification. Do not close #19 or #20 based solely on
this documentation or an unexecuted workflow.
