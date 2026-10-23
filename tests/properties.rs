// SPDX-License-Identifier: MIT
//! Invariants that hold under any sequence of operations.

use core::num::NonZeroU16;
use deglitch::{Debouncer, Edge};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Sample(bool),
    SetTrip(u16),
    SetFallTrip(u16),
    Enable,
    Disable,
    Reset,
    ClearLatch,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        8 => any::<bool>().prop_map(Op::Sample),
        1 => (1u16..=8).prop_map(Op::SetTrip),
        1 => (0u16..=8).prop_map(Op::SetFallTrip),
        1 => Just(Op::Enable),
        1 => Just(Op::Disable),
        1 => Just(Op::Reset),
        1 => Just(Op::ClearLatch),
    ]
}

fn nz(n: u16) -> NonZeroU16 {
    NonZeroU16::new(n).expect("non-zero")
}

proptest! {
    #[test]
    fn invariants_hold_under_any_operation_sequence(
        trip in 1u16..=8,
        fall_trip in 0u16..=8,
        ops in prop::collection::vec(op(), 0..200),
    ) {
        let mut db = Debouncer::symmetric(nz(trip), fall_trip);
        for op in ops {
            let before = db.level();
            let edge = match op {
                Op::Sample(s) => db.update(s),
                Op::SetTrip(t) => { db.set_trip(nz(t)); Edge::None }
                Op::SetFallTrip(t) => { db.set_fall_trip(t); Edge::None }
                Op::Enable => { db.enable(); Edge::None }
                Op::Disable => { db.disable(); Edge::None }
                Op::Reset => { db.reset(); Edge::None }
                Op::ClearLatch => { db.clear_latch(); Edge::None }
            };
            prop_assert!(db.counter() <= db.trip().get());
            prop_assert!(db.fall_counter() <= db.fall_trip());
            prop_assert!(!db.level() || db.latched() || before);
            if matches!(op, Op::Sample(_)) {
                let expected = match (before, db.level()) {
                    (false, true) => Edge::Rose,
                    (true, false) => Edge::Fell,
                    _ => Edge::None,
                };
                prop_assert_eq!(edge, expected);
            }
            if !db.enabled() {
                prop_assert!(!db.level());
                prop_assert_eq!(db.counter(), 0);
            }
        }
    }

    #[test]
    fn level_asserts_exactly_at_trip_consecutive_true_samples(
        trip in 1u16..=64,
        run in 0u16..=70,
    ) {
        let mut db = Debouncer::new(nz(trip));
        let mut rose_at = None;
        for tick in 1..=run {
            if db.update(true) == Edge::Rose {
                rose_at = Some(tick);
            }
        }
        prop_assert_eq!(rose_at, (run >= trip).then_some(trip));
        prop_assert_eq!(db.level(), run >= trip);
    }

    #[test]
    fn symmetric_level_clears_exactly_at_fall_trip_false_samples(
        fall_trip in 1u16..=64,
        run in 0u16..=70,
    ) {
        let mut db = Debouncer::symmetric(nz(1), fall_trip);
        db.update(true);
        let mut fell_at = None;
        for tick in 1..=run {
            if db.update(false) == Edge::Fell {
                fell_at = Some(tick);
            }
        }
        prop_assert_eq!(fell_at, (run >= fall_trip).then_some(fall_trip));
        prop_assert_eq!(db.level(), run < fall_trip);
    }
}
