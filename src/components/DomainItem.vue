<script setup lang="ts">
const props = defineProps<{
    ip: string;
    hostname: string;
    disabled?: boolean;
}>();

const emit = defineEmits<{
    remove: [ip: string, hostname: string];
}>();

function handleRemove() {
    if (!props.disabled) {
        emit("remove", props.ip, props.hostname);
    }
}
</script>

<template>
    <div
        class="flex items-center justify-between p-4 bg-bg-secondary border border-border rounded-lg hover:border-accent/50 transition-colors">
        <div class="flex-1 min-w-0">
            <span class="text-text-primary font-medium truncate block">{{ hostname }}</span>
        </div>
        <button @click="handleRemove" :disabled="disabled"
            class="ml-4 px-3 py-1.5 text-sm text-red-400 hover:text-red-300 hover:bg-red-400/10 rounded-md transition-colors shrink-0 disabled:opacity-50 disabled:cursor-not-allowed"
            :title="disabled ? 'Administrator privileges required' : 'Remove domain'">
            Remove
        </button>
    </div>
</template>
