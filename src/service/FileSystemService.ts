import { Volumn, FileInfo } from '@/types/fs';
import { invoke } from '@tauri-apps/api/core';

// Mock API for development environment
// In a real application, these would be imported from Tauri
const dialog = {
    confirm: async (title: string, message: string): Promise<boolean> => {
        console.log('[Mock] dialog.confirm', { title, message });
        return window.confirm(`${title}: ${message}`);
    }
};

const fs = {
    readDir: async (path: string, options?: { recursive?: boolean }): Promise<FileInfo> => {
        console.log('fs.readDir', { path, options });
        return await invoke('scan_directory', { path: path });
    },

    /**
     * get system volumns
     * */
    getAvailableDrivers: async (): Promise<Volumn[]> => {
        console.log('fs.getAvailableDrivers');
        // 假设 result 是一个数组，这里定义一个通用的类型来明确其元素类型
        type ResultItem = {
            path: string;
            name: string;
            totalSize: number;
            availableSize: number;
        };

        var typedResult: ResultItem[] = [];
        try {
            typedResult = await invoke('get_available_drivers', {});
            console.log('fs.getAvailableDrivers result:', { typedResult });
        } catch (error) {
            console.error('getAvailableDrivers failed:', error);
            throw error;
        }

        return typedResult.map((item: ResultItem) => {
            return {
                path: item.path,
                name: item.name,
                icon: '💻',
                totalSize: item.totalSize,
                availableSize: item.availableSize
            };
        });
    },

    removeFile: async (path: string): Promise<void> => {
        console.log('[Mock] fs.removeFile', { path });
    },

    removeDir: async (path: string, options?: { recursive?: boolean }): Promise<void> => {
        console.log('[Mock] fs.removeDir', { path, options });
    }
};

export class FileSystemService {
    static async getAvailableVolumns(): Promise<Volumn[]> {
        try {
            return fs.getAvailableDrivers();
        } catch (error) {
            console.error('load system volumns failed:', error);
            throw error;
        }
    }

    /**
     * 扫描指定目录
     * @param path 目录路径
     * @param recursive 是否递归扫描子目录
     */
    static async scanDirectory(path: string, recursive: boolean = true): Promise<FileInfo[]> {
        try {
            const dir = await fs.readDir(path, { recursive });
            return this.processEntries(dir);
        } catch (error) {
            console.error('扫描目录出错:', error);
            throw error;
        }
    }

    /**
     * 处理文件入口转换为FileInfo
     */
    private static processEntries(dir: FileInfo): FileInfo[] {
        const result: FileInfo[] = [];
        console.log('entry:', dir);
        for (const entry of dir.children || []) {
            console.log('entry:', entry);
            const fileInfo: FileInfo = {
                name: entry.name || '未命名',
                path: entry.path,
                isDirectory: entry.isDirectory || entry.children !== undefined,
                size: entry.size || 0,
                created: 0,
                modified: 0,
                readonly: false
            };

            // 如果是目录且有子项，递归处理
            // if (fileInfo.isDir && entry.children) {
            //   fileInfo.children = this.processEntries(entry.children);
            // }

            // 为文件设置类型
            if (!fileInfo.isDirectory) {
                fileInfo.type = this.getFileType(fileInfo.name);
            }

            result.push(fileInfo);
        }

        return result;
    }

    /**
     * 根据文件名获取文件类型
     */
    private static getFileType(fileName: string): string {
        const ext = fileName.split('.').pop()?.toLowerCase() || '';

        // 图片文件
        if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) {
            return 'image';
        }

        // 视频文件
        if (['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv'].includes(ext)) {
            return 'video';
        }

        // 音频文件
        if (['mp3', 'wav', 'ogg', 'flac', 'm4a'].includes(ext)) {
            return 'audio';
        }

        // 文档文件
        if (['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'rtf'].includes(ext)) {
            return 'document';
        }

        // 压缩文件
        if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) {
            return 'archive';
        }

        // 代码文件
        if (['js', 'ts', 'py', 'java', 'c', 'cpp', 'html', 'css', 'rs', 'go', 'php'].includes(ext)) {
            return 'code';
        }

        // 可执行文件
        if (['exe', 'dll', 'app', 'msi', 'sh', 'bat'].includes(ext)) {
            return 'executable';
        }

        return 'other';
    }

    /**
     * 删除文件或目录
     * @param path 文件或目录路径
     * @param askConfirmation 是否请求确认
     */
    static async deletePath(path: string, askConfirmation: boolean = true): Promise<boolean> {
        try {
            // 请求确认
            if (askConfirmation) {
                const confirmed = await dialog.confirm('确认删除', `确定要删除此项吗？\n${path}`);
                if (!confirmed) {
                    return false;
                }
            }

            // 判断是文件还是目录
            // 在实际应用中，我们需要先检查路径是文件还是目录
            // 但在模拟环境中，我们根据路径是否包含扩展名来判断
            const hasExtension = path.split('/').pop()?.includes('.');

            if (!hasExtension) {
                await fs.removeDir(path, { recursive: true });
            } else {
                await fs.removeFile(path);
            }

            return true;
        } catch (error) {
            console.error('删除文件/目录出错:', error);
            throw error;
        }
    }

    /**
     * 批量删除文件
     * @param paths 文件路径数组
     */
    static async deleteMultiplePaths(paths: string[]): Promise<boolean> {
        try {
            // 请求确认
            const confirmed = await dialog.confirm('确认批量删除', `确定要删除这 ${paths.length} 个项目吗？`);

            if (!confirmed) {
                return false;
            }

            // 批量删除
            for (const path of paths) {
                // 这里不再单独请求确认
                await this.deletePath(path, false);
            }

            return true;
        } catch (error) {
            console.error('批量删除出错:', error);
            throw error;
        }
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
}
