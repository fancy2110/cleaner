<!-- eslint-disable vue/no-parsing-error -->
<script setup lang="ts">
import { ScannerService, TrashListener } from '@/service/ScannerService';
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
    // ScannerService.addFileToTrash(value);
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
    <div class="items-center w-full gap-2 p-2 place-content-end">
        <Panel toggleable :collapsed="true">
            <template #header>
                <div class="flex items-center gap-2">
                    <Avatar image="https://primefaces.org/cdn/primevue/images/avatar/amyelsner.png" shape="circle" />
                    <span class="font-bold">To Delete: {{ delelteSize }}</span>
                </div>
            </template>

            <template #icons>
                <Button icon="pi pi-trash" severity="secondary" rounded text @click="$router.push('/pages/empty')" />
            </template>

            <template #default>
                <DataTable :value="items" selectionMode="single" @rowSelect="onRowSelect" scrollable dataKey="path"
                    scrollHeight="100%" :metaKeySelection="false">
                    <Column :field="(rowData: FileInfo) => rowData.name" style="width: 250px"> </Column>
                    <Column :field="(rowData: FileInfo) => formatFileSize(rowData.size)" style="width: 150px"> </Column>
                    <Column style="width: 10rem">
                        <template #body="slotProps">
                            <div class="flex flex-wrap gap-2">
                                <Button type="button" icon="pi pi-restore" rounded severity="success"
                                    v-on:click="removeFileFromTrash(slotProps.data)" />
                            </div>
                        </template>
                    </Column>
                </DataTable>
            </template>
        </Panel>
    </div>
</template>
