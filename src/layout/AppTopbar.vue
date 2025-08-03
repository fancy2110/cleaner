<script setup lang="ts">
import { ref } from 'vue';
import { FileInfo, Volumn } from '@/types/fs';
import router from '@/router';
interface Props { }

const props = defineProps<Props>();

// Define custom events for use outside the component
const emit = defineEmits<{
    (e: 'pathChange', path: string): void;
    (e: 'startScan', path: string): void;
    (e: 'scanComplete', success: boolean, info: FileInfo | null): void;
}>();

const showDriveSelector = ref(true);
const selectedDrive = ref<string>('/');

// 选择驱动器
function selectDrive(drive: { name: string; path: string }) {
    selectedDrive.value = drive.name;
    emit('pathChange', drive.path);
    showDriveSelector.value = false;
}

const availableDrives = ref([
    {
        // path: '/',
        label: 'Disk 1',
        icon: 'pi pi-save',
        command: () => {
            selectDrive({ name: 'Home1', path: '/' });
        }
        // totalSize: 100
    },
    {
        // path: '/',
        icon: 'pi pi-save',
        // totalSize: 100,
        label: 'Disk 2',
        command: () => {
            selectDrive({ name: 'Home2', path: '/' });
        }
    },
    {
        // path: '/',
        // totalSize: 100,
        label: 'Disk 3',
        icon: 'pi pi-trash',
        command: () => {
            selectDrive({ name: 'Home3', path: '/' });
        }
    }
]);

const menu = ref(null);

function toggleMenu(event: Event) {
    menu.value.toggle(event);
}

const loading = ref([false, false, false]);

function load(index: number) {
    loading.value[index] = true;
    setTimeout(() => {
        loading.value[index] = false;
        console.log('scan complete');
        router.push('/main');
    }, 1000);
}
</script>

<template>
    <div class="flex p-2">
        <div>
            <Menu ref="menu" :model="availableDrives" :popup="true" />
            <Button class="w-full" type="button" :label="selectedDrive" icon="pi pi-angle-down" @click="toggleMenu" />
        </div>

        <div class="w-full flex items-center mx-4 border-2 border-black">
            <!-- disk scan progress-->
            <!-- <ProgressBar :value="50" :showValue="true" class="h-1 w-full items-center"></ProgressBar> -->
            <div class="text-sm w-1/2 text-left">Used 35GB</div>
            <div class="text-sm w-1/2 text-right">Total 2000TB</div>
        </div>

        <Button type="button" label="Search" icon="pi pi-search" :loading="loading[0]" @click="load(0)" />

        <!-- <Button type="button" label="Scan" @click="$router.push('/uikit/button')" /> -->
    </div>
</template>
