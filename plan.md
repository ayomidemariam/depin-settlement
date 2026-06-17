# DePIN Settlement — Contribution Plan

## Overview

The DePIN Settlement contract automates SLA-backed payouts for decentralized infrastructure nodes. Work items are tracked as scoped GitHub issues and assigned during sprint cycles. Each issue includes clear acceptance criteria, a defined scope of work, and links to relevant code.

---

## Issue Types

### Bug Fixes

- **Reentrancy edge cases**: Identify and patch paths where the reentrancy guard (`InProgress` flag) could be bypassed during cross-contract oracle calls.
- **Payout rounding**: Handle remainder dust when `actual_payout` does not divide evenly by the settlement token's minimum unit.
- **Oracle verification failure recovery**: Ensure contract state resets cleanly when an oracle call fails mid-report.
- **Token decimal mismatch**: Validate that deposit/payout logic works correctly across tokens with non-standard decimal places.
- **Storage TTL expiry**: Ensure temporary storage entries for node info are bumped correctly before expiry on Stellar.

### New Features

- **Multi-token support**: Allow each node to pick a settlement token at registration time rather than using a single contract-wide token.
- **Slashing conditions**: Implement a penalty mechanism that deducts from escrow when uptime falls below a severe threshold.
- **Oracle rotation**: Allow the admin to update the oracle contract address after initialization.
- **Batch payouts**: Add a function that processes multiple node reports in a single transaction to reduce fee overhead.
- **Pause / emergency stop**: Implement a circuit breaker that freezes deposits and payouts during critical incidents.

### Documentation

- **Inline docs**: Add Soroban contract function documentation (`#[doc]`) for all public methods.
- **README improvements**: Document deployment steps, oracle contract interface requirements, and testnet addresses.
- **Architecture diagram**: Create a Mermaid diagram showing the oracle-settlement interaction flow.

### Testing

- **Fuzz testing**: Add property-based tests for the payout calculation logic with random uptime values.
- **Integration tests**: Write end-to-end tests that deploy both contracts and simulate a full sprint cycle.
- **Edge case coverage**: Test maximum escrow amounts, zero-uptime payouts, and simultaneous node registrations.
- **Failure simulation**: Test oracle timeout scenarios, invalid signatures, and unregistered node reports.

### Performance & Optimization

- **WASM size reduction**: Audit dependencies and remove unused imports to keep the compiled contract under 64 KB.
- **Storage optimization**: Merge adjacent storage keys where possible to reduce ledger entry footprint.
- **Gas profiling**: Benchmark `submit_oracle_report` and `deposit` to identify costly operations.

---

## Contributor Guidelines

- All PRs must include tests for new logic and pass `cargo test --features testutils`.
- Run `cargo fmt --check` and `cargo clippy --lib` before submitting.
- Reference the issue number in the PR title and description.
- Tag maintainers for review once CI is green.
