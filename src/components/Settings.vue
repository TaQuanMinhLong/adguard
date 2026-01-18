<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";
import { useTheme } from "../composables/useTheme";

const { theme, setTheme } = useTheme();

const hostFilePath = ref("");
const historyDir = ref("");
const maxHistoryEntries = ref(50);
const isLoading = ref(false);
const isSaving = ref(false);

async function loadSettings() {
    isLoading.value = true;
    try {
        const config = await invoke<{
            host_file_path: string | null;
            history_dir: string | null;
            max_history_entries: number;
            theme: string;
        }>("get_config");

        hostFilePath.value = config.host_file_path || "";
        historyDir.value = config.history_dir || "";
        maxHistoryEntries.value = config.max_history_entries || 5;
    } catch (error) {
        console.error("Failed to load settings:", error);
        toast.error("Failed to load settings", {
            description: String(error),
        });
    } finally {
        isLoading.value = false;
    }
}

async function saveSettings() {
    isSaving.value = true;
    try {
        await invoke("update_config", {
            configJson: {
                host_file_path: hostFilePath.value || null,
                history_dir: historyDir.value || null,
                max_history_entries: maxHistoryEntries.value,
                theme: theme.value,
            },
        });
        toast.success("Settings saved successfully!");
    } catch (error) {
        console.error("Failed to save settings:", error);
        toast.error("Failed to save settings", {
            description: String(error),
        });
    } finally {
        isSaving.value = false;
    }
}

async function getDefaultHostPath() {
    try {
        const path = await invoke<string>("get_host_file_path");
        return path;
    } catch (error) {
        console.error("Failed to get default host path:", error);
        return "";
    }
}

onMounted(async () => {
    await loadSettings();
    if (!hostFilePath.value) {
        hostFilePath.value = await getDefaultHostPath();
    }
});
</script>

<template>
    <div class="space-y-6">
        <div class="bg-card border border-border rounded-xl p-6">
            <h2 class="text-xl font-bold text-text-primary mb-6">
                Settings
            </h2>

            <div v-if="isLoading" class="text-center py-8 text-text-secondary">
                Loading settings...
            </div>

            <div v-else class="space-y-6">
                <!-- Paths Section -->
                <div>
                    <h3 class="text-lg font-semibold text-text-primary mb-4">
                        Paths
                    </h3>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-text-secondary mb-2">
                                Host File Path
                            </label>
                            <input v-model="hostFilePath" type="text"
                                class="w-full px-4 py-2 bg-bg-secondary border border-border rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-accent focus:border-transparent"
                                placeholder="Leave empty for default" />
                            <p class="mt-1 text-xs text-text-muted">
                                Path to the hosts file. Leave empty to use the platform default.
                            </p>
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-text-secondary mb-2">
                                History Directory
                            </label>
                            <input v-model="historyDir" type="text"
                                class="w-full px-4 py-2 bg-bg-secondary border border-border rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-accent focus:border-transparent"
                                placeholder="Leave empty for default" />
                            <p class="mt-1 text-xs text-text-muted">
                                Directory where backup history files are stored. Leave empty to use the platform
                                default.
                            </p>
                        </div>
                    </div>
                </div>

                <!-- History Section -->
                <div>
                    <h3 class="text-lg font-semibold text-text-primary mb-4">
                        History
                    </h3>
                    <div>
                        <label class="block text-sm font-medium text-text-secondary mb-2">
                            Maximum History Entries
                        </label>
                        <input v-model.number="maxHistoryEntries" type="number" min="1" max="1000"
                            class="w-full px-4 py-2 bg-bg-secondary border border-border rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-accent focus:border-transparent" />
                        <p class="mt-1 text-xs text-text-muted">
                            Maximum number of backup history entries to keep (1-1000).
                        </p>
                    </div>
                </div>

                <!-- Appearance Section -->
                <div>
                    <h3 class="text-lg font-semibold text-text-primary mb-4">
                        Appearance
                    </h3>
                    <div>
                        <label class="block text-sm font-medium text-text-secondary mb-2">
                            Theme
                        </label>
                        <div class="flex gap-3">
                            <button @click="setTheme('dark')" :class="[
                                'px-4 py-2 rounded-lg font-medium transition-colors',
                                theme === 'dark'
                                    ? 'bg-accent text-white'
                                    : 'bg-bg-secondary text-text-secondary border border-border hover:bg-bg-tertiary',
                            ]">
                                Dark
                            </button>
                            <button @click="setTheme('light')" :class="[
                                'px-4 py-2 rounded-lg font-medium transition-colors',
                                theme === 'light'
                                    ? 'bg-accent text-white'
                                    : 'bg-bg-secondary text-text-secondary border border-border hover:bg-bg-tertiary',
                            ]">
                                Light
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Save Button -->
                <div class="pt-4 border-t border-border">
                    <button @click="saveSettings" :disabled="isSaving"
                        class="px-6 py-2 bg-accent text-white rounded-lg hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium">
                        {{ isSaving ? "Saving..." : "Save Settings" }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>
