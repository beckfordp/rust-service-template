// SPDX-License-Identifier: MIT

//! The docs every port serves (gluon all-Rust Stage A, D1 and D2): Swagger UI
//! on the OpenAPI document of the routes served, and the AsyncAPI document and
//! page, wired into the app.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use support::Memory;
use tower::ServiceExt;
use {{crate_name}}::server::app;

async fn get(uri: &str) -> (StatusCode, Option<String>, String) {
    let response = app(Memory::default())
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned());
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, content_type, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn swagger_ui_loads_the_openapi_document() {
    let (status, _, page) = get("/docs/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("swagger-ui"), "{page}");
    let (status, _, yaml) = get("/docs/docs.yaml").await;
    assert_eq!(status, StatusCode::OK);
    for path in ["/examples/{id}", "/health", "/health/ready", "ExampleResponse"] {
        assert!(yaml.contains(path), "{path} missing from\n{yaml}");
    }
}

#[tokio::test]
async fn the_asyncapi_document_and_its_page_are_served() {
    let (status, content_type, json) = get("/docs/asyncapi.json").await;
    assert_eq!((status, content_type.as_deref()), (StatusCode::OK, Some("application/json")));
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(document["asyncapi"], "3.0.0");
    let (status, _, page) = get("/docs/asyncapi/").await;
    assert_eq!(status, StatusCode::OK);
{%- if kafka or outbox %}
    assert!(page.contains("{{short_name}}-events"), "{page}");
{%- endif %}
{%- if outbox %}
    assert!(page.contains("ExampleCreated"), "{page}");
{%- endif %}
{%- if kafka == false and outbox == false %}
    assert!(page.contains("publishes and consumes nothing"), "{page}");
{%- endif %}
}
