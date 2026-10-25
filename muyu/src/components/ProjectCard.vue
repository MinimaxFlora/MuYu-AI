<script setup lang="ts">
import { computed, ref } from "vue";

import { assetUrl } from "../api";
import { formatDuration, formatRelative, modeLabel } from "../format";
import type { ProjectSummary } from "../types";
import Icon from "./Icon.vue";

const props = defineProps<{ project: ProjectSummary }>();
defineEmits<{ open: []; remove: [] }>();

const failed = ref(false);

const palettes = [
  ["#c08434", "#3d2f6b", "#0d2f3a"],
  ["#2f6f7a", "#193a57", "#0a1a24"],
  ["#7a5a2c", "#2f4a3d", "#101c18"],
  ["#6a4a8a", "#243a5c", "#0b1620"],
];

const index = computed(() => {
  let hash = 0;
  for (const char of props.project.id) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  return hash % palettes.length;
});

const posterStyle = computed(() => {
  const [p1, p2, p3] = palettes[index.value];
  return { "--p1": p1, "--p2": p2, "--p3": p3 };
});

const coverUrl = computed(() => assetUrl(props.project.coverPath));
const mark = computed(() => props.project.name.replace(/[《》\s]/g, "").slice(0, 1) || "幕");
</script>

<template>
  <article class="project-card" @click="$emit('open')">
    <div class="cover">
      <img
        v-if="coverUrl && !failed"
        :src="coverUrl"
        alt=""
        loading="lazy"
        @error="failed = true"
      />
      <div v-else class="poster" :style="posterStyle">
        <span class="cover-mark">{{ mark }}</span>
      </div>
      <span class="cover-mode">{{ modeLabel(project.mode) }}</span>
      <span class="cover-dur">{{ formatDuration(project.duration) }}</span>
      <button class="card-remove" title="删除项目" @click.stop="$emit('remove')">
        <Icon name="trash" :size="13" />
      </button>
    </div>
    <div class="body">
      <div class="name">{{ project.name }}</div>
      <div class="meta">
        {{ project.videoCount }} 集 · {{ project.subtitleCount }} 条字幕 · {{ formatRelative(project.updatedAt) }}
      </div>
      <div class="bar"><i /></div>
    </div>
  </article>
</template>
