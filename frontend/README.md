# Loyalty Member PWA

Vue 3 + Vite + Tailwind v4 progressive web app. Members sign in with **Google
via Auth0**, get a loyalty **session QR code** to present at the till, and see
their **point balance**.

## Stack

- Vue 3.5 + TypeScript + Vite 8
- Tailwind CSS v4 (`@tailwindcss/vite`)
- `@auth0/auth0-vue` (Universal Login → Google)
- Pinia (member / session state), Vue Router
- `qrcode.vue` for the QR code, `vite-plugin-pwa` for installability

The UI is composed: views (`HomeView`, `MemberView`, `CallbackView`) assemble
components (`MemberHeader`, `SessionQr`, `BalanceFooter`, `AuthButtons`, …),
which assemble small `ui/` primitives.

## Run locally

```bash
cp .env.example .env   # fill in Auth0 values if different
pnpm install
pnpm dev               # http://localhost:5173
```

The backend must be running (see `../backend`) at `VITE_API_BASE`.

## Auth0 setup (one-time)

In the Auth0 dashboard:

1. **Enable the Google social connection** on the SPA application.
2. **Create an API** with identifier (audience) `https://loyalty-api` — this
   matches `VITE_AUTH0_AUDIENCE` and the backend's `AUTH0_AUDIENCE`, so access
   tokens are verifiable JWTs.
3. In the SPA app settings, add to **Allowed Callback URLs**, **Allowed Logout
   URLs**, and **Allowed Web Origins**:
   - `http://localhost:5173` (dev)
   - `http://localhost:8080` (docker compose)

## Environment

| Var | Purpose |
| --- | --- |
| `VITE_AUTH0_DOMAIN` | Auth0 tenant domain |
| `VITE_AUTH0_CLIENT_ID` | SPA application client id |
| `VITE_AUTH0_AUDIENCE` | API audience (`https://loyalty-api`) |
| `VITE_API_BASE` | Loyalty backend base URL |

`VITE_*` values are inlined at build time, so the Docker image takes them as
build args (wired in `docker-compose.yaml`).

## Build / Docker

```bash
pnpm build             # type-check + production build to dist/
```

Shipped as the `loyalty-pwa` compose service (nginx serving `dist/`), exposed on
`http://localhost:8080`.
