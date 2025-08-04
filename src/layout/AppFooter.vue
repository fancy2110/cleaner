<script setup lang="ts">
import AppConfigurator from './AppConfigurator.vue';
import { FileInfo, Volumn } from '@/types/fs';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
</script>

<template>
    <div class="items-center w-full gap-2 p-2 place-content-end">
        <Panel header="To Delete: 13.8 GB" :toggleable="true">
            <p class="leading-normal m-0 w-full">
                Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et
                dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip
                ex ea commodo consequat.
                Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
                Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est
                laborum.
            </p>
        </Panel>

        <div class="place-content-center">
            <Button icon="pi pi-trash" label="Delete" @click="$router.push('/pages/empty')"> </Button>
            <!-- <Button type="button" label="Delete" icon="pi pi-search" :loading="loading[0]" /> -->
        </div>
    </div>
</template>

<style lang="scss" scoped>
.p-progressbar {
    height: 1rem;
}
</style>
