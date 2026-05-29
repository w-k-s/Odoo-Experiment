import { useAuth0 } from "@auth0/auth0-vue";

/**
 * Wraps Auth0 redirect-login. With a Google-only tenant there is no separate
 * sign-up: the first Google login creates the account, so a single entry point
 * is all we need.
 */
export function useLogin() {
  const { loginWithRedirect } = useAuth0();

  async function signIn() {
    try {
      await loginWithRedirect();
    } catch (err) {
      console.error("[auth] login failed:", err);
    }
  }

  return { signIn };
}
