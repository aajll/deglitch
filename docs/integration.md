# Integration guide

How to add `deglitch` to a Rust project. The [API reference](https://docs.rs/deglitch) describes every public item and [design.md](design.md) explains the behaviour.

## Requirements

- Rust 1.85 or later (edition 2024). The minimum is recorded as `rust-version` in `Cargo.toml` and checked in CI.
- No runtime dependencies. The crate is `#![no_std]` and does not use `alloc`.

## Git dependency

Until the crate is published, depend on a release tag:

```toml
[dependencies]
deglitch = { git = "https://github.com/aajll/deglitch", tag = "v0.1.0" }
```

Pinning a tag rather than a branch keeps builds reproducible and makes an upgrade a deliberate change.

## Published crate

Once published, use the version from crates.io:

```toml
[dependencies]
deglitch = "0.1"
```

## Use

One `Debouncer` per monitored condition, owned by whoever runs the tick:

```rust
use core::num::NonZeroU16;
use deglitch::{Debouncer, Edge};

struct Monitor {
    overtemp: Debouncer,
}

impl Monitor {
    fn tick(&mut self, temperature_over_limit: bool) {
        match self.overtemp.update(temperature_over_limit) {
            Edge::Rose => { /* open the contactor, log the fault */ }
            Edge::Fell => { /* condition cleared; the latch still shows it happened */ }
            Edge::None => {}
        }
    }
}

let mut monitor = Monitor {
    overtemp: Debouncer::symmetric(NonZeroU16::new(5).unwrap(), 20),
};
monitor.tick(false);
```

A `Debouncer` is `Send` and plain data. To share its outputs with another thread or task, publish `level()` and `latched()` through the synchronisation the application already uses; the crate does not provide one.

## Build and test

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the coverage, dependency and audit gates CI runs.

## Cross builds

CI builds the library for `thumbv7em-none-eabihf` to prove it stays `no_std`:

```sh
rustup target add thumbv7em-none-eabihf
cargo build --target thumbv7em-none-eabihf
```

Any target with `core` works. Tests run on the host.
