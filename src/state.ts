import { ref } from "vue";

export const activeTab = ref<"blocked" | "history" | "settings">("blocked");
export const hasAdminPrivileges = ref<boolean | null>(null);
export const showAdminWarning = ref(true);
