# Backend conventions

## External integrations are consumed through role-based traits (ports)

Every external system the backend talks to is reached through a trait named for
the **capability**, never the vendor. The trait lives in its own module under
`src/middleware/integrations/`; the vendor client lives in a separate module and
is the *only* place vendor-specific code is allowed.

Current ports:

- `Crm` (`integrations/crm.rs`) — provisioning contacts. Implemented by `Odoo`
  (`integrations/odoo.rs`).
- `IdentityProvider` (`integrations/identity.rs`) — resolving a caller's profile
  from their `sub`. Implemented by `Auth0` (`integrations/auth0.rs`).

Rules:

1. **Name traits for the role, not the vendor** (`Crm`, `IdentityProvider`).
   Renaming would be needed if we ever swap Odoo/Auth0 — the rest of the app
   shouldn't have to.
2. **Depend on the trait, never the concrete type.** `AppState` holds
   `Arc<dyn Crm>` / `Arc<dyn IdentityProvider>`; handlers go through those. Only
   `main.rs` names the concrete `Odoo`/`Auth0` when constructing state.
3. **Don't leak vendor types across the trait boundary.** Wrap ids/responses in
   neutral domain types (`ContactId`, `Profile`) and accept neutral inputs
   (`NewContact`, a subject id) — e.g. `Crm::create_contact` returns
   `ContactId(String)`, not Odoo's `res.partner` `i32`.
4. **An integration owns the credentials it needs.** Callers pass domain
   identifiers, not transport credentials. The identity provider resolves a
   profile from a `sub` and manages its own Management-API M2M token internally;
   handlers never forward the caller's access token to it.

Traits use `#[axum::async_trait]` (already a dependency via axum) so they stay
object-safe for `Arc<dyn _>`.

## New env vars must be added to the example env file

Whenever a change reads a new environment variable (e.g. a new
`std::env::var(...)` in `config.rs`), add it to the related example file
(`backend/.env.example`) with a placeholder value and a short comment explaining
what it is and how to obtain it. Keep the example in sync with what
`Config::from_env` requires so a fresh checkout can boot from the example alone.
