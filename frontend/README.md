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




## Build / Docker

```bash
pnpm build             # type-check + production build to dist/
```

Shipped as the `loyalty-pwa` compose service (nginx serving `dist/`), exposed on
`http://localhost:8080`.
