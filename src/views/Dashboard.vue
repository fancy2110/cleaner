<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ScannerService } from '@/service/ScannerService';
import { FileInfo, formatFileSize, formatDate } from '@/types/fs';
import File from '@/views/uikit/File.vue';

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

function addFileToTrash(value: FileInfo) {
    ScannerService.addFileToTrash(value);
    console.log('on selected item ', value);
}

const onRowSelect = (event: any) => {
    let file = event.data.path;
    ScannerService.setCurrentDirectory(file);
};
</script>

<template>
    <div v-if="(files ?? []).length > 0">
        <DataTable :value="files" selectionMode="single" tableStyle="min-width: 50rem" @rowSelect="onRowSelect"
            scrollable dataKey="path" scrollHeight="100%" :metaKeySelection="false">
            <Column :header="t('fileList.filename')">
                <template #body="slotProps">
                    <File :file="slotProps.data" />
                </template>
            </Column>

            <Column :field="(rowData: FileInfo) => formatFileSize(rowData.size)" :header="t('fileList.size')"
                style="width: 150px">
            </Column>

            <Column :field="(rowData: FileInfo) => formatDate(rowData.created)" :header="t('fileList.createTime')"
                style="width: 150px"> </Column>

            <Column style="width: 5rem">
                <template #body="slotProps">
                    <div class="flex flex-wrap gap-2">
                        <Button type="button" icon="pi pi-trash" rounded severity="success" size="small"
                            v-on:click="addFileToTrash(slotProps.data)" />
                    </div>
                </template>
            </Column>
        </DataTable>
    </div>
    <div v-else class="p-4 text-center text-gray-500">这是一个空文件夹</div>
</template>

<style lang="scss" scoped></style>
