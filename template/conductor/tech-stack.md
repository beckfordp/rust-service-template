# Tech Stack — {{project-name}}-rs

- Rust 1.99.1 (`rust-toolchain.toml`), edition 2024; workspace lints: `unsafe_code`
  forbidden, clippy pedantic, no `unwrap`/`expect`/`panic` outside tests.
- HTTP: axum 0.8. The health routes, tapir's plain-text errors and the HTTP edge (CORS,
  the Allow-less 405) are gluon-rs `service-basics`.
- Postgres: sqlx 0.8 (runtime queries, `migrations/` applied at startup, recorded in
  `_sqlx_migrations`); the pool and readiness ping are `service-basics::postgres`.
- Telemetry: gluon-rs `telemetry` (JSON log lines, OTLP traces, Prometheus metrics).
{%- if kafka %}
- Kafka: gluon-rs `kafka-client` (`supervised::run_supervised`: at-least-once, DLQ
  guard, restart with backoff) and `kafka-event-consumer` (envelope routing, typed
  `Routes::Error`).
{%- endif %}
- Tests: gluon-rs `service-testing` (a throwaway PostgreSQL 16 from the local binaries;
  the binary under test); coverage with cargo-llvm-cov.
- gluon-rs is a path dependency (`../../lib/gluon-rs` in gluon's layout; `GLUON_RS`
  elsewhere); the Docker build passes it as a named build context.
