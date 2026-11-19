// SPDX-License-Identifier: MIT

//! The service's lifecycle (Scala `KafkaConsumers.watched`): the HTTP server
//! and the Kafka consumers run side by side until the stop signal; then both
//! are told to stop and the service ends well. Either stopping or failing by
//! itself stops the other, is logged ("Kafka consumers stopped" / "… failed",
//! as Scala's), and fails the service: a pod answering `/health` with no
//! consumer is what `watched` prevents. Generic over the two tasks and their
//! errors, so its exits are tested with stand-ins (`tests/lifecycle.rs`).

use std::fmt::Display;
use std::future::Future;

use telemetry::logging::Logger;
use tokio::sync::watch;
use tokio::task::JoinError;

/// Runs `server` and `consumers` until `signal`, then sets `stop` (which both
/// must watch) and waits for them. True when the service ends well: stopped
/// by the signal, both tasks ending cleanly.
pub async fn run_until_stopped<S, SE, C, CE, G>(
    server: S,
    consumers: C,
    signal: G,
    stop: watch::Sender<bool>,
    logger: &Logger,
) -> bool
where
    S: Future<Output = Result<(), SE>> + Send + 'static,
    SE: Display + Send + 'static,
    C: Future<Output = Result<(), CE>> + Send + 'static,
    CE: Display + Send + 'static,
    G: Future<Output = ()>,
{
    let mut server = tokio::spawn(server);
    let mut consumers = tokio::spawn(consumers);
    let (mut server_ended, mut consumers_ended) = (None, None);
    let first = tokio::select! {
        () = signal => First::Signal,
        result = &mut consumers => { consumers_ended = Some(result); First::Consumers }
        result = &mut server => { server_ended = Some(result); First::Server }
    };
    // Nobody listening just means both tasks have ended already.
    let _ = stop.send(true);
    let consumers_ended = match consumers_ended {
        Some(result) => result,
        None => consumers.await,
    };
    let server_ended = match server_ended {
        Some(result) => result,
        None => server.await,
    };
    // Only the task that ended first ended by itself; the other was told to.
    let consumers_ok = ended_well(
        logger,
        "Kafka consumers",
        consumers_ended,
        first != First::Consumers,
    );
    let server_ok = ended_well(logger, "HTTP server", server_ended, first != First::Server);
    consumers_ok && server_ok && first == First::Signal
}

/// What ended the wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum First {
    /// SIGTERM or SIGINT.
    Signal,
    /// The consumers, by themselves.
    Consumers,
    /// The server, by itself.
    Server,
}

/// Whether a task ended as it should: cleanly, because it was told to stop.
/// Anything else is logged and fails the service.
fn ended_well<E: Display>(
    logger: &Logger,
    what: &str,
    result: Result<Result<(), E>, JoinError>,
    told_to_stop: bool,
) -> bool {
    let error = match result {
        Ok(Ok(())) if told_to_stop => return true,
        Ok(Ok(())) => {
            logger.error(&[], &format!("{what} stopped"));
            return false;
        }
        Ok(Err(error)) => error.to_string(),
        Err(error) => error.to_string(),
    };
    logger.error(&[("error", &error)], &format!("{what} failed"));
    false
}
