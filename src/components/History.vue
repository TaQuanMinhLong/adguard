<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";
import { Trash2 } from "lucide-vue-next";
import { Checkbox } from "./ui/checkbox";

interface HistoryEntry {
    filename: string;
    path: string;
    entry_count: number;
    file_size: number;
    timestamp: number;
}

const historyEntries = ref<HistoryEntry[]>([]);
const isLoading = ref(false);
const isRollingBack = ref(false);
const isDeleting = ref(false);
const selectedEntries = ref<Set<string>>(new Set());
const selectAllChecked = ref(false);
const entryCheckedStates = ref<Record<string, boolean>>({});

async function loadHistory() {
    isLoading.value = true;
    try {
        const entries = await invoke<HistoryEntry[]>("get_history_list");
        historyEntries.value = entries;
        // Clear selection when reloading
        selectedEntries.value.clear();
    } catch (error) {
        console.error("Failed to load history:", error);
        toast.error("Failed to load history", {
            description: String(error),
        });
    } finally {
        isLoading.value = false;
    }
}

function formatDate(timestamp: number) {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
}

function formatFileSize(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// Watch selectAllChecked and update all entry states
watch(selectAllChecked, (checked) => {
    historyEntries.value.forEach((entry) => {
        entryCheckedStates.value[entry.filename] = checked;
    });
    updateSelectedEntries();
});

// Watch entryCheckedStates and update selectedEntries
watch(entryCheckedStates, () => {
    updateSelectedEntries();
}, { deep: true });

// Watch historyEntries to initialize checked states
watch(historyEntries, (entries) => {
    entries.forEach((entry) => {
        if (!(entry.filename in entryCheckedStates.value)) {
            entryCheckedStates.value[entry.filename] = false;
        }
    });
    // Remove entries that no longer exist
    Object.keys(entryCheckedStates.value).forEach((filename) => {
        if (!entries.find((e) => e.filename === filename)) {
            delete entryCheckedStates.value[filename];
        }
    });
    updateSelectedEntries();
}, { immediate: true });

function updateSelectedEntries() {
    selectedEntries.value = new Set(
        Object.entries(entryCheckedStates.value)
            .filter(([, checked]) => checked)
            .map(([filename]) => filename)
    );
    selectAllChecked.value = historyEntries.value.length > 0 && selectedEntries.value.size === historyEntries.value.length;
}

function setEntryChecked(filename: string, checked: boolean | "indeterminate" | undefined) {
    entryCheckedStates.value[filename] = checked === true;
    updateSelectedEntries();
}

const selectedCount = computed(() => selectedEntries.value.size);

async function deleteSelected() {
    if (selectedEntries.value.size === 0) {
        toast.error("No entries selected");
        return;
    }

    const count = selectedEntries.value.size;
    const message = count === 1
        ? `Are you sure you want to delete this history entry?`
        : `Are you sure you want to delete ${count} history entries?`;

    if (!confirm(message)) {
        return;
    }

    isDeleting.value = true;
    try {
        const filenames = Array.from(selectedEntries.value);
        await invoke("delete_history_files", { filenames });
        toast.success(`Successfully deleted ${count} ${count === 1 ? "entry" : "entries"}`);
        await loadHistory();
    } catch (error) {
        console.error("Failed to delete history files:", error);
        toast.error("Failed to delete history files", {
            description: String(error),
        });
    } finally {
        isDeleting.value = false;
    }
}

async function rollbackTo(filename: string) {
    if (!confirm(`Are you sure you want to rollback to ${filename}? This will replace your current hosts file.`)) {
        return;
    }

    isRollingBack.value = true;
    try {
        await invoke("rollback_to", { filename });
        toast.success("Rollback successful!", {
            description: "The hosts file has been restored.",
        });
        await loadHistory();
    } catch (error) {
        console.error("Failed to rollback:", error);
        toast.error("Failed to rollback", {
            description: String(error),
        });
    } finally {
        isRollingBack.value = false;
    }
}

onMounted(loadHistory);
</script>

<template>
    <div class="space-y-6">
        <div class="bg-card border border-border rounded-xl p-6">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-xl font-bold text-text-primary">
                    Backup History
                </h2>
                <div class="flex items-center gap-2">
                    <button v-if="selectedCount > 0" @click="deleteSelected" :disabled="isDeleting"
                        class="px-4 py-2 bg-red-500/20 text-red-400 rounded-lg hover:bg-red-500/30 disabled:opacity-50 transition-colors font-medium flex items-center gap-2">
                        <Trash2 class="h-4 w-4" />
                        {{ isDeleting ? "Deleting..." : `Delete (${selectedCount})` }}
                    </button>
                    <button @click="loadHistory" :disabled="isLoading"
                        class="px-4 py-2 bg-accent/20 text-accent rounded-lg hover:bg-accent/30 disabled:opacity-50 transition-colors font-medium">
                        {{ isLoading ? "Loading..." : "Refresh" }}
                    </button>
                </div>
            </div>

            <div v-if="isLoading" class="text-center py-8 text-text-secondary">
                Loading history...
            </div>

            <div v-else-if="historyEntries.length === 0" class="text-center py-8 text-text-muted">
                No backup history found. History entries will appear here after you save changes.
            </div>

            <div v-else class="space-y-3">
                <!-- Select All Checkbox -->
                <div class="flex items-center gap-3 p-3 bg-bg-secondary border border-border rounded-lg">
                    <Checkbox v-model="selectAllChecked" />
                    <label class="text-sm font-medium text-text-secondary cursor-pointer"
                        @click="selectAllChecked = !selectAllChecked">
                        Select All ({{ selectedCount }} selected)
                    </label>
                </div>

                <!-- History Entries -->
                <div v-for="entry in historyEntries" :key="entry.filename"
                    class="flex items-center gap-3 p-4 bg-bg-secondary border border-border rounded-lg hover:border-accent/50 transition-colors">
                    <Checkbox :model-value="entryCheckedStates[entry.filename] ?? false"
                        @update:model-value="(checked) => setEntryChecked(entry.filename, checked)" />
                    <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-3 mb-2">
                            <span class="text-sm font-mono text-text-secondary truncate">
                                {{ entry.filename }}
                            </span>
                        </div>
                        <div class="flex items-center gap-4 text-xs text-text-muted">
                            <span>{{ formatDate(entry.timestamp) }}</span>
                            <span>•</span>
                            <span>{{ entry.entry_count }} entries</span>
                            <span>•</span>
                            <span>{{ formatFileSize(entry.file_size) }}</span>
                        </div>
                    </div>
                    <button @click="rollbackTo(entry.filename)" :disabled="isRollingBack"
                        class="ml-4 px-4 py-2 bg-accent text-white rounded-lg hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium text-sm shrink-0">
                        {{ isRollingBack ? "Rolling back..." : "Restore" }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>
