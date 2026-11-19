// SPDX-License-Identifier: MIT

//! The example consumer's route, less the stream: which envelopes it acts on
//! and the line it logs. The rest of the per-record decision is gluon-rs
//! `kafka-event-consumer`'s; the consumer against a broker is
//! tests/contract's.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use kafka_event_consumer::envelope::{Envelope, Routes};
use serde_json::{Value, json};
use support::{captured_logger, log_lines};
use {{crate_name}}::consumers::{LAG_INTERVAL, settings};
use {{crate_name}}::events::{EventsError, ExampleEvents, GROUP, TOPIC};

fn envelope(event_type: &str, payload: Value) -> Envelope {
    serde_json::from_value(json!({
        "eventId": "e1",
        "eventType": event_type,
        "timestamp": "2026-11-20T10:00:00Z",
        "source": "test",
        "correlationId": "c1",
        "payload": payload,
    }))
    .unwrap()
}

#[tokio::test]
async fn example_happened_is_logged_and_other_types_skipped() {
    let (logger, buffer) = captured_logger();
    let route = ExampleEvents::new(logger);
    assert!(route.decode(&envelope("SomethingElse", json!({}))).is_none());
    let event = route
        .decode(&envelope("ExampleHappened", json!({"exampleId": "x1"})))
        .unwrap()
        .unwrap();
    assert!(route.handle("e1", &event).await.is_ok());
    let lines = log_lines(&buffer);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["message"], "Example happened");
    assert_eq!(lines[0]["example_id"], "x1");
    assert_eq!(lines[0]["event_id"], "e1");
}

#[test]
fn a_payload_that_isnt_the_events_is_poison() {
    let (logger, _) = captured_logger();
    let decoded = ExampleEvents::new(logger).decode(&envelope("ExampleHappened", json!({})));
    assert!(
        matches!(decoded, Some(Err(EventsError::Payload(_)))),
        "{decoded:?}"
    );
}

#[test]
fn the_consumer_reads_its_topic_as_its_group() {
    let settings = settings("kafka:9092").unwrap();
    assert_eq!(settings.bootstrap, "kafka:9092");
    assert_eq!(settings.group_id.as_str(), GROUP);
    assert_eq!(settings.topic.as_str(), TOPIC);
    assert_eq!(settings.lag_interval, LAG_INTERVAL);
}
