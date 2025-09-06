<script setup lang="ts">
import { FILE_TYPES, FileInfo, getFileType } from '@/types/fs';

// 定义组件 props，接收 FileInfo 输入
// eslint-disable-next-line no-unused-vars
const props = defineProps<{
    file: FileInfo;
}>();

// 根据文件类型获取对应的图标
const getFileIcon = (file: FileInfo): string => {
    if (file.isDirectory) {
        return 'pi pi-folder';
    }

    const fileType = file.type || getFileType(file.path);

    switch (fileType) {
        case FILE_TYPES.IMAGE:
            return 'pi pi-image';
        case FILE_TYPES.VIDEO:
            return 'pi pi-video';
        case FILE_TYPES.AUDIO:
            return 'pi pi-music';
        case FILE_TYPES.DOCUMENT:
            return 'pi pi-file-pdf';
        case FILE_TYPES.ARCHIVE:
            return 'pi pi-file-archive';
        case FILE_TYPES.CODE:
            return 'pi pi-file-code';
        default:
            return 'pi pi-file';
    }
};

// 根据文件类型获取对应的图标颜色
const getFileIconColor = (file: FileInfo): string => {
    if (file.isDirectory) {
        return 'text-blue-500';
    }

    const fileType = file.type || getFileType(file.path);

    switch (fileType) {
        case FILE_TYPES.IMAGE:
            return 'text-green-500';
        case FILE_TYPES.VIDEO:
            return 'text-red-500';
        case FILE_TYPES.AUDIO:
            return 'text-purple-500';
        case FILE_TYPES.DOCUMENT:
            return 'text-orange-500';
        case FILE_TYPES.ARCHIVE:
            return 'text-yellow-500';
        case FILE_TYPES.CODE:
            return 'text-teal-500';
        default:
            return 'text-gray-500';
    }
};
</script>

<template>
    <div class="file-item p-3 flex items-center gap-3 hover:bg-gray-100 rounded-md transition-colors">
        <!-- 文件图标 -->
        <i :class="`${getFileIcon(file)} ${getFileIconColor(file)} text-2xl`"></i>

        <!-- 文件信息 -->
        <div class="file-info flex-1 min-w-0">
            <div class="file-name font-medium truncate" :title="file.name">
                {{ file.name }}
            </div>
        </div>
    </div>
</template>

<style scoped>
.file-item {
    cursor: pointer;
}
</style>
