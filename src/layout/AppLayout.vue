<script setup lang="ts">
import AppFooter from './AppFooter.vue';
import AppTopbar from './AppTopbar.vue';

function handlePathChange(event: string | null) {
    console.log('on path changed', event);
}

function handleStartScan(event: string | null) {
    console.log('on path changed', event);
}
</script>

<template>
    <div class="flex flex-col h-full">
        <div class="w-full shadow-md">
            <app-topbar v-on:path-change="handlePathChange" v-on:start-scan="handleStartScan"></app-topbar>
        </div>

        <div class="flex-1 overflow-auto shadow-md mt-2">
            <router-view></router-view>
        </div>

        <div class="flex-none w-full items-center">
            <app-footer></app-footer>
        </div>
    </div>
    <Toast />
</template>
