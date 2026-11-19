# Product — {{project-name}}-rs

The Rust port of gluon's Scala `{{project-name}}` (ADR 0029), generated from
`tools/rust-service-template`. It replaces the Scala service in gluon-local
(`<SVC>_IMPL=rust` in gluon-kube) without gshop, the acceptance suite or any other
service noticing.

## Goal
The Scala service's functional contract first (gshop and the Playwright acceptance
suite green against it), then its wire contract (see
[product-guidelines.md](./product-guidelines.md)).

## Source of truth
The Scala service's sources and test suites, gluon's acceptance features, the full-stack
capture (`tests/contract/contracts/full-stack/`), and gluon's ADRs.
