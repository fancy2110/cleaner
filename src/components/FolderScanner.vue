<template>
  <div class="folder-scanner">
    <div class="scanner-header">
      <h2>文件夹扫描器</h2>
      <div class="scanner-controls">
        <input
          v-model="scanPath"
          type="text"
          placeholder="输入要扫描的路径，如 /Users 或 C:\"
          class="path-input"
          :disabled="isScanning"
        />
        <button
          @click="startScan"
          :disabled="!scanPath || isScanning"
          class="btn btn-primary"
        >
          <span v-if="!isScanning">开始扫描</span>
          <span v-else>扫描中...</span>
        </button>
        <button
          @click="stopScan"
          :disabled="!isScanning"
          class="btn btn-danger"
        >
          停止扫描
        </button>
        <button
          @click="clearData"
          :disabled="isScanning"
          class="btn btn-secondary"
        >
          清除数据
        </button>
      </div>
    </div>

    <div class="scanner-progress" v-if="progress">
      <div class="progress-info">
        <div class="progress-stat">
          <span class="label">文件数:</span>
          <span class="value">{{ progress.total_files.toLocaleString() }}</span>
        </div>
        <div class="progress-stat">
          <span class="label">文件夹数:</span>
          <span class="value">{{ progress.total_directories.toLocaleString() }}</span>
        </div>
        <div class="progress-stat">
          <span class="label">总大小:</span>
          <span class="value">{{ formatSize(progress.total_size) }}</span>
        </div>
      </div>
      <div class="current-path" v-if="progress.current_path">
        <span class="label">当前扫描:</span>
        <span class="path">{{ progress.current_path }}</span>
      </div>
      <div class="progress-bar">
        <div class="progress-bar-fill" :class="{ active: isScanning }"></div>
      </div>
    </div>

    <div class="scanner-results" v-if="fileTree.length > 0">
      <h3>扫描结果</h3>
      <div class="file-tree">
        <FileTreeNode
          v-for="node in fileTree"
          :key="node.path"
          :node="node"
          :depth="0"
        />
      </div>
    </div>

    <div class="scanner-empty" v-else-if="!isScanning && scanCompleted">
      <p>扫描完成，未找到任何文件</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue';
import { ScannerService, FileStats, ScanProgress } from '@/services/ScannerService';

// 状态
const scanPath = ref('');
const isScanning = ref(false);
const scanCompleted = ref(false);
const progress = ref<ScanProgress>({
  total_files: 0,
  total_directories: 0,
  total_size: 0,
  is_scanning: false
});
const allStats = ref<Map<string, FileStats>>(new Map());

// 计算属性
const fileTree = computed(() => {
  return ScannerService.buildFileTree(allStats.value);
});

// 方法
const formatSize = (bytes: number): string => {
  return ScannerService.formatSize(bytes);
};

const startScan = async () => {
  if (!scanPath.value || isScanning.value) return;

  try {
    isScanning.value = true;
    scanCompleted.value = false;
    allStats.value.clear();

    await ScannerService.startScan(
      scanPath.value,
      // 文件更新回调
      (stats: FileStats) => {
        allStats.value.set(stats.path, stats);
      },
      // 进度更新回调
      (prog: ScanProgress) => {
        progress.value = prog;
      },
      // 扫描完成回调
      async (message: string) => {
        console.log('扫描完成:', message);
        isScanning.value = false;
        scanCompleted.value = true;
        
        // 更新目录大小
        await ScannerService.updateDirectorySizes();
        
        // 重新获取所有统计信息
        const updatedStats = await ScannerService.getAllStats();
        allStats.value = updatedStats;
      }
    );
  } catch (error) {
    console.error('启动扫描失败:', error);
    isScanning.value = false;
  }
};

const stopScan = async () => {
  try {
    await ScannerService.stopScan();
    isScanning.value = false;
  } catch (error) {
    console.error('停止扫描失败:', error);
  }
};

const clearData = async () => {
  try {
    await ScannerService.clearScanData();
    allStats.value.clear();
    progress.value = {
      total_files: 0,
      total_directories: 0,
      total_size: 0,
      is_scanning: false
    };
    scanCompleted.value = false;
  } catch (error) {
    console.error('清除数据失败:', error);
  }
};

// 生命周期
onUnmounted(() => {
  if (isScanning.value) {
    stopScan();
  }
});
</script>

<script lang="ts">
// 文件树节点组件
import { defineComponent, PropType } from 'vue';

export const FileTreeNode = defineComponent({
  name: 'FileTreeNode',
  props: {
    node: {
      type: Object as PropType<FileStats>,
      required: true
    },
    depth: {
      type: Number,
      default: 0
    }
  },
  setup(props) {
    const isExpanded = ref(false);
    
    const toggleExpanded = () => {
      if (props.node.is_directory && props.node.children) {
        isExpanded.value = !isExpanded.value;
      }
    };
    
    const formatSize = (bytes: number): string => {
      return ScannerService.formatSize(bytes);
    };
    
    const getFileName = (path: string): string => {
      return path.split('/').pop() || path.split('\\').pop() || path;
    };
    
    return {
      isExpanded,
      toggleExpanded,
      formatSize,
      getFileName
    };
  },
  template: `
    <div class="tree-node" :style="{ paddingLeft: depth * 20 + 'px' }">
      <div class="node-content" @click="toggleExpanded">
        <span class="node-icon">
          <span v-if="node.is_directory">
            {{ isExpanded ? '📂' : '📁' }}
          </span>
          <span v-else>📄</span>
        </span>
        <span class="node-name">{{ getFileName(node.path) }}</span>
        <span class="node-size">{{ formatSize(node.size) }}</span>
      </div>
      <div v-if="isExpanded && node.children" class="node-children">
        <FileTreeNode
          v-for="child in node.children"
          :key="child.path"
          :node="child"
          :depth="depth + 1"
        />
      </div>
    </div>
  `
});
</script>

<style scoped>
.folder-scanner {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.scanner-header {
  margin-bottom: 20px;
}

.scanner-header h2 {
  margin-bottom: 15px;
  color: #333;
}

.scanner-controls {
  display: flex;
  gap: 10px;
  align-items: center;
}

.path-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.path-input:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background-color: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #0056b3;
}

.btn-danger {
  background-color: #dc3545;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background-color: #c82333;
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #5a6268;
}

.scanner-progress {
  background-color: #f8f9fa;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.progress-info {
  display: flex;
  gap: 30px;
  margin-bottom: 15px;
}

.progress-stat {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-stat .label {
  color: #666;
  font-weight: 500;
}

.progress-stat .value {
  color: #333;
  font-weight: bold;
  font-size: 18px;
}

.current-path {
  margin-bottom: 10px;
  font-size: 14px;
}

.current-path .label {
  color: #666;
  margin-right: 8px;
}

.current-path .path {
  color: #007bff;
  word-break: break-all;
}

.progress-bar {
  width: 100%;
  height: 4px;
  background-color: #e9ecef;
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background-color: #007bff;
  width: 100%;
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s ease;
}

.progress-bar-fill.active {
  animation: progress-animation 2s ease-in-out infinite;
}

@keyframes progress-animation {
  0% {
    transform: scaleX(0);
  }
  50% {
    transform: scaleX(0.5);
  }
  100% {
    transform: scaleX(1);
  }
}

.scanner-results {
  margin-top: 20px;
}

.scanner-results h3 {
  margin-bottom: 15px;
  color: #333;
}

.file-tree {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 15px;
  background-color: #fff;
  max-height: 600px;
  overflow-y: auto;
}

.scanner-empty {
  text-align: center;
  padding: 40px;
  color: #666;
}

/* 文件树节点样式 */
:deep(.tree-node) {
  margin-bottom: 2px;
}

:deep(.node-content) {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.2s;
}

:deep(.node-content:hover) {
  background-color: #f0f0f0;
}

:deep(.node-icon) {
  margin-right: 8px;
  font-size: 16px;
}

:deep(.node-name) {
  flex: 1;
  color: #333;
  font-size: 14px;
}

:deep(.node-size) {
  color: #666;
  font-size: 12px;
  margin-left: 10px;
}

:deep(.node-children) {
  margin-top: 2px;
}
</style>