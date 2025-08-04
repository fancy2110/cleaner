<script setup lang="ts">
import router from '@/router';

function open(path: string) {
    console.log('scan complete');
    router.push(path);
}
</script>

<template>
    <div className="card flex flex-col gap-2">
        <Button type="button" label="Panel" @click="open('/uikit/panel')" />
        <Button type="button" label="Button" @click="open('/uikit/button')" />
        <Button type="button" label="List" @click="open('/uikit/list')" />
        <Button type="button" label="Formlayout" @click="open('/uikit/formlayout')" />
        <Button type="button" label="input" @click="open('/uikit/input')" />
        <Button type="button" label="table" @click="open('/uikit/table')" />
        <Button type="button" label="tree" @click="open('/uikit/tree')" />
        <Button type="button" label="overlay" @click="open('/uikit/overlay')" />
        <Button type="button" label="media" @click="open('/uikit/media')" />
        <Button type="button" label="message" @click="open('/uikit/message')" />
        <Button type="button" label="file" @click="open('/uikit/file')" />
        <Button type="button" label="menu" @click="open('/uikit/menu')" />
        <Button type="button" label="charts" @click="open('/uikit/charts')" />
        <Button type="button" label="misc" @click="open('/uikit/misc')" />
        <Button type="button" label="timeline" @click="open('/uikit/timeline')" />
        <Button type="button" label="crud" @click="open('/pages/crud')" />
    </div>
</template>
