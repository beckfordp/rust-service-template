// SPDX-License-Identifier: MIT

//! What the service publishes through the outbox: `ExampleCreated` on
//! [`TOPIC`], written in the transaction that stores the example
//! (`store.rs`). Replaced by the Scala service's events (its `*Events.scala`).

use schemars::JsonSchema;
use serde::Serialize;

/// The topic Debezium routes the outbox to.
pub const TOPIC: &str = "{{short_name}}-events";

/// The event type.
pub const EXAMPLE_CREATED: &str = "ExampleCreated";

/// The service's name as the envelope's `source`.
pub const SOURCE: &str = "{{project-name}}";

/// `ExampleCreated`'s payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExampleCreated {
    /// Which example.
    pub example_id: String,
    /// Its name.
    pub name: String,
}
