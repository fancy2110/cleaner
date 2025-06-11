<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import type { PropType } from "vue";
import type { FileTypeStats } from "../types";
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
});

const { t } = useI18n();

// 颜色映射表
const typeColors = {
  image: "#4CAF50",
  video: "#2196F3",
  audio: "#FF9800",
  document: "#9C27B0",
  archive: "#795548",
  code: "#607D8B",
  executable: "#F44336",
  other: "#9E9E9E",
};

// 文件类型映射
const fileExtensionMap = {
  // 图片
  jpg: "image",
  jpeg: "image",
  png: "image",
  gif: "image",
  svg: "image",
  webp: "image",
  bmp: "image",
  // 视频
  mp4: "video",
  avi: "video",
  mov: "video",
  wmv: "video",
  mkv: "video",
  webm: "video",
  // 音频
  mp3: "audio",
  wav: "audio",
  ogg: "audio",
  flac: "audio",
  m4a: "audio",
  // 文档
  pdf: "document",
  doc: "document",
  docx: "document",
  xls: "document",
  xlsx: "document",
  ppt: "document",
  pptx: "document",
  txt: "document",
  rtf: "document",
  odt: "document",
  // 压缩包
  zip: "archive",
  rar: "archive",
  "7z": "archive",
  tar: "archive",
  gz: "archive",
  // 代码
  js: "code",
  ts: "code",
  py: "code",
  java: "code",
  c: "code",
  cpp: "code",
  html: "code",
  css: "code",
  json: "code",
  xml: "code",
  rs: "code",
  go: "code",
  php: "code",
  // 可执行文件
  exe: "executable",
  dll: "executable",
  app: "executable",
  msi: "executable",
  sh: "executable",
  bat: "executable",
};

// 统计数据
const statistics = ref<{
  totalFiles: number;
  totalDirs: number;
  totalSize: number;
  typeStats: FileTypeStats[];
}>({
  totalFiles: 0,
  totalDirs: 0,
  totalSize: 0,
  typeStats: [],
});

// 图表尺寸
const chartSize = ref(300);
const centerRadius = computed(() => chartSize.value * 0.3);
const chartWidth = computed(() => chartSize.value - 40); // 留出边距
const chartHeight = computed(() => chartSize.value - 40);

const isDataAvailable = computed(() => {
  return statistics.value.totalFiles > 0 || statistics.value.totalDirs > 0;
});

// 处理文件数据，计算统计信息
function processFileData() {
  if (!props.files || props.files.length === 0) {
    resetStatistics();
    return;
  }

  // 重置统计数据
  resetStatistics();

  // 临时存储各类型统计
  const typeMap = new Map<string, { count: number; size: number }>();

  // 递归处理文件
  const processFile = (file: FileInfo) => {
    if (file.isDirectory) {
      statistics.value.totalDirs++;
      if (file.children) {
        file.children.forEach(processFile);
      }
    } else {
      statistics.value.totalFiles++;
      statistics.value.totalSize += file.size || 0;

      // 获取文件类型
      const ext = file.name.split(".").pop()?.toLowerCase() || "";
      const type =
        fileExtensionMap[ext as keyof typeof fileExtensionMap] || "other";

      // 更新类型统计
      if (!typeMap.has(type)) {
        typeMap.set(type, { count: 0, size: 0 });
      }
      const typeData = typeMap.get(type)!;
      typeData.count++;
      typeData.size += file.size || 0;
    }
  };

  // 处理所有文件
  props.files.forEach(processFile);

  // 转换为数组格式
  statistics.value.typeStats = Array.from(typeMap.entries()).map(
    ([type, data]) => ({
      type,
      count: data.count,
      size: data.size,
      color: typeColors[type as keyof typeof typeColors] || "#9E9E9E",
    }),
  );

  // 按大小排序
  statistics.value.typeStats.sort((a, b) => b.size - a.size);
}

// 重置统计数据
function resetStatistics() {
  statistics.value = {
    totalFiles: 0,
    totalDirs: 0,
    totalSize: 0,
    typeStats: [],
  };
}

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

// 获取文件类型的本地化名称
function getLocalizedTypeName(type: string): string {
  return t(`fileStatistics.fileTypes.${type}`);
}

// 计算饼图数据
const pieChartData = computed(() => {
  if (!statistics.value.typeStats.length) return [];

  let total = statistics.value.totalSize;
  if (total === 0) total = 1; // 避免除以零

  let startAngle = 0;
  return statistics.value.typeStats.map((stat) => {
    const percentage = stat.size / total;
    const angle = percentage * 360;
    const largeArcFlag = angle > 180 ? 1 : 0;

    // 计算SVG弧形路径
    const endAngle = startAngle + angle;
    const startRad = ((startAngle - 90) * Math.PI) / 180;
    const endRad = ((endAngle - 90) * Math.PI) / 180;

    const x1 = centerRadius.value + (chartWidth.value / 2) * Math.cos(startRad);
    const y1 =
      centerRadius.value + (chartHeight.value / 2) * Math.sin(startRad);
    const x2 = centerRadius.value + (chartWidth.value / 2) * Math.cos(endRad);
    const y2 = centerRadius.value + (chartHeight.value / 2) * Math.sin(endRad);

    // 计算标签位置（在弧的中间点）
    const middleRad = ((startAngle + angle / 2 - 90) * Math.PI) / 180;
    const labelRadius = (chartWidth.value / 2) * 0.7; // 标签位置比半径稍短
    const labelX = centerRadius.value + labelRadius * Math.cos(middleRad);
    const labelY = centerRadius.value + labelRadius * Math.sin(middleRad);

    const path = `M ${centerRadius.value} ${centerRadius.value} L ${x1} ${y1} A ${chartWidth.value / 2} ${chartHeight.value / 2} 0 ${largeArcFlag} 1 ${x2} ${y2} Z`;

    const result = {
      type: stat.type,
      color: stat.color,
      percentage,
      angle,
      path,
      labelX,
      labelY,
      count: stat.count,
      size: stat.size,
      startAngle,
    };

    startAngle += angle;
    return result;
  });
});

// 监听文件列表变化
watch(
  () => props.files,
  () => {
    processFileData();
  },
  { deep: true },
);

// 组件挂载时处理数据
onMounted(() => {
  processFileData();
});
</script>

<template>
  <div class="statistics-container">
    <div class="chart-container">
      <div v-if="isLoading" class="loading-overlay">
        <div class="spinner"></div>
        <span>{{ t("fileStatistics.loading") }}</span>
      </div>

      <div v-else-if="!isDataAvailable" class="empty-data">
        <span>{{ t("fileStatistics.noData") }}</span>
        <p>{{ t("app.pleaseSelect") }}</p>
      </div>

      <template v-else>
        <div class="chart-header">
          <h3>{{ t("fileStatistics.title") }}</h3>
          <div class="stats-summary">
            <div class="stat-item">
              <span class="stat-label">{{ t("fileStatistics.files") }}:</span>
              <span class="stat-value">{{ statistics.totalFiles }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t("fileStatistics.folders") }}:</span>
              <span class="stat-value">{{ statistics.totalDirs }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label"
                >{{ t("fileStatistics.totalSize") }}:</span
              >
              <span class="stat-value">{{
                formatSize(statistics.totalSize)
              }}</span>
            </div>
          </div>
        </div>

        <!-- 饼图 -->
        <div class="pie-chart">
          <svg :width="chartSize" :height="chartSize" viewBox="0 0 600 600">
            <g :transform="`translate(150, 150)`">
              <!-- 绘制各个扇形 -->
              <path
                v-for="(slice, index) in pieChartData"
                :key="index"
                :d="slice.path"
                :fill="slice.color"
                :stroke="'#fff'"
                stroke-width="1"
                class="pie-slice"
              />

              <!-- 中心白色圆形 -->
              <circle
                :cx="centerRadius"
                :cy="centerRadius"
                :r="chartWidth / 4"
                fill="#ffffff"
                stroke="#e0e0e0"
                stroke-width="1"
              />
            </g>
          </svg>
        </div>

        <!-- 图例 -->
        <div class="chart-legend">
          <div
            v-for="(stat, index) in statistics.typeStats"
            :key="index"
            class="legend-item"
          >
            <div
              class="legend-color"
              :style="{ backgroundColor: stat.color }"
            ></div>
            <div class="legend-text">
              <div class="legend-type">
                {{ getLocalizedTypeName(stat.type) }}
              </div>
              <div class="legend-details">
                <span>{{ stat.count }} {{ t("fileStatistics.files") }}</span>
                <span>{{ formatSize(stat.size) }}</span>
                <span
                  >{{
                    ((stat.size / statistics.totalSize) * 100).toFixed(1)
                  }}%</span
                >
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.statistics-container {
  display: flex;
  flex-direction: column;
  padding: 16px;
  height: 100%;
  overflow-y: auto;
  background-color: #f9f9f9;
  border-right: 1px solid #e0e0e0;
}

.chart-container {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  padding: 16px;
  margin-bottom: 16px;
  min-height: 400px;
}

.chart-header {
  width: 100%;
  margin-bottom: 16px;
}

.chart-header h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  color: #333;
}

.stats-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.stat-item {
  display: flex;
  gap: 4px;
}

.stat-label {
  font-weight: 500;
  color: #555;
}

.stat-value {
  color: #2196f3;
}

.pie-chart {
  margin: 16px 0;
}

.pie-slice {
  transition: opacity 0.3s;
}

.pie-slice:hover {
  opacity: 0.8;
}

.chart-legend {
  width: 100%;
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}

.legend-color {
  width: 16px;
  height: 16px;
  border-radius: 3px;
}

.legend-text {
  display: flex;
  flex-direction: column;
  flex: 1;
}

.legend-type {
  font-weight: 500;
  text-transform: capitalize;
}

.legend-details {
  display: flex;
  gap: 8px;
  font-size: 12px;
  color: #666;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: rgba(255, 255, 255, 0.8);
  border-radius: 8px;
}

.spinner {
  width: 30px;
  height: 30px;
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

.empty-data {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #999;
}

.empty-data span {
  font-size: 18px;
  margin-bottom: 8px;
}

.empty-data p {
  margin: 8px 0 0 0;
  font-size: 14px;
}
</style>
