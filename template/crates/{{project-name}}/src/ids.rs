// SPDX-License-Identifier: MIT

//! Where new row ids come from: injected (`rust.md` §5), so a test can pin
//! them; random (UUID v4, as the Scala services' `UUID.randomUUID`) in the
//! service.

use uuid::Uuid;

/// Gives out new ids.
pub trait IdGen: Send + Sync {
    /// A new id.
    fn next_id(&self) -> Uuid;
}

/// Random version-4 UUIDs.
#[derive(Debug, Clone, Copy, Default)]
pub struct RandomIds;

impl IdGen for RandomIds {
    fn next_id(&self) -> Uuid {
        Uuid::new_v4()
    }
}
