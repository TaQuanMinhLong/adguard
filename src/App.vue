<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <div>
    <input type="text" v-model="name" placeholder="Enter a name..." />
    <button @click="greet">
      Hello World
    </button>
    <p v-if="greetMsg">{{ greetMsg }}</p>
  </div>
</template>