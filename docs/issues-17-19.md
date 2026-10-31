# SoroBond seller funding, claim dashboard and Testnet proof (#17–#19)

**Status:** Frontend controllers, user-facing routes, deterministic tests and an opt-in Testnet proof verifier are implemented. Real deployment and actual transaction evidence have NOT been collected or verified. Issue #19 cannot be closed on the basis of unit tests.

Run this implementation in the existing lightweight Node 22 project:

    node --test packages/sdk/test/*.test.mjs apps/api/test/*.test.mjs apps/web/tests/*.test.mjs tests/e2e/*.test.mjs
    node apps/api/src/web-server.mjs

Open /warranties/fund and /warranties/claims (or /warranties/new for the pre-existing immutable terms review). These are static HTML/ES module routes, not a newly invented Next.js workspace.

## Trusted integration required

The UI expects a trusted application integration at window.sorobondApp containing:

- network: "testnet"; contractId: exact deployed Soroban contract address
- rpc: instance of the existing packages/sdk/src/client.mjs SorobanClient, network-bound to Testnet
- chain: an audited Soroban contract adapter declaring network and contractId, with readPinnedToken, readBalance, readLockedTotal, readWarranty, readCreatedWarranty, readClaim, ledgerTimestamp, readVerdict, and buildInvocation
- wallet: authenticated connect(), signTransaction(prepared, details), and signJointTransaction(prepared, {requiredSigners:[buyer,seller]}) for mutual settlements; never substitute a single wallet signature
- evidenceVault: a vetted authenticated adapter with clientSideEncryption===true and encryptAndStore({warrantyId,actor,parties,bytes}) that encrypts evidence **before network transport**, returns a real private receipt and the correct SHA-256 warranty-bound commitment

No live wallet integration, compiled XDR builder, transaction signer, persistent encrypted evidence provider or deployed contract is included here; buttons fail closed until supplied. Do not invent a wallet confirmation or expose a test key via window variables.

The contract's create method performs an **atomic seller-authorized token transfer**; do not separately transfer tokens to the contract address. Read and display the exact pinned asset contract, source amount in token base units and liabilities. No funded label before RPC status CONFIRMED and a matching on-chain created warranty. A Pending warranty is funded but is not yet activated; seller/buyer approvals and activation remain separate actions from #5–#8.

All claim roles are determined from actual on-chain addresses: only buyer claims at or before expires_at (inclusive), only the preselected resolver arbitrates, both seller and buyer authorize identical mutual settlement, and only seller withdraws after expiry + 86400 seconds (strict boundary) if no claim exists. Transaction state is re-read after every successful RPC status. Contract authentication is the final authority, never UI button visibility.

The evidence uploader limits files to 1 MiB, hashes warrantyId + ":" + original bytes and requires a matching encrypted-vault commitment before enabling claim submission. The on-chain claim stores **only a 32-byte commitment**. The backend EvidenceVault in #15 was in-memory and is not a production evidence provider. Actors must originate from authenticated wallet sessions, never arbitrary strings supplied by browser request parameters. Contract status and verdict data must be verified against official Stellar Testnet RPC receipts, not synthesized adapter data.

## Remaining blockers

The browser is an integration shell rather than a deployed end-to-end wallet app. Before public/Testnet demonstration: connect an audited wallet/XDR builder, token decimals metadata, authenticated encrypted vault, contract event decoding, deployed contract identity and secure off-chain evidence retention. Confirm TTL restoration, unresolved pending offers and legal warranty limitations. **Use Testnet test assets only.**

For issue #19 see docs/verification/testnet.md. Real confirmed transaction hashes and negative outcome receipts are required; no evidence means a deliberately SKIPPED live proof test, not green proof of a full lifecycle.

