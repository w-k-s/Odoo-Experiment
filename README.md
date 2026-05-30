# Odoo POS Loyalty System with Temporal Saga

A distributed system integrating Odoo POS with a loyalty platform using Kafka, Debezium, and Temporal orchestration.

## Setup Checklist

### Phase 1: Core Infrastructure (Easy)
- [x] **Odoo + PostgreSQL in Docker Compose**
  - Spin up Odoo 18 + Postgres with docker-compose
  - Configure `.env` and `.pg.env` files
  - Verify Odoo runs at `localhost:8069`

### Phase 2: Odoo Configuration (Easy)
- [x] **Activate Sales/Customers modules**
  - Install via Odoo Apps UI
- [x] **Enable POS module** (~15 min)
  - Install Point of Sale from Odoo Apps

### Phase 3: Loyalty Backend Service (Moderate)
- [x] **Build a standalone loyalty backend with REST APIs**
  - `POST /loyalty/programs` — create a loyalty program
  - `POST /loyalty/members` — create/register a loyalty member
  - `POST /loyalty/sessions` — create a loyalty session, returns a session ID
  - `GET  /loyalty/sessions/{id}` — resolve a session to its member/customer details
  - Own datastore for programs, members, sessions
  - Ship as a Docker Compose service

### Phase 4: Loyalty Member PWA (Moderate)
- [x] **Build a frontend PWA for loyalty members**
  - Member login (authenticate against the loyalty backend)
  - Create a loyalty session → display the session ID (e.g. QR / short code)
  - Installable PWA, served as its own Compose service
  - The session ID is what the member presents at the till

### Phase 5: POS ↔ Loyalty Integration (Moderate)
- [ ] **Capture the session at the POS and link the customer**
  - Add a `loyalty_session_code` field on `pos.order` (custom module)
  - OWL/JS button + popup in the POS UI to enter/scan the session ID
    (see `odoo/tutorials/1-pos-loyalty-session-field.md`)
  - On entry, call the loyalty backend `GET /loyalty/sessions/{id}` to fetch
    customer details
  - Link the customer to the order via Odoo's existing "set customer" POS feature
    (create the `res.partner` if they don't exist yet)
  - Test that both the session code and the customer persist on the posted order

### Phase 6: Event Capture (Moderate-to-Hard)
- [x] **Kafka + Debezium PostgreSQL CDC**
  - Kafka (KRaft, no Zookeeper) + Kafbat UI in compose (`kafka`, `kafka-ui` on
    `localhost:8081`); `odoo-postgres` runs with `wal_level=logical`
  - Debezium captures `public.pos_order` via `pgoutput`; a Groovy `Filter` SMT
    keeps only **confirmed** orders (`state` in `paid`/`done`/`invoiced`) and a
    `RegexRouter` sends them to the **`confirmed-orders`** topic
  - Connector config in `debezium/confirmed-orders-connector.json`, auto-registered
    by the `debezium-init` service (custom `debezium/Dockerfile` adds Groovy)
  - Verified: flipping a `pos_order` to `paid` emits one message on
    `confirmed-orders`; `draft` is filtered out
  - _Still open: surfacing `loyalty_session_code` + `partner_id` on the row
    (Phase 5) and the outbox pattern as a future hardening_

### Phase 7: Event Processing (Easy-to-Moderate)
- [ ] **Build KFunc/transform service**
  - Kafka consumer that reshapes the Debezium event into a clean payload
  - Enrich/flatten loyalty session + customer fields as needed
  - Deploy as a Docker Compose container

### Phase 8: Temporal Integration (Moderate) — _optional / stretch_
- [ ] **Forward processed events into Temporal**
  - Thin HTTP gateway that calls the Temporal SDK `StartWorkflow`
  - Workflow records the loyalty earn/transaction for the session
  - Configure retries, error tolerance, DLQ
  - Test end-to-end POS order → CDC → KFunc → Temporal


## Quick Start

```bash
# 1. Start the stack. The `odoo-init` service seeds/initializes the
#    `odoo` database (installs the base module) before the main `odoo`
#    service starts, so the DB is ready on first boot.
docker compose up -d

# 2. Open the UI
#    http://localhost:8069   (master password: see admin_passwd in odoo/etc/odoo.conf)

# 3. Configure Odoo (Apps UI)
#    Install Sales, Customers, POS modules

# 4. Deploy remaining services (Kafka, Debezium, Temporal, etc.)
#    (see docker-compose extensions or separate compose files)
```

### Database initialization

The DB is seeded automatically by the `odoo-init` one-shot service in
`docker-compose.yaml`. It runs `odoo -d odoo -i base --stop-after-init`
once Postgres is healthy, then exits; the main `odoo` service waits for it
via `service_completed_successfully`.

- **Add more modules at init time:** extend the `-i` list, e.g.
  `-i base,point_of_sale,sale_management`.
- **Re-seed from scratch:** stop the stack and remove the Postgres volume,
  then start again:
  ```bash
  docker compose down
  rm -rf ./pg-data
  docker compose up -d
  ```

## Notes

- **Passwords**: Change `odoo/etc/odoo.conf` and `odoo/.pg.env` credentials before production
- **Scope**: Points redemption, discounts, and reservation IDs are intentionally deferred — current flow is: create session (PWA) → enter at POS → link customer → capture order downstream
- **Outbox Pattern**: Recommended for reliable, commit-time Debezium event capture


