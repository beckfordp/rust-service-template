// SPDX-License-Identifier: MIT

//! The service's lifecycle (Scala `KafkaConsumers.watched`): the HTTP server
//! and the Kafka consumers run until SIGTERM/SIGINT, which stops both and ends
//! well; either stopping or failing by itself stops the other, is logged, and
//! fails the service, so a pod never answers `/health` with no consumer. The
//! tasks here are stand-ins that stop when told.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::future::{Future, pending, ready};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use {{crate_name}}::lifecycle::run_until_stopped;
use serde_json::Value;
use support::{captured_logger, log_lines};
use tokio::sync::watch;

/// A stand-in task: runs until told to stop, then ends `Ok`; records that it
/// was stopped.
fn until_stopped(
    stopped: &watch::Receiver<bool>,
    was_stopped: &Arc<AtomicBool>,
) -> impl Future<Output = Result<(), String>> + Send + 'static {
    let (mut stopped, was_stopped) = (stopped.clone(), was_stopped.clone());
    async move {
        let _ = stopped.wait_for(|stop| *stop).await;
        was_stopped.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/// The ERROR lines' messages and errors.
fn errors(lines: &[Value]) -> Vec<(String, Option<String>)> {
    lines
        .iter()
        .filter(|line| line["level"] == "ERROR")
        .map(|line| {
            (
                line["message"].as_str().unwrap().to_owned(),
                line["error"].as_str().map(ToOwned::to_owned),
            )
        })
        .collect()
}

#[tokio::test]
async fn a_signal_stops_both_and_the_service_ends_well() {
    let (stop, stopped) = watch::channel(false);
    let (server_stopped, consumers_stopped) = (Arc::default(), Arc::default());
    let (logger, buffer) = captured_logger();

    let well = run_until_stopped(
        until_stopped(&stopped, &server_stopped),
        until_stopped(&stopped, &consumers_stopped),
        ready(()),
        stop,
        &logger,
    )
    .await;

    assert!(well);
    assert!(
        server_stopped.load(Ordering::SeqCst),
        "the server was stopped"
    );
    assert!(
        consumers_stopped.load(Ordering::SeqCst),
        "the consumers were stopped"
    );
    assert_eq!(errors(&log_lines(&buffer)), []);
}

#[tokio::test]
async fn consumers_that_stop_by_themselves_fail_the_service() {
    let (stop, stopped) = watch::channel(false);
    let server_stopped = Arc::default();
    let (logger, buffer) = captured_logger();

    let well = run_until_stopped(
        until_stopped(&stopped, &server_stopped),
        ready(Ok(())),
        pending(),
        stop,
        &logger,
    )
    .await;

    assert!(!well);
    assert!(
        server_stopped.load(Ordering::SeqCst),
        "the server was stopped"
    );
    assert_eq!(
        errors(&log_lines(&buffer)),
        [("Kafka consumers stopped".to_owned(), None)]
    );
}

#[tokio::test]
async fn consumers_that_fail_fail_the_service_with_the_error() {
    let (stop, stopped) = watch::channel(false);
    let server_stopped = Arc::default();
    let (logger, buffer) = captured_logger();

    let well = run_until_stopped(
        until_stopped(&stopped, &server_stopped),
        ready(Err("DLQ producer: bad config".to_owned())),
        pending(),
        stop,
        &logger,
    )
    .await;

    assert!(!well);
    assert!(server_stopped.load(Ordering::SeqCst));
    assert_eq!(
        errors(&log_lines(&buffer)),
        [(
            "Kafka consumers failed".to_owned(),
            Some("DLQ producer: bad config".to_owned())
        )]
    );
}

#[tokio::test]
async fn a_server_that_fails_stops_the_consumers_and_fails_the_service() {
    let (stop, stopped) = watch::channel(false);
    let consumers_stopped = Arc::default();
    let (logger, buffer) = captured_logger();

    let well = run_until_stopped(
        ready(Err("address in use".to_owned())),
        until_stopped(&stopped, &consumers_stopped),
        pending(),
        stop,
        &logger,
    )
    .await;

    assert!(!well);
    assert!(
        consumers_stopped.load(Ordering::SeqCst),
        "the consumers were stopped"
    );
    assert_eq!(
        errors(&log_lines(&buffer)),
        [(
            "HTTP server failed".to_owned(),
            Some("address in use".to_owned())
        )]
    );
}

#[tokio::test]
async fn a_task_that_panics_fails_the_service() {
    let (stop, stopped) = watch::channel(false);
    let server_stopped = Arc::default();
    let (logger, buffer) = captured_logger();

    let well = run_until_stopped(
        until_stopped(&stopped, &server_stopped),
        async { panic!("consumer bug") },
        pending(),
        stop,
        &logger,
    )
    .await;

    assert!(!well);
    let failures = errors(&log_lines(&buffer));
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].0, "Kafka consumers failed");
}
