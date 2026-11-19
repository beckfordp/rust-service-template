// SPDX-License-Identifier: MIT

//! The routes over an in-memory store: the example's answers, readiness,
//! and the edge every Scala service has (gluon-rs `service-basics`, tested
//! there; here, that it's wired).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::sync::atomic::Ordering;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use support::Memory;
use time::macros::datetime;
use tower::ServiceExt;
use uuid::Uuid;
use {{crate_name}}::domain::Example;
use {{crate_name}}::server::app;

const ID: &str = "11111111-1111-4111-8111-111111111111";

struct Answer {
    status: StatusCode,
    headers: Vec<(String, String)>,
    body: String,
}

impl Answer {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

async fn send(app: Router, method: &str, uri: &str, headers: &[(&str, &str)]) -> Answer {
    let mut request = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = app
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| (name.as_str().to_owned(), value.to_str().unwrap().to_owned()))
        .collect();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Answer {
        status,
        headers,
        body: String::from_utf8(body.to_vec()).unwrap(),
    }
}

fn example() -> Example {
    Example {
        id: Uuid::parse_str(ID).unwrap().into(),
        name: "first".to_owned(),
        created_at: datetime!(2026-11-20 10:00:00 UTC),
    }
}

#[tokio::test]
async fn an_example_is_its_json() {
    let answer = send(app(Memory::with(vec![example()])), "GET", &format!("/examples/{ID}"), &[]).await;
    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.header("content-type"), Some("application/json"));
    assert_eq!(
        serde_json::from_str::<Value>(&answer.body).unwrap(),
        json!({"id": ID, "name": "first", "createdAt": "2026-11-20T10:00:00Z"})
    );
}

#[tokio::test]
async fn a_bad_id_is_400_an_unknown_one_404_a_failed_store_500() {
    let store = Memory::with(vec![example()]);
    let bad = send(app(store.clone()), "GET", "/examples/x", &[]).await;
    assert_eq!(
        (bad.status, bad.body.as_str()),
        (
            StatusCode::BAD_REQUEST,
            "Invalid value for: path parameter id"
        )
    );
    let unknown = send(
        app(store.clone()),
        "GET",
        "/examples/22222222-2222-4222-8222-222222222222",
        &[],
    )
    .await;
    assert_eq!(
        (unknown.status, unknown.body.as_str()),
        (StatusCode::NOT_FOUND, "Not found")
    );
    store.failing.store(true, Ordering::SeqCst);
    let failed = send(app(store), "GET", &format!("/examples/{ID}"), &[]).await;
    assert_eq!(
        (failed.status, failed.body.as_str()),
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    );
    assert_eq!(
        failed.header("content-type"),
        Some("text/plain; charset=UTF-8")
    );
}

#[tokio::test]
async fn readiness_follows_the_stores_ping() {
    let store = Memory::default();
    let ready = send(app(store.clone()), "GET", "/health/ready", &[]).await;
    assert_eq!(ready.status, StatusCode::OK);
    store.down.store(true, Ordering::SeqCst);
    let down = send(app(store), "GET", "/health/ready", &[]).await;
    assert_eq!(down.status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn the_edge_is_wired_cors_outermost_and_405_without_allow() {
    let post = send(
        app(Memory::default()),
        "POST",
        &format!("/examples/{ID}"),
        &[("origin", "http://localhost:5173")],
    )
    .await;
    assert_eq!(post.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(post.header("allow"), None);
    assert_eq!(post.header("access-control-allow-origin"), Some("*"));
    let preflight = send(
        app(Memory::default()),
        "OPTIONS",
        "/examples/x",
        &[
            ("origin", "http://localhost:5173"),
            ("access-control-request-method", "GET"),
        ],
    )
    .await;
    assert_eq!(preflight.status, StatusCode::OK);
    assert_eq!(
        preflight.header("access-control-allow-methods"),
        Some("PATCH, PUT, GET, POST, DELETE")
    );
    let unknown = send(app(Memory::default()), "GET", "/nowhere", &[]).await;
    assert_eq!(
        (unknown.status, unknown.body.as_str()),
        (StatusCode::NOT_FOUND, "Not found")
    );
}
