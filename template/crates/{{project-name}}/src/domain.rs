// SPDX-License-Identifier: MIT

//! The pure core: values parsed at the edge, so the rest of the service holds
//! only valid ones (`rust.md` "parse, don't validate"). The `Example` here
//! shows the shape; the Scala service's domain replaces it.

use std::fmt;
use std::str::FromStr;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::clock::Clock;
use crate::ids::IdGen;

/// An example's id: a UUID, as the path carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExampleId(Uuid);

impl ExampleId {
    /// The UUID.
    #[must_use]
    pub fn uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for ExampleId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// A path segment that isn't a UUID.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid value for: path parameter id")]
pub struct InvalidId;

impl FromStr for ExampleId {
    type Err = InvalidId;

    fn from_str(text: &str) -> Result<Self, InvalidId> {
        Uuid::parse_str(text).map(Self).map_err(|_| InvalidId)
    }
}

impl fmt::Display for ExampleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// An example row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    /// Its id.
    pub id: ExampleId,
    /// Its name.
    pub name: String,
    /// When it was made.
    pub created_at: OffsetDateTime,
}

impl Example {
    /// A new example called `name`, its id from `ids`, made `clock`'s now.
    pub fn new(name: impl Into<String>, ids: &dyn IdGen, clock: &dyn Clock) -> Self {
        Self {
            id: ExampleId(ids.next_id()),
            name: name.into(),
            created_at: clock.now(),
        }
    }
}
