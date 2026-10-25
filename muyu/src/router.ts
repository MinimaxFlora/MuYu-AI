import { createRouter, createWebHashHistory } from "vue-router";

import HomeView from "./views/HomeView.vue";
import BatchView from "./views/BatchView.vue";
import ExportView from "./views/ExportView.vue";
import SettingsView from "./views/SettingsView.vue";
import StudioView from "./views/StudioView.vue";
import VoiceView from "./views/VoiceView.vue";
import WriteView from "./views/WriteView.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    { path: "/projects", name: "projects", component: HomeView },
    { path: "/studio/:id", name: "studio", component: StudioView },
    { path: "/studio/:id/write", name: "write", component: WriteView },
    { path: "/studio/:id/voice", name: "voice", component: VoiceView },
    { path: "/studio/:id/export", name: "export", component: ExportView },
    { path: "/tasks", name: "tasks", component: BatchView },
    { path: "/settings", name: "settings", component: SettingsView },
    { path: "/:pathMatch(.*)*", redirect: "/" },
  ],
});

export default router;
