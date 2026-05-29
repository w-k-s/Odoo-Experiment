import { createAuth0 } from "@auth0/auth0-vue";

/**
 * The Auth0 client/plugin, exported as a singleton so non-component code (the
 * API layer) can read the token without `useAuth0()`.
 *
 * POC choice: we do NOT request an API `audience`. The app authenticates the
 * member with the **ID token** (a JWT signed by the tenant, carrying
 * name/email/sub). This avoids needing a custom Auth0 API and sidesteps the
 * consent loop the Management API audience caused.
 *
 * `localstorage` + refresh tokens keep the session across reloads, so the user
 * isn't bounced back to Google on every page load.
 */
export const auth0 = createAuth0({
  domain: import.meta.env.VITE_AUTH0_DOMAIN,
  clientId: import.meta.env.VITE_AUTH0_CLIENT_ID,
  authorizationParams: {
    redirect_uri: `${window.location.origin}/callback`,
    scope: "openid profile email",
  },
  cacheLocation: "localstorage",
  useRefreshTokens: true,
});
