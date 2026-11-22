// SPDX-License-Identifier: MIT

//! The {{project-name}} binary: the Rust port of gluon's Scala
//! {{project-name}} (ADR 0029). Generated from `tools/rust-service-template`:
//! the `example` table and `GET /examples/{id}` show the shape and go when
//! the first real route arrives.

{%- if redis %}
pub mod cache;
{%- endif %}
pub mod clock;
pub mod config;
{%- if kafka %}
pub mod consumers;
{%- endif %}
pub mod db;
pub mod docs;
pub mod domain;
{%- if kafka %}
pub mod events;
{%- endif %}
pub mod examples;
pub mod ids;
{%- if kafka %}
pub mod lifecycle;
{%- endif %}
{%- if outbox %}
pub mod published;
{%- endif %}
pub mod server;
pub mod store;

/// The package version, as built.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
