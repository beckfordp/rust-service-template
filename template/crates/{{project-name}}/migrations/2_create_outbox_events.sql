-- SPDX-License-Identifier: MIT
-- The transactional outbox (ADR 0008), as every Scala service creates it;
-- Debezium's EventRouter publishes each row (gluon-rs service-basics::outbox).
-- Replace with the Scala service's own Flyway file, copied verbatim.
CREATE TABLE outbox_events (
    id UUID PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    payload JSONB NOT NULL,
    trace_parent TEXT,
    correlation_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
