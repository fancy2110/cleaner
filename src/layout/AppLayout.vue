<script setup lang="ts">
import { ref } from 'vue';
import AppFooter from './AppFooter.vue';
import AppTopbar from './AppTopbar.vue';
import { ScannerService, ScanProgress } from '@/service/ScannerService';
import router from '@/router';
import { Volumn } from '@/types/fs';

const currentProgress = ref<ScanProgress | null>(null);
const currentPath = ref<string | null>('/');

function handlePathChange(path: string | null) {
    console.log('on path changed', path);
    ScannerService.setCurrentDirectory(path ?? '/');
}

function handleStartScan(volumn: Volumn | null) {
    let path = volumn?.path ?? '/';
    console.log('start path scan', path);
    currentPath.value = path;
    ScannerService.startScan(
        (stats) => {
            console.log('scan update:', stats);
        },
        (progress) => {
            currentProgress.value = progress;
        },
        (message) => {
            console.log('scan progress:', { message });
            router.push('/main');
            let progress = currentProgress?.value;
            let newProgress: ScanProgress = {
                is_scanning: false,
                total_files: progress?.total_files ?? 0,
                total_directories: progress?.total_directories ?? 0,
                scaned_size: progress?.scaned_size ?? 0
            };
            currentProgress.value = newProgress;

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
            <app-topbar :scan-progress="currentProgress" :path="currentPath" @path-change="handlePathChange"
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
