import { create } from "zustand";
import type { FileItem } from "./types";
import * as api from "./api/tager";
import type { FileEntry, ManagerStatus, TagEntry } from "./types/tager";

type FileStore = {
  selectedIndex: number | null,
  files: FileItem[],
  setFiles: (filesInfos: FileItem[]) => void;
  select: (index: number | null) => void;
  updateFile: (index: number | null, data: Partial<FileItem>) => void;
}

export const useFileStore = create<FileStore>((set) => ({
  selectedIndex: null,
  files: [],
  setFiles: (files) => set({ files }),
  select: (index) => set({ selectedIndex: index }),
  updateFile: (index, data) =>
    set((state) => ({
      files: state.files.map((n, i) =>
        i === index ? { ...n, ...data } : n
      ),
    }))
}))

const ensureSelectionValid = (
  files: FileEntry[],
  selectedFileId: number | null
): number | null => {
  if (selectedFileId === null) return null;
  return files.some(f => f.id === selectedFileId)
    ? selectedFileId
    : null;
};


type TagerState = {
  // dane
  files: FileEntry[];
  tags: TagEntry[];
  status: ManagerStatus | null;

  selectedFileId: number | null;

  // UI / control
  loading: boolean;
  error: string | null;
  rootPath: string | null;

  // akcje
  init: (path: string) => Promise<void>;
  updateFile: (id: number | null, data: Partial<FileEntry>) => void;
  refresh: () => Promise<void>;
  loadTags: () => Promise<void>;
  select: (id: number | null) => void;
  filter: (name?: string, tags?: string[]) => Promise<void>;
  assignTag: (filePath: string, tagName: string) => Promise<void>;
  disconnect: () => Promise<void>;
};

export const useTagerStore = create<TagerState>((set) => ({
  files: [],
  tags: [],
  status: null,

  selectedFileId: null,

  loading: false,
  error: null,
  rootPath: null,

  init: async (path) => {
    set({ loading: true, error: null });

    try {
      const files = await api.initTagerManager(path);
      const status = await api.getManagerStatus();

      set({
        files,
        status,
        rootPath: path,
        selectedFileId: null, // NOWY ROOT = brak selekcji
      });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  updateFile: (id, data) =>
    set((state) => ({
      files: state.files.map((n) =>
        n.id === id ? { ...n, ...data } : n
      ),
  })),

  refresh: async () => {
    set({ loading: true });

    try {
      const files = await api.syncAndGetFiles();
      set((state) => ({
        files,
        selectedFileId: ensureSelectionValid(files, state.selectedFileId),
      }));
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  loadTags: async () => {
    try {
      const tags = await api.getAllTags();
      set({ tags });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  filter: async (name, tags) => {
    set({ loading: true });

    try {
      const files = await api.getFilteredFiles(name, tags);
      set((state) => ({
        files,
        selectedFileId: ensureSelectionValid(files, state.selectedFileId),
      }));
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  assignTag: async (filePath, tagName) => {
    try {
      await api.assignTagToFile(filePath, tagName);

      // szybki refresh
      const files = await api.syncAndGetFiles();
      set({ files });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  select: (id) => set({ selectedFileId: id }),

  disconnect: async () => {
    await api.disconnectManager();
    set({
      files: [],
      tags: [],
      status: null,
      rootPath: null,
    });
  },
}));