// SPDX-License-Identifier: MIT
//! One test per rule in `docs/design.md`, named for the rule it falsifies.

use core::num::NonZeroU16;
use deglitch::{Debouncer, Edge};

fn nz(n: u16) -> NonZeroU16 {
    NonZeroU16::new(n).expect("non-zero")
}

fn feed(db: &mut Debouncer, sample: bool, ticks: usize) -> Edge {
    let mut last = Edge::None;
    for _ in 0..ticks {
        last = db.update(sample);
    }
    last
}

#[test]
fn new_starts_clear_and_enabled() {
    let db = Debouncer::new(nz(3));
    assert!(!db.level());
    assert!(!db.latched());
    assert!(db.enabled());
    assert_eq!(db.counter(), 0);
    assert_eq!(db.fall_counter(), 0);
    assert_eq!(db.trip(), nz(3));
    assert_eq!(db.fall_trip(), 0);
}

#[test]
fn new_equals_symmetric_with_zero_fall_trip() {
    assert_eq!(Debouncer::new(nz(4)), Debouncer::symmetric(nz(4), 0));
}

#[test]
fn level_asserts_only_at_trip_consecutive_true_samples() {
    let mut db = Debouncer::new(nz(3));
    assert_eq!(db.update(true), Edge::None);
    assert_eq!(db.update(true), Edge::None);
    assert!(!db.level());
    assert_eq!(db.update(true), Edge::Rose);
    assert!(db.level());
}

#[test]
fn trip_of_one_asserts_on_the_first_true_sample() {
    let mut db = Debouncer::new(nz(1));
    assert_eq!(db.update(true), Edge::Rose);
}

#[test]
fn a_single_false_sample_restarts_the_assertion_count() {
    let mut db = Debouncer::new(nz(3));
    feed(&mut db, true, 2);
    assert_eq!(db.counter(), 2);
    db.update(false);
    assert_eq!(db.counter(), 0);
    feed(&mut db, true, 2);
    assert!(!db.level());
}

#[test]
fn counter_saturates_at_trip() {
    let mut db = Debouncer::new(nz(2));
    feed(&mut db, true, 1000);
    assert_eq!(db.counter(), 2);
    assert!(db.level());
}

#[test]
fn one_sided_level_clears_on_the_first_false_sample() {
    let mut db = Debouncer::new(nz(2));
    feed(&mut db, true, 2);
    assert_eq!(db.update(false), Edge::Fell);
    assert!(!db.level());
    assert_eq!(db.fall_counter(), 0);
}

#[test]
fn edge_is_reported_for_exactly_one_update() {
    let mut db = Debouncer::new(nz(2));
    assert_eq!(feed(&mut db, true, 2), Edge::Rose);
    assert_eq!(db.update(true), Edge::None);
    assert_eq!(db.update(false), Edge::Fell);
    assert_eq!(db.update(false), Edge::None);
}

#[test]
fn latch_sets_with_the_level_and_outlives_it() {
    let mut db = Debouncer::new(nz(2));
    feed(&mut db, true, 2);
    assert!(db.latched());
    db.update(false);
    assert!(!db.level());
    assert!(db.latched());
}

#[test]
fn clear_latch_clears_only_the_latch() {
    let mut db = Debouncer::new(nz(2));
    feed(&mut db, true, 2);
    db.clear_latch();
    assert!(!db.latched());
    assert!(db.level());
    assert_eq!(db.counter(), 2);
}

#[test]
fn a_persisting_fault_sets_the_latch_again_after_clear() {
    let mut db = Debouncer::new(nz(2));
    feed(&mut db, true, 2);
    db.clear_latch();
    db.update(true);
    assert!(db.latched());
}

#[test]
fn symmetric_level_clears_after_fall_trip_false_samples() {
    let mut db = Debouncer::symmetric(nz(2), 3);
    feed(&mut db, true, 2);
    assert_eq!(db.update(false), Edge::None);
    assert_eq!(db.update(false), Edge::None);
    assert!(db.level());
    assert_eq!(db.fall_counter(), 2);
    assert_eq!(db.update(false), Edge::Fell);
    assert!(!db.level());
    assert_eq!(db.fall_counter(), 0);
}

#[test]
fn a_single_true_sample_restarts_the_deassertion_count() {
    let mut db = Debouncer::symmetric(nz(1), 3);
    db.update(true);
    feed(&mut db, false, 2);
    assert_eq!(db.fall_counter(), 2);
    db.update(true);
    assert_eq!(db.fall_counter(), 0);
    feed(&mut db, false, 2);
    assert!(db.level());
}

#[test]
fn fall_counter_does_not_run_while_the_level_is_clear() {
    let mut db = Debouncer::symmetric(nz(2), 3);
    feed(&mut db, false, 10);
    assert_eq!(db.fall_counter(), 0);
    db.update(true);
    db.update(false);
    assert_eq!(db.fall_counter(), 0);
}

#[test]
fn set_trip_clears_count_and_level_but_keeps_the_latch() {
    let mut db = Debouncer::symmetric(nz(2), 4);
    feed(&mut db, true, 2);
    feed(&mut db, false, 1);
    db.set_trip(nz(5));
    assert_eq!(db.trip(), nz(5));
    assert_eq!(db.counter(), 0);
    assert!(!db.level());
    assert!(db.latched());
    assert!(db.enabled());
    assert_eq!(db.fall_trip(), 4);
    assert_eq!(db.fall_counter(), 1);
}

#[test]
fn set_trip_does_not_produce_an_edge_on_the_next_update() {
    let mut db = Debouncer::new(nz(2));
    feed(&mut db, true, 2);
    db.set_trip(nz(3));
    assert_eq!(db.update(false), Edge::None);
}

#[test]
fn set_fall_trip_clears_only_the_fall_counter() {
    let mut db = Debouncer::symmetric(nz(1), 5);
    db.update(true);
    feed(&mut db, false, 2);
    db.set_fall_trip(0);
    assert_eq!(db.fall_trip(), 0);
    assert_eq!(db.fall_counter(), 0);
    assert!(db.level());
    assert!(db.latched());
    assert_eq!(db.update(false), Edge::Fell);
}

#[test]
fn disabled_debouncer_ignores_samples() {
    let mut db = Debouncer::new(nz(1));
    db.disable();
    assert!(!db.enabled());
    assert_eq!(db.update(true), Edge::None);
    assert!(!db.level());
    assert_eq!(db.counter(), 0);
}

#[test]
fn disable_clears_counts_and_level_but_keeps_the_latch() {
    let mut db = Debouncer::symmetric(nz(2), 3);
    feed(&mut db, true, 2);
    db.update(false);
    db.disable();
    assert_eq!(db.counter(), 0);
    assert_eq!(db.fall_counter(), 0);
    assert!(!db.level());
    assert!(db.latched());
}

#[test]
fn enable_restarts_from_a_clean_state_without_an_edge() {
    let mut db = Debouncer::new(nz(2));
    feed(&mut db, true, 2);
    db.disable();
    db.enable();
    assert!(db.enabled());
    assert_eq!(db.update(false), Edge::None);
    assert_eq!(feed(&mut db, true, 2), Edge::Rose);
}

#[test]
fn reset_clears_everything_but_thresholds_and_gate() {
    let mut db = Debouncer::symmetric(nz(2), 3);
    feed(&mut db, true, 2);
    db.update(false);
    db.reset();
    assert_eq!(db.counter(), 0);
    assert_eq!(db.fall_counter(), 0);
    assert!(!db.level());
    assert!(!db.latched());
    assert!(db.enabled());
    assert_eq!(db.trip(), nz(2));
    assert_eq!(db.fall_trip(), 3);
    assert_eq!(db.update(false), Edge::None);
}

#[test]
fn debouncers_compose_through_the_enable_gate() {
    let mut ready = Debouncer::new(nz(2));
    let mut fault = Debouncer::new(nz(2));
    fault.disable();
    let mut fault_rose_at = None;
    for tick in 1..=4 {
        ready.update(true);
        if ready.level() {
            fault.enable();
        }
        if fault.update(true) == Edge::Rose {
            fault_rose_at = Some(tick);
        }
    }
    // Ready asserts on tick 2 and arms the fault debouncer in the same tick,
    // which then needs two samples of its own.
    assert_eq!(fault_rose_at, Some(3));
}

#[test]
fn edge_default_is_none_and_types_are_plain_data() {
    assert_eq!(Edge::default(), Edge::None);
    let db = Debouncer::new(nz(1));
    let copy = db.clone();
    assert_eq!(db, copy);
    assert!(format!("{db:?}").contains("Debouncer"));
    assert!(format!("{:?}", Edge::Rose).contains("Rose"));
}
