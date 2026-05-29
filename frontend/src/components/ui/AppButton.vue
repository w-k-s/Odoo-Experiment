<script setup lang="ts">
withDefaults(defineProps<{ variant?: "primary" | "ghost" }>(), {
  variant: "primary",
});

const emit = defineEmits<{ (e: "click", ev: MouseEvent): void }>();
</script>

<template>
  <button
    type="button"
    class="inline-flex items-center justify-center rounded-xl px-6 py-3 text-sm font-semibold transition active:scale-95 focus:outline-none focus-visible:ring-2 focus-visible:ring-white/40"
    :class="
      variant === 'primary'
        ? 'bg-white text-black hover:bg-zinc-200'
        : 'bg-transparent text-zinc-300 hover:bg-zinc-900 hover:text-white'
    "
    @click="emit('click', $event)"
  >
    <slot />
  </button>
</template>
