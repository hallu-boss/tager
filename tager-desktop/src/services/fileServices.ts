// services/fileSystemService.ts
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface FileSystemItem {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  modified: string;
  extension?: string;
}

export class FileSystemService {
  static async selectDirectory(): Promise<string | null> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Wybierz katalog do zarządzania tagami'
      });
      
      return selected as string | null;
    } catch (error) {
      console.error('Błąd podczas wybierania katalogu:', error);
      return null;
    }
  }

  static async readDirectory(path: string): Promise<FileSystemItem[]> {
    try {
      // Wywołanie komendy backendu w Rust
      const files: FileSystemItem[] = await invoke('read_directory_with_metadata', { 
        path 
      });
      
      // Sortowanie: katalogi pierwsze, potem pliki alfabetycznie
      return files.sort((a, b) => {
        if (a.isDir && !b.isDir) return -1;
        if (!a.isDir && b.isDir) return 1;
        return a.name.localeCompare(b.name);
      });
    } catch (error) {
      console.error('Błąd podczas czytania katalogu:', error);
      throw error;
    }
  }

  static async getSupportedExtensions(): Promise<string[]> {
    return [
      'jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg',
      'pdf', 'docx', 'doc', 'txt', 'rtf', 'odt', 'md',
      'mp4', 'avi', 'mov', 'mkv', 'wmv', 'flv', 'webm'
    ];
  }

  static isImageFile(extension?: string): boolean {
    if (!extension) return false;
    const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg'];
    return imageExtensions.includes(extension.toLowerCase());
  }

  static isVideoFile(extension?: string): boolean {
    if (!extension) return false;
    const videoExtensions = ['mp4', 'avi', 'mov', 'mkv', 'wmv', 'flv', 'webm'];
    return videoExtensions.includes(extension.toLowerCase());
  }

  static isDocumentFile(extension?: string): boolean {
    if (!extension) return false;
    const docExtensions = ['pdf', 'docx', 'doc', 'txt', 'rtf', 'odt', 'md'];
    return docExtensions.includes(extension.toLowerCase());
  }

  static getFileType(extension?: string): 'image' | 'document' | 'video' | 'other' {
    if (!extension) return 'other';
    
    if (this.isImageFile(extension)) return 'image';
    if (this.isVideoFile(extension)) return 'video';
    if (this.isDocumentFile(extension)) return 'document';
    
    return 'other';
  }
}