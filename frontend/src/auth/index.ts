import { createAuth0 } from "@auth0/auth0-vue";

/**
 * The Auth0 client/plugin. Exported as a singleton so non-component code (the
 * API layer) can request access tokens without `useAuth0()`.
 */
export const auth0 = createAuth0({
  domain: import.meta.env.VITE_AUTH0_DOMAIN,
  clientId: import.meta.env.VITE_AUTH0_CLIENT_ID,
  authorizationParams: {
    redirect_uri: `${window.location.origin}/callback`,
    audience: import.meta.env.VITE_AUTH0_AUDIENCE,
    scope: "openid profile email",
  },
});
