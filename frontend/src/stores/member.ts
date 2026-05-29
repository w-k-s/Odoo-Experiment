import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";

/** The authenticated member's profile + point balance. */
export const useMemberStore = defineStore("member", () => {
  const memberId = ref<string | null>(null);
  const name = ref("");
  const email = ref<string | null>(null);
  const points = ref(0);
  const loading = ref(false);

  async function fetchMe() {
    loading.value = true;
    try {
      const m = await api.me();
      memberId.value = m.member_id;
      name.value = m.name;
      email.value = m.email;
      points.value = m.points;
    } finally {
      loading.value = false;
    }
  }

  function reset() {
    memberId.value = null;
    name.value = "";
    email.value = null;
    points.value = 0;
  }

  return { memberId, name, email, points, loading, fetchMe, reset };
});
