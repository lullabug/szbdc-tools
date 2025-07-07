<script setup lang="ts">
import "vfonts/FiraCode.css";
</script>

<template>
    <div class="header">
        <img
            src="../assets/szbgcAvatar.webp"
            alt="szbgcAvatar"
            class="header-avatar"
        />
        <div>Shenzhen BanG Dream! Club</div>
    </div>
</template>

<style scoped>
.header {
    height: 6rem;
    width: 100%;
    border-bottom: 1px solid rgba(212, 201, 198, 0.5);
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 2rem;
    font-family: "Fira Code", monospace;
    font-weight: bold;
    font-size: 2rem;
    color: rgba(212, 201, 198, 1);
}

.header-avatar {
    height: 80%;
}
</style>
