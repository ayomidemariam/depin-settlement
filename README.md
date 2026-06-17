# 📡 DePIN Oracle-Triggered SLA Settlement

A conditional payment and settlement smart contract built on **Stellar's Soroban** network. This contract holds stablecoins (via the Stellar Asset Contract interface) in escrow and conditionally unlocks stream payouts to Decentralized Physical Infrastructure Network (DePIN) hardware nodes strictly based on verified uptime metrics reported by an authorized on-chain Oracle.

---

## 🌊 Drips Wave Program

This repository actively participates in the **Drips Wave Program** — recurring 7-day contribution sprints where developers earn rewards based on issue complexity.

### The Workflow

1. **Find an Issue** — Look for issues tagged with `wave` and a complexity label (`trivial`, `medium`, `high`).
2. **Assign** — Comment `.take` on the issue to assign it to yourself.
3. **Fix & Test** — Submit a PR meeting our strict testing standards.
4. **Merge & Earn** — Once merged, points are automatically credited to your Drips profile.

### 🎯 Current Wave Issues & Point Matrix

#### 🔴 High Complexity — 200 Points
Requires deep architectural understanding of the Soroban SDK and cross-contract interactions.

| Issue | Description |
|-------|-------------|
| **#22** | **Implement Cross-Contract Oracle Verification** — Build the cross-contract call interface that ingests and verifies uptime data from the external Oracle contract. The logic must securely update the node's payout state in persistent storage only if the Oracle's authorization signature is valid. **Requirements:** Prevent reentrancy attacks and optimize state updates to minimize ledger read/write fees. |

#### 🟡 Medium Complexity — 150 Points
Standard smart contract feature development and rigorous test engineering.

| Issue | Description |
|-------|-------------|
| **#25** | **Develop Edge-Case Test Suite for Payout Streams** — Create comprehensive Rust unit tests utilizing Soroban's `testutils` feature. Mock the environment (`Env`) to simulate scenarios like 0% uptime, delayed Oracle reports, and network fee spikes to ensure the contract never locks funds permanently. **Requirements:** Achieve 95% line coverage for the `payout.rs` module. |

#### 🟢 Trivial Complexity — 100 Points
Clear, bounded structural changes ideal for first-time Soroban contributors.

| Issue | Description |
|-------|-------------|
| **#29** | **Optimize WASM Output and Implement `#![no_std]`** — Audit workspace crates to ensure all Rust modules begin with `#![no_std]` to prevent the standard library from bloating the contract. Update `Cargo.toml` release profile with `opt-level = "z"` to minimize compiled WebAssembly size. **Requirements:** The compiled `.wasm` file must be under 64 KB. |

---

## ✅ Global Acceptance Criteria

| Criterion | Requirement |
|-----------|-------------|
| **Test Coverage** | Strict **95%** line coverage on new code via `cargo test` |
| **WASM Optimization** | Must pass CI build step checking for optimized binary size (Soroban's 64 KB limit) |
| **Linting & Formatting** | Run `cargo fmt` and `cargo clippy --all-targets` before committing |
| **Descriptive PRs** | Pull requests must link to the specific Wave issue (e.g., `Closes #22`) |

---

## 🛠️ Local Development Setup

### Prerequisites

- **Rust** (v1.84.0+) and the `wasm32-unknown-unknown` target
- **Soroban CLI** installed

### 1. Clone & Build

```bash
git clone https://github.com/yourusername/depin-oracle-settlement.git
cd depin-oracle-settlement
cargo build --target wasm32-unknown-unknown --release
```

### 2. Run Tests

```bash
cargo test --features testutils
```

---

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
