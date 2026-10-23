// SPDX-License-Identifier: MIT
#![doc = include_str!("../README.md")]
#![no_std]

use core::num::NonZeroU16;

/// What one [`Debouncer::update`] did to the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Edge {
    /// The level did not change on this update.
    #[default]
    None,
    /// The level went from `false` to `true` on this update.
    Rose,
    /// The level went from `true` to `false` on this update.
    Fell,
}

/// A saturating-counter debouncer with a sticky latch.
///
/// Feed it one boolean sample per tick with [`update`](Self::update). The
/// [`level`](Self::level) asserts once the sample has been `true` for
/// `trip` consecutive ticks, and clears at once, or after `fall_trip`
/// consecutive `false` ticks when a fall threshold is set. The
/// [`latched`](Self::latched) flag records that the level asserted and
/// stays set until [`clear_latch`](Self::clear_latch) or
/// [`reset`](Self::reset).
///
/// Every method does a fixed amount of work with no loops, so the cost of an
/// update does not depend on the thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Debouncer {
    trip: NonZeroU16,
    fall_trip: u16,
    counter: u16,
    fall_counter: u16,
    level: bool,
    latched: bool,
    enabled: bool,
}

impl Debouncer {
    /// A one-sided debouncer: the level asserts after `trip` consecutive
    /// `true` samples and clears on the first `false` sample.
    #[must_use]
    pub const fn new(trip: NonZeroU16) -> Self {
        Self::symmetric(trip, 0)
    }

    /// A debouncer with independent thresholds: the level asserts after
    /// `rise_trip` consecutive `true` samples and clears after `fall_trip`
    /// consecutive `false` samples. A `fall_trip` of zero clears at once,
    /// which is what [`new`](Self::new) builds.
    #[must_use]
    pub const fn symmetric(rise_trip: NonZeroU16, fall_trip: u16) -> Self {
        Self {
            trip: rise_trip,
            fall_trip,
            counter: 0,
            fall_counter: 0,
            level: false,
            latched: false,
            enabled: true,
        }
    }

    /// Apply one sample and report whether the level changed.
    ///
    /// A `true` sample restarts the de-assertion count and advances the
    /// assertion count up to `trip`; reaching `trip` asserts the level and
    /// sets the latch. A `false` sample restarts the assertion count and,
    /// with a `fall_trip` of zero, clears the level at once; otherwise it
    /// advances the de-assertion count while the level is asserted and
    /// clears the level on reaching `fall_trip`.
    ///
    /// A disabled debouncer ignores the sample and returns [`Edge::None`].
    pub fn update(&mut self, sample: bool) -> Edge {
        if !self.enabled {
            return Edge::None;
        }
        let before = self.level;
        if sample {
            self.fall_counter = 0;
            self.counter = self.counter.saturating_add(1).min(self.trip.get());
            if self.counter == self.trip.get() {
                self.level = true;
                self.latched = true;
            }
        } else {
            self.counter = 0;
            if self.fall_trip == 0 {
                self.level = false;
            } else if self.level {
                self.fall_counter = self.fall_counter.saturating_add(1).min(self.fall_trip);
                if self.fall_counter == self.fall_trip {
                    self.level = false;
                    self.fall_counter = 0;
                }
            } else {
                self.fall_counter = 0;
            }
        }
        match (before, self.level) {
            (false, true) => Edge::Rose,
            (true, false) => Edge::Fell,
            _ => Edge::None,
        }
    }

    /// The debounced level.
    #[must_use]
    pub const fn level(&self) -> bool {
        self.level
    }

    /// Whether the level has asserted since the latch was last cleared.
    #[must_use]
    pub const fn latched(&self) -> bool {
        self.latched
    }

    /// Clear the latch. If the level is still asserted, the next `true`
    /// sample sets the latch again, so a fault that is still present is not
    /// hidden by an acknowledgement.
    pub const fn clear_latch(&mut self) {
        self.latched = false;
    }

    /// Consecutive `true` samples counted so far, in `0..=trip`.
    #[must_use]
    pub const fn counter(&self) -> u16 {
        self.counter
    }

    /// Consecutive `false` samples counted toward de-assertion, in
    /// `0..=fall_trip`.
    #[must_use]
    pub const fn fall_counter(&self) -> u16 {
        self.fall_counter
    }

    /// The assertion threshold.
    #[must_use]
    pub const fn trip(&self) -> NonZeroU16 {
        self.trip
    }

    /// The de-assertion threshold; zero means the level clears at once.
    #[must_use]
    pub const fn fall_trip(&self) -> u16 {
        self.fall_trip
    }

    /// Change the assertion threshold.
    ///
    /// The assertion count and the level are cleared, because a count made
    /// under a larger threshold could exceed a smaller one. The latch, the
    /// enable gate and the de-assertion settings are kept.
    pub const fn set_trip(&mut self, trip: NonZeroU16) {
        self.trip = trip;
        self.counter = 0;
        self.level = false;
    }

    /// Change the de-assertion threshold. The de-assertion count is cleared
    /// for the same reason as in [`set_trip`](Self::set_trip); nothing else
    /// changes.
    pub const fn set_fall_trip(&mut self, fall_trip: u16) {
        self.fall_trip = fall_trip;
        self.fall_counter = 0;
    }

    /// Resume processing samples. The debouncer starts from the clean state
    /// that [`disable`](Self::disable) left.
    pub const fn enable(&mut self) {
        self.enabled = true;
    }

    /// Stop processing samples and clear the counts and the level. The latch
    /// is kept, so a disable and enable cycle around a start-up transient
    /// does not discard a fault record.
    pub const fn disable(&mut self) {
        self.enabled = false;
        self.counter = 0;
        self.fall_counter = 0;
        self.level = false;
    }

    /// Whether samples are being processed.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Clear the counts, the level and the latch. The thresholds and the
    /// enable gate are kept.
    pub const fn reset(&mut self) {
        self.counter = 0;
        self.fall_counter = 0;
        self.level = false;
        self.latched = false;
    }
}

#[cfg(doctest)]
#[doc = include_str!("../docs/integration.md")]
mod doc_integration {}

#[cfg(doctest)]
#[doc = include_str!("../docs/design.md")]
mod doc_design {}
