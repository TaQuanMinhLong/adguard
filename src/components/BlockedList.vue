<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";
import DomainItem from "./DomainItem.vue";
import StatisticsCard from "./StatisticsCard.vue";

interface BlockedDomain {
    ip: string;
    hostname: string;
}

const props = defineProps<{
    disabled?: boolean;
}>();

const blockedDomains = ref<BlockedDomain[]>([]);
const statistics = ref({ total_blocked: 0, unique_ips: 0 });
const isLoading = ref(false);
const isSaving = ref(false);
const showAddInput = ref(false);
const newDomain = ref("");
const isAdding = ref(false);

async function loadBlockedDomains() {
    isLoading.value = true;
    try {
        const domains = await invoke<[string, string][]>("get_blocked_domains");
        blockedDomains.value = domains.map(([ip, hostname]) => ({ ip, hostname }));

        const stats = await invoke<{
            total_blocked: number;
            unique_ips: number;
        }>("get_statistics");
        statistics.value = stats;
    } catch (error) {
        console.error("Failed to load blocked domains:", error);
    } finally {
        isLoading.value = false;
    }
}

async function addDomain() {
    const domain = newDomain.value.trim();
    if (!domain) {
        toast.error("Please enter a domain name");
        return;
    }

    // Basic domain validation
    if (!/^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/.test(domain)) {
        toast.error("Please enter a valid domain name");
        return;
    }

    isAdding.value = true;
    try {
        // Use 127.0.0.1 as the default IP for blocking
        await invoke("add_domain", { ip: "127.0.0.1", hostname: domain });
        await loadBlockedDomains();
        newDomain.value = "";
        showAddInput.value = false;
        toast.success("Domain added successfully");
    } catch (error) {
        console.error("Failed to add domain:", error);
        toast.error("Failed to add domain", {
            description: String(error),
        });
    } finally {
        isAdding.value = false;
    }
}

async function removeDomain(ip: string, hostname: string) {
    try {
        await invoke("remove_domain", { ip, hostname });
        await loadBlockedDomains();
        toast.success("Domain removed successfully");
    } catch (error) {
        console.error("Failed to remove domain:", error);
        toast.error("Failed to remove domain", {
            description: String(error),
        });
    }
}

async function saveChanges() {
    isSaving.value = true;
    try {
        await invoke("save_changes");
        await loadBlockedDomains();
        toast.success("Changes saved successfully!");
    } catch (error) {
        console.error("Failed to save changes:", error);
        toast.error("Failed to save changes", {
            description: String(error),
        });
    } finally {
        isSaving.value = false;
    }
}

function handleAddKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
        addDomain();
    } else if (e.key === "Escape") {
        showAddInput.value = false;
        newDomain.value = "";
    }
}

onMounted(loadBlockedDomains);
</script>

<template>
    <div class="space-y-6">
        <!-- Statistics -->
        <StatisticsCard title="Total Blocked Domains" :value="statistics.total_blocked" icon="🚫" />

        <!-- Blocked Domains List -->
        <div class="bg-card border border-border rounded-xl p-6">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-xl font-bold text-text-primary">
                    Blocked Domains
                </h2>
                <div class="flex items-center gap-2">
                    <button v-if="!showAddInput" @click="showAddInput = true" :disabled="props.disabled"
                        class="px-4 py-2 bg-accent/20 text-accent rounded-lg hover:bg-accent/30 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed">
                        + Add Domain
                    </button>
                    <button @click="saveChanges" :disabled="isSaving || props.disabled"
                        class="px-4 py-2 bg-accent text-white rounded-lg hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium">
                        {{ isSaving ? "Saving..." : "Save Changes" }}
                    </button>
                </div>
            </div>

            <!-- Add Domain Input -->
            <div v-if="showAddInput" class="mb-4 p-4 bg-bg-secondary border border-border rounded-lg">
                <div class="flex items-center gap-2">
                    <input v-model="newDomain" @keydown="handleAddKeydown" type="text"
                        placeholder="Enter domain name (e.g., example.com)"
                        class="flex-1 px-4 py-2 bg-bg-primary border border-border rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-accent focus:border-transparent"
                        :disabled="isAdding" />
                    <button @click="addDomain" :disabled="isAdding || !newDomain.trim() || props.disabled"
                        class="px-4 py-2 bg-accent text-white rounded-lg hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium">
                        {{ isAdding ? "Adding..." : "Add" }}
                    </button>
                    <button @click="showAddInput = false; newDomain = ''"
                        class="px-4 py-2 bg-bg-tertiary text-text-secondary rounded-lg hover:bg-bg-tertiary/80 transition-colors font-medium">
                        Cancel
                    </button>
                </div>
            </div>

            <div v-if="isLoading" class="text-center py-8 text-text-secondary">
                Loading...
            </div>

            <div v-else-if="blockedDomains.length === 0" class="text-center py-8 text-text-muted">
                No blocked domains yet. Click "Add Domain" to block a domain.
            </div>

            <!-- Grid Layout with Container Queries -->
            <div v-else class="@container">
                <div class="grid grid-cols-1 @[32rem]:grid-cols-2 @[48rem]:grid-cols-3 gap-3">
                    <DomainItem v-for="(domain, index) in blockedDomains"
                        :key="`${domain.ip}-${domain.hostname}-${index}`" :ip="domain.ip" :hostname="domain.hostname"
                        :disabled="props.disabled" @remove="removeDomain" />
                </div>
            </div>
        </div>
    </div>
</template>
