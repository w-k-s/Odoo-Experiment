<script setup lang="ts">
import { watchEffect } from "vue";
import { useAuth0 } from "@auth0/auth0-vue";
import { useRouter } from "vue-router";

const { isLoading, isAuthenticated } = useAuth0();
const router = useRouter();

watchEffect(() => {
  if (!isLoading.value) {
    router.replace(isAuthenticated.value ? "/app" : "/");
  }
});
</script>

<template>
  <div class="flex min-h-dvh items-center justify-center bg-black text-zinc-400">
    <p class="text-sm">Signing you in…</p>
  </div>
</template>
