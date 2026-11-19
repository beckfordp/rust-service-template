// SPDX-License-Identifier: MIT

//! `/health/ready` follows Postgres: stopped, paused (bounded at 2 s) and
//! back.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::thread::sleep;
use std::time::{Duration, Instant};

use support::{Postgres, Service};

fn ready_within(service: &Service, seconds: u64, want: u16) {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        let got = service.get("/health/ready").map(|(status, _, _)| status);
        if got == Some(want) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "not {want} within {seconds} s (last {got:?})"
        );
        sleep(Duration::from_millis(100));
    }
}

#[test]
fn readiness_follows_a_stopped_postgres() {
    let postgres = Postgres::start();
    let service = Service::start(&postgres.env());
    assert_eq!(
        service.get("/health/ready").unwrap(),
        (200, None, String::new())
    );
    postgres.stop();
    ready_within(&service, 5, 503);
    assert_eq!(
        service.get("/health").unwrap().0,
        200,
        "liveness doesn't depend on the database"
    );
    postgres.start_again();
    ready_within(&service, 10, 200);
}

#[test]
fn a_paused_postgres_is_503_within_3_seconds() {
    let postgres = Postgres::start();
    let service = Service::start(&postgres.env());
    ready_within(&service, 5, 200);
    postgres.pause();
    let started = Instant::now();
    let status = service.get("/health/ready").unwrap().0;
    let took = started.elapsed();
    postgres.resume();
    assert_eq!(status, 503);
    assert!(took < Duration::from_secs(3), "took {took:?}");
    ready_within(&service, 10, 200);
}
