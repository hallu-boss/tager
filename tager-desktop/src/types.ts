export interface FileItem {
  id: number;
  name: string;
  path: string;
  thumbnail?: string;
  tags: string[];
  size: number;
  modified: string;
  type: 'image' | 'document' | 'video' | 'other';
}

export interface DirectoryInfo {
  path: string;
  totalFiles: number;
  size: number;
  lastScanned: string;
}

// Mock stała ścieżki - później będzie z backendu
export const MOCK_DIRECTORY_PATH = "/home/hallu/Documents";