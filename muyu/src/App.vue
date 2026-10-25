<script setup lang="ts">
import { dateZhCN, darkTheme, zhCN, type GlobalThemeOverrides } from "naive-ui";
import { onMounted, onUnmounted, ref } from "vue";
import { useRouter } from "vue-router";

import { isTauri } from "./api";
import IconSprite from "./components/IconSprite.vue";
import type { UpdateInfo } from "./types";

const router = useRouter();
const updateInfo = ref<UpdateInfo | null>(null);

const overrides: GlobalThemeOverrides = {
  common: {
    primaryColor: "#e9b949",
    primaryColorHover: "#f7d98b",
    primaryColorPressed: "#c9922f",
    primaryColorSuppl: "#e9b949",
    bodyColor: "#08090c",
    cardColor: "#12151d",
    modalColor: "#141823",
    popoverColor: "#161a24",
    tableColor: "#0f1219",
    inputColor: "rgba(255,255,255,0.035)",
    borderColor: "rgba(255,255,255,0.09)",
    dividerColor: "rgba(255,255,255,0.07)",
    textColorBase: "#edeff5",
    textColor1: "#edeff5",
    textColor2: "#a3a9b8",
    textColor3: "#767e90",
    placeholderColor: "#5f6675",
    borderRadius: "10px",
    borderRadiusSmall: "8px",
    fontFamily:
      '"MiSans", "HarmonyOS Sans SC", "Microsoft YaHei UI", "PingFang SC", "Segoe UI", sans-serif',
    fontSize: "13px",
    heightMedium: "34px",
  },
  Slider: {
    railColor: "rgba(255,255,255,0.09)",
    railColorHover: "rgba(255,255,255,0.14)",
    fillColor: "#e9b949",
    fillColorHover: "#f7d98b",
    handleColor: "#ffffff",
    handleBoxShadow: "0 0 0 3px rgba(233,185,73,0.18)",
    handleSize: "14px",
    railHeight: "3px",
  },
  Input: {
    color: "rgba(255,255,255,0.035)",
    colorFocus: "rgba(255,255,255,0.05)",
    border: "1px solid rgba(255,255,255,0.09)",
    borderFocus: "1px solid rgba(233,185,73,0.5)",
    boxShadowFocus: "0 0 0 2px rgba(233,185,73,0.12)",
  },
  Message: {
    color: "#161a24",
    textColor: "#edeff5",
    boxShadow: "0 12px 32px rgba(0,0,0,.45)",
  },
};

let unlisten: (() => void) | undefined;
let unlistenTray: (() => void) | undefined;
let unlistenUpdate: (() => void) | undefined;

async function logClientError(message: string) {
  try {
    const { error } = await import("@tauri-apps/plugin-log");
    await error(message);
  } catch {
    // 日志不可用时不影响业务
  }
}

function onWindowError(event: ErrorEvent) {
  void logClientError(`[frontend] ${event.message} @ ${event.filename}:${event.lineno}:${event.colno}`);
}

function onUnhandledRejection(event: PromiseRejectionEvent) {
  void logClientError(`[frontend] unhandled rejection: ${String(event.reason)}`);
}

onMounted(async () => {
  window.addEventListener("error", onWindowError);
  window.addEventListener("unhandledrejection", onUnhandledRejection);
  if (!isTauri()) return;
  const { getCurrentWebview } = await import("@tauri-apps/api/webview");
  const { listen } = await import("@tauri-apps/api/event");
  unlisten = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === "drop") {
      window.dispatchEvent(
        new CustomEvent<string[]>("muyu:files-dropped", { detail: event.payload.paths }),
      );
    }
  });
  unlistenTray = await listen<string>("tray://navigate", (event) => {
    if (event.payload === "projects") router.push("/projects");
    if (event.payload === "settings") router.push("/settings");
  });
  unlistenUpdate = await listen<UpdateInfo>("updater://available", (event) => {
    updateInfo.value = event.payload;
  });
});

onUnmounted(() => {
  window.removeEventListener("error", onWindowError);
  window.removeEventListener("unhandledrejection", onUnhandledRejection);
  unlisten?.();
  unlistenTray?.();
  unlistenUpdate?.();
});
</script>

<template>
  <n-config-provider
    :theme="darkTheme"
    :theme-overrides="overrides"
    :locale="zhCN"
    :date-locale="dateZhCN"
  >
    <n-dialog-provider>
      <n-message-provider>
        <IconSprite />
        <router-view />
        <button v-if="updateInfo" class="update-banner" @click="router.push('/settings')">
          <span>发现新版本</span>
          <b>v{{ updateInfo.version }}</b>
          <em>去更新</em>
        </button>
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>
