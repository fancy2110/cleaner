<script setup lang="ts">
import AppConfigurator from './AppConfigurator.vue';
import { FileInfo, Volumn } from '@/types/fs';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

interface Props {
    currentPath: string;
}
</script>

<template>
    <div class="flex flex-row items-center w-full gap-2 p-2">
        <div class="place-content-center w-full">
            <i>4.5GB</i>
            <!-- <Button type="button" label="Delete" icon="pi pi-search" :loading="loading[0]" /> -->
        </div>

        <div class="place-content-center">
            <Button icon="pi pi-check w-10" text raised rounded @click="$router.push('/uikit/panel')">
                <i class="p-3">Delete</i>
            </Button>

            <!-- <Button type="button" label="Delete" icon="pi pi-search" :loading="loading[0]" /> -->
        </div>
    </div>
</template>

<style lang="scss" scoped>
.p-progressbar {
    height: 1rem;
}
</style>
