# Required real Stellar Testnet lifecycle verification (#19)

**Status:** NOT EXECUTED. No actual deployed SoroBond contract, funded Testnet warranty, ledger receipt or live proof is claimed in this branch. The automated verifier is implemented, but an operator must supply real transactions and a trusted receipt decoder.

## Setup and execution

1. Install a supported Stellar CLI and compile the Soroban contract (contracts/Cargo.toml specifies soroban-sdk 29.0.1). Check cargo fmt, cargo test and cargo clippy first.
2. Configure Stellar **Testnet** only, establish three independent funded *test* accounts (seller, buyer and resolver) plus an issuer/admin for a test asset; never commit seed phrases or sign with real customer accounts. Validate the network passphrase.
3. Deploy the pinned test asset contract and SoroBond with that exact token address, record the contract ID and token ID. Validate actual allowlisted identity and token decimals.
4. For the unclaimed expiry scenario: seller create, seller approve, buyer approve, activate. Wait for at least 1 hour (MIN_DURATION) **plus 24 hours (EXPIRY_GRACE)** of real Testnet ledger time, then seller withdraw_expired. At exact expiry + grace the call must fail.
5. For mutual settlement: create a distinct bond, approve twice, activate, buyer submit_claim with privately stored evidence commitment, both parties authorize settle; verify terminal status and exact token balances.
6. For arbitrated dispute: another distinct bond, approve twice, activate, buyer submit_claim, only the preselected resolver authorizes arbitrate; verify the immutable verdict and payout.
7. Negative proofs: send an unauthorized settle or arbitration attempt and a second attempt to withdraw an already closed bond. Record actual failed Testnet transaction hashes and rejection outcome; simulation failures **are not submitted transactions** and cannot be substituted for real failed receipts.
8. Verify all receipts through an independently reviewed reader using official Soroban RPC events (contract ID, transaction hash, ledger, contract event and exact token identity). Record actual transaction hashes with the 13-entry manifest.

## Evidence format

Create a local JSON file (not a fake filled-in example) with this structure:

    {
      "network":"testnet",
      "contractId":"<actual C... contract StrKey>",
      "token":"<actual C... token StrKey>",
      "scenarios":{
        "expiry":{"warrantyId":1,"transactions":{"created":{"hash":"<real SHA-256>"},"activated":{"hash":"<real SHA-256>"},"expired":{"hash":"<real SHA-256>"}}},
        "mutual":{"warrantyId":2,"transactions":{"created":{"hash":"<real SHA-256>"},"activated":{"hash":"<real SHA-256>"},"claimed":{"hash":"<real SHA-256>"},"settled":{"hash":"<real SHA-256>"}}},
        "arbitration":{"warrantyId":3,"transactions":{"created":{"hash":"<real SHA-256>"},"activated":{"hash":"<real SHA-256>"},"claimed":{"hash":"<real SHA-256>"},"arbitrated":{"hash":"<real SHA-256>"}}}
      },
      "negative":{"unauthorized_settlement":{"hash":"<real failed SHA-256>"},"double_withdraw":{"hash":"<real failed SHA-256>"}}
    }

Warranty IDs above are **illustrative structure, not evidence**; replace all of them with actual chain data. Store the manifest without private keys, transaction envelopes, personal evidence or encrypted evidence decryption keys.

Provide an operator-reviewed module exporting async createTestnetReaders(manifest), returning:
- rpc: a Testnet SorobanClient instance, with checkNetwork() and status(hash)
- reader: {network:"testnet", contractId:manifest.contractId, readReceipt(hash)} using genuine RPC event data. The receipt must contain exact contractId, transactionHash, warrantyId, ledger, phase and token; terminal receipts additionally include decimal-string bondAmount, buyerPayout and sellerReturn.

This contract reader is **not provided in this repo** because XDR decoding and signed transaction tooling are not yet integrated and no Testnet deployment identifiers are known.

Run the live validator on an explicitly opted-in environment:

    SOROBOND_TESTNET_MANIFEST=/absolute/path/to/real-manifest.json \
    SOROBOND_TESTNET_ADAPTER=/absolute/path/to/verified-reader.mjs \
    node --test tests/e2e/testnet-warranty.test.mjs

A skipped live test **does not satisfy** issue #19. The verifier asserts the official Testnet passphrase, confirmed transactions, independent contract events, increasing ledgers, asset identity, strict payout conservation, two failed transactions, unique identifiers and distinct warranties. Review the verified output and external explorer links in the PR before marking the issue complete.

Testnet ledgers cannot be locally fast-forwarded; duration plus expiry grace is real elapsed time. The protocol still has unresolved pending-offer, TTL and security-audit risks, even after verification. DO NOT DEPOSIT REAL MONEY.

