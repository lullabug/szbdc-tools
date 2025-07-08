<script setup lang="ts">
import { NModal, NCard, NSpace, NInput, NButton } from "naive-ui";
import { computed, ref } from "vue";
import BarCode from "./Barcode.vue";
interface Props {
    visible: boolean;
}

const props = defineProps<Props>();

interface Emits {
    sku: string;
    metadata: string;
}

const emit = defineEmits<{
    (e: "close"): void;
    (e: "confirm", data: Emits): void;
}>();

const isConfirmDisabled = computed(() => {
    return !sku.value || !metadata.value;
});

const sku = ref<string>("");
const metadata = ref<string>("");

function clearInputs() {
    sku.value = "";
    metadata.value = "";
}

function handleClose() {
    emit("close");
    clearInputs();
}

function handleMaskClick() {
    handleClose();
}

function handleConfirm() {
    emit("confirm", {
        sku: sku.value,
        metadata: metadata.value,
    });
    clearInputs();
}

function handleSkuBarcodeScanned(barcode: string) {
    sku.value = barcode;
}
</script>

<template>
    <n-modal
        :show="props.visible"
        :mask-closable="true"
        @update:show="handleMaskClick"
        role="dialog"
    >
        <n-card class="add-modal-card">
            <div class="add-modal-content">
                <n-space vertical>
                    <n-space>
                        <n-input
                            v-model:value="sku"
                            placeholder="Enter SKU"
                            clearable
                            :style="{ width: '28rem' }"
                        />
                        <BarCode @barcode="handleSkuBarcodeScanned" />
                    </n-space>
                    <n-input
                        v-model:value="metadata"
                        placeholder="Enter Metadata"
                        clearable
                        :style="{ width: '28rem' }"
                    />
                    <n-button
                        type="primary"
                        :disabled="isConfirmDisabled"
                        @click="handleConfirm"
                        >Confirm</n-button
                    >
                </n-space>
            </div>
        </n-card>
    </n-modal>
</template>

<style scoped>
.add-modal-card {
    width: 42rem;
    padding: 1rem;
}
</style>
