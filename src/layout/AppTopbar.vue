<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { createDefaultVolumn, Volumn, formatFileSize } from '@/types/fs';
import { FileSystemService } from '@/service/FileSystemService';
import { MenuItem } from 'primevue/menuitem';
import { ScannerService, ScanProgress } from '@/service/ScannerService';

const props = defineProps<{
    currentPath: string | null;
    scanProgress: ScanProgress | null;
}>();
// Define custom events for use outside the component
const emit = defineEmits<{
    pathChange: [path: string]; // named tuple syntax
    startScan: [path: string]; // named tuple syntax
    cancelScan: []; // named tuple syntax
}>();

const showDriveSelector = ref(true);
const selectedDrive = ref<{ driver: Volumn | null; path: string }>({ driver: null, path: '' });
const availableDrives = ref<MenuItem[]>([]);
const isScanning = ref(false);

// Platform detection utility
const isWindows = navigator.userAgent.toUpperCase().indexOf('WIN') >= 0 || navigator.platform.toUpperCase().indexOf('WIN') >= 0;
const isMac = navigator.userAgent.toUpperCase().indexOf('MAC') >= 0 || navigator.platform.toUpperCase().indexOf('MAC') >= 0;

// 获取可用的磁盘/驱动器
async function getAvailableDrives(): Promise<Volumn[]> {
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
        let item = isWindows ? { name: 'C:', path: 'C:\\', icon: '💻' } : { name: 'Root', path: '/', icon: '🖥️' };
        drives = [createDefaultVolumn(item)];
    }

    availableDrives.value = drives.map((drive) => ({
        label: drive.name,
        icon: 'pi pi-save',
        command: () => {
            selectDrive(drive);
        }
    }));
    console.log('availableDrives:', availableDrives.value);
    return drives;
}

// 选择驱动器
function selectDrive(volumn: Volumn) {
    console.log('selectDrive:', volumn);
    selectedDrive.value = { driver: volumn, path: volumn.path };
    emit('pathChange', volumn.path);
    showDriveSelector.value = false;
}

// 选择驱动器
function selectPath(path: string) {
    let driver = selectedDrive.value.driver;
    selectedDrive.value = { driver: driver, path: path };
    emit('pathChange', path);
    showDriveSelector.value = false;
}

const menu = ref();

function toggleMenu(event: Event | null) {
    if (event == null) return;
    menu.value.toggle(event);
}

const scan_progress = ref(0);

// 开始扫描
async function startScan() {
    const drive = selectedDrive;
    if (!drive.value || isScanning.value) return;
    scan_progress.value = 0;
    emit('startScan', drive.value.path);
}

async function cancelScan() {
    let ret = await ScannerService.stopScan();
    console.log('app topbar cancelScan:', ret);
    emit('cancelScan');
}

// 组件挂载时获取可用驱动器
onMounted(async () => {
    let volums = await getAvailableDrives();
    if (volums.length > 0) {
        selectDrive(volums[0]);
    } else {
        // Add click outside listener

        // Set initial selected drive if we have a currentPath
        // Set a default selected drive if no current path
        let driver = isWindows ? 'C:' : isMac ? 'Macintosh HD' : 'Root';
        let volumn = createDefaultVolumn({ name: driver });
        selectedDrive.value = { driver: volumn, path: '/' };
    }
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

/**
 * 获取当前的磁盘使用信息
 */
function getUsedSize(): string {
    let driver = selectedDrive.value.driver ?? { totalSize: 0, availableSize: 0 };
    let usedSize = driver.totalSize - driver.availableSize;
    return formatFileSize(usedSize);
}

const pathHome = ref({
    icon: 'pi pi-home',
    to: '/',
    command: () => {
        selectPath('/');
    }
});
/**
 * 更新扫描进度
 */
watch(
    () => props,
    (newValue) => {
        let progress = newValue.scanProgress;
        if (progress == null) return;

        isScanning.value = progress.is_scanning;
        let driver = selectedDrive.value.driver;
        if (progress.is_scanning) {
            scan_progress.value = progress.total_size / (driver?.totalSize ?? Infinity);
        } else {
            scan_progress.value = 0;
        }
    }
);
// 假设 path 是从某处获取的当前路径变量
const pathItems = ref<MenuItem[] | undefined>();

// 分割路径并生成 pathItems
watch(
    selectedDrive,
    (newValue) => {
        let newPath = newValue.path;
        const paths = newPath.split('/').filter(Boolean); // 过滤空字符串
        const items: MenuItem[] = [];
        let currentPath = '';

        paths.forEach((segment) => {
            currentPath += `/${segment}`;
            items.push({
                label: segment,
                command: () => {
                    selectPath(currentPath);
                }
            });
        });

        pathItems.value = items;
    },
    { immediate: true }
);
</script>

<template>
    <div class="flex p-2">
        <div class="flex flex-none">
            <Menu ref="menu" :model="availableDrives" :popup="true" />
            <Button class="w-full" type="button" :label="selectedDrive.driver?.name" icon="pi pi-angle-down"
                @click="toggleMenu" />
        </div>

        <div class="w-full flex items-center mx-4 gap-5">
            <div class="flex flex-none items-center">
                <i class="fas fa-hdd" style="color: #95a5a6"></i>
                <span class="ml-2">Total: {{ formatFileSize(selectedDrive.driver?.totalSize) }}</span>
            </div>
            <div class="flex flex-none items-center">
                <i class="fas fa-chart-pie" style="color: #f39c12"></i>
                <span class="ml-2">Used: {{ getUsedSize() }}</span>
            </div>
        </div>

        <div class="flex w-full items-center place-content-end">
            <div class="w-full" v-if="isScanning">
                <ProgressBar :value="scan_progress" :showValue="true"></ProgressBar>
            </div>

            <div class="ml-4">
                <Button type="button" :label="scanButtonLabel()" :icon="scanButtonIcon()"
                    @click="isScanning ? cancelScan() : startScan()" />
            </div>
        </div>

        <!-- <Button type="button" label="Scan" @click="$router.push('/uikit/button')" /> -->
    </div>
    <Breadcrumb class="flex w-full h-10" :home="pathHome" :model="pathItems"></Breadcrumb>
</template>
