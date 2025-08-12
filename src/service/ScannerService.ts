import { FileInfo } from '@/types/fs';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface ScanProgress {
    total_files: number;
    total_directories: number;
    total_size: number;
    current_path?: string;
    is_scanning: boolean;
}

export interface ScanResult {
    files: FileInfo[];
    progress: ScanProgress;
}

export type ScanUpdateCallback = (stats: FileInfo) => void;
export type ScanProgressCallback = (progress: ScanProgress) => void;
export type ScanCompleteCallback = (message: string) => void;

export class ScannerService {
    private static updateListeners: UnlistenFn[] = [];
    private static progressListeners: UnlistenFn[] = [];
    private static completeListeners: UnlistenFn[] = [];

    /**
     * 开始扫描文件夹
     * @param path 要扫描的文件夹路径
     * @param onUpdate 文件更新回调
     * @param onProgress 进度更新回调
     * @param onComplete 扫描完成回调
     */
    static async startScan(onUpdate?: ScanUpdateCallback, onProgress?: ScanProgressCallback, onComplete?: ScanCompleteCallback): Promise<boolean> {
        try {
            // 清理之前的监听器
            await this.cleanup();

            // 设置文件更新监听器
            if (onUpdate) {
                const unlistenUpdate = await listen<FileInfo>('folder-scan-update', (event) => {
                    onUpdate(event.payload);
                });
                this.updateListeners.push(unlistenUpdate);
            }

            // 设置进度监听器
            if (onProgress) {
                const unlistenProgress = await listen<ScanProgress>('folder-scan-progress', (event) => {
                    onProgress(event.payload);
                });
                this.progressListeners.push(unlistenProgress);
            }

            // 设置完成监听器
            if (onComplete) {
                const unlistenComplete = await listen<string>('folder-scan-complete', (event) => {
                    onComplete(event.payload);
                });
                this.completeListeners.push(unlistenComplete);
            }

            // 开始扫描
            await invoke('start_scan', { path: '/' });
            return true;
        } catch (error) {
            console.error('启动文件夹扫描失败:', error);
            return false;
        }
    }

    /**
     * 停止当前扫描
     */
    static async stopScan(): Promise<boolean> {
        try {
            console.log('call stop scan from vue');
            await invoke('stop_folder_scan');
            return await this.cleanup();
        } catch (error) {
            console.error('停止扫描失败:', error);
            return false;
        }
    }

    /**
     * 获取扫描进度
     */
    static async getProgress(): Promise<ScanProgress> {
        try {
            return await invoke<ScanProgress>('get_scan_progress');
        } catch (error) {
            console.error('获取扫描进度失败:', error);
            throw error;
        }
    }

    /**
     * 检查是否正在扫描
     */
    static async isScanning(): Promise<boolean> {
        try {
            return await invoke<boolean>('is_scanning');
        } catch (error) {
            console.error('检查扫描状态失败:', error);
            return false;
        }
    }

    /**
     * 获取指定路径的文件统计信息
     */
    static async getFileStats(path: string): Promise<FileInfo | null> {
        try {
            const stats = await invoke<FileInfo | null>('get_folder_stats', { path });
            return stats;
        } catch (error) {
            console.error('获取文件统计信息失败:', error);
            return null;
        }
    }

    /**
     * 获取所有已扫描文件的统计信息
     */
    static async getAllStats(): Promise<Map<string, FileInfo>> {
        try {
            const stats = await invoke<Record<string, FileInfo>>('get_all_folder_stats');
            return new Map(Object.entries(stats));
        } catch (error) {
            console.error('获取所有文件统计信息失败:', error);
            return new Map();
        }
    }

    /**
     * 更新目录大小（在扫描完成后调用）
     */
    static async updateDirectorySizes(): Promise<void> {
        try {
            await invoke('update_directory_sizes');
        } catch (error) {
            console.error('更新目录大小失败:', error);
            throw error;
        }
    }

    /**
     * 清理扫描数据
     */
    static async clearScanData(): Promise<void> {
        try {
            await invoke('clear_folder_scan');
            await this.cleanup();
        } catch (error) {
            console.error('清理扫描数据失败:', error);
            throw error;
        }
    }

    /**
     * 清理事件监听器
     */
    private static async cleanup(): Promise<boolean> {
        // 清理更新监听器
        for (const unlisten of this.updateListeners) {
            unlisten();
        }
        this.updateListeners = [];

        // 清理进度监听器
        for (const unlisten of this.progressListeners) {
            unlisten();
        }
        this.progressListeners = [];

        // 清理完成监听器
        for (const unlisten of this.completeListeners) {
            unlisten();
        }
        this.completeListeners = [];
        return true;
    }

    /**
     * 格式化文件大小
     */
    static formatSize(bytes: number): string {
        if (bytes === 0) return '0 B';

        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));

        return parseFloat((bytes / Math.pow(1024, i)).toFixed(2)) + ' ' + units[i];
    }

    /**
     * 格式化时间戳
     */
    static formatTimestamp(timestamp?: number): string {
        if (!timestamp) return '未知';

        const date = new Date(timestamp * 1000);
        return date.toLocaleString();
    }

    static async getCurrentDirFiles(): Promise<FileInfo[]> {
        return Promise.resolve(this.getData().slice(0, 200));
    }
}

// 扩展 FileStats 接口以支持树结构
declare module './ScannerService';
