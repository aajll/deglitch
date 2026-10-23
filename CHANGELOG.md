# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Debouncer`, a saturating-counter debouncer with a sticky latch, one-sided or symmetric thresholds, an enable gate, runtime threshold changes and a reset.
- `Edge`, returned from every update to report a rising or falling transition on that tick.
- `no_std` with no dependencies and `unsafe` forbidden.
- Contract and property tests, with CI gates on formatting, clippy pedantic, rustdoc, a `thumbv7em-none-eabihf` build, `cargo deny`, `cargo audit` and 100% line and region coverage.
