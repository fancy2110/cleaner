<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { PropType } from "vue";
import { FileInfo } from "@/types/fs";

const props = defineProps({
  files: {
    type: Array as PropType<FileInfo[]>,
    default: () => [],
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
  selectedFiles: {
    type: Array as PropType<string[]>,
    default: () => [],
  },
});

const emit = defineEmits(["select", "unselect", "delete", "openDirectory"]);
const { t } = useI18n();

// 状态变量
const sortBy = ref<"name" | "size" | "type">("name");
const sortDirection = ref<"asc" | "desc">("asc");
const showContextMenu = ref(false);
const contextMenuPosition = ref({ x: 0, y: 0 });
const contextMenuFile = ref<FileInfo | null>(null);

// 格式化文件大小
function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";

  const units = [
    t("sizes.byte"),
    t("sizes.kilobyte"),
    t("sizes.megabyte"),
    t("sizes.gigabyte"),
    t("sizes.terabyte"),
  ];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));

  return parseFloat((bytes / Math.pow(1024, i)).toFixed(2)) + " " + units[i];
}

// 获取文件图标
function getFileIcon(file: FileInfo): string {
  if (file.isDirectory) return "📁";

  const ext = file.name.split(".").pop()?.toLowerCase() || "";

  switch (ext) {
    case "pdf":
      return "📄";
    case "jpg":
    case "jpeg":
    case "png":
    case "gif":
    case "webp":
    case "svg":
      return "🖼️";
    case "mp3":
    case "wav":
    case "ogg":
    case "flac":
      return "🎵";
    case "mp4":
    case "mov":
    case "avi":
    case "mkv":
      return "🎬";
    case "doc":
    case "docx":
      return "📝";
    case "xls":
    case "xlsx":
      return "📊";
    case "ppt":
    case "pptx":
      return "📊";
    case "zip":
    case "rar":
    case "7z":
    case "tar":
    case "gz":
      return "🗜️";
    case "js":
    case "ts":
    case "py":
    case "java":
    case "c":
    case "cpp":
    case "rs":
    case "go":
    case "php":
      return "📜";
    case "html":
    case "css":
    case "xml":
      return "🌐";
    case "exe":
    case "dll":
    case "app":
      return "⚙️";
    default:
      return "📄";
  }
}

// 获取文件类型
function getFileType(file: FileInfo): string {
  if (file.isDirectory) return t("fileList.fileTypes.folder");

  const ext = file.name.split(".").pop()?.toLowerCase() || "";

  // 图片文件
  if (["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp"].includes(ext)) {
    return t("fileList.fileTypes.image");
  }

  // 视频文件
  if (["mp4", "mov", "avi", "mkv", "webm", "flv"].includes(ext)) {
    return t("fileList.fileTypes.video");
  }

  // 音频文件
  if (["mp3", "wav", "ogg", "flac", "m4a"].includes(ext)) {
    return t("fileList.fileTypes.audio");
  }

  // 文档文件
  if (
    ["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf"].includes(
      ext,
    )
  ) {
    return t("fileList.fileTypes.document");
  }

  // 压缩文件
  if (["zip", "rar", "7z", "tar", "gz"].includes(ext)) {
    return t("fileList.fileTypes.archive");
  }

  // 代码文件
  if (
    [
      "js",
      "ts",
      "py",
      "java",
      "c",
      "cpp",
      "html",
      "css",
      "rs",
      "go",
      "php",
    ].includes(ext)
  ) {
    return t("fileList.fileTypes.code");
  }

  // 可执行文件
  if (["exe", "dll", "app", "msi", "sh", "bat"].includes(ext)) {
    return t("fileList.fileTypes.executable");
  }

  return t("fileList.fileTypes.unknown", { ext: ext.toUpperCase() });
}

// 文件排序
const sortedFiles = computed(() => {
  if (!props.files || props.files.length === 0) return [];

  const sorted = [...props.files];

  sorted.sort((a, b) => {
    // 始终将文件夹排在前面
    if (a.isDirectory && !b.isDirectory) return -1;
    if (!a.isDirectory && b.isDirectory) return 1;

    if (sortBy.value === "name") {
      const compare = a.name.localeCompare(b.name);
      return sortDirection.value === "asc" ? compare : -compare;
    }

    if (sortBy.value === "size") {
      const sizeA = a.size || 0;
      const sizeB = b.size || 0;
      return sortDirection.value === "asc" ? sizeA - sizeB : sizeB - sizeA;
    }

    if (sortBy.value === "type") {
      const typeA = getFileType(a);
      const typeB = getFileType(b);
      const compare = typeA.localeCompare(typeB);
      return sortDirection.value === "asc" ? compare : -compare;
    }

    return 0;
  });

  return sorted;
});

// 文件是否被选中
function isFileSelected(filePath: string): boolean {
  return props.selectedFiles.includes(filePath);
}

// 切换排序方式
function toggleSort(field: "name" | "size" | "type") {
  if (sortBy.value === field) {
    // 如果已经按照该字段排序，则切换排序方向
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    // 否则，更改排序字段并默认为升序
    sortBy.value = field;
    sortDirection.value = "asc";
  }
}

// 选择文件
function toggleFileSelection(file: FileInfo) {
  if (file.isDirectory) {
    emit("openDirectory", file.path);
    return;
  }

  if (isFileSelected(file.path)) {
    emit("unselect", file.path);
  } else {
    emit("select", file);
  }
}

// 显示上下文菜单
function showFileContextMenu(file: FileInfo, event: MouseEvent) {
  event.preventDefault();
  contextMenuFile.value = file;
  contextMenuPosition.value = {
    x: event.clientX,
    y: event.clientY,
  };
  showContextMenu.value = true;

  // 点击页面其他地方关闭菜单
  window.addEventListener("click", closeContextMenu, { once: true });
}

// 关闭上下文菜单
function closeContextMenu() {
  showContextMenu.value = false;
  contextMenuFile.value = null;
}

// 删除文件
function deleteFile() {
  if (contextMenuFile.value) {
    emit("delete", contextMenuFile.value.path);
    closeContextMenu();
  }
}

// 在窗口关闭时清理事件监听器
watch(
  () => showContextMenu.value,
  (newValue) => {
    if (!newValue) {
      window.removeEventListener("click", closeContextMenu);
    }
  },
);
</script>

<template>
  <div class="file-list-container">
    <!-- 文件列表头部 -->
    <div class="file-list-header">
      <div class="header-cell icon-cell"></div>
      <div class="header-cell name-cell" @click="toggleSort('name')">
        {{ t("fileList.filename") }}
        <span class="sort-indicator" v-if="sortBy === 'name'">
          {{ sortDirection === "asc" ? "↑" : "↓" }}
        </span>
      </div>
      <div class="header-cell type-cell" @click="toggleSort('type')">
        {{ t("fileList.type") }}
        <span class="sort-indicator" v-if="sortBy === 'type'">
          {{ sortDirection === "asc" ? "↑" : "↓" }}
        </span>
      </div>
      <div class="header-cell size-cell" @click="toggleSort('size')">
        {{ t("fileList.size") }}
        <span class="sort-indicator" v-if="sortBy === 'size'">
          {{ sortDirection === "asc" ? "↑" : "↓" }}
        </span>
      </div>
    </div>

    <!-- 加载中提示 -->
    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ t("fileList.loading") }}</span>
    </div>

    <!-- 空文件列表提示 -->
    <div v-else-if="files.length === 0" class="empty-state">
      <span>{{ t("fileList.empty") }}</span>
    </div>

    <!-- 文件列表 -->
    <div v-else class="file-list">
      <div
        v-for="(file, index) in sortedFiles"
        :key="index"
        class="file-item"
        :class="{ selected: isFileSelected(file.path) }"
        @click="toggleFileSelection(file)"
        @contextmenu="showFileContextMenu(file, $event)"
      >
        <div class="file-cell icon-cell">
          <span class="file-icon">{{ getFileIcon(file) }}</span>
        </div>
        <div class="file-cell name-cell">
          <span class="file-name">{{ file.name }}</span>
        </div>
        <div class="file-cell type-cell">
          <span class="file-type">{{ getFileType(file) }}</span>
        </div>
        <div class="file-cell size-cell">
          <span class="file-size">{{
            file.isDirectory ? "-" : formatSize(file.size || 0)
          }}</span>
        </div>
      </div>
    </div>

    <!-- 上下文菜单 -->
    <div
      v-if="showContextMenu"
      class="context-menu"
      :style="{
        top: `${contextMenuPosition.y}px`,
        left: `${contextMenuPosition.x}px`,
      }"
    >
      <div class="menu-item" @click="deleteFile">
        <span class="menu-icon">🗑️</span>
        <span>{{ t("fileList.actions.delete") }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-list-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #fff;
  border-radius: 4px;
  overflow: hidden;
}

.file-list-header {
  display: flex;
  background-color: #f5f5f5;
  border-bottom: 1px solid #e0e0e0;
  padding: 10px 16px;
  font-weight: 500;
}

.header-cell {
  cursor: pointer;
  user-select: none;
}

.header-cell:hover {
  color: #2196f3;
}

.sort-indicator {
  margin-left: 4px;
  font-weight: bold;
}

.file-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.file-item {
  display: flex;
  padding: 8px 16px;
  border-bottom: 1px solid #f5f5f5;
  cursor: pointer;
  transition: background-color 0.2s;
}

.file-item:hover {
  background-color: #f9f9f9;
}

.file-item.selected {
  background-color: #e3f2fd;
}

.file-cell,
.header-cell {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0 8px;
}

.icon-cell {
  width: 40px;
  flex-shrink: 0;
}

.name-cell {
  flex: 3;
  min-width: 200px;
}

.type-cell {
  flex: 1;
  min-width: 100px;
}

.size-cell {
  flex: 1;
  min-width: 80px;
  text-align: right;
}

.file-icon {
  font-size: 20px;
}

.file-name {
  font-weight: 400;
}

.file-type,
.file-size {
  color: #666;
  font-size: 0.9rem;
}

.empty-state,
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  color: #999;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid rgba(0, 0, 0, 0.1);
  border-radius: 50%;
  border-top-color: #2196f3;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.context-menu {
  position: fixed;
  background-color: #fff;
  border-radius: 4px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
  padding: 4px 0;
  min-width: 150px;
  z-index: 1000;
}

.menu-item {
  padding: 8px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.menu-item:hover {
  background-color: #f5f5f5;
}

.menu-icon {
  font-size: 16px;
}
</style>
