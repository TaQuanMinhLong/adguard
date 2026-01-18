<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import type { AlertVariants } from "./index"
import { cn } from "@/styles"
import { alertVariants } from "./index"
import { X } from "lucide-vue-next"

const props = defineProps<{
    class?: HTMLAttributes["class"]
    variant?: AlertVariants["variant"]
    dismissible?: boolean
}>()

const emit = defineEmits<{
    close: []
}>()

</script>

<template>
    <div data-slot="alert" :class="cn(alertVariants({ variant }), props.class, 'relative')" role="alert">
        <slot />
        <button v-if="dismissible" @click="emit('close')"
            class="absolute right-3 top-3 rounded-md p-1 text-text-secondary hover:bg-bg-tertiary hover:text-text-primary transition-colors focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-card"
            aria-label="Close alert">
            <X class="h-4 w-4" />
        </button>
    </div>
</template>