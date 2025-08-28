<!-- eslint-disable vue/no-parsing-error -->
<script setup lang="ts"></script>

<template>
    <div class="items-center w-full gap-2 p-2 place-content-end">
        <Panel toggleable :collapsed="true">
            <template #header>
                <div class="flex items-center gap-2">
                    <Avatar image="https://primefaces.org/cdn/primevue/images/avatar/amyelsner.png" shape="circle" />
                    <span class="font-bold">To Delete: 13.8 GB</span>
                </div>
            </template>

            <template #icons>
                <Button icon="pi pi-trash" severity="secondary" rounded text @click="$router.push('/pages/empty')" />
            </template>

            <template #default>
                <!-- <p class="leading-normal m-0 w-full">
                    <li class="ml-10 mr-10">
                        <div class="flex justify-between">
                            <p>File1</p>
                            <p>13.MB</p>
                        </div>
                    </li>
                </p> -->
            </template>
        </Panel>
    </div>
</template>
