<script setup lang="ts">
import { useAuth0 } from "@auth0/auth0-vue";
import AppButton from "@/components/ui/AppButton.vue";

const { loginWithRedirect } = useAuth0();
</script>

<template>
  <AppButton variant="primary" @click="loginWithRedirect()">Sign in</AppButton>
</template>
