import { auth0 } from "@/auth";

const API_BASE = import.meta.env.VITE_API_BASE;

export interface MemberProfile {
  member_id: string;
  name: string;
  email: string | null;
  points: number;
}

export interface SessionResponse {
  id: string;
  member_id: string;
  status: string;
  created_at: string;
  expires_at: string | null;
}

export interface SessionDetail {
  session_id: string;
  status: string;
  member: { name: string; email: string | null };
}

/** The raw ID-token JWT (used as the bearer for our API — POC). */
async function idToken(): Promise<string> {
  // Refresh the session so the ID token isn't stale, then read its raw JWT.
  await auth0.checkSession();
  const raw = auth0.idTokenClaims.value?.__raw;
  if (!raw) throw new Error("not authenticated");
  return raw;
}

/** Fetch with the caller's Auth0 ID token attached. Does not throw on status. */
async function request(path: string, init: RequestInit = {}): Promise<Response> {
  const token = await idToken();
  return fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      ...(init.headers ?? {}),
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
  });
}

async function expectOk(res: Response, path: string): Promise<Response> {
  if (!res.ok) {
    throw new Error(`${path} failed: ${res.status}`);
  }
  return res;
}

export const api = {
  async me(): Promise<MemberProfile> {
    const res = await expectOk(await request("/loyalty/me"), "GET /loyalty/me");
    return res.json();
  },

  async createSession(): Promise<SessionResponse> {
    const res = await expectOk(
      await request("/loyalty/sessions", { method: "POST" }),
      "POST /loyalty/sessions",
    );
    return res.json();
  },

  /** Returns null when the session is unknown (404) so callers can re-mint. */
  async getSession(id: string): Promise<SessionDetail | null> {
    const res = await request(`/loyalty/sessions/${encodeURIComponent(id)}`);
    if (res.status === 404) return null;
    await expectOk(res, "GET /loyalty/sessions/{id}");
    return res.json();
  },
};
