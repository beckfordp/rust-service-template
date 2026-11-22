// SPDX-License-Identifier: MIT

//! The service's API documentation (gluon all-Rust Stage A, decisions D1, D2):
//! the OpenAPI document of exactly the routes served, with Swagger UI at
//! `/docs/` (`/docs/docs.yaml`), and the AsyncAPI document at
//! `/docs/asyncapi.json` with its page at `/docs/asyncapi/`, built into the
//! binary (gluon-rs `telemetry::docs`; no npm). Extend both with every route
//! and event the port adds.

use axum::Router;
use serde_json::{Value, json};
use telemetry::docs::openapi::{Api, Operation};
use telemetry::docs::{asyncapi, swagger};
{%- if kafka %}

use crate::events::{EXAMPLE_HAPPENED, ExampleHappened};
{%- endif %}
{%- if outbox %}
use crate::published::{EXAMPLE_CREATED, ExampleCreated};
{%- endif %}
use crate::examples::ExampleResponse;

const TITLE: &str = "{{project-name | title_case}}";
const EXAMPLE_ID: &str = "11111111-1111-4111-8111-111111111111";

/// The OpenAPI document: the example route, then the two health checks.
#[must_use]
pub fn api() -> Api {
    Api::new(TITLE, crate::version())
        .schema::<ExampleResponse>()
        .operation(
            Operation::get("/examples/{id}")
                .tag("examples")
                .summary("Get an example")
                .description("The example with this id.")
                .path_parameter("id", "The example's id (a UUID)", json!(EXAMPLE_ID))
                .response(
                    200,
                    "The example",
                    Some((
                        "ExampleResponse",
                        json!({"id": EXAMPLE_ID, "name": "first", "createdAt": "2026-11-22T12:00:00Z"}),
                    )),
                )
                .response(400, "The id isn't a UUID (plain text)", None)
                .response(404, "No example has this id (plain text)", None),
        )
        .operation(
            Operation::get("/health")
                .tag("health")
                .summary("Liveness")
                .description("200 while the process is up.")
                .response(200, "Up", None),
        )
        .operation(
            Operation::get("/health/ready")
                .tag("health")
                .summary("Readiness")
                .description("200 while Postgres answers; 503 while it doesn't.")
                .response(200, "Ready", None)
                .response(503, "Not ready", None),
        )
}

/// The AsyncAPI document: {% if kafka or outbox %}what the service {% if outbox %}publishes through the outbox{% if kafka %} and {% endif %}{% endif %}{% if kafka %}consumes{% endif %}{% else %}no channels — the service publishes and consumes nothing{% endif %}.
#[must_use]
pub fn asyncapi_document() -> Value {
    let channels = [
{%- if outbox %}
        asyncapi::Channel::publish_cdc::<ExampleCreated>(crate::published::TOPIC, EXAMPLE_CREATED),
{%- endif %}
{%- if kafka %}
        asyncapi::Channel::subscribe::<ExampleHappened>(crate::events::TOPIC, EXAMPLE_HAPPENED),
{%- endif %}
    ];
    asyncapi::document(TITLE, crate::version(), &channels)
}

/// `/docs…`: Swagger UI and the OpenAPI YAML, the AsyncAPI JSON and page.
pub fn router() -> Router {
    swagger::router(api().to_yaml()).merge(asyncapi::router_with_page(&asyncapi_document()))
}
