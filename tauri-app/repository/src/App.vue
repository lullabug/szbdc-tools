<script setup lang="ts">
import { NConfigProvider } from "naive-ui";
import Header from "./components/Header.vue";

import { sampleRepo } from "./components/plugin/sample.ts";
import { fumoRepo } from "./components/plugin/fumo.ts";
import DataTable from "./components/DataTable.vue";
</script>

<template>
    <n-config-provider>
        <div class="app">
            <Header class="app-header"></Header>
            <DataTable :items="[sampleRepo, fumoRepo]" />
        </div>
    </n-config-provider>
</template>

<style scoped>
.app {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    align-items: center;
}

.app-header {
    width: 90vw;
    margin-top: 1rem;
    margin-bottom: 2rem;
}

.data-table {
    margin: 1rem;
    width: 80%;
}
</style>
