<script setup lang="ts">
import SignInButton from "@/components/auth/SignInButton.vue";
import SignUpButton from "@/components/auth/SignUpButton.vue";
</script>

<template>
  <div class="flex w-full max-w-xs flex-col gap-3">
    <SignInButton />
    <SignUpButton />
  </div>
</template>
