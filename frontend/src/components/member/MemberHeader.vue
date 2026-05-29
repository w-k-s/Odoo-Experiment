<script setup lang="ts">
import LogoutButton from "@/components/auth/LogoutButton.vue";

defineProps<{ name: string }>();
</script>

<template>
  <header class="flex items-center justify-between px-6 py-5">
    <div class="min-w-0">
      <p class="text-xs uppercase tracking-widest text-zinc-500">Welcome</p>
      <h1 class="truncate text-lg font-semibold text-white">
        Hi {{ name || "there" }}
      </h1>
    </div>
    <LogoutButton />
  </header>
</template>
