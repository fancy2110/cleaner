<script setup lang="ts">
import { NodeService } from '@/service/NodeService';
import { CustomerService } from '@/service/CustomerService';
import { useToast } from 'primevue/usetoast';

import { onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ScannerService } from '@/service/ScannerService';
import { FileInfo } from '@/types/fs';

onMounted(() => {
    ScannerService.getCurrentDirFiles().then((data) => (files.value = data));
});

// function onRowSelect(value: any) {
//     console.log('on selected item ', value);
// }

// function onRowUnSelect(value: any) {
//     console.log('on un selected item ', value);
// }

const toast = useToast();
const onRowSelect = (event: any) => {
    toast.add({ severity: 'info', summary: 'Product Selected', detail: 'Name: ' + event.data.name, life: 3000 });
};

const files = ref<FileInfo[] | null>(null);

const { t } = useI18n();
</script>

<template>
    <!-- <div class="h-full overflow-auto"> -->
    <!-- <TreeTable :value="treeTableValue" v-model:selectionKeys="selectedTreeTableValue">
            <Column field="name" header="Name" :expander="true"></Column>
            <Column field="size" header="Size"></Column>
            <Column field="type" header="Type"></Column>
        </TreeTable> -->

    <DataTable :value="files" selectionMode="single" @rowSelect="onRowSelect" scrollable dataKey="id"
        scrollHeight="100%" :metaKeySelection="false">
        <Column field="name" :header="t('fileList.filename')" style="min-width: 200px" frozen class="font-bold">
        </Column>
        <Column field="id" header="Id" style="min-width: 100px"></Column>
        <Column field="name" header="Name" style="min-width: 200px"></Column>
        <Column field="country.name" header="Country" style="min-width: 200px"></Column>
        <Column field="date" header="Date" style="min-width: 200px"></Column>
        <Column field="company" header="Company" style="min-width: 200px"></Column>
        <Column field="status" header="Status" style="min-width: 200px"></Column>
        <Column field="activity" header="Activity" style="min-width: 200px"></Column>
        <Column field="representative.name" header="Representative" style="min-width: 200px"></Column>
    </DataTable>

    <!-- <Splitter class="h-full">
            <SplitterPanel :size="30" :minSize="30">
                <div class="p-3 aspect-1/1 overflow-auto place-content-center">
                    <Chart class="w-full place-content-center" type="pie" :data="pieData" :options="pieOptions">
                    </Chart>
                </div>
            </SplitterPanel>
            <SplitterPanel :size="70" :minSize="60">
                <div class="p-3 overflow-auto">
                    <TreeTable :value="treeTableValue" selectionMode="checkbox"
                        v-model:selectionKeys="selectedTreeTableValue">
                        <Column field="name" header="Name" :expander="true"></Column>
                        <Column field="size" header="Size"></Column>
                        <Column field="type" header="Type"></Column>
                    </TreeTable>
                </div>
            </SplitterPanel>
        </Splitter> -->
    <!-- </div> -->
</template>

<style lang="css" scoped>
.p-chart {
    display: flex;
}

.p-splitter {
    border-radius: 0;
}
</style>
