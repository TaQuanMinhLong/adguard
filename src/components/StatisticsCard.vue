<script setup lang="ts">
defineProps<{
    title: string;
    value: string | number;
    icon?: string;
}>();
</script>

<template>
    <div class="bg-card border border-border rounded-xl p-6 shadow-sm">
        <div class="flex items-center justify-between">
            <div>
                <p class="text-sm text-text-secondary mb-1">{{ title }}</p>
                <p class="text-2xl font-bold text-text-primary">{{ value }}</p>
            </div>
            <div v-if="icon" class="w-12 h-12 rounded-full bg-accent/10 flex items-center justify-center">
                <span class="text-2xl">{{ icon }}</span>
            </div>
        </div>
    </div>
</template>
