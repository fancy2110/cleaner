# FolderScannerService 文档

## 概述

`FolderScannerService` 是一个高性能的异步文件夹扫描服务，用于递归扫描文件系统并收集文件统计信息。该服务使用 Rust 编写，通过 Tauri 插件集成到应用中，支持并发扫描和实时进度更新。

## 主要特性

- **异步并发扫描**：支持多个工作线程并发扫描，提高扫描效率
- **实时进度更新**：通过事件系统实时推送扫描进度和文件信息
- **内存高效**：使用队列管理扫描任务，避免内存溢出
- **跨平台支持**：支持 Windows、macOS 和 Linux
- **权限处理**：优雅处理文件系统权限错误
- **目录大小计算**：自动计算目录总大小

## 架构设计

### 后端架构（Rust）

```rust
pub struct FolderScannerService {
    queue: Arc<Mutex<VecDeque<ScanQueueItem>>>,     // 扫描队列
    stats: Arc<Mutex<HashMap<PathBuf, FileStats>>>, // 文件统计信息
    workers: Vec<JoinHandle<()>>,                   // 工作线程
    tx: Option<Sender<FileStats>>,                  // 消息发送器
    concurrency: usize,                             // 并发数
    is_scanning: Arc<Mutex<bool>>,                  // 扫描状态
    progress: Arc<Mutex<ScanProgress>>,             // 扫描进度
}
```

### 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    pub path: PathBuf,
    pub size: u64,
    pub is_directory: bool,
    pub parent: Option<PathBuf>,
    pub modified: Option<u64>,
    pub created: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size: u64,
    pub current_path: Option<PathBuf>,
    pub is_scanning: bool,
}
```

## API 接口

### Tauri 命令

1. **start_folder_scan**
   ```rust
   #[command]
   async fn start_folder_scan(path: String) -> Result<(), String>
   ```
   开始扫描指定路径

2. **stop_folder_scan**
   ```rust
   #[command]
   async fn stop_folder_scan() -> Result<(), String>
   ```
   停止当前扫描

3. **get_folder_stats**
   ```rust
   #[command]
   async fn get_folder_stats(path: String) -> Result<Option<FileStats>, String>
   ```
   获取指定路径的统计信息

4. **get_all_folder_stats**
   ```rust
   #[command]
   async fn get_all_folder_stats() -> Result<HashMap<PathBuf, FileStats>, String>
   ```
   获取所有已扫描文件的统计信息

5. **get_scan_progress**
   ```rust
   #[command]
   async fn get_scan_progress() -> Result<ScanProgress, String>
   ```
   获取当前扫描进度

6. **update_directory_sizes**
   ```rust
   #[command]
   async fn update_directory_sizes() -> Result<(), String>
   ```
   更新所有目录的大小（递归计算子文件大小）

### 事件

1. **folder-scan-update**
   - 触发时机：每当扫描到新文件/目录时
   - 数据类型：`FileStats`

2. **folder-scan-progress**
   - 触发时机：扫描进度更新时
   - 数据类型：`ScanProgress`

3. **folder-scan-complete**
   - 触发时机：扫描完成时
   - 数据类型：`string`

## 前端使用示例

### TypeScript 服务类

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export class FolderScannerService {
  static async startScan(
    path: string,
    onUpdate?: (stats: FileStats) => void,
    onProgress?: (progress: ScanProgress) => void,
    onComplete?: (message: string) => void
  ): Promise<void> {
    // 设置事件监听器
    if (onUpdate) {
      await listen<FileStats>("folder-scan-update", (event) => {
        onUpdate(event.payload);
      });
    }

    // 开始扫描
    await invoke("start_folder_scan", { path });
  }

  static async stopScan(): Promise<void> {
    await invoke("stop_folder_scan");
  }

  static async getProgress(): Promise<ScanProgress> {
    return await invoke<ScanProgress>("get_scan_progress");
  }
}
```

### Vue 组件示例

```vue
<template>
  <div class="folder-scanner">
    <input v-model="scanPath" placeholder="输入扫描路径" />
    <button @click="startScan" :disabled="isScanning">
      {{ isScanning ? '扫描中...' : '开始扫描' }}
    </button>
    
    <div v-if="progress">
      <p>文件数: {{ progress.total_files }}</p>
      <p>文件夹数: {{ progress.total_directories }}</p>
      <p>总大小: {{ formatSize(progress.total_size) }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { FolderScannerService } from '@/services/FolderScannerService';

const scanPath = ref('');
const isScanning = ref(false);
const progress = ref<ScanProgress | null>(null);

const startScan = async () => {
  isScanning.value = true;
  
  await FolderScannerService.startScan(
    scanPath.value,
    (stats) => {
      console.log('新文件:', stats);
    },
    (prog) => {
      progress.value = prog;
    },
    (message) => {
      console.log('扫描完成:', message);
      isScanning.value = false;
    }
  );
};
</script>
```

## 性能优化

1. **并发控制**
   - 默认使用 3 个工作线程
   - 可根据系统资源调整并发数

2. **队列管理**
   - 使用 `VecDeque` 实现高效的任务队列
   - 避免递归调用导致的栈溢出

3. **内存管理**
   - 使用 `Arc<Mutex<>>` 实现安全的共享状态
   - 定期清理已完成的扫描数据

4. **错误处理**
   - 优雅处理权限错误，不中断扫描
   - 记录错误日志便于调试

## 注意事项

1. **权限问题**
   - 在 macOS 上扫描某些系统目录需要额外权限
   - Windows 上需要管理员权限访问某些目录
   - Linux 上需要适当的用户权限

2. **性能影响**
   - 扫描大型目录可能消耗较多 CPU 和内存
   - 建议在后台线程中执行扫描操作

3. **安全考虑**
   - 避免扫描敏感目录
   - 实现适当的访问控制
   - 考虑添加扫描深度限制

## 扩展功能

1. **文件过滤**
   - 支持按文件类型、大小、日期过滤
   - 支持正则表达式匹配

2. **增量扫描**
   - 缓存扫描结果
   - 只扫描变更的文件

3. **导出功能**
   - 支持导出扫描结果为 JSON/CSV
   - 生成目录树报告

4. **可视化**
   - 文件大小树图
   - 目录结构图表
   - 实时扫描动画

## 故障排除

### 常见问题

1. **扫描速度慢**
   - 增加并发工作线程数
   - 排除不必要的目录
   - 使用 SSD 硬盘

2. **内存使用过高**
   - 减少并发数
   - 定期清理扫描结果
   - 分批处理大目录

3. **权限错误过多**
   - 以管理员身份运行
   - 检查文件系统权限
   - 跳过系统保护目录

## 总结

`FolderScannerService` 提供了一个强大而灵活的文件系统扫描解决方案，适用于需要分析文件系统结构、计算目录大小或构建文件索引的应用场景。通过合理的架构设计和性能优化，它能够高效地处理大规模文件系统扫描任务。