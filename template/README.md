# {{project-name}}-rs

The Rust port of gluon's Scala `{{project-name}}` (ADR 0029), generated from
gluon's `tools/rust-service-template`. Conductor-driven: start at
[`conductor/index.md`](conductor/index.md).

## After generating
1. `mv {{project-name}} {{project-name}}-rs` (in gluon's `services/`).
2. `cargo update -w --offline`: the lock covers every option (versions the sandbox
   mirror serves); this drops what the service doesn't use and adds nothing.
3. `git init`, then add `("services/{{project-name}}", "services/{{project-name}}-rs")` to
   gluon `bin/lib/sandbox.py` `PORTS`.
4. Copy `deploy/{{project-name}}-rs.values.yaml` to gluon `environments/local/`, its env
   from `{{project-name}}.values.yaml`.
5. `scripts/ci.sh`: green before the first change.
6. Replace the example: `migrations/` (the Scala service's Flyway files, verbatim),
   `domain.rs`, `store.rs`, `examples.rs`{% if kafka %}, `events.rs`{% endif %}, and their tests.

## Layout
- `src/config.rs`: the env contract (`{{project-name | shouty_snake_case}}_PORT`, `POSTGRES_*`{% if kafka %},
  `KAFKA_BOOTSTRAP_SERVERS`{% endif %}{% if redis %}, `REDIS_URI`{% endif %}, `OTEL_*`); every problem reported at once.
- `src/domain.rs`: the pure core; `src/ids.rs`, `src/clock.rs`: injected ids and time.
- `src/store.rs`: the tables behind a trait (Postgres here, in memory in tests).
- `src/server.rs`: routes behind gluon-rs `service-basics::edge` (CORS, the Allow-less
  405); `src/examples.rs`: one route and its error enum; `src/docs.rs`: the OpenAPI and
  AsyncAPI documents and their pages (`/docs/`, `/docs/asyncapi/`): extend both per route
  and event.
{%- if redis %}
- `src/cache.rs`: reads cached in Redis (gluon-rs `redis-cache`); Redis down is a miss.
{%- endif %}
{%- if outbox %}
- `src/published.rs`, `migrations/2_create_outbox_events.sql`: the event the store writes
  to the outbox in its transaction (gluon-rs `service-basics::outbox`); Debezium
  publishes it.
{%- endif %}
{%- if kafka %}
- `src/events.rs`, `src/consumers.rs`: one consumer on gluon-rs `kafka-client`;
  `src/lifecycle.rs`: the server and consumers stop together.
{%- endif %}
- `tests/`: in-memory route tests; Postgres-backed tests on a throwaway cluster (gluon-rs
  `service-testing`); the binary's exits.
