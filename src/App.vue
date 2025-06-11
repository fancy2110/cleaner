<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import PathNavigator from "./components/PathNavigator.vue";
import FileStatistics from "./components/FileStatistics.vue";
import FileList from "./components/FileList.vue";
import SelectionSummary from "./components/SelectionSummary.vue";
import LanguageSelector from "./components/LanguageSelector.vue";
import FolderSizeView from "./components/FolderSizeView.vue";
import { FileSystemService } from "./services/FileSystemService";
import { FileInfo } from "./types/fs";

// Get i18n composition API
const { t } = useI18n();

// 应用状态
const currentPath = ref("");
const isLoading = ref(false);
const files = ref<FileInfo[]>([]);
const selectedFiles = ref<FileInfo[]>([]);
const error = ref("");

// 扫描当前目录
async function scanCurrentDirectory() {
    if (!currentPath.value) return;
    console.log("scanning directory:", currentPath.value);

    try {
        isLoading.value = true;
        error.value = "";

        // 清空选择
        selectedFiles.value = [];

        // 扫描目录
        files.value = await FileSystemService.scanDirectory(currentPath.value);
    } catch (err) {
        console.error("扫描目录出错:", err);
        error.value = t("app.error", { message: err });
        files.value = [];
    } finally {
        isLoading.value = false;
    }
}

// 处理路径变化
function handlePathChange(path: string) {
    console.log("new path change:", path);
    currentPath.value = path;
}

// 处理打开目录
async function handleOpenDirectory(path: string) {
    currentPath.value = path;
}

// 处理文件选择
function handleFileSelect(file: FileInfo) {
    selectedFiles.value.push(file);
}

// 处理取消选择
function handleFileUnselect(filePath: string) {
    selectedFiles.value = selectedFiles.value.filter(
        (file) => file.path !== filePath,
    );
}

// 处理清空选择
function handleClearSelection() {
    selectedFiles.value = [];
}

// 处理删除选中文件
async function handleDeleteSelected() {
    if (selectedFiles.value.length === 0) return;

    try {
        const paths = selectedFiles.value.map((file) => file.path);
        const success = await FileSystemService.deleteMultiplePaths(paths);

        if (success) {
            // 重新扫描当前目录
            await scanCurrentDirectory();
        }
    } catch (err) {
        console.error("删除文件出错:", err);
        error.value = t("app.error", { message: err });
    }
}

// 处理删除单个文件
async function handleDeleteFile(path: string) {
    try {
        const success = await FileSystemService.deletePath(path);

        if (success) {
            // 如果删除的文件是已选中的，也从选中列表中移除
            selectedFiles.value = selectedFiles.value.filter(
                (file) => file.path !== path,
            );

            // 重新扫描当前目录
            await scanCurrentDirectory();
        }
    } catch (err) {
        console.error("删除文件出错:", err);
        error.value = t("app.error", { message: err });
    }
}

// 组件加载时
onMounted(() => {
    // 可以在这里设置默认路径或做其他初始化
    // 例如，设置默认路径为用户主目录
    currentPath.value = "/";
});
</script>

<template>
    <div class="app-container">
        <!-- 顶部标题栏 -->
        <header class="app-header">
            <h1>{{ t("app.title") }}</h1>
            <LanguageSelector class="language-selector" />
        </header>

        <!-- 路径导航 -->
        <PathNavigator
            :current-path="currentPath"
            @path-change="handlePathChange"
        />

        <!-- 主内容区 -->
        <main class="app-content">
            <!-- 错误提示 -->
            <div v-if="error" class="error-message">
                {{ error }}
            </div>

            <!-- 内容布局 -->
            <div class="content-layout">
                <!-- 左侧统计图表 -->
                <div class="stats-section">
                    <FileStatistics :files="files" :is-loading="isLoading" />
                    <FolderSizeView
                        :current-path="currentPath"
                        @path-change="handlePathChange"
                    />
                </div>

                <!-- 右侧文件列表 -->
                <div class="files-section">
                    <FileList
                        :files="files"
                        :is-loading="isLoading"
                        :selected-files="selectedFiles.map((file) => file.path)"
                        @select="handleFileSelect"
                        @unselect="handleFileUnselect"
                        @delete="handleDeleteFile"
                        @open-directory="handleOpenDirectory"
                    />
                </div>
            </div>
        </main>

        <!-- 底部选择统计 -->
        <footer class="app-footer">
            <SelectionSummary
                :selected-files="selectedFiles"
                @clear-selection="handleClearSelection"
                @delete-selected="handleDeleteSelected"
            />
        </footer>
    </div>
</template>

<style scoped>
.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
}

.app-header {
    background-color: #2c3e50;
    color: white;
    padding: 0.5rem 1rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.app-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 500;
}

.language-selector {
    margin-left: auto;
}

.tab-navigation {
    display: flex;
    background-color: #34495e;
}

.tab-button {
    padding: 12px 24px;
    background-color: transparent;
    color: white;
    border: none;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background-color 0.2s;
    border-bottom: 3px solid transparent;
}

.tab-button:hover {
    background-color: rgba(255, 255, 255, 0.1);
}

.tab-button.active {
    background-color: rgba(255, 255, 255, 0.2);
    border-bottom-color: #3498db;
}

.tab-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.root-mode {
    padding: 0;
}

.scanner-mode {
    padding: 0;
    overflow-y: auto;
}

.app-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.error-message {
    background-color: #ffebee;
    color: #d32f2f;
    padding: 8px 16px;
    margin: 8px;
    border-radius: 4px;
    border-left: 4px solid #d32f2f;
}

.content-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.stats-section {
    width: 40%;
    min-width: 300px;
    max-width: 500px;
    border-right: 1px solid #e0e0e0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.files-section {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}

.app-footer {
    border-top: 1px solid #e0e0e0;
    max-height: 200px;
}

/* 响应式布局 */
@media (max-width: 768px) {
    .content-layout {
        flex-direction: column;
    }

    .stats-section {
        width: 100%;
        max-width: none;
        border-right: none;
        border-bottom: 1px solid #e0e0e0;
        max-height: 300px;
    }
}
</style>

<style>
:root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;

    --primary-color: #2196f3;
    --secondary-color: #ff9800;
    --error-color: #f44336;
    --success-color: #4caf50;
    --text-color: #333333;
    --border-color: #e0e0e0;
    --background-color: #f5f5f5;
}

body {
    margin: 0;
    padding: 0;
    min-height: 100vh;
    position: fixed;
    width: 100%;
    height: 100%;
    overflow: hidden;
}

* {
    box-sizing: border-box;
}

button {
    cursor: pointer;
}

/* 美化滚动条 */
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-track {
    background: #f1f1f1;
}

::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
    background: #a8a8a8;
}
</style>
