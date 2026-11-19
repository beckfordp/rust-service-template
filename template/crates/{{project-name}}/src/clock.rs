// SPDX-License-Identifier: MIT

//! What "now" is: injected (`rust.md` §5), so a test can pin it; the wall
//! clock in the service.

use time::OffsetDateTime;

/// The current time.
pub trait Clock: Send + Sync {
    /// Now, in UTC.
    fn now(&self) -> OffsetDateTime;
}

/// The wall clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
