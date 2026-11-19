-- SPDX-License-Identifier: MIT
-- The example table (tools/rust-service-template): replace it with the Scala
-- service's Flyway migrations, copied verbatim (V1 → 1_…, V2 → 2_…).
CREATE TABLE example (
    id         UUID        PRIMARY KEY,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
