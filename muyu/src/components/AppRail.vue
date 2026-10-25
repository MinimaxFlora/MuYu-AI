<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";

import Icon from "./Icon.vue";

const route = useRoute();
const router = useRouter();

const active = computed(() => route.name);
const inStudio = computed(() => route.name === "studio");

const items = [
  { name: "home", label: "首页", icon: "home", to: "/" },
  { name: "projects", label: "项目", icon: "folder", to: "/projects" },
  { name: "tasks", label: "任务", icon: "layers", to: "/tasks" },
] as const;

function isActive(name: string) {
  if (name === "projects") return active.value === "projects" || inStudio.value;
  return active.value === name;
}
</script>

<template>
  <aside class="rail">
    <button class="rail-logo" title="幕语" @click="router.push('/')">
      <img src="/logo.png" alt="幕语" />
    </button>

    <nav class="rail-nav">
      <button
        v-for="item in items"
        :key="item.name"
        class="rail-item"
        :class="{ active: isActive(item.name) }"
        @click="router.push(item.to)"
      >
        <Icon :name="item.icon" :size="20" />
        <span>{{ item.label }}</span>
      </button>

      <n-tooltip trigger="hover" placement="right">
        <template #trigger>
          <button class="rail-item disabled" disabled>
            <Icon name="film" :size="20" />
            <span>素材</span>
          </button>
        </template>
        素材库将在 P1 开放
      </n-tooltip>
    </nav>

    <div class="rail-bottom">
      <button
        class="rail-item"
        :class="{ active: active === 'settings' }"
        @click="router.push('/settings')"
      >
        <Icon name="gear" :size="20" />
        <span>设置</span>
      </button>
      <div class="rail-avatar">林</div>
    </div>
  </aside>
</template>
