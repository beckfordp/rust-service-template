// SPDX-License-Identifier: MIT

//! The config contract: the Scala service's environment variables and defaults.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;

use {{crate_name}}::config::{Config, PostgresConfig, TracingConfig, from_env};

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

#[test]
fn defaults() {
    assert_eq!(
        from_env(&HashMap::new()).unwrap(),
        Config {
            port: 8080,
            service_name: "{{project-name}}".to_owned(),
            postgres: PostgresConfig {
                host: "localhost".to_owned(),
                port: 5432,
                database: "{{short_name}}".to_owned(),
                user: "{{short_name}}".to_owned(),
                password: "{{short_name}}".to_owned(),
            },
{%- if kafka %}
            kafka_bootstrap: "localhost:9092".to_owned(),
{%- endif %}
            tracing: TracingConfig {
                otlp_endpoint: None,
                sampling_ratio: 1.0,
            },
        }
    );
}

#[test]
fn every_variable_overrides_its_default() {
    let config = from_env(&env(&[
        ("{{project-name | shouty_snake_case}}_PORT", "18080"),
        ("{{project-name | shouty_snake_case}}_NAME", "{{project-name}}-rs"),
        ("POSTGRES_HOST", "127.0.0.1"),
        ("POSTGRES_PORT", "15432"),
        ("POSTGRES_DB", "db"),
        ("POSTGRES_USER", "u"),
        ("POSTGRES_PASSWORD", "p"),
        ("KAFKA_BOOTSTRAP_SERVERS", "127.0.0.1:19092"),
        ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://127.0.0.1:4318"),
        ("OTEL_TRACES_SAMPLER_ARG", "0.25"),
    ]))
    .unwrap();
    assert_eq!(config.port, 18080);
    assert_eq!(config.service_name, "{{project-name}}-rs");
    assert_eq!(config.postgres.port, 15432);
{%- if kafka %}
    assert_eq!(config.kafka_bootstrap, "127.0.0.1:19092");
{%- endif %}
    assert_eq!(config.tracing.sampling_ratio, 0.25);
}

#[test]
fn every_problem_is_reported() {
    let error = from_env(&env(&[
        ("{{project-name | shouty_snake_case}}_PORT", "http"),
        ("POSTGRES_PORT", "x"),
    ]))
    .unwrap_err();
    let text = error.to_string();
    assert!(text.contains("{{project-name | shouty_snake_case}}_PORT"), "{text}");
    assert!(text.contains("POSTGRES_PORT"), "{text}");
}
