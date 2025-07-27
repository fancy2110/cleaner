<script setup lang="ts">
import { FileInfo } from '@/types/fs';
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
    currentPath: string;
}>();

const emit = defineEmits<{
    (e: 'pathChange', path: string): void;
}>();

const isScanning = ref(false);
const currentStats = ref<FileInfo | null>(null);

const sortedChildren = computed(() => {
    if (!currentStats.value?.children) return [];
    return [...currentStats.value.children].sort((a, b) => {
        // 文件夹优先
        if (a.isDirectory !== b.isDirectory) {
            return a.isDirectory ? -1 : 1;
        }
        // 大小降序
        return b.size - a.size;
    });
});

function startScan() {
    // TODO: Implement folder scanning
    console.log('Start scan:', props.currentPath);
}

function clearScan() {
    // TODO: Implement clear scan
    console.log('Clear scan');
}

function navigateToPath(path: string) {
    emit('pathChange', path);
}

function getFileName(path: string): string {
    return path.split('/').pop() || path;
}

function formatSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024;
        unitIndex++;
    }

    return `${size.toFixed(2)} ${units[unitIndex]}`;
}

// 监听路径变化，自动开始扫描
watch(
    () => props.currentPath,
    (newPath) => {
        if (newPath) {
            startScan();
        }
    }
);
</script>
<template>
    <div class="folder-size-view">
        <div class="header">
            <h3>{{ t('folderSize.title') }}</h3>
            <div class="controls">
                <button @click="startScan" :disabled="isScanning" class="scan-button">
                    {{ t('folderSize.scanButton') }}
                </button>
                <button @click="clearScan" :disabled="isScanning" class="clear-button">
                    {{ t('folderSize.clearButton') }}
                </button>
            </div>
        </div>

        <div v-if="isScanning" class="scanning-status">
            {{ t('folderSize.scanningStatus') }}
        </div>

        <div v-if="currentStats" class="stats-container">
            <div class="size-info">
                <span class="label">{{ t('folderSize.totalSize') }}:</span>
                <span class="value">{{ formatSize(currentStats.size) }}</span>
            </div>

            <div class="children-list" v-if="currentStats.children">
                <div v-for="child in sortedChildren" :key="child.path" class="child-item"
                    :class="{ 'is-directory': child.isDirectory }">
                    <div class="item-content" @click="child.isDirectory && navigateToPath(child.path)">
                        <span class="item-name">{{ getFileName(child.path) }}</span>
                        <span class="item-size">{{ formatSize(child.size) }}</span>
                    </div>
                </div>
            </div>
        </div>

        <div v-else-if="!isScanning" class="empty-state">
            {{ t('folderSize.emptyState') }}
        </div>
    </div>
</template>

<style scoped>
.folder-size-view {
    padding: 16px;
    height: 100%;
    display: flex;
    flex-direction: column;
}

.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
}

.header h3 {
    margin: 0;
}

.controls {
    display: flex;
    gap: 8px;
}

.scan-button,
.clear-button {
    padding: 6px 12px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 14px;
    transition: background-color 0.2s;
}

.scan-button {
    background-color: var(--primary-color);
    color: white;
}

.scan-button:hover {
    background-color: #1976d2;
}

.clear-button {
    background-color: var(--error-color);
    color: white;
}

.clear-button:hover {
    background-color: #d32f2f;
}

button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.scanning-status {
    color: var(--primary-color);
    margin-bottom: 16px;
    font-style: italic;
}

.stats-container {
    flex: 1;
    overflow: auto;
}

.size-info {
    margin-bottom: 16px;
    padding: 12px;
    background-color: var(--background-color);
    border-radius: 4px;
}

.size-info .label {
    font-weight: 500;
    margin-right: 8px;
}

.children-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.child-item {
    padding: 8px;
    border-radius: 4px;
    background-color: white;
    border: 1px solid var(--border-color);
}

.child-item.is-directory {
    cursor: pointer;
    transition: background-color 0.2s;
}

.child-item.is-directory:hover {
    background-color: var(--background-color);
}

.item-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.item-name {
    font-weight: 500;
}

.item-size {
    color: #666;
}

.empty-state {
    text-align: center;
    color: #666;
    margin-top: 32px;
}
</style>
