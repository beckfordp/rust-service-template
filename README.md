# rust-service-template

A [cargo-generate](https://github.com/cargo-generate/cargo-generate) template for a
Rust port of a gluon service (all-Rust prep P3): notification-service-rs's shape —
Postgres store behind a trait, sqlx migrations, health/readiness, one example route and
its error enum, the HTTP edge from gluon-rs `service-basics`, injected ids and clock,
JSON logs and OTLP traces, conductor — and none of its domain. No CRUD surface
(TD-12's lesson). With `kafka=true`, one consumer on gluon-rs `kafka-client` and the
lifecycle that stops the server and consumers together. With `redis=true`, reads cached
in Redis (`REDIS_URI`) through gluon-rs `redis-cache`, failing fast to Postgres. With
`outbox=true`, the outbox table and an event written in the store's transaction
(gluon-rs `service-basics::outbox`). Every variant serves its docs: Swagger UI at
`/docs/` and the AsyncAPI document and page, built in (all-Rust Stage A P0.3).

```sh
cd services
cargo generate --path ../tools/rust-service-template template \
  --name cart-service --define short_name=cart --define kafka=false \
  --define redis=true --define outbox=false
```

Then follow the generated `README.md`, "After generating".

`scripts/verify-template.sh` generates four variants (none; kafka; redis+outbox; all) into a scratch gluon layout and
runs each one's `scripts/ci.sh` and gluon `bin/check-locks` on its lock: run it after
any change here.
