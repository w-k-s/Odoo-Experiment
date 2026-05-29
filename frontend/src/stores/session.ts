import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";

const STORAGE_KEY = "loyalty.session";

/**
 * The loyalty session presented at the till. Persisted locally and re-validated
 * on every load: if the stored code is still active we keep it, otherwise we
 * mint a fresh one.
 */
export const useSessionStore = defineStore("session", () => {
  const code = ref<string | null>(localStorage.getItem(STORAGE_KEY));
  const status = ref("");
  const loading = ref(false);

  function persist(value: string | null) {
    code.value = value;
    if (value) localStorage.setItem(STORAGE_KEY, value);
    else localStorage.removeItem(STORAGE_KEY);
  }

  async function create() {
    const session = await api.createSession();
    persist(session.id);
    status.value = session.status;
  }

  async function ensure() {
    loading.value = true;
    try {
      if (code.value) {
        const detail = await api.getSession(code.value);
        if (detail && detail.status === "active") {
          status.value = detail.status;
          return;
        }
      }
      await create();
    } finally {
      loading.value = false;
    }
  }

  function clear() {
    persist(null);
    status.value = "";
  }

  return { code, status, loading, ensure, clear };
});
