<script setup>
import { ref } from 'vue';
const breadcrumbHome = ref({ icon: 'pi pi-home', to: '/' });
const breadcrumbItems = ref([{ label: 'Computer' }, { label: 'Notebook' }, { label: 'Accessories' }, { label: 'Backpacks' }, { label: 'Item' }]);
</script>

<template>
    <Breadcrumb :home="breadcrumbHome" :model="breadcrumbItems" />
</template>
