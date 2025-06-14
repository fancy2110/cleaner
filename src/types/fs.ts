/**
 * File Information model
 */
export interface FileInfo {
  name: string;
  path: string;
  isDirectory: boolean;
  size: number;
  created: number;
  modified: number;
  readonly: boolean;
  type?: string;
  children?: FileInfo[];
}

/**
 * Volume Information model
 * for macos is Macintosh HD
 * for windows is C:, D:
 * for linux is /
 **/
export interface Volumn {
  path: string;
  name: string;
  icon: string;
  totalSize: number;
}

export interface FileSystemAPI {
  createDir(path: string): Promise<void>;
  removeDir(path: string, recursive: boolean): Promise<void>;
  removeFile(path: string): Promise<void>;
  copyFile(src: string, dst: string): Promise<void>;
  moveFile(src: string, dst: string): Promise<void>;
  fileInfo(path: string): Promise<FileInfo>;
}

export const FILE_TYPES = {
  DIRECTORY: "directory",
  IMAGE: "image",
  VIDEO: "video",
  AUDIO: "audio",
  DOCUMENT: "document",
  ARCHIVE: "archive",
  CODE: "code",
  OTHER: "other",
} as const;

export type FileType = (typeof FILE_TYPES)[keyof typeof FILE_TYPES];

export const FILE_EXTENSIONS: Record<FileType, string[]> = {
  [FILE_TYPES.DIRECTORY]: [],
  [FILE_TYPES.IMAGE]: ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp"],
  [FILE_TYPES.VIDEO]: ["mp4", "mov", "avi", "mkv", "webm", "flv"],
  [FILE_TYPES.AUDIO]: ["mp3", "wav", "ogg", "flac", "m4a"],
  [FILE_TYPES.DOCUMENT]: [
    "pdf",
    "doc",
    "docx",
    "xls",
    "xlsx",
    "ppt",
    "pptx",
    "txt",
    "rtf",
  ],
  [FILE_TYPES.ARCHIVE]: ["zip", "rar", "7z", "tar", "gz"],
  [FILE_TYPES.CODE]: [
    "js",
    "ts",
    "py",
    "java",
    "c",
    "cpp",
    "rs",
    "go",
    "php",
    "html",
    "css",
  ],
  [FILE_TYPES.OTHER]: [],
};

export function getFileType(path: string): FileType {
  if (path.endsWith("/")) return FILE_TYPES.DIRECTORY;

  const extension = path.split(".").pop()?.toLowerCase();
  if (!extension) return FILE_TYPES.OTHER;

  for (const [type, extensions] of Object.entries(FILE_EXTENSIONS)) {
    if (extensions.includes(extension)) {
      return type as FileType;
    }
  }

  return FILE_TYPES.OTHER;
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";

  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));

  return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
}
