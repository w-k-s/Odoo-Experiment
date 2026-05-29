<script setup lang="ts">
import { useLogin } from "@/composables/useLogin";
import AppButton from "@/components/ui/AppButton.vue";

const { signIn } = useLogin();
</script>

<template>
  <AppButton variant="primary" @click="signIn">Continue with Google</AppButton>
</template>
