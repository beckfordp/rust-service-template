// SPDX-License-Identifier: MIT

//! Reads cached in Redis in front of the store (cache-aside, gluon-rs
//! `redis-cache`): a found example is cached for [`TTL`]; a write deletes its
//! key. Redis is never the source of truth: down or slow, every read goes to
//! the store, within the cache's timeout. Replaced by the Scala service's
//! cache (its keys and TTLs).

use std::sync::Arc;
use std::time::Duration;

use redis_cache::Cache;

use crate::domain::{Example, ExampleId};
use crate::store::{Examples, StoreError};

/// How long a cached example stands.
pub const TTL: Duration = Duration::from_secs(300);

/// How long a cache call may take before the store answers instead.
pub const TIMEOUT: Duration = Duration::from_millis(500);

/// The key an example is cached at.
#[must_use]
pub fn key(id: ExampleId) -> String {
    format!("{{short_name}}:example:{id}")
}

/// `store`, its reads cached in `cache`.
#[derive(Debug, Clone)]
pub struct CachedExamples<S> {
    store: S,
    cache: Arc<Cache>,
}

impl<S> CachedExamples<S> {
    /// `store`, cached in `cache`.
    pub fn new(store: S, cache: Arc<Cache>) -> Self {
        Self { store, cache }
    }
}

/// Why a load didn't give an example to cache.
enum Miss {
    Absent,
    Store(StoreError),
}

impl<S: Examples> Examples for CachedExamples<S> {
    async fn get(&self, id: ExampleId) -> Result<Option<Example>, StoreError> {
        let load = || async {
            match self.store.get(id).await {
                Ok(Some(example)) => Ok(example),
                Ok(None) => Err(Miss::Absent),
                Err(error) => Err(Miss::Store(error)),
            }
        };
        match self.cache.get_or_load(&key(id), TTL, load).await {
            Ok(example) => Ok(Some(example)),
            Err(Miss::Absent) => Ok(None),
            Err(Miss::Store(error)) => Err(error),
        }
    }

    async fn insert(&self, example: &Example) -> Result<(), StoreError> {
        self.store.insert(example).await?;
        // A stale entry expires within the TTL if Redis is down now.
        let _ = self.cache.delete(&[&key(example.id)]).await;
        Ok(())
    }

    async fn ping(&self, within: Duration) -> bool {
        self.store.ping(within).await
    }
}
