<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { FileInfo, Volumn } from '@/types/fs';
import router from '@/router';
import { FileSystemService } from '@/service/FileSystemService';
import { ScannerService } from '@/service/ScannerService';

// Define custom events for use outside the component
const emit = defineEmits<{
    (e: 'pathChange', path: string): void;
    (e: 'startScan', path: string): void;
    (e: 'scanComplete', success: boolean, info: FileInfo | null): void;
}>();

const showDriveSelector = ref(true);
const selectedDrive = ref<string>('/');
const availableDrives = ref([
    {
        label: 'Disk',
        icon: 'pi pi-save',
        command: () => {
            selectDrive({ name: 'Root', path: '/' });
        }
    }
]);

// Platform detection utility
const isWindows = navigator.userAgent.toUpperCase().indexOf('WIN') >= 0 || navigator.platform.toUpperCase().indexOf('WIN') >= 0;
const isMac = navigator.userAgent.toUpperCase().indexOf('MAC') >= 0 || navigator.platform.toUpperCase().indexOf('MAC') >= 0;

// 获取可用的磁盘/驱动器
async function getAvailableDrives() {
    var drives: Volumn[] = [];
    try {
        // In a real implementation, this would use Tauri's API to get actual drives
        // For example, using the Tauri fs plugin or a custom command

        drives = await FileSystemService.getAvailableVolumns();
        // If no drive is selected yet, select the first one
        // if (!selectedDrive.value && availableDrives.value.length > 0) {
        //     selectedDrive.value = availableDrives.value[0].name;
        // }
    } catch (error) {
        // Error handling without console logging
        // Fallback to basic drives
        drives = isWindows ? [{ name: 'C:', path: 'C:\\', icon: '💻', totalSize: 0 }] : [{ name: 'Root', path: '/', icon: '🖥️', totalSize: 0 }];
    }

    availableDrives.value = drives.map((drive) => ({
        label: drive.name,
        icon: 'pi pi-save',
        command: () => {
            selectDrive({ name: drive.name, path: drive.path });
        }
    }));
}

// 选择驱动器
function selectDrive(drive: { name: string; path: string }) {
    selectedDrive.value = drive.name;
    emit('pathChange', drive.path);
    showDriveSelector.value = false;
}

const menu = ref<Event>(null);

function toggleMenu(event: Event | null) {
    menu.value.toggle(event);
}

const isScanning = ref(false);
const progress = ref(0);
let interval: any = null;

// 开始扫描
function startScan() {
    const drive = selectDrive;
    if (!drive || isScanning.value) return;
    isScanning.value = true;
    progress.value = 0;
    interval = setInterval(() => {
        let newValue = progress.value + Math.floor(Math.random() * 10) + 1;
        if (newValue >= 100) {
            newValue = 100;
            cancelScan();
        }
        console.log('scan progress:', { newValue });
        progress.value = newValue;
    }, 1000);

    // ScannerService.startScan(
    //     (stats) => {
    //         console.log('scan update:', stats);
    //     },
    //     (progress) => {
    //         console.log('scan progress:', { progress });
    //     },
    //     (message) => {
    //         console.log('scan progress:', { message });
    //         // ScannerService.getFileStats(drive).then((info) => {
    //         //     console.log('scan complete:', info);
    //         //     emit('scanComplete', true, info);
    //         // });
    //         isScanning.value = false;
    //     }
    // ).then(() => {
    //     console.log('call finished:');
    //     isScanning.value = false;
    //     router.push('/main');
    // });
}

function cancelScan() {
    clearInterval(interval);
    isScanning.value = false;
    interval = null;
}

// 组件挂载时获取可用驱动器
onMounted(() => {
    getAvailableDrives();

    // Add click outside listener

    // Set initial selected drive if we have a currentPath
    // Set a default selected drive if no current path
    selectedDrive.value = isWindows ? 'C:' : isMac ? 'Macintosh HD' : 'Root';
    // No additional setup needed for dropdown positioning
});

// 组件卸载时移除事件监听器
onUnmounted(() => { });

function scanButtonLabel(): string {
    return isScanning.value ? 'Cancel' : 'Search';
}

function scanButtonIcon(): string {
    return isScanning.value ? 'pi pi-times' : 'pi pi-search';
}
</script>

<template>
    <div class="flex p-2">
        <div class="flex flex-none">
            <Menu ref="menu" :model="availableDrives" :popup="true" />
            <Button class="w-full" type="button" :label="selectedDrive" icon="pi pi-angle-down" @click="toggleMenu" />
        </div>

        <div class="w-full flex items-center mx-4 gap-5">
            <div class="flex flex-none items-center">
                <i class="fas fa-hdd" style="color: #95a5a6"></i>
                <span class="ml-2">Total: 2 TB</span>
            </div>
            <div class="flex flex-none items-center">
                <i class="fas fa-chart-pie" style="color: #f39c12"></i>
                <span class="ml-2">Used: 388.82 GB</span>
            </div>
        </div>

        <div class="flex w-full items-center place-content-end">
            <div class="w-full" v-if="isScanning">
                <ProgressBar :value="progress" :showValue="true"></ProgressBar>
            </div>

            <div class="ml-4">
                <Button type="button" :label="scanButtonLabel()" :icon="scanButtonIcon()"
                    @click="isScanning ? cancelScan() : startScan()" />
            </div>
        </div>

        <!-- <Button type="button" label="Scan" @click="$router.push('/uikit/button')" /> -->
    </div>
</template>
