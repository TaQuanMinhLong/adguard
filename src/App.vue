<script setup lang="ts">
import { onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Toaster } from "vue-sonner";
import { useTheme } from "./composables/useTheme";
import BlockedList from "./components/BlockedList.vue";
import History from "./components/History.vue";
import Settings from "./components/Settings.vue";
import { activeTab, hasAdminPrivileges, showAdminWarning } from "./state";
import { Alert, AlertDescription, AlertTitle } from "./components/ui/alert";

const { loadTheme } = useTheme();



onMounted(async () => {
  loadTheme();
  // Check admin privileges
  try {
    hasAdminPrivileges.value = await invoke<boolean>("check_admin_privileges");
  } catch (error) {
    console.error("Failed to check admin privileges:", error);
    hasAdminPrivileges.value = false;
  }
});


</script>

<template>
  <div class="min-h-screen bg-bg-primary">
    <!-- Header -->
    <header class="border-b border-border bg-bg-secondary">
      <div class="container mx-auto px-6 py-4">
        <div class="flex items-center justify-between">
          <h1 class="text-2xl font-bold text-text-primary">
            AdBlock Manager
          </h1>
          <div class="flex items-center gap-2">
            <span class="text-sm text-text-secondary">v0.1.0</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Navigation Tabs -->
    <nav class="border-b border-border bg-bg-secondary">
      <div class="container mx-auto px-6">
        <div class="flex gap-1">
          <button @click="activeTab = 'blocked'" :class="[
            'px-6 py-3 font-medium transition-colors border-b-2',
            activeTab === 'blocked'
              ? 'text-accent border-accent'
              : 'text-text-secondary border-transparent hover:text-text-primary',
          ]">
            Blocked Domains
          </button>
          <button @click="activeTab = 'history'" :class="[
            'px-6 py-3 font-medium transition-colors border-b-2',
            activeTab === 'history'
              ? 'text-accent border-accent'
              : 'text-text-secondary border-transparent hover:text-text-primary',
          ]">
            History
          </button>
          <button @click="activeTab = 'settings'" :class="[
            'px-6 py-3 font-medium transition-colors border-b-2',
            activeTab === 'settings'
              ? 'text-accent border-accent'
              : 'text-text-secondary border-transparent hover:text-text-primary',
          ]">
            Settings
          </button>
        </div>
      </div>
    </nav>

    <!-- Main Content -->
    <main class="container mx-auto px-6 py-8">
      <!-- Admin Privileges Warning -->
      <Alert v-if="hasAdminPrivileges === false && showAdminWarning" variant="warning" class="mb-6" dismissible
        @close="showAdminWarning = false">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          stroke-linecap="round" stroke-linejoin="round">
          <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
          <path d="M12 9v4" />
          <path d="M12 17h.01" />
        </svg>
        <AlertTitle>Administrator Privileges Required</AlertTitle>
        <AlertDescription>
          This application requires administrator privileges to modify the hosts file. Please run the application as
          administrator to save changes. You can still view and manage blocked domains, but changes won't be saved to
          the hosts file without admin privileges.
        </AlertDescription>
      </Alert>

      <Transition name="fade" mode="out-in">
        <BlockedList v-if="activeTab === 'blocked'" :key="'blocked'" :disabled="hasAdminPrivileges === false" />
        <History v-else-if="activeTab === 'history'" :key="'history'" />
        <Settings v-else-if="activeTab === 'settings'" :key="'settings'" />
      </Transition>
    </main>

    <!-- Toast Notifications -->
    <Toaster richColors position="top-right" />
  </div>
</template>
