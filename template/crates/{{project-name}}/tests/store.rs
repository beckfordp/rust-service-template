// SPDX-License-Identifier: MIT

//! `example` in a throwaway Postgres, migrated as the service migrates it.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::time::Duration;

use support::Postgres;
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;
use {{crate_name}}::clock::Clock;
use {{crate_name}}::domain::Example;
use {{crate_name}}::ids::IdGen;
use {{crate_name}}::store::Examples;

struct Fixed;

impl IdGen for Fixed {
    fn next_id(&self) -> Uuid {
        Uuid::from_u128(7)
    }
}

impl Clock for Fixed {
    fn now(&self) -> OffsetDateTime {
        datetime!(2026-11-20 10:00:00.123456 UTC)
    }
}

#[tokio::test]
async fn an_example_reads_back_as_written() {
    let postgres = Postgres::start();
    let store = support::store(&postgres).await;
    let example = Example::new("first", &Fixed, &Fixed);
    store.insert(&example).await.unwrap();
    assert_eq!(store.get(example.id).await.unwrap(), Some(example));
    assert_eq!(store.get(Uuid::from_u128(8).into()).await.unwrap(), None);
}

#[tokio::test]
async fn the_ping_follows_the_database() {
    let postgres = Postgres::start();
    let store = support::store(&postgres).await;
    assert!(store.ping(Duration::from_secs(2)).await);
    postgres.stop();
    assert!(!store.ping(Duration::from_secs(2)).await);
    assert!(store.get(Uuid::from_u128(7).into()).await.is_err());
}
{%- if outbox %}

#[tokio::test]
async fn storing_an_example_writes_its_event_in_the_same_transaction() {
    let postgres = Postgres::start();
    let store = support::store(&postgres).await;
    let example = Example::new("first", &Fixed, &Fixed);
    store.insert(&example).await.unwrap();
    let id = example.id.to_string();
    assert_eq!(postgres.query("select aggregate_id from outbox_events").trim(), id);
    let payload = postgres.query("select payload::text from outbox_events");
    let envelope: serde_json::Value = serde_json::from_str(payload.trim()).unwrap();
    assert_eq!(envelope["eventType"], "ExampleCreated");
    assert_eq!(envelope["source"], "{{project-name}}");
    assert_eq!(envelope["correlationId"], id.as_str());
    assert_eq!(envelope["timestamp"], "2026-11-20T10:00:00.123456Z");
    assert_eq!(envelope["payload"], serde_json::json!({"exampleId": id, "name": "first"}));
    // A failed insert (the same id) leaves no second event.
    assert!(store.insert(&example).await.is_err());
    assert_eq!(postgres.query("select count(*) from outbox_events").trim(), "1");
}
{%- endif %}
