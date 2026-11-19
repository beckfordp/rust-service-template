// SPDX-License-Identifier: MIT

//! The pure core, with plain values.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use time::OffsetDateTime;
use uuid::Uuid;
use {{crate_name}}::clock::{Clock, SystemClock};
use {{crate_name}}::domain::{Example, ExampleId, InvalidId};
use {{crate_name}}::ids::{IdGen, RandomIds};

#[test]
fn an_id_is_a_uuid_in_the_path() {
    let id: ExampleId = "11111111-1111-4111-8111-111111111111".parse().unwrap();
    assert_eq!(id.to_string(), "11111111-1111-4111-8111-111111111111");
    assert_eq!(id.uuid(), Uuid::from_u128(0x1111_1111_1111_4111_8111_1111_1111_1111));
    assert_eq!("x".parse::<ExampleId>(), Err(InvalidId));
}

#[test]
fn a_new_example_takes_its_id_and_time_from_what_it_is_given() {
    let before = OffsetDateTime::now_utc();
    let example = Example::new("first", &RandomIds, &SystemClock);
    assert_eq!(example.name, "first");
    assert!(example.created_at >= before && example.created_at <= SystemClock.now());
    assert_ne!(RandomIds.next_id(), RandomIds.next_id());
}
