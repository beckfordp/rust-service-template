// SPDX-License-Identifier: MIT

//! The example consumer's route: `ExampleHappened` is logged; every other
//! event type is skipped. The rest of the per-record decision (null values,
//! undecodable envelopes and payloads to the DLQ, the DLQ guard) is gluon-rs
//! `kafka-event-consumer`'s `handle_record`, as it is `EnvelopeHandler`'s in
//! Scala. Replaced by the Scala service's consumers.

use kafka_event_consumer::envelope::{Envelope, Routes};
use serde::Deserialize;
use telemetry::logging::Logger;

/// The topic.
pub const TOPIC: &str = "{{short_name}}-events";

/// The consumer group.
pub const GROUP: &str = "{{project-name}}-{{short_name}}-events";

/// The event type routed.
pub const EXAMPLE_HAPPENED: &str = "ExampleHappened";

/// `ExampleHappened`'s payload: required fields, others ignored, as circe's
/// derived decoder does.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleHappened {
    /// Which example.
    pub example_id: String,
}

/// Why an event wasn't handled: logged, retried and dead-lettered by the DLQ
/// guard.
#[derive(Debug, thiserror::Error)]
pub enum EventsError {
    /// The payload isn't the event's (poison).
    #[error(transparent)]
    Payload(#[from] serde_json::Error),
}

/// The route: `ExampleHappened` → a log line.
#[derive(Debug, Clone)]
pub struct ExampleEvents {
    logger: Logger,
}

impl ExampleEvents {
    /// Logs to `logger`.
    #[must_use]
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }
}

impl Routes for ExampleEvents {
    type Event = ExampleHappened;
    type Error = EventsError;

    fn decode(&self, envelope: &Envelope) -> Option<Result<ExampleHappened, EventsError>> {
        (envelope.event_type == EXAMPLE_HAPPENED).then(|| {
            serde_json::from_value(envelope.payload.clone()).map_err(EventsError::from)
        })
    }

    async fn handle(&self, event_id: &str, event: &ExampleHappened) -> Result<(), EventsError> {
        self.logger.info(
            &[
                ("event_id", event_id),
                ("example_id", event.example_id.as_str()),
            ],
            "Example happened",
        );
        Ok(())
    }
}
