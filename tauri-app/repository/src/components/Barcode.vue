<script setup lang="ts">
import { NModal, NButton, NCard, NPopselect } from "naive-ui";
import { nextTick, onMounted, ref, watch } from "vue";
import { Icon } from "@vicons/utils";
import QrCode from "@vicons/material/QrCodeScannerRound";
import { SelectMixedOption } from "naive-ui/es/select/src/interface";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{
    (e: "barcode", barcode: string): void;
}>();

const camaraList = ref<MediaDeviceInfo[]>([]);
const videoRef = ref<HTMLVideoElement | null>(null);

const curCameraId = ref<string | null>(null);
const curCamara = ref<MediaDeviceInfo | null>(null);

watch(curCameraId, (id) => {
    if (id) {
        curCamara.value =
            camaraList.value.find((device) => device.deviceId === id) || null;
    } else {
        curCamara.value = null;
    }
});

async function getCamaraList() {
    try {
        await navigator.mediaDevices.getUserMedia({
            video: true,
        });
        const devices = await navigator.mediaDevices.enumerateDevices();
        camaraList.value = devices.filter(
            (device) => device.kind === "videoinput"
        );
        if (camaraList.value.length > 0) {
            curCameraId.value = camaraList.value[0].deviceId;
        }
    } catch (e) {
        console.error("Error getting camera list:", e);
    }
}

const popselectOptions = ref<SelectMixedOption[]>([]);

watch(camaraList, (list) => {
    popselectOptions.value = list.map((device) => ({
        label: device.label || "Unnamed Camera",
        value: device.deviceId,
    }));
});

const modalShow = ref(false);

function handleButtonClick() {
    modalShow.value = true;
}

async function startCamera() {
    await stopCamera();
    if (curCamara.value && videoRef.value) {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({
                video: {
                    width: { min: 320, ideal: 640, max: 1280 },
                    height: { min: 240, ideal: 480, max: 720 },
                    deviceId: curCamara.value.deviceId,
                },
            });
            videoRef.value.srcObject = stream;
        } catch (e) {
            console.error("Error accessing camera:", e);
        }
    }
}

async function stopCamera() {
    if (videoRef.value && videoRef.value.srcObject) {
        const stream = videoRef.value.srcObject as MediaStream;
        stream.getTracks().forEach((track) => track.stop());
        videoRef.value.srcObject = null;
    }
}

watch(modalShow, async (show) => {
    if (show) {
        await nextTick();
        await startCamera();
        startScan();
    } else {
        stopScan();
        await stopCamera();
    }
});

interface LumaData {
    luma: Uint8Array;
    width: number;
    height: number;
}

function captureLuma(): LumaData | null {
    if (!videoRef.value || !videoRef.value.srcObject) {
        return null;
    }
    const canvas = document.createElement("canvas");
    canvas.width = videoRef.value.videoWidth;
    canvas.height = videoRef.value.videoHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
        return null;
    }
    ctx.drawImage(videoRef.value, 0, 0, canvas.width, canvas.height);
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const data = imageData.data;
    const luma = new Uint8Array(canvas.width * canvas.height);
    for (let i = 0, j = 0; i < data.length; i += 4, j++) {
        const lumaValue = Math.round(
            0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2]
        );
        luma[j] = lumaValue;
    }
    return { luma, width: canvas.width, height: canvas.height };
}

const timer = ref<number | null>(null);

const scanInterval = 500;

function stopScan() {
    if (timer.value !== null) {
        clearInterval(timer.value);
        timer.value = null;
    }
}

function startScan() {
    timer.value = setInterval(async () => {
        const captureData = captureLuma();
        if (!captureData) {
            return;
        }
        try {
            const rs = await invoke<string | null>("scan_barcode", {
                luma: Array.from(captureData.luma),
                width: captureData.width,
                height: captureData.height,
            });

            if (rs) {
                emit("barcode", rs);
                modalShow.value = false;
            }
        } catch (e) {
            console.error("Error scanning barcode:", e);
            return;
        }
    }, scanInterval);
}

onMounted(async () => {
    await getCamaraList();
});
</script>

<template>
    <n-button @click="handleButtonClick">
        <Icon><QrCode /></Icon>
    </n-button>
    <n-modal v-model:show="modalShow">
        <n-card class="modal-card">
            <div class="modal-card-content">
                <n-popselect
                    :options="popselectOptions"
                    v-model:value="curCameraId"
                    trigger="click"
                >
                    <n-button>{{
                        curCamara?.label || "No camera selected"
                    }}</n-button>
                </n-popselect>
                <video
                    ref="videoRef"
                    autoplay
                    playsinline
                    class="barcode-video"
                ></video>
            </div>
        </n-card>
    </n-modal>
</template>

<style scoped>
.modal-card {
    width: 50vw;
}
.modal-card-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
}
.barcode-video {
    width: 90%;
    padding-bottom: 2rem;
}
</style>
