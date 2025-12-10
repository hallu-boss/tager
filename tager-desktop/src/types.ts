export interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number;
  extension?: string;
}

export interface FileItem {
  id: number;
  name: string;
  path: string;
  thumbnail?: string;
  tags: string[];
  size: number;
  modified: string;
  type: EntryType;
}

export type EntryType = 'image' | 'document' | 'video' | 'other' | 'directory'

export interface DirectoryInfo {
  path: string;
  totalFiles: number;
  size: number;
  lastScanned: string;
}

// Mock stała ścieżki - później będzie z backendu
export const MOCK_DIRECTORY_PATH = "/home/hallu/Documents";