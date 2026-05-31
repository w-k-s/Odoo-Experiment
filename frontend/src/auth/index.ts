import { createAuth0 } from "@auth0/auth0-vue";

/**
 * The Auth0 client/plugin, exported as a singleton so non-component code (the
 * API layer) can read the token without `useAuth0()`.
 *
 * We request the `https://wks-bakery.eu.auth0.com/api/v2/` audience so Auth0
 * mints a JWT access token; the API layer sends that as the bearer.
 *
 * `localstorage` + refresh tokens keep the session across reloads, so the user
 * isn't bounced back to Google on every page load.
 */
export const auth0 = createAuth0({
  domain: import.meta.env.VITE_AUTH0_DOMAIN,
  clientId: import.meta.env.VITE_AUTH0_CLIENT_ID,
  authorizationParams: {
    redirect_uri: `${window.location.origin}/callback`,
    audience: import.meta.env.VITE_AUTH0_AUDIENCE,
    scope: "openid profile email",
  },
  cacheLocation: "localstorage",
  useRefreshTokens: true,
});
