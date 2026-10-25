# Contributing to ThreeD-Rust (`threed-rs`)

Thank you for your interest in contributing to `threed-rs`! This document provides guidelines and workflows for contributing to the project.

---

## 🛠️ Development Setup

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (edition 2021, Rust $\ge$ 1.80)
- `rustfmt` and `clippy` components:
  ```bash
  rustup component add rustfmt clippy
  ```

### Clone and Build
```bash
git clone https://github.com/eraysevgen/threed-rs.git
cd threed-rs
cargo build --all-targets --all-features
```

---

## 🧪 Testing & Code Quality

Before opening a pull request, ensure your code passes all checks:

### 1. Run All Tests
```bash
cargo test --all --verbose
```

### 2. Check Formatting
```bash
cargo fmt --all -- --check
```
To auto-format:
```bash
cargo fmt --all
```

### 3. Clippy Linter
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 🌿 Git & Branching Strategy

1. **Fork** the repository on GitHub.
2. Create a feature or bugfix branch from `main`:
   ```bash
   git checkout -b feat/my-new-feature
   # or
   git checkout -b fix/issue-description
   ```
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat: add support for XYZ feature`
   - `fix: resolve bounding box calculation edge case`
   - `docs: update CLI usage examples`
   - `perf: optimize KD-tree batch search memory layout`
   - `refactor: clean up feature extraction internal helpers`
   - `test: add unit test for PLY endianness parser`

---

## 🚀 Submitting a Pull Request

1. Push your branch to your fork:
   ```bash
   git push origin feat/my-new-feature
   ```
2. Open a Pull Request against `main`.
3. Fill out the PR template checklist completely.
4. Ensure all CI checks (Ubuntu, Windows, macOS) pass.
5. Address any review comments or feedback.

---

## 📄 License
By contributing to `threed-rs`, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).

