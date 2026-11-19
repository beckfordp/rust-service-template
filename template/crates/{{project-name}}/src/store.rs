// SPDX-License-Identifier: MIT

//! The `example` table. Handlers reach it through the [`Examples`] trait, so
//! they are testable with an in-memory store (`tests/support`); [`PgExamples`]
//! is the Postgres one.

use std::future::Future;
use std::time::Duration;

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{Example, ExampleId};

/// Why a store call failed.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The database refused or failed.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// What the service needs from `example`.
pub trait Examples: Clone + Send + Sync + 'static {
    /// The example with `id`, if there is one.
    fn get(&self, id: ExampleId)
    -> impl Future<Output = Result<Option<Example>, StoreError>> + Send;

    /// Stores `example`.
    fn insert(&self, example: &Example) -> impl Future<Output = Result<(), StoreError>> + Send;

    /// True when the database answers within `within` (readiness).
    fn ping(&self, within: Duration) -> impl Future<Output = bool> + Send;
}

/// `example` in Postgres.
#[derive(Debug, Clone)]
pub struct PgExamples {
    pool: PgPool,
}

impl PgExamples {
    /// Over `pool`, migrated.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Examples for PgExamples {
    async fn get(&self, id: ExampleId) -> Result<Option<Example>, StoreError> {
        let row: Option<(Uuid, String, OffsetDateTime)> =
            sqlx::query_as("select id, name, created_at from example where id = $1")
                .bind(id.uuid())
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(id, name, created_at)| Example {
            id: id.into(),
            name,
            created_at,
        }))
    }

    async fn insert(&self, example: &Example) -> Result<(), StoreError> {
        sqlx::query("insert into example (id, name, created_at) values ($1, $2, $3)")
            .bind(example.id.uuid())
            .bind(&example.name)
            .bind(example.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn ping(&self, within: Duration) -> bool {
        service_basics::postgres::ping(&self.pool, within)
            .await
            .is_ok()
    }
}
