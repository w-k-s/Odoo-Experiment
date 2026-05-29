<script setup lang="ts">
import { useAuth0 } from "@auth0/auth0-vue";
import AppButton from "@/components/ui/AppButton.vue";

const { loginWithRedirect } = useAuth0();

function signUp() {
  loginWithRedirect({ authorizationParams: { screen_hint: "signup" } });
}
</script>

<template>
  <AppButton variant="ghost" @click="signUp()">Sign up</AppButton>
</template>
