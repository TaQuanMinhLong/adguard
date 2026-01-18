<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { cn } from "@/styles"

const props = defineProps<{
    class?: HTMLAttributes["class"]
}>()
</script>

<template>
    <div data-slot="alert-description"
        :class="cn('text-text-secondary col-start-2 grid justify-items-start gap-1 text-sm [&_p]:leading-relaxed', props.class)">
        <slot />
    </div>
</template>