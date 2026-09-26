# Contributing to Orbity

Thank you for your interest in contributing to **Orbity**! As a high-performance, deterministic multi-agent harness written in Rust, we welcome contributions ranging from bug reports and documentation enhancements to major features and performance optimizations.

---

## 1. Development Principles

- **Zero-Unchecked Invariants:** Every task execution, state transition, and FinOps budget update must be strictly validated.
- **Strict Confinement:** Sandboxing and security boundaries (Bubblewrap) must never be bypassed or weakened.
- **Immutable Auditability:** Any new event type must integrate into the SHA-256 tamper-evident audit store.
- **Topological Determinism:** Workflows must follow deterministic DAG graph evaluation without uncontrolled cycles.

---

## 2. Local Setup & Toolchain

### Prerequisites
- **Rust Toolchain:** Version $\ge 1.80$ (2021/2024 edition compatible):
  ```bash
  rustup update stable
  rustup component add rustfmt clippy
  ```
- **System Packages (Linux):** `bubblewrap`, `sqlite3`, `pkg-config`, `gcc`.

### Build & Test Commands
```bash
# Build the entire workspace
cargo build --workspace

# Run all unit and integration tests
cargo test --workspace

# Run formatter check
cargo fmt --all -- --check

# Run linter checks with zero warnings
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 3. Pull Request Guidelines

1. **Fork & Branch:** Create a feature branch with a descriptive name (`feat/my-feature` or `fix/my-bug`).
2. **Commit Messages:** Follow Conventional Commits format:
   - `feat(scope): add new feature`
   - `fix(scope): resolve bug`
   - `docs(scope): update documentation`
   - `test(scope): add integration tests`
3. **Format & Lint:** Ensure `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass before submitting.
4. **Testing:** Include unit tests in the appropriate crate and integration tests under `tests/`.
5. **Documentation:** Update relevant markdown files and comments if contracts or behaviors change.

---

## 4. Code of Conduct

All contributors are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md). Please report any unacceptable behavior to [team@meza.ai](mailto:team@meza.ai).
