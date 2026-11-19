# Product Guidelines — {{project-name}}-rs

## Contracts, in layers
- The Scala `{{project-name}}` is the reference (ADR 0029). Its behaviour is met in two
  layers, in order (gluon `tests/contract` `docs/behaviours.md`):
  1. **functional** (`<SVC>-F<n>`): what a caller or consumer can observe — status and
     meaning, state change, event and its meaning. Enough for gshop and the Playwright
     acceptance suite to pass: the first bar.
  2. **wire** (`<SVC>-W<n>`): byte identity with the Scala service — field names and
     order, headers, envelopes, rows, metric/log/trace names. After the functional layer.
- Every behaviour a track meets cites its source (acceptance scenario, Scala test, ADR,
  capture) and how it is reachable (route or topic, and the input that drives it). A
  behaviour with no way to reach it yet isn't a done criterion.
- Documenting existing behaviour is allowed; changing it is not. A gap found here is
  recorded in the track and raised in gluon.

## Code & engineering conventions
Canonical source: [`code_styleguides/rust.md`](./code_styleguides/rust.md)
(functional core, errors as enums, parse don't validate, no panics on the
request path, explicit wiring). It is the Rust counterpart of the Scala
services' `scala-fp.md`.

## API design conventions
- Plain resource JSON responses, standard HTTP status codes, no response
  envelope; error bodies as the Scala service writes them.
- Each domain error maps to a status explicitly at the edge, one mapping per
  case (`IntoResponse` on a per-endpoint error enum); no catch-all translator.
- The HTTP edge (CORS, the Allow-less 405) is gluon-rs `service-basics::edge`, as
  in every Scala service; never a copy here.
- One dedicated Postgres database (`{{short_name}}`); never another service's database.

## Testing
- `scripts/ci.sh` before every commit (fmt, clippy, tests with coverage >= 80%).
- Unit tests for the pure core; handlers against in-memory stores; Postgres-backed
  tests against a throwaway PostgreSQL 16 cluster from the local binaries (gluon-rs
  `service-testing`). No Docker or Testcontainers: the sandbox has no Docker.
- The acceptance suite on gluon-kube for the functional layer; the `tests/contract`
  harness for the wire layer.

## Security conventions
- Customer data is never logged beyond what the Scala service logs.
- No secret is committed; configuration comes from env vars only.
