// SPDX-License-Identifier: MIT

//! Shared by the integration tests: gluon-rs `service-testing`'s throwaway
//! PostgreSQL cluster, this service's store over it and its binary under
//! test, an in-memory store for route tests, and a logger whose lines the
//! test can read back.

#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unused_async_trait_impl
)]

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;
use telemetry::logging::Logger;
use {{crate_name}}::config::{PORT_VARIABLE, PostgresConfig};
use {{crate_name}}::db;
use {{crate_name}}::domain::{Example, ExampleId};
use {{crate_name}}::store::{Examples, PgExamples, StoreError};

// Not every test file uses both.
#[allow(unused_imports)]
pub use service_testing::{Postgres, free_port};

/// The {{project-name}} binary, on a fresh port.
pub struct Service(service_testing::Service);

impl Service {
    /// Starts it with exactly `extra` (plus its port); does not wait.
    pub fn spawn(extra: &[(&str, String)]) -> Self {
        Self(service_testing::Service::spawn(
            env!("CARGO_BIN_EXE_{{project-name}}"),
            PORT_VARIABLE,
            extra,
        ))
    }

    /// Starts it and waits until `/health` answers.
    pub fn start(extra: &[(&str, String)]) -> Self {
        Self(service_testing::Service::start(
            env!("CARGO_BIN_EXE_{{project-name}}"),
            PORT_VARIABLE,
            extra,
        ))
    }
}

impl Deref for Service {
    type Target = service_testing::Service;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Service {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The service's store over `postgres`, migrated as the service migrates it.
pub async fn store(postgres: &Postgres) -> PgExamples {
    let env: HashMap<_, _> = postgres.env().into_iter().collect();
    let config = PostgresConfig {
        host: env["POSTGRES_HOST"].clone(),
        port: env["POSTGRES_PORT"].parse().unwrap(),
        database: env["POSTGRES_DB"].clone(),
        user: env["POSTGRES_USER"].clone(),
        password: env["POSTGRES_PASSWORD"].clone(),
    };
    let pool = db::connect(&config).await.unwrap();
    db::migrate(&pool).await.unwrap();
    PgExamples::new(pool)
}

/// An in-memory `example` table; `failing` makes every call fail as a
/// database would, `down` makes the ping fail.
#[derive(Clone, Default)]
pub struct Memory {
    rows: Arc<Mutex<Vec<Example>>>,
    pub failing: Arc<AtomicBool>,
    pub down: Arc<AtomicBool>,
}

impl Memory {
    /// Holding `rows`.
    pub fn with(rows: Vec<Example>) -> Self {
        Self {
            rows: Arc::new(Mutex::new(rows)),
            ..Self::default()
        }
    }

    fn fail(&self) -> Result<(), StoreError> {
        if self.failing.load(Ordering::SeqCst) {
            return Err(StoreError::Database(sqlx::Error::PoolClosed));
        }
        Ok(())
    }
}

impl Examples for Memory {
    async fn get(&self, id: ExampleId) -> Result<Option<Example>, StoreError> {
        self.fail()?;
        Ok(self.rows.lock().unwrap().iter().find(|row| row.id == id).cloned())
    }

    async fn insert(&self, example: &Example) -> Result<(), StoreError> {
        self.fail()?;
        self.rows.lock().unwrap().push(example.clone());
        Ok(())
    }

    async fn ping(&self, _within: Duration) -> bool {
        !self.down.load(Ordering::SeqCst)
    }
}

/// Where a [`captured_logger`] writes.
pub type LogBuffer = Arc<Mutex<Vec<u8>>>;

/// A logger named as the service's, writing to a buffer the test reads back.
pub fn captured_logger() -> (Logger, LogBuffer) {
    let buffer = LogBuffer::default();
    (Logger::to_writer("{{project-name}}", buffer.clone()), buffer)
}

/// The lines logged to `buffer` so far, each as JSON.
pub fn log_lines(buffer: &LogBuffer) -> Vec<Value> {
    let text = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    text.lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
