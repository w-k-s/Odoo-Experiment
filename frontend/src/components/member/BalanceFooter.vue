<script setup lang="ts">
defineProps<{ points: number; loading: boolean }>();
</script>

<template>
  <footer class="border-t border-zinc-900 px-6 py-6">
    <div class="flex items-end justify-between">
      <span class="text-xs uppercase tracking-widest text-zinc-500">
        Points balance
      </span>
      <span class="text-3xl font-bold tabular-nums text-white">
        {{ loading ? "…" : points }}
      </span>
    </div>
  </footer>
</template>
