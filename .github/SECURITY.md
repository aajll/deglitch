# Security Policy

## Reporting a vulnerability

Use GitHub's private vulnerability reporting on this repository. Expect a response within 7 days. A confirmed issue is fixed in a patch release.

## Scope

`deglitch` is a small `no_std` library with no dependencies, no `unsafe`, no allocation and no I/O. Its state is a handful of saturating counters and flags, so the realistic attack surface is a caller feeding it samples it should not trust. The library cannot overflow, panic or loop on any input; what a caller does with the level and latch is the caller's concern.
