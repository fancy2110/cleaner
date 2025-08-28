<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ScannerService } from '@/service/ScannerService';
import { FileInfo, formatFileSize, formatDate, formatFileType } from '@/types/fs';

const { t } = useI18n();
const files = ref<FileInfo[] | null>(null);
const subscriber = ref<() => void>();

const onFileListChanged = (data: FileInfo | null) => {
    console.log('onFileListChanged', data);
    files.value = data?.children || [];
};

onMounted(() => {
    let job = ScannerService.subscribe(onFileListChanged);
    console.log('onMounted', job);
    subscriber.value = job;
});

onUnmounted(() => {
    let job = subscriber.value;
    console.log('onUnmounted');
    if (job) {
        job();
    }
});

// function onRowSelect(value: any) {
//     console.log('on selected item ', value);
// }

// function onRowUnSelect(value: any) {
//     console.log('on un selected item ', value);
// }

const onRowSelect = (event: any) => {
    let file = event.data.path;
    ScannerService.setCurrentDirectory(file);
};
</script>

<template>
    <DataTable :value="files" selectionMode="single" @rowSelect="onRowSelect" scrollable dataKey="path"
        scrollHeight="100%" :metaKeySelection="false">
        <Column :field="(rowData: FileInfo) => rowData.name" :header="t('fileList.filename')" style="width: 250px">
        </Column>
        <Column :field="(rowData: FileInfo) => formatFileSize(rowData.size)" header="Size" style="width: 150px">
        </Column>
        <Column :field="(rowData: FileInfo) => formatFileType(rowData)" header="Type" style="width: 150px"></Column>
        <Column :field="(rowData: FileInfo) => formatDate(rowData.created)" header="Created" style="width: 150px">
        </Column>
        <Column style="width: 10rem">
            <template #body>
                <div class="flex flex-wrap gap-2">
                    <Button type="button" icon="pi pi-search" rounded />
                    <Button type="button" icon="pi pi-pencil" rounded severity="success" />
                </div>
            </template>
        </Column>
        <template #footer>
            <div class="flex justify-start">
                <Button icon="pi pi-refresh" label="Reload" severity="warn" />
            </div>
        </template>
    </DataTable>
</template>

<style lang="css" scoped>
.p-chart {
    display: flex;
}

.p-splitter {
    border-radius: 0;
}
</style>
