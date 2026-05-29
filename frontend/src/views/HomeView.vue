<script setup lang="ts">
import { watchEffect } from "vue";
import { useAuth0 } from "@auth0/auth0-vue";
import { useRouter } from "vue-router";
import AppHero from "@/components/AppHero.vue";
import AuthButtons from "@/components/auth/AuthButtons.vue";

const { isAuthenticated, isLoading } = useAuth0();
const router = useRouter();

// Already signed in? Skip the landing page.
watchEffect(() => {
  if (!isLoading.value && isAuthenticated.value) router.replace("/app");
});
</script>

<template>
  <div
    class="flex min-h-dvh flex-col items-center justify-center gap-12 bg-black px-6 text-white"
  >
    <AppHero />
    <AuthButtons />
  </div>
</template>
