<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { PropType } from "vue";
import { FileInfo } from "@/types/fs";

const props = defineProps({
  selectedFiles: {
    type: Array as PropType<FileInfo[]>,
    default: () => [],
  },
});

const emit = defineEmits(["clearSelection", "deleteSelected"]);
const { t } = useI18n();

// 计算选中文件的总数
const selectedCount = computed(() => props.selectedFiles.length);

// 计算选中文件的总大小
const totalSize = computed(() => {
  return props.selectedFiles.reduce((sum, file) => sum + (file.size || 0), 0);
});

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

// 清空选择
function clearSelection() {
  emit("clearSelection");
}

// 删除选中的文件
function deleteSelected() {
  emit("deleteSelected");
}

// 计算选中的文件类型分布
const fileTypeDistribution = computed(() => {
  const distribution = new Map<string, number>();

  props.selectedFiles.forEach((file) => {
    if (file.isDirectory) {
      const count = distribution.get(t("fileList.fileTypes.folder")) || 0;
      distribution.set(t("fileList.fileTypes.folder"), count + 1);
      return;
    }

    const ext = file.name.split(".").pop()?.toLowerCase() || "";
    let type = t("fileList.fileTypes.other");

    // 图片文件
    if (["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp"].includes(ext)) {
      type = t("fileList.fileTypes.image");
    }
    // 视频文件
    else if (["mp4", "mov", "avi", "mkv", "webm", "flv"].includes(ext)) {
      type = t("fileList.fileTypes.video");
    }
    // 音频文件
    else if (["mp3", "wav", "ogg", "flac", "m4a"].includes(ext)) {
      type = t("fileList.fileTypes.audio");
    }
    // 文档文件
    else if (
      [
        "pdf",
        "doc",
        "docx",
        "xls",
        "xlsx",
        "ppt",
        "pptx",
        "txt",
        "rtf",
      ].includes(ext)
    ) {
      type = t("fileList.fileTypes.document");
    }
    // 压缩文件
    else if (["zip", "rar", "7z", "tar", "gz"].includes(ext)) {
      type = t("fileList.fileTypes.archive");
    }
    // 代码文件
    else if (
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
      type = t("fileList.fileTypes.code");
    }

    const count = distribution.get(type) || 0;
    distribution.set(type, count + 1);
  });

  return Array.from(distribution.entries())
    .map(([type, count]) => ({ type, count }))
    .sort((a, b) => b.count - a.count);
});
</script>

<template>
  <div class="selection-summary">
    <div v-if="selectedCount === 0" class="empty-selection">
      <span>{{ t("selectionSummary.noSelection") }}</span>
    </div>

    <template v-else>
      <div class="summary-header">
        <h3>
          {{ t("selectionSummary.selected", { count: selectedCount }) }}
        </h3>
        <div class="summary-actions">
          <button
            class="action-button delete-button"
            @click="deleteSelected"
            :title="t('selectionSummary.actions.delete')"
          >
            <span class="button-icon">🗑️</span>
            <span>{{ t("selectionSummary.actions.delete") }}</span>
          </button>
          <button
            class="action-button clear-button"
            @click="clearSelection"
            :title="t('selectionSummary.actions.clear')"
          >
            <span class="button-icon">❌</span>
            <span>{{ t("selectionSummary.actions.clear") }}</span>
          </button>
        </div>
      </div>

      <div class="summary-details">
        <div class="total-size">
          <span class="detail-label"
            >{{ t("selectionSummary.totalSize") }}:</span
          >
          <span class="detail-value">{{ formatSize(totalSize) }}</span>
        </div>

        <div class="type-distribution">
          <span class="detail-label"
            >{{ t("selectionSummary.typeDistribution") }}:</span
          >
          <div class="distribution-list">
            <div
              v-for="(item, index) in fileTypeDistribution"
              :key="index"
              class="distribution-item"
            >
              <span class="type-name">{{ item.type }}:</span>
              <span class="type-count">{{ item.count }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="selected-files-list">
        <h4>{{ t("selectionSummary.selectedFiles") }}</h4>
        <div class="files-list-container">
          <div
            v-for="(file, index) in selectedFiles"
            :key="index"
            class="selected-file-item"
          >
            <span class="selected-file-name">{{ file.name }}</span>
            <span class="selected-file-size">{{ formatSize(file.size) }}</span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.selection-summary {
  background-color: #f9f9f9;
  border-top: 1px solid #e0e0e0;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.empty-selection {
  display: flex;
  justify-content: center;
  padding: 8px;
  color: #999;
  font-style: italic;
}

.summary-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.summary-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.summary-actions {
  display: flex;
  gap: 8px;
}

.action-button {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  border: none;
  font-size: 12px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.delete-button {
  background-color: #f44336;
  color: white;
}

.delete-button:hover {
  background-color: #d32f2f;
}

.clear-button {
  background-color: #9e9e9e;
  color: white;
}

.clear-button:hover {
  background-color: #757575;
}

.button-icon {
  font-size: 14px;
}

.summary-details {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 8px;
}

.total-size,
.type-distribution {
  display: flex;
  gap: 4px;
  align-items: center;
}

.detail-label {
  font-weight: 500;
  color: #555;
}

.detail-value {
  color: #2196f3;
}

.distribution-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.distribution-item {
  display: flex;
  gap: 4px;
  background-color: #e0e0e0;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
}

.type-name {
  color: #555;
}

.type-count {
  font-weight: 500;
}

.selected-files-list {
  display: flex;
  flex-direction: column;
  margin-top: 8px;
}

.selected-files-list h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 500;
}

.files-list-container {
  max-height: 120px;
  overflow-y: auto;
  background-color: #fff;
  border-radius: 4px;
  border: 1px solid #e0e0e0;
}

.selected-file-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 10px;
  border-bottom: 1px solid #f0f0f0;
}

.selected-file-item:last-child {
  border-bottom: none;
}

.selected-file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 70%;
}

.selected-file-size {
  color: #666;
  font-size: 12px;
}
</style>
