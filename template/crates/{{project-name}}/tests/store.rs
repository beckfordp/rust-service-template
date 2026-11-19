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
