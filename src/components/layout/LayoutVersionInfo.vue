<script setup lang="ts">
defineProps<{
  version: string;
}>();
</script>

<template>
  <div class="fixed bottom-2 right-3 text-[10px] font-mono text-app-text-mute opacity-30 select-none pointer-events-none z-0">
    v{{ version }}
  </div>
</template>
