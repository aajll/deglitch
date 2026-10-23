# AGENTS.md

---

## 1) Project-specific instructions

**Project:** `deglitch`
**Primary goal:** A saturating-counter debounce filter with a sticky latch, in `no_std` Rust with no dependencies and no `unsafe`.

### 1.1 Essential commands

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo test
cargo build --target thumbv7em-none-eabihf
cargo llvm-cov --all-targets --fail-under-lines 100 --fail-under-regions 100
cargo deny check
cargo audit
```

### 1.2 Notes

- The README and `docs/*.md` are doctested through `include_str!` in `src/lib.rs`; a Rust fence there must compile and run.
- The crate is at `0.1.0` until its first release; do not bump the version.

## 2) Non-negotiable design invariants

- `update` is constant time: no loops, no work that scales with a threshold.
- `counter <= trip` and `fall_counter <= fall_trip` after every operation.
- The latch is set whenever the level asserts and cleared only by `clear_latch` or `reset`; `disable` and the threshold setters keep it.
- `Edge::Rose` and `Edge::Fell` come only from `update`, for exactly the update in which the level changed.
- `#![no_std]`, no `alloc`, no runtime dependencies, `unsafe_code = "forbid"`.

## 3) CI / source of truth

`.github/workflows/ci.yml` is the source of truth for the gates: format, clippy pedantic, rustdoc, tests on stable and the `rust-version` toolchain, a `thumbv7em-none-eabihf` build, 100% line and region coverage, `cargo deny` and `cargo audit`.

## 4) Docs / commit conventions

### Documentation layout

- `README.md` is a short landing page. Detail goes in `docs/`.
- `docs/integration.md` covers adding the dependency, MSRV and cross builds.
- Rustdoc is the API reference and changes with the interface.
- `docs/design.md` is the single design record; a behaviour change updates it and `tests/contract.rs` together, and a design choice goes under its Decisions section with the alternatives.
- Prose is British English, one line per paragraph, no hard wrapping, no em dashes joining clauses.

### Commits

Conventional Commits, subject under about 70 characters, body explains why. No tool attribution lines.

## 5) Rust style expectations

- Edition 2024, `rust-version = "1.85"`; do not use a feature newer than the MSRV.
- Clippy pedantic through `[lints]`; justify any `#[allow]` in a comment.
- Prefer `const fn` for accessors and field-only mutators.
- Validated types over runtime checks; no error type unless a value cannot be made unrepresentable.
- Tests are named for the rule they falsify.
