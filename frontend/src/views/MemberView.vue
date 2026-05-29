<script setup lang="ts">
import { onMounted, ref } from "vue";
import MemberHeader from "@/components/member/MemberHeader.vue";
import SessionQr from "@/components/member/SessionQr.vue";
import BalanceFooter from "@/components/member/BalanceFooter.vue";
import { useMemberStore } from "@/stores/member";
import { useSessionStore } from "@/stores/session";
import { usePullToRefresh } from "@/composables/usePullToRefresh";

const member = useMemberStore();
const session = useSessionStore();

const root = ref<HTMLElement | null>(null);
const refreshing = ref(false);

// Every load: refresh the balance and (re)validate the session.
async function load() {
  await Promise.all([member.fetchMe(), session.ensure()]);
}

async function refresh() {
  refreshing.value = true;
  try {
    await load();
  } finally {
    refreshing.value = false;
  }
}

onMounted(load);
usePullToRefresh(root, refresh);
</script>

<template>
  <div
    ref="root"
    class="flex min-h-dvh flex-col overflow-y-auto bg-black text-white"
  >
    <p
      v-if="refreshing"
      class="py-2 text-center text-xs uppercase tracking-widest text-zinc-500"
    >
      Refreshing…
    </p>

    <MemberHeader :name="member.name" />

    <main class="flex flex-1 flex-col items-center justify-center px-6">
      <SessionQr
        :code="session.code"
        :status="session.status"
        :loading="session.loading"
      />
    </main>

    <BalanceFooter :points="member.points" :loading="member.loading" />
  </div>
</template>
