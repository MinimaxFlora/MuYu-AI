<script setup lang="ts">
import Icon from "./Icon.vue";

defineProps<{ current: number }>();

const steps = ["素材", "剧情理解", "解说文案", "分镜与配音", "成片导出"];
</script>

<template>
  <div class="stepper">
    <div
      v-for="(step, index) in steps"
      :key="step"
      class="step"
      :class="{ done: index + 1 < current, active: index + 1 === current }"
    >
      <span class="idx">
        <Icon v-if="index + 1 < current" name="check" :size="10" />
        <template v-else>{{ index + 1 }}</template>
      </span>
      {{ step }}
    </div>
    <div class="stepper-grow" />
    <slot name="right" />
  </div>
</template>
