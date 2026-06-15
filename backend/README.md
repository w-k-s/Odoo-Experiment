# Loyalty Backend

Rust (axum + Diesel) REST service for loyalty programs, members, and sessions.

## Run locally (without Docker)

1. **Postgres** — you need a reachable database. The simplest path is to run just
   the dockerized DB from the repo root and use the host-exposed port `5433`:

   ```bash
   docker compose up -d loyalty-postgres
   ```

   (Or point `DATABASE_URL` at any local Postgres.)

2. **Configure env** — copy the example and adjust if needed:

   ```bash
   cp .env.example .env
   ```

3. **Run** — migrations run automatically on startup, then a default program is
   bootstrapped:

   ```bash
   cargo run
   ```

The server listens on `BIND_ADDR` (default `0.0.0.0:8000`).

## Endpoints

Public / admin:
- `GET  /health`
- `POST /loyalty/programs` — `{ "name": "..." }`
- `POST /loyalty/members` — `{ "name": "...", "email"?: "...", "program_id"?: "..." }`

Member-facing (require a valid Auth0 access token for audience `https://loyalty-api`):
- `GET  /loyalty/me` — profile + point balance; provisions the member (Odoo
  `res.partner` + loyalty row) on first call, keyed by the Auth0 `sub`.
- `POST /loyalty/sessions` — mints a session code for the caller.
- `GET  /loyalty/sessions/{id}` — resolves a session the caller owns to its
  member details (404 otherwise).

```bash
curl localhost:8000/health
curl -H "Authorization: Bearer <auth0-access-token>" localhost:8000/loyalty/me
```

## Configuration

See `.env.example`. Beyond the database settings:

- `AUTH0_DOMAIN`, `AUTH0_AUDIENCE` — tokens are verified against the tenant JWKS
  and must carry this audience. Create an **API** in Auth0 with identifier
  `https://loyalty-api` so access tokens are JWTs.
- `ODOO_URL`, `ODOO_DB`, `ODOO_LOGIN`, `ODOO_API_KEY` — Odoo JSON-RPC connection.

### Odoo API access (best practice)

Don't use the master admin password. Set up a scoped integration identity:

1. **Enable developer mode**: Settings → General Settings → Developer Tools →
   *Activate the developer mode*.
2. **Create a dedicated user** (e.g. `loyalty-bot`): Settings → Users & Companies
   → Users → New, granting only the rights it needs (Contacts: create/write).
   You can leave its password empty to disable interactive login.
3. **Generate an API key for that user**: open the user → *Account Security* →
   *New API Key*. It's shown once — copy it into `ODOO_API_KEY`.
4. The backend authenticates with `login = ODOO_LOGIN` and `password = the API
   key`. Keys can be revoked/rotated without touching the login.
5. Always use HTTPS for real deployments (in-cluster compose to `http://odoo:8069`
   is fine for local dev).

---

1. To understate the complexity: where do we put loyalty_engine?
2. What belong in app state (pool and ...)
3. Where do we put the response schemas
~4. How do we read configuration~

integrations
- traits for crm, auth
- impl for crm, auth

crates/integrations/odoo
crates/integrations/auth0
crates/integrations/loyalty_engine
