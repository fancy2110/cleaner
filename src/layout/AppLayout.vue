<script setup lang="ts">
import { ref } from 'vue';
import AppFooter from './AppFooter.vue';
import AppTopbar from './AppTopbar.vue';
import { ScannerService, ScanProgress } from '@/service/ScannerService';
import router from '@/router';

const currentProgress = ref<ScanProgress | null>(null);
const currentPath = ref<string | null>('/');

function handlePathChange(path: string | null) {
    console.log('on path changed', path);
}

function handleStartScan(path: string | null) {
    console.log('start path scan', path);
    currentPath.value = path;
    ScannerService.startScan(
        (stats) => {
            console.log('scan update:', stats);
        },
        (progress) => {
            console.log('scan progress:', { progress });
            currentProgress.value = progress;
        },
        (message) => {
            console.log('scan progress:', { message });
            router.push('/main');

            if (path == null) return;

            ScannerService.getFileStats(path).then(
                (info) => {
                    console.log('scan complete:', info);
                },
                (error) => {
                    console.log('scan complete error:', error);
                }
            );
        }
    ).then(() => {
        console.log('call finished:');
    });
}
</script>

<template>
    <div class="flex flex-col h-full">
        <div class="w-full shadow-md">
            <app-topbar :scan-progress="currentProgress" :current-path="currentPath" @path-change="handlePathChange"
                @start-scan="handleStartScan"></app-topbar>
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
