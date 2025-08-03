<script setup>
import AppFooter from './AppFooter.vue';
import AppTopbar from './AppTopbar.vue';
</script>

<template>
    <div class="flex flex-col h-full">
        <app-topbar></app-topbar>
        <div class="flex-1 flex-col bg-black overflow-auto">
            <router-view></router-view>
        </div>
        <div class="flex-none w-full items-center">
            <app-footer></app-footer>
        </div>
    </div>
    <Toast />
</template>
