<script setup lang="ts">
import type { CheckboxRootEmits, CheckboxRootProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { Check } from "lucide-vue-next"
import { CheckboxIndicator, CheckboxRoot, useForwardPropsEmits } from "reka-ui"
import { cn } from "@/styles"

const props = defineProps<CheckboxRootProps & { class?: HTMLAttributes["class"] }>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = reactiveOmit(props, "class")

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
    <CheckboxRoot v-slot="slotProps" data-slot="checkbox" v-bind="forwarded" :class="cn('peer border-border data-[state=checked]:bg-accent data-[state=checked]:text-white data-[state=checked]:border-accent focus-visible:border-accent focus-visible:ring-accent/50 aria-invalid:ring-red-400/20 aria-invalid:border-red-400 size-4 shrink-0 rounded-[4px] border bg-bg-secondary shadow-xs transition-shadow outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
        props.class)">
        <CheckboxIndicator data-slot="checkbox-indicator"
            class="grid place-content-center text-current transition-none">
            <slot v-bind="slotProps">
                <Check class="size-3.5" />
            </slot>
        </CheckboxIndicator>
    </CheckboxRoot>
</template>