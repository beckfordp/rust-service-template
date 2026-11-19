// SPDX-License-Identifier: MIT

//! The config contract: the Scala service's environment variables and
//! defaults (its `application.conf`). Pure: an environment map in, a
//! [`Config`] or every problem out; the pieces every service reads the same
//! way are gluon-rs `service-basics` and `telemetry`.

use std::collections::HashMap;
use std::hash::BuildHasher;

pub use service_basics::config::{ConfigError, PostgresConfig};
use service_basics::config::{port, postgres};
pub use telemetry::config::TracingConfig;
use telemetry::config::tracing_from_env;

/// The HTTP port's variable.
pub const PORT_VARIABLE: &str = "{{project-name | shouty_snake_case}}_PORT";

/// Everything the service reads from its environment.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// `{{project-name | shouty_snake_case}}_PORT`: the HTTP API.
    pub port: u16,
    /// `{{project-name | shouty_snake_case}}_NAME`: `service.name` on spans, the logger name.
    pub service_name: String,
    /// `POSTGRES_*`.
    pub postgres: PostgresConfig,
{%- if kafka %}
    /// `KAFKA_BOOTSTRAP_SERVERS`: the consumers' and DLQ producer's brokers.
    pub kafka_bootstrap: String,
{%- endif %}
    /// `OTEL_*`.
    pub tracing: TracingConfig,
}

/// Reads the config from an environment map.
///
/// # Errors
/// [`ConfigError`] listing every invalid variable.
pub fn from_env<S: BuildHasher>(env: &HashMap<String, String, S>) -> Result<Config, ConfigError> {
    let mut problems = Vec::new();
    let text = |name: &str, default: &str| env.get(name).map_or(default, String::as_str).to_owned();
    let http_port = port(env, PORT_VARIABLE, 8080, &mut problems);
    let postgres = postgres(env, "{{short_name}}", &mut problems);
    let tracing = tracing_from_env(env, &mut problems);
    if !problems.is_empty() {
        return Err(ConfigError { problems });
    }
    Ok(Config {
        port: http_port,
        service_name: text("{{project-name | shouty_snake_case}}_NAME", "{{project-name}}"),
        postgres,
{%- if kafka %}
        kafka_bootstrap: text("KAFKA_BOOTSTRAP_SERVERS", "localhost:9092"),
{%- endif %}
        tracing,
    })
}
