<script setup lang="ts">
import { useAuth0 } from "@auth0/auth0-vue";
import { useSessionStore } from "@/stores/session";
import AppButton from "@/components/ui/AppButton.vue";

const { logout } = useAuth0();
const session = useSessionStore();

function onLogout() {
  session.clear();
  logout({ logoutParams: { returnTo: window.location.origin } });
}
</script>

<template>
  <AppButton variant="ghost" @click="onLogout()">Log out</AppButton>
</template>
