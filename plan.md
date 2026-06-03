# Plan: Points transform/consumer service ("KFunc")

## Context

The README's last open task (line 14) is to *"Map events to points earned or redeemed; aggregate them to calculate balance; store balance."* Today the pipeline is complete up to the event backbone:

```
Odoo POS → Debezium CDC (public.pos_order) → Kafka topic `confirmed-orders` (JSON envelope)
```

…but **nothing consumes `confirmed-orders`**. `loyalty_members.points` exists but is static (always the seeded `0`), and `GET /loyalty/me` just returns it. We need the consumer that turns each confirmed order into a points delta, records it durably, and keeps the member balance current.

The confirmed-order event already carries everything we need on `value.after`:
`loyalty_member_ref` (the loyalty member id, denormalized from `partner_id.ref`), `amount_total`, `state`, and the `pos_order` `id`.

> **Note on "KFunc":** the README's "KFunc/transform service" is treated as a *role* (a transform/consumer), not a mandate to adopt Knative. A real Knative Function needs Kubernetes + Knative Serving/Eventing + a KafkaSource, and scale-to-zero fights a long-lived Kafka consumer that must hold partition assignments — disproportionate for this docker-compose stack. We satisfy the role natively instead.

### Decisions (from user)
1. **Form factor:** the consumer runs **in-process inside the existing `loyalty-backend`** as a `tokio` background task (`tokio::spawn`) started from `main.rs`, using `rdkafka`. No new binary, no new container — shared lifecycle with the API, and it reuses `loyalty_engine` models/schema/pool directly.
2. **Balance model:** event-sourced **ledger + materialized balance**. New `points_transactions` table (one row per order, UNIQUE on the source order id → idempotent against Debezium redelivery / create+update duplicates). `loyalty_members.points` is the running aggregate, updated in the same DB transaction.
3. **Points rule:** **earn with a configurable rate** stored on `loyalty_programs` (`points_per_unit`, default 1). `points = floor(amount_total * points_per_unit)`. Redemption deferred (Odoo emits no redeem signal yet) — the ledger's signed `delta` leaves the door open.

## Changes

### 1. Migration — new table + program rate column
New migration dir `backend/migrations/2026-06-03-000001_points_ledger/` (`up.sql` / `down.sql`). Embedded migrations (`db.rs:MIGRATIONS`) run automatically on startup.

`up.sql`:
- `ALTER TABLE loyalty_programs ADD COLUMN points_per_unit INTEGER NOT NULL DEFAULT 1;`
- New table:
  ```sql
  CREATE TABLE points_transactions (
      id           TEXT PRIMARY KEY,
      member_id    TEXT NOT NULL REFERENCES loyalty_members(id) ON DELETE CASCADE,
      source_order TEXT NOT NULL,          -- pos_order id (as text)
      delta        INTEGER NOT NULL,        -- +earn / -redeem
      amount_total NUMERIC(12,2) NOT NULL,
      created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
      UNIQUE (source_order)                 -- idempotency key
  );
  CREATE INDEX idx_points_tx_member ON points_transactions (member_id);
  ```

### 2. Schema + models (reuse existing patterns)
- `backend/src/loyalty_engine/schema.rs`: add `points_transactions` `table!`, add `points_per_unit` to `loyalty_programs`, add `joinable!`/`allow_tables_to_appear_in_same_query!`.
- `backend/src/loyalty_engine/models.rs`: add `points_per_unit: i32` to `Program` (and any insert struct); add `PointsTransaction` (`Queryable/Selectable`) + `NewPointsTransaction` (`Insertable`) mirroring the `Member`/`NewMember` style.

### 3. New service — `PointsService`
`backend/src/loyalty_engine/services/points.rs` (registered in `services/mod.rs`), following the `MemberService` shape (`pool`, `conn.interact`). One core method, fully transactional + idempotent:

```rust
/// Record an earn for a confirmed order. No-ops if the order was already
/// processed (UNIQUE(source_order)). Returns the member's new balance.
async fn record_order(&self, member_id: &str, source_order: &str, amount_total: Decimal) -> EngineResult<Option<i32>>
```
Inside one `conn.interact` + `conn.transaction(...)`:
1. Look up member → its program's `points_per_unit`.
2. `delta = floor(amount_total * points_per_unit)`.
3. `INSERT INTO points_transactions … ON CONFLICT (source_order) DO NOTHING`. If 0 rows inserted → already processed, return `Ok(None)` (skip the balance bump).
4. `UPDATE loyalty_members SET points = points + delta WHERE id = member_id` and return new balance.

This keeps "store balance" correct under at-least-once delivery and the create+update duplication Debezium produces per row.

### 4. The consumer — in-process `tokio` task
New module `backend/src/consumer/mod.rs` (logic + envelope parsing kept testable), spawned from `main.rs`:
- Extend `Config::from_env` with consumer vars: `KAFKA_BROKERS`, `KAFKA_TOPIC` (default `confirmed-orders`), `KAFKA_GROUP_ID` (default `loyalty-points`). Document each in `backend/.env.example` per the backend CLAUDE.md rule.
- In `main.rs`, after `AppState` is built (so migrations have already run and `PointsService`/pool exist), start the consumer with `tokio::spawn(consumer::run(points_service, consumer_cfg))` **before** `axum::serve`. The API and consumer then share one process and the same connection pool.
- `consumer::run`: build an `rdkafka` `StreamConsumer`, subscribe to the topic, loop:
  - Parse the JSON envelope; take `value.after`. Skip if `after` is null (delete) or `loyalty_member_ref` is null/empty (anonymous order).
  - Parse `amount_total` (number) and the order `id` (→ string).
  - Call `PointsService::record_order`; log earned / skipped-duplicate.
  - **Commit the offset only after** the DB write succeeds (manual `commit_message`) so a crash reprocesses rather than drops — safe because the write is idempotent.
  - On transient errors, log and continue (don't commit); let `tracing` surface failures. Consider a top-level restart-on-panic wrapper so a consumer crash doesn't silently kill the task while the API keeps serving.
- Trait-port note: the Kafka consumer is an inbound transport, not an outbound integration, so it doesn't need a `dyn` port; keep `rdkafka` confined to `src/consumer/`.

### 5. Cargo + Dockerfile
- `backend/Cargo.toml`: add `rdkafka = { version = "0.36", features = ["cmake-build", "ssl"] }` and `rust_decimal` (+ its `diesel` Postgres feature) for `NUMERIC`. **No new `[[bin]]`** — single binary unchanged.
- `backend/Dockerfile`: add librdkafka build deps to the **builder** stage (`cmake`, `g++`, `libsasl2-dev`, `libssl-dev`, `pkg-config`). The dependency-cache stub step and runtime `CMD ["loyalty-backend"]` stay as-is. Check whether the slim runtime needs `libsasl2-2`/`libssl3` at runtime (the `ssl`/`cmake-build` features may statically vendor librdkafka — verify after first build).

### 6. docker-compose
No new service. Add the Kafka env vars to the existing `loyalty-backend` service (`backend/.env.docker` / compose `environment`): `KAFKA_BROKERS=kafka:9092`, `KAFKA_TOPIC=confirmed-orders`. Add `kafka` to `loyalty-backend`'s `depends_on` so it starts after the broker.

## Critical files
- `backend/migrations/2026-06-03-000001_points_ledger/{up,down}.sql` (new)
- `backend/src/loyalty_engine/schema.rs`, `models.rs` (edit)
- `backend/src/loyalty_engine/services/points.rs` (new), `services/mod.rs` (edit)
- `backend/src/consumer/mod.rs` (new), `backend/src/main.rs` (edit — `tokio::spawn` the consumer)
- `backend/src/config.rs` (edit — Kafka vars), `backend/.env.example` (edit)
- `backend/Cargo.toml`, `backend/Dockerfile`, `docker-compose.yaml` (edit)

## Verification (end-to-end)
1. `docker compose up -d --build` → confirm `loyalty-backend` logs both "listening on …" and the consumer's partition assignment (subscribed to `confirmed-orders`).
2. In Odoo (`localhost:8069`) create a POS order: start a session in the PWA (`localhost:8080`) to mint a code, attach it at the till so `loyalty_member_ref` is set, pay it (state→`paid`).
3. Watch the `confirmed-orders` topic in kafka-ui (`localhost:8081`) for the event; consumer log should show `earned N points for member …`.
4. `GET http://localhost:8000/loyalty/me` (authenticated) → `points` reflects `floor(amount_total * points_per_unit)`.
5. **Idempotency:** let Debezium emit the order's update event (or restart `loyalty-backend` to reprocess uncommitted offsets) → balance does **not** double; `points_transactions` has exactly one row per `source_order` (`psql` on `localhost:5433`).
6. Optionally add a unit test for `consumer` envelope-parsing + the `floor(amount*rate)` mapping.

## Out of scope / follow-ups
- Redemption events (need an Odoo-side signal; ledger `delta` is already signed for it).
- Exposing transaction history via the API/PWA.
- The optional Temporal sign-up orchestration (separate README item).
