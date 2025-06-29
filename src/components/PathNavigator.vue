<script setup lang="ts">
import { FileSystemService } from '@/service/FileSystemService';
import { ScannerService } from '@/service/ScannerService';
import { FileInfo, Volumn } from '@/types/fs';
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';

// Define custom events for use outside the component
const emit = defineEmits<{
    (e: 'pathChange', path: string): void;
    (e: 'startScan', path: string): void;
    (e: 'scanComplete', success: boolean, info: FileInfo | null): void;
}>();

// Platform detection utility
const isMac = navigator.userAgent.toUpperCase().indexOf('MAC') >= 0 || navigator.platform.toUpperCase().indexOf('MAC') >= 0;
const isWindows = navigator.userAgent.toUpperCase().indexOf('WIN') >= 0 || navigator.platform.toUpperCase().indexOf('WIN') >= 0;

interface Props {
    currentPath: string;
}

const props = defineProps<Props>();
const { t } = useI18n();

const pathSegments = ref<{ name: string; fullPath: string }[]>([]);
const availableDrives = ref<Volumn[]>([]);
const selectedDrive = ref<string>('');
const showDriveSelector = ref(false);
const isScanning = ref(false);

// 监听路径变化，并解析为路径段
watch(
    () => props.currentPath,
    (newPath) => {
        if (!newPath) {
            pathSegments.value = [];
            return;
        }
        // 分割路径并创建导航路径段
        const segments = newPath.split(/[/\\]/);
        let currentFullPath = '';

        pathSegments.value = segments
            .filter((segment) => segment)
            .map((segment, index) => {
                // 对于Windows路径，需要特殊处理盘符
                if (index === 0 && segment.includes(':')) {
                    currentFullPath = segment + '\\';
                    return {
                        name: segment,
                        fullPath: currentFullPath
                    };
                }

                // 为Unix路径添加根目录
                if (index === 0 && !currentFullPath) {
                    currentFullPath = '/' + segment;
                } else {
                    currentFullPath = currentFullPath + (currentFullPath.endsWith('/') || currentFullPath.endsWith('\\') ? '' : '/') + segment;
                }

                return {
                    name: segment,
                    fullPath: currentFullPath
                };
            });

        // Update selected drive based on current path
        updateSelectedDriveFromPath(newPath);
    },
    { immediate: true }
);

// 导航到特定路径段
function navigateToSegment(fullPath: string) {
    emit('pathChange', fullPath);
}

// 获取可用的磁盘/驱动器
async function getAvailableDrives() {
    try {
        // In a real implementation, this would use Tauri's API to get actual drives
        // For example, using the Tauri fs plugin or a custom command

        const drives = await FileSystemService.getAvailableVolumns();
        availableDrives.value = drives;
        // If no drive is selected yet, select the first one
        if (!selectedDrive.value && availableDrives.value.length > 0) {
            selectedDrive.value = availableDrives.value[0].name;
        }
    } catch (error) {
        // Error handling without console logging
        // Fallback to basic drives
        const volumns: Volumn[] = isWindows ? [{ name: 'C:', path: 'C:\\', icon: '💻', totalSize: 0 }] : [{ name: 'Root', path: '/', icon: '🖥️', totalSize: 0 }];
        availableDrives.value = volumns;
    }
}

// 根据路径更新选中的驱动器
function updateSelectedDriveFromPath(path: string) {
    if (!path) return;

    // Find matching drive
    const drive = availableDrives.value.find((drive) => {
        if (isWindows && path.toUpperCase().startsWith(drive.name.toUpperCase())) {
            return true;
        } else if (isMac && drive.name === 'Macintosh HD' && (path === '/' || path.startsWith('/System') || path.startsWith('/Users'))) {
            return true;
        } else if (isMac && path.includes('/Volumes/') && path.includes(drive.name)) {
            return true;
        } else if (!isWindows && !isMac && path.startsWith(drive.path)) {
            return true;
        }
        return false;
    });

    if (drive) {
        selectedDrive.value = drive.name;
    } else {
        // Default fallback
        selectedDrive.value = isWindows ? 'C:' : isMac ? 'Macintosh HD' : 'Root';
    }
}

// 选择驱动器
function selectDrive(drive: { name: string; path: string }) {
    selectedDrive.value = drive.name;
    emit('pathChange', drive.path);
    showDriveSelector.value = false;
}

// 切换驱动器选择器的显示状态
function toggleDriveSelector(event: Event) {
    showDriveSelector.value = !showDriveSelector.value;

    if (showDriveSelector.value) {
        // Use event to get button position
        const button = event.currentTarget as HTMLElement;
        const rect = button.getBoundingClientRect();

        // Set timeout to allow Vue to render the dropdown
        setTimeout(() => {
            const dropdown = document.querySelector('.drive-dropdown');
            if (dropdown && dropdown instanceof HTMLElement) {
                dropdown.style.position = 'fixed';
                dropdown.style.top = `${rect.bottom + 2}px`;
                dropdown.style.left = `${rect.left}px`;
                dropdown.style.zIndex = '99999';
            }
        }, 0);
    }
}

// 点击外部时关闭驱动器选择器
function handleClickOutside(event: Event) {
    if (!event.target) return;

    const target = event.target as Element;
    if (showDriveSelector.value && !target.closest('.drive-selector') && !target.closest('.drive-dropdown')) {
        showDriveSelector.value = false;
    }
}

// 自定义节流函数
function throttle(fn: Function, delay: number) {
    let lastCall = 0;
    return function (...args: any[]) {
        const now = Date.now();
        if (now - lastCall >= delay) {
            lastCall = now;
            return fn(...args);
        }
    };
}

// 开始扫描
const startScan = throttle(function () {
    if (!props.currentPath || isScanning.value) return;
    isScanning.value = true;
    ScannerService.startScan(
        (stats) => {
            console.log('scan update:', stats);
        },
        (progress) => {
            console.log('scan progress:', { progress });
        },
        (message) => {
            console.log('scan progress:', { message });
            ScannerService.getFileStats(props.currentPath).then((info) => {
                console.log('scan complete:', info);
                emit('scanComplete', true, info);
            });
            isScanning.value = false;
        }
    ).then(() => {
        console.log('call finished:');
    });
}, 500);

// 获取切断的路径名（用于显示在面包屑中）
function getTruncatedName(name: string, maxLength = 20): string {
    if (name.length <= maxLength) return name;
    return name.substring(0, maxLength - 3) + '...';
}

// 格式化路径显示
function formatPathForDisplay(segment: string, index: number): string {
    // 如果是第一个段并且是根目录
    if (index === 0) {
        if (segment === '/') return 'Root';
        if (segment.includes(':')) return segment.toUpperCase(); // Windows drives
        if (segment === 'Users') return 'Users';
        if (segment === 'Volumes') return 'Volumes';
        return segment;
    }

    // 特殊目录美化
    if (segment === 'Users' || segment === 'user') return 'Users';
    if (segment === 'Documents') return 'Documents';
    if (segment === 'Downloads') return 'Downloads';
    if (segment === 'Desktop') return 'Desktop';
    if (segment === 'Pictures') return 'Pictures';
    if (segment === 'Music') return 'Music';
    if (segment === 'Videos') return 'Videos';
    if (segment === 'Applications') return 'Apps';
    if (segment === 'Program Files') return 'Programs';

    // 普通路径段
    return segment;
}

// 组件挂载时获取可用驱动器
onMounted(() => {
    getAvailableDrives();

    // Add click outside listener
    document.addEventListener('click', handleClickOutside);

    // Set initial selected drive if we have a currentPath
    if (props.currentPath) {
        updateSelectedDriveFromPath(props.currentPath);
    } else {
        // Set a default selected drive if no current path
        selectedDrive.value = isWindows ? 'C:' : isMac ? 'Macintosh HD' : 'Root';
    }

    // No additional setup needed for dropdown positioning
});

// 组件卸载时移除事件监听器
onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
    <div class="path-navigator">
        <div class="path-display">
            <div v-if="!currentPath" class="empty-path">
                <span>{{ t('pathNavigator.pleaseSelect') }}</span>
            </div>
            <div v-else class="directory-buttons">
                <div class="path-container">
                    <!-- Drive selector dropdown -->
                    <div class="drive-selector">
                        <button class="directory-button drive-button" @click="toggleDriveSelector"
                            :title="t('pathNavigator.selectDrive')" :class="{ 'with-separator': true }">
                            {{ selectedDrive || '...' }}
                        </button>

                        <div v-show="showDriveSelector" class="drive-dropdown">
                            <div v-for="drive in availableDrives" :key="drive.name" class="drive-option"
                                @click="selectDrive(drive)">
                                {{ drive.name }}
                            </div>
                        </div>
                    </div>

                    <!-- Path segments -->
                    <button v-for="(segment, index) in pathSegments" :key="index" class="directory-button" :class="{
                        active: index === pathSegments.length - 1,
                        'with-separator': index < pathSegments.length - 1
                    }" @click="navigateToSegment(segment.fullPath)" :title="segment.fullPath"
                        :disabled="index === pathSegments.length - 1">
                        {{ getTruncatedName(formatPathForDisplay(segment.name, index)) }}
                    </button>
                </div>
            </div>
        </div>

        <div class="action-buttons">
            <button class="control-button scan-button" @click="startScan" :disabled="!currentPath"
                :title="t('pathNavigator.startScan')">
                <span class="icon">
                    <span v-if="isScanning" class="loading-spinner"></span>
                    <span v-else>🔍</span>
                </span>
                <span class="button-text">
                    {{ isScanning ? t('pathNavigator.scanning') : t('pathNavigator.scan') }}
                </span>
            </button>
        </div>
    </div>
</template>

<style scoped>
.path-navigator {
    display: flex;
    align-items: center;
    background-color: var(--bg-color, #f5f5f5);
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    padding: 8px 16px;
    width: 100%;
    min-height: 50px;
    gap: 16px;
}

.drive-selector {
    position: relative;
    z-index: 9999;
    display: inline-block;
    height: 100%;
    vertical-align: top;
}

.drive-dropdown {
    width: 150px;
    max-height: 250px;
    overflow-y: auto;
    background-color: var(--dropdown-bg, white);
    border: 1px solid var(--dropdown-border, #ddd);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    animation: fadeIn 0.2s ease-in-out;
    position: fixed;
    z-index: 99999;
    top: 0;
    left: 0;
}

.drive-button {
    min-width: 80px;
    max-width: 120px;
    padding: 0 16px 0 10px;
    border: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
    background-color: var(--drive-button-bg, #333333);
    color: white;
    cursor: pointer;
    box-shadow: none;
    height: 100%;
    align-self: stretch;
    display: flex;
    align-items: center;
    position: relative;
    border-top-left-radius: 4px;
    border-bottom-left-radius: 4px;
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
    font-size: 13px;
    vertical-align: top;
}

.drive-button:hover {
    background-color: var(--button-hover-bg, #444444);
}

.drive-button.with-separator::after {
    color: rgba(255, 255, 255, 0.7);
    z-index: 5;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(-10px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.drive-option {
    padding: 10px 12px;
    cursor: pointer;
    transition: background-color 0.2s;
    border-bottom: 1px solid var(--option-border, #f0f0f0);
    color: var(--text-color, inherit);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 13px;
    user-select: none;
}

.drive-option:last-child {
    border-bottom: none;
}

.drive-option:hover {
    background-color: var(--option-hover-bg, #f5f5f5);
}

.drive-button:hover {
    background-color: var(--button-hover-bg, rgba(0, 0, 0, 0.05));
}

.icon {
    font-size: 16px;
}

.path-display {
    flex: 1;
    overflow: visible;
    min-height: 36px;
    display: flex;
    align-items: stretch;
    background-color: transparent;
    position: relative;
}

.empty-path {
    color: #999;
    font-style: italic;
    padding: 0 8px;
    font-size: 13px;
}

.directory-buttons {
    display: flex;
    max-width: 100%;
    align-items: center;
    padding: 0;
    margin: 0;
    flex-wrap: nowrap;
    scroll-behavior: smooth;
    -webkit-overflow-scrolling: touch;
    position: relative;
    z-index: 1;
}

.path-container {
    display: flex;
    height: 36px;
    white-space: nowrap;
    scrollbar-width: thin;
    position: relative;
    background-color: var(--path-bg, #f5f5f5);
    border-radius: 4px;
    border: 1px solid var(--path-border, #ddd);
    flex: 1;
    align-items: stretch;
    box-shadow: none;
    padding: 0;
    overflow-x: auto;
    overflow-y: hidden;
    font-size: 0;
    /* Remove space between inline elements */
}

.path-container::-webkit-scrollbar {
    height: 4px;
}

.path-container::-webkit-scrollbar-thumb {
    background-color: #ccc;
    border-radius: 2px;
}

.directory-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0 16px 0 10px;
    white-space: nowrap;
    font-weight: 400;
    color: var(--text-color, #333);
    transition: all 0.2s ease;
    overflow: hidden;
    text-overflow: ellipsis;
    background-color: var(--path-bg, #f5f5f5);
    border: none;
    font-size: 13px;
    height: 100%;
    margin: 0;
    position: relative;
    align-self: stretch;
    box-sizing: border-box;
    vertical-align: top;
}

/* Button name styling removed */

.directory-button.active {
    font-weight: 600;
    color: var(--active-text-color, white);
    background-color: var(--active-button-bg, #0066cc);
    pointer-events: none;
    padding-right: 16px;
    padding-left: 10px;
    box-shadow: none;
    margin: 0;
    z-index: 2;
}

.directory-button.active.with-separator::after {
    color: var(--active-text-color, white);
    opacity: 0.5;
}

.directory-button:hover:not(.active):not(.drive-button) {
    background-color: rgba(0, 0, 0, 0.05);
    box-shadow: none;
}

.drive-button:hover {
    background-color: var(--button-hover-bg, #d0d0d0);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
    z-index: 20;
}

.directory-button:active:not(.active) {
    background-color: transparent;
}

.drive-button:active {
    background-color: var(--button-active-bg, #cccccc);
    box-shadow: 0 1px 1px rgba(0, 0, 0, 0.1) inset;
}

.action-buttons {
    display: flex;
    margin-left: 16px;
    flex-shrink: 0;
}

.scan-button {
    background-color: var(--drive-button-bg, #333333);
    color: white;
    border: none;
    padding: 0 16px;
    transition: all 0.2s ease;
    min-width: 100px;
    font-size: 13px;
    font-weight: 500;
    border-radius: 4px;
    box-shadow: none;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-left: 8px;
}

.scan-button:hover:not(:disabled) {
    background-color: var(--button-hover-bg, #444444);
    border-color: transparent;
    box-shadow: none;
    transform: none;
}

.scan-button:active:not(:disabled) {
    background-color: var(--button-active-bg, #222222);
    box-shadow: none;
    transform: none;
}

.button-text {
    margin-left: 6px;
    font-weight: 500;
    transition: all 0.3s ease;
}

.scan-button:disabled {
    background-color: var(--disabled-bg, #555555);
    border-color: transparent;
    color: var(--disabled-text, #999999);
    cursor: not-allowed;
    box-shadow: none;
    opacity: 0.7;
}

.loading-spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-radius: 50%;
    border-top-color: #fff;
    animation: spin 1s ease-in-out infinite;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
