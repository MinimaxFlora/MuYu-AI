<script setup lang="ts">
defineProps<{ name: string; size?: number }>();
</script>

<template>
  <svg class="i" :style="{ width: `${size ?? 16}px`, height: `${size ?? 16}px` }">
    <use :href="`#i-${name}`" />
  </svg>
</template>
