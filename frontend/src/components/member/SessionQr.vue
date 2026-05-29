<script setup lang="ts">
import QrcodeVue from "qrcode.vue";

defineProps<{ code: string | null; status: string; loading: boolean }>();
</script>

<template>
  <div class="flex flex-col items-center gap-5">
    <div class="rounded-3xl bg-white p-5 shadow-2xl shadow-white/10">
      <QrcodeVue v-if="code" :value="code" :size="240" level="M" />
      <div
        v-else
        class="h-[240px] w-[240px] animate-pulse rounded-xl bg-zinc-200"
      />
    </div>
    <p class="font-mono text-2xl tracking-[0.3em] text-white">
      {{ code ?? "…" }}
    </p>
    <p class="text-xs uppercase tracking-widest text-zinc-500">
      Show this at the till
    </p>
  </div>
</template>
