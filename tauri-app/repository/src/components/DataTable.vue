<script setup lang="ts">
import { NDropdown, NButton, NInput, NDataTable, NPopselect } from "naive-ui";
import { computed, nextTick, ref, VNodeRef, watch } from "vue";

import "vfonts/FiraCode.css";
import Barcode from "./Barcode.vue";
import AddModal from "./AddModal.vue";
import { columnHeaders, RepositoryCallback, RowData } from "./plugin/interface";

import Add from "@vicons/material/PlaylistAddRound";
import { Icon } from "@vicons/utils";

interface PropItem {
    repo_name: string;
    callback: RepositoryCallback;
}

const props = defineProps<{
    items: PropItem[];
}>();

const curRepoName = ref<string | null>(null);

const repoOptions = computed(() => {
    return props.items.map((item) => ({
        label: item.repo_name,
        value: item.repo_name,
    }));
});

watch(curRepoName, async (v) => {
    if (v) {
        await handleRepoSelect(v);
    }
});

const curFilterValue = ref<string>("uid");

const filterOptions = ref([
    { label: "Filter by UID", value: "uid" },
    { label: "Filter by SKU", value: "sku" },
]);

watch(curFilterValue, () => {
    clearFilterInput();
});

const rows = ref<RowData[]>([]);

async function handleRepoSelect(key: string) {
    rows.value = await Promise.resolve(
        props.items.find((item) => item.repo_name === key)?.callback.load() ||
            []
    );
    console.log("Loaded rows:", rows.value);
}

const tableRef = ref<VNodeRef | null>(null);

function handleFilterInput(v: string) {
    if (curFilterValue.value === "uid") {
        tableRef.value.filter({
            uid: [v],
        });
    } else if (curFilterValue.value === "sku") {
        tableRef.value.filter({
            sku: [v],
        });
    }
}

const filterInputValue = ref<string>("");

watch(filterInputValue, (v) => {
    handleFilterInput(v);
});

function clearFilterInput() {
    filterInputValue.value = "";
    handleFilterInput("");
}

function handleBarcodeScanned(barcode: string) {
    console.log("Barcode scanned:", barcode);
    filterInputValue.value = barcode;
    handleFilterInput(barcode);
}

const showContextMenu = ref(false);
const contextMenuOptions = ref([{ label: "Remove", key: "remove" }]);
const contextMenuX = ref(0);
const contextMenuY = ref(0);

const selectedRow = ref<RowData | null>(null);

function contextMenuRowProps(row: RowData) {
    return {
        onContextmenu: (e: MouseEvent) => {
            e.preventDefault();

            selectedRow.value = row;

            showContextMenu.value = false;
            nextTick().then(() => {
                contextMenuX.value = e.clientX;
                contextMenuY.value = e.clientY;
                showContextMenu.value = true;
            });
        },
    };
}

function handleClickOutsideContextMenu() {
    showContextMenu.value = false;
}

async function handleSelectContextMenuSelect(key: string) {
    if (selectedRow.value) {
        const callback = (
            props.items.find(
                (item) => item.repo_name === curRepoName.value
            ) as PropItem
        ).callback;

        if (key === "remove") {
            await Promise.resolve(callback.rm(selectedRow.value.uid));
            const index = rows.value.findIndex(
                (row) => row.uid === selectedRow.value!.uid
            );
            rows.value.splice(index, 1);
            console.log("Removed row:", selectedRow.value);
        }
    }
    showContextMenu.value = false;
}

const addModalShow = ref(false);

async function handleAddButtonClick() {
    console.log("Add button clicked");
    addModalShow.value = true;
}

async function handleAddModalConfirm(data: { sku: string; metadata: string }) {
    if (curRepoName.value) {
        const callback = (
            props.items.find(
                (item) => item.repo_name === curRepoName.value
            ) as PropItem
        ).callback;

        await Promise.resolve(callback.add(data.sku, data.metadata));
        rows.value = await Promise.resolve(callback.load());
    }
    addModalShow.value = false;
}

async function handleAddModalClose() {
    addModalShow.value = false;
}
</script>

<template>
    <div class="switcher">
        <n-popselect
            :options="repoOptions"
            v-model:value="curRepoName"
            trigger="click"
        >
            <n-button class="switcher-button">{{
                curRepoName || "Choose Repository"
            }}</n-button>
        </n-popselect>
        <div class="filter">
            <n-popselect
                :options="filterOptions"
                v-model:value="curFilterValue"
                trigger="click"
            >
                <n-button class="switcher-button"
                    >Filter by
                    {{ curFilterValue.toLocaleUpperCase() }}</n-button
                >
            </n-popselect>
            <n-input
                type="text"
                class="filter-input"
                v-model:value="filterInputValue"
            />
            <Barcode @barcode="handleBarcodeScanned" />
        </div>
        <n-button
            strong
            secondary
            type="primary"
            v-if="curRepoName"
            @click="handleAddButtonClick"
        >
            <Icon size="1.5rem"><Add /></Icon>
        </n-button>
    </div>
    <n-data-table
        :columns="columnHeaders"
        :data="rows"
        :pagination="{ pageSize: 10 }"
        :row-key="(row) => row.uid"
        :row-props="contextMenuRowProps"
        striped
        ref="tableRef"
        class="data-table"
    />
    <n-dropdown
        placement="bottom-start"
        trigger="manual"
        :x="contextMenuX"
        :y="contextMenuY"
        :options="contextMenuOptions"
        :show="showContextMenu"
        @clickoutside="handleClickOutsideContextMenu"
        @select="handleSelectContextMenuSelect"
    ></n-dropdown>
    <AddModal
        :visible="addModalShow"
        @close="handleAddModalClose"
        @confirm="handleAddModalConfirm"
    />
</template>

<style scoped>
.switcher {
    display: flex;
    flex-direction: column;
    align-items: start;
    width: 80vw;
    gap: 1rem;
    font-family: "Fira Code", monospace;
}

.switcher-button {
    width: 12rem;
}

.filter {
    width: 80vw;
    display: flex;
    gap: 1rem;
}

.data-table {
    margin: 1rem;
    width: 80%;
}

.mono-column {
    font-family: "Fira Code", monospace;
}

.filter-input {
    width: 24rem;
}
</style>
