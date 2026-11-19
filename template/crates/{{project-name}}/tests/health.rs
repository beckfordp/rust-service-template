// SPDX-License-Identifier: MIT

//! The started binary: liveness, the JSON start line, and the exits (SIGTERM
//! ends it 0, bad config 1 listing every problem).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use support::{Postgres, Service};
use {{crate_name}}::config::PORT_VARIABLE;

#[test]
fn health_is_200_with_no_body() {
    let postgres = Postgres::start();
    let service = Service::start(&postgres.env());
    assert_eq!(service.get("/health").unwrap(), (200, None, String::new()));
}

#[test]
fn the_start_is_a_json_log_line_and_sigterm_exits_0() {
    let postgres = Postgres::start();
    let mut service = Service::start(&postgres.env());
    service.terminate();
    assert_eq!(service.wait_exit().code(), Some(0));

    let stdout = service.stdout();
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|_| panic!("not JSON: {line}")))
        .collect();
    let start = lines
        .iter()
        .find(|line| line["message"] == "{{project-name}} starting")
        .unwrap_or_else(|| panic!("no start line in {stdout}"));
    assert_eq!(start["logger_name"], "{{project-name}}");
    assert_eq!(start["level"], "INFO");
    assert_eq!(start["port"], service.port.to_string());
}

#[test]
fn bad_config_exits_1_listing_every_problem() {
    let mut service = Service::spawn(&[
        (PORT_VARIABLE, "http".to_owned()),
        ("POSTGRES_PORT", "x".to_owned()),
    ]);
    assert_eq!(service.wait_exit().code(), Some(1));
    let stderr = service.stderr();
    assert!(stderr.contains(PORT_VARIABLE), "{stderr}");
    assert!(stderr.contains("POSTGRES_PORT"), "{stderr}");
}

#[test]
fn an_unreachable_database_exits_1() {
    let mut service = Service::spawn(&[
        ("POSTGRES_HOST", "127.0.0.1".to_owned()),
        ("POSTGRES_PORT", support::free_port().to_string()),
    ]);
    assert_eq!(service.wait_exit().code(), Some(1));
    assert!(service.stderr().contains("unreachable"));
}
