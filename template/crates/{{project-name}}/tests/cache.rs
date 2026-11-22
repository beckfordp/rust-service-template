// SPDX-License-Identifier: MIT

//! The read cache over an in-memory store and a throwaway Redis (gluon-rs
//! `service-testing`): hits skip the store, writes invalidate, and Redis down
//! is only slower, never wrong.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use redis_cache::Cache;
use service_testing::Redis;
use support::Memory;
use time::macros::datetime;
use uuid::Uuid;
use {{crate_name}}::cache::{CachedExamples, TIMEOUT, TTL, key};
use {{crate_name}}::domain::Example;
use {{crate_name}}::store::Examples;

fn example() -> Example {
    Example {
        id: Uuid::from_u128(7).into(),
        name: "first".to_owned(),
        created_at: datetime!(2026-11-22 12:00:00.123456 UTC),
    }
}

fn cached(memory: &Memory, redis: &Redis) -> (CachedExamples<Memory>, Arc<Cache>) {
    let cache = Arc::new(Cache::new(&redis.url(), TIMEOUT).unwrap());
    (CachedExamples::new(memory.clone(), cache.clone()), cache)
}

#[tokio::test]
async fn a_found_example_is_cached_and_a_hit_skips_the_store() {
    let redis = Redis::start();
    let memory = Memory::with(vec![example()]);
    let (store, _) = cached(&memory, &redis);
    assert_eq!(store.get(example().id).await.unwrap(), Some(example()));
    memory.failing.store(true, Ordering::SeqCst);
    assert_eq!(store.get(example().id).await.unwrap(), Some(example()));
}

#[tokio::test]
async fn an_absent_example_is_not_cached() {
    let redis = Redis::start();
    let memory = Memory::default();
    let (store, cache) = cached(&memory, &redis);
    assert_eq!(store.get(example().id).await.unwrap(), None);
    assert_eq!(cache.get(&key(example().id)).await.unwrap(), None);
}

#[tokio::test]
async fn a_write_deletes_the_cached_entry() {
    let redis = Redis::start();
    let memory = Memory::default();
    let (store, cache) = cached(&memory, &redis);
    cache.set(&key(example().id), "stale", TTL).await.unwrap();
    store.insert(&example()).await.unwrap();
    assert_eq!(cache.get(&key(example().id)).await.unwrap(), None);
}

#[tokio::test]
async fn with_redis_down_reads_and_writes_go_to_the_store() {
    let redis = Redis::start();
    let memory = Memory::default();
    let (store, _) = cached(&memory, &redis);
    redis.stop();
    store.insert(&example()).await.unwrap();
    assert_eq!(store.get(example().id).await.unwrap(), Some(example()));
    memory.failing.store(true, Ordering::SeqCst);
    assert!(store.get(example().id).await.is_err());
}
