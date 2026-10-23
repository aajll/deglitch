# deglitch

[![CI](https://github.com/aajll/deglitch/actions/workflows/ci.yml/badge.svg)](https://github.com/aajll/deglitch/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

**A saturating-counter debounce filter with a sticky latch, for `no_std` control code.**

## Quick start

Add the crate as a git dependency; see the [integration guide](docs/integration.md) for the tag to pin and the other ways to depend on it.

```rust
use core::num::NonZeroU16;
use deglitch::{Debouncer, Edge};

// Require 10 consecutive asserted ticks before tripping.
let mut pressure_fault = Debouncer::new(NonZeroU16::new(10).unwrap());

// Called once per control-loop tick with the raw condition.
let mut log_fault_event = || {};
for over_limit in [true; 12] {
    if pressure_fault.update(over_limit) == Edge::Rose {
        log_fault_event();
    }
}
assert!(pressure_fault.level());

// The level follows the input; the latch waits for an acknowledgement.
pressure_fault.update(false);
assert!(!pressure_fault.level());
assert!(pressure_fault.latched());
pressure_fault.clear_latch();
assert!(!pressure_fault.latched());
```

## Description

`deglitch` turns a noisy boolean condition, sampled once per tick, into a clean one. The level asserts only after the condition holds for a configured number of consecutive ticks, and a separate sticky latch records that it asserted until the application clears it.

- A level that tracks the debounced input, and a latch for fault-hold and operator acknowledgement
- One-sided debounce by default, or independent rise and fall thresholds
- Each update returns the edge it produced, so a transition is handled where the sample is applied
- Runtime threshold changes and an enable gate that keeps the latch across a disable
- Caller-owned state, no allocation, no loops, and no `unsafe`, so it is usable from interrupt context
- `no_std` with no dependencies; tests cover every line and region, with property tests for the counter invariants

The crate does not sample inputs, measure time, or share state between threads. A tick is whatever the caller's update rate is, and a `Debouncer` shared across contexts needs the caller's own synchronisation.

## Validated toolchains

| Toolchain | Target | Validation |
| --- | --- | --- |
| Rust stable and 1.85 | `x86_64` Linux | Unit, contract and property tests, clippy pedantic, coverage gate |
| Rust stable | `thumbv7em-none-eabihf` | `no_std` library build |

See [cross builds](docs/integration.md#cross-builds).

## Documentation

- [Integration guide](docs/integration.md) — adding the dependency, minimum Rust version, and cross builds
- [API reference](https://docs.rs/deglitch) — rustdoc for every public item, or `cargo doc --open`
- [Design](docs/design.md) — state model, update rules, decisions, and verification
- [Changelog](CHANGELOG.md) — release history

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before you open a pull request. Report security issues as described in [SECURITY.md](.github/SECURITY.md).

## License

MIT. See [LICENSE](./LICENSE).
