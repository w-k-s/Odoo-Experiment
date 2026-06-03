# Odoo POS Loyalty System with Temporal Saga

A distributed system integrating Odoo POS with a loyalty platform using Kafka, Debezium, and Temporal orchestration.

## To Do

- [x] Odoo + PostgreSQL in Docker Compose
- [x] Enable POS module in Odoo
- [x] Loyalty Backend
- [x] Loyalty Frontend (PWA) (Authenticate with Auth0)
- [x] Capture the session at the POS and link the customer
- [x] Setup Kafka + Debezium PostgreSQL CDC
- [x] Event Processing (Easy-to-Moderate)
- [ ] Build KFunc/transform service (Map events to points earned or redeemed; aggregate them to calculate balance; store balance)
- [ ] Use Temporal.io to orchestrate sign-up between Auth0, Odoo (Optional)

## Commands

```bash
docker compose up -d
```

### Database initialization

The DB is seeded automatically by the `odoo-init` one-shot service in
`docker-compose.yaml`. It runs `odoo -d odoo -i base --stop-after-init`
once Postgres is healthy, then exits; the main `odoo` service waits for it via `service_completed_successfully`.

To Add more modules at init time:** extend the `-i` list, e.g.
  `-i base,point_of_sale,sale_management`.



