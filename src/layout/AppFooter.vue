<!-- eslint-disable vue/no-parsing-error -->
<script setup lang="ts">
import { ScannerService, TrashListener } from '@/service/ScannerService';
import File from '@/views/uikit/File.vue';
import { FileInfo, formatFileSize } from '@/types/fs';
import { onMounted, onUnmounted, ref } from 'vue';

let listener = ref<() => void>();
const onTrashChanged: TrashListener = (files: FileInfo[], size: number) => {
    items.value = files;
    delelteSize.value = formatFileSize(size);
    console.log('onTrashChanged', files, size);
};

const delelteSize = ref<string>('0 GB');
const items = ref<FileInfo[]>([]);

function onRowSelect(event: any) {
    let file = event.data.path;
    console.log('on item selected in trash', file);
    // ScannerService.setCurrentDirectory(file);
}

function removeFileFromTrash(value: FileInfo) {
    ScannerService.removeFileFromTrash(value);
    console.log('on selected item ', value);
}

onMounted(() => {
    listener.value = ScannerService.subscribeTrash(onTrashChanged);
});

onUnmounted(() => {
    let job = listener.value;
    console.log('onUnmounted');
    if (job) {
        job();
    }
});
</script>

<template>
    <!-- <div class="items-center w-full gap-2 p-2 place-content-end"> -->
    <Panel toggleable :collapsed="true">
        <template #header>
            <div class="flex items-center gap-2">
                <span class="pi pi-trash" style="font-size: 2rem"></span>
                <span class="font-bold">To Delete: {{ delelteSize }}</span>
            </div>
        </template>

        <template #icons>
            <Button icon="pi pi-trash" severity="secondary" rounded text @click="$router.push('/pages/empty')" />
        </template>

        <template #default>
            <div v-if="items.length > 0">
                <DataTable :value="items" selectionMode="single" @rowSelect="onRowSelect" scrollable dataKey="path"
                    scrollHeight="100%" :metaKeySelection="false">
                    <Column>
                        <template #body="slotProps">
                            <File :file="slotProps.data" />
                        </template>
                    </Column>
                    <Column :field="(rowData: FileInfo) => formatFileSize(rowData.size)" style="width: 150px"> </Column>
                    <Column style="width: 3rem">
                        <template #body="slotProps">
                            <Button type="button" icon="pi pi-times" rounded severity="warn" size="small"
                                v-on:click="removeFileFromTrash(slotProps.data)" />
                        </template>
                    </Column>
                </DataTable>
            </div>
            <div v-else class="p-4 text-center text-gray-500">回收站中没有文件</div>
        </template>
    </Panel>
    <!-- </div> -->
</template>

<style lang="scss" scoped>
.p-panel {
    border: 1px solid var(--p-panel-border-color);
    border-radius: 0px;
    background: var(--p-panel-background);
    color: var(--p-panel-color);
}
</style>
