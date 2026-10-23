# Design

`deglitch` is a saturating-counter debouncer. Each tick the caller supplies one boolean sample; the debouncer counts consecutive samples of one value, asserts or clears a level when a count reaches its threshold, and reports the edge it produced. A sticky latch records that the level asserted until the caller clears it. Every update does a fixed amount of work with no loops, so its cost does not depend on the thresholds.

## Goals and non-goals

Goals:

- Deterministic, constant-time updates usable from interrupt context and hard real-time loops.
- Separate level and fault-hold outputs, so one object serves both signal cleaning and fault reporting.
- Caller-owned state with no allocation, no global state and no `unsafe`.
- Invalid configurations are unrepresentable, so there is no error path.

Non-goals:

- Sampling hardware inputs or measuring time. A tick is whatever the caller's update rate is.
- Analogue filtering, hysteresis on a numeric value, or majority voting over a window.
- Combining several conditions in one object. Compose objects instead.
- Sharing one object between threads or with an interrupt handler. The caller owns the synchronisation.

## State model

A `Debouncer` holds two thresholds, two counters and three flags.

- `trip` is the number of consecutive `true` samples needed to assert. It is a `NonZeroU16`, so a zero threshold cannot be constructed.
- `counter` counts consecutive `true` samples and saturates at `trip`, so it always lies in `0..=trip` and never wraps, however long the condition holds.
- `fall_trip` is the number of consecutive `false` samples needed to clear. Zero means the level clears at once.
- `fall_counter` counts consecutive `false` samples while the level is asserted and saturates at `fall_trip`.
- `level` is the debounced output.
- `latched` becomes `true` whenever `level` does, and only `clear_latch` and `reset` clear it.
- `enabled` is the processing gate.

The level and the latch answer different questions. The level says whether the condition is currently, cleanly true; it needs no acknowledgement and suits a debounced switch or threshold. The latch says whether the condition has asserted since the last acknowledgement; it suits a fault indicator that must stay visible after the fault clears. `clear_latch` clears only the latch: if the level is still asserted, the next `true` sample sets the latch again, which keeps an acknowledgement from hiding a fault that is still present.

## Update rules

`update` applies one sample and returns an `Edge`:

- A `true` sample resets `fall_counter` and advances `counter` up to `trip`. Once `counter` reaches `trip`, `level` and `latched` become `true`.
- A `false` sample resets `counter`. With `fall_trip == 0`, `level` clears at once. Otherwise, while `level` is `true`, `fall_counter` advances, and `level` clears when it reaches `fall_trip`.

A single `false` sample therefore restarts the assertion count, and with a fall threshold a single `true` sample restarts the de-assertion count.

The edge is the comparison of `level` before and after the update, so `Edge::Rose` and `Edge::Fell` are returned for exactly the one update in which the level changed. `reset`, `disable` and the threshold setters return nothing, so a forced clear of the level is never reported as a falling edge.

### Enable gate

A disabled debouncer ignores samples and returns `Edge::None`. `disable` also clears the counters and the level, so `enable` restarts from a clean state. It keeps the latch, because a disable and enable cycle, such as around a start-up transient, must not discard a fault record.

The gate is how two debouncers compose: one debounces a "ready" condition, and its level enables the second. Keeping arming outside the object keeps each object a single concern and each condition separately observable.

### Configuration changes

`set_trip` clears `counter` and `level`, because a count accumulated under a larger threshold could exceed a smaller one and break the `counter <= trip` invariant. `set_fall_trip` clears `fall_counter` for the same reason. Neither touches the latch.

## Decisions

### Edges are returned, not queried or called back

The C reference offers single-tick `rose` and `fell` queries and an optional transition callback. Both exist because C has no cheap way to return a small enum from the update. Returning `Edge` from `update` gives the caller the transition where the sample is applied, removes the previous-output field and the compile-time callback option, and cannot be read a tick late. A caller that only wants the level ignores the return value.

### Thresholds are validated by type

`trip` is a `NonZeroU16`, so construction and `set_trip` cannot fail and there is no error type. `fall_trip` is a plain `u16` because zero is a meaningful setting. The counters are `u16` to match the thresholds; a wider count would never be reached because both saturate at their threshold.

### No shared-state mode

The C reference has an atomics mode for one writer and concurrent readers. Rust expresses the single writer through `&mut self`, and a consumer that needs the outputs elsewhere publishes `level()` and `latched()` through its own cell or channel. Building a particular sharing mechanism into the debouncer would pick one for every consumer; leaving it out keeps the crate dependency-free and `unsafe`-free.

## Portability

The crate is `#![no_std]`, uses only `core`, has no dependencies and forbids `unsafe`. It builds for any Rust target with `core`; CI builds it for `thumbv7em-none-eabihf`.

## Cost

`update` performs a handful of integer compares, saturating adds and stores, with no branches whose count depends on the thresholds and no loops. A `Debouncer` is seven fields, ten bytes of payload before alignment.

## Verification

- `tests/contract.rs` holds one test per rule in this document, named for the rule it falsifies.
- `tests/properties.rs` holds property tests that, under any sequence of samples, threshold changes, enables, disables and resets, the counters stay within their thresholds, the latch is set whenever the level is, and the returned edge matches the change in the level.
- Doc examples in the README and the guides compile and run as doctests.
- CI gates on `cargo fmt`, clippy pedantic with warnings denied, `cargo doc` with warnings denied, the stable and minimum toolchains, a `no_std` target build, `cargo deny`, `cargo audit` and 100% line and region coverage of `src/`.
