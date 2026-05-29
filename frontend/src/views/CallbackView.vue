<script setup lang="ts">
import { watchEffect } from "vue";
import { useAuth0 } from "@auth0/auth0-vue";
import { useRouter } from "vue-router";

const { isLoading, isAuthenticated, error } = useAuth0();
const router = useRouter();

watchEffect(() => {
  if (error.value) {
    // Stay on this view so the message is readable instead of bouncing home.
    console.error("[auth] callback error:", error.value);
    return;
  }
  if (!isLoading.value) {
    router.replace(isAuthenticated.value ? "/app" : "/");
  }
});
</script>

<template>
  <div
    class="flex min-h-dvh flex-col items-center justify-center gap-4 bg-black px-6 text-center"
  >
    <template v-if="error">
      <p class="text-sm font-semibold text-red-400">Sign-in failed</p>
      <p class="max-w-sm text-xs text-zinc-400">{{ error.message }}</p>
      <RouterLink to="/" class="text-xs text-zinc-500 underline">Back</RouterLink>
    </template>
    <p v-else class="text-sm text-zinc-400">Signing you in…</p>
  </div>
</template>
