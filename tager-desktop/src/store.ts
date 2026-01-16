import { create } from "zustand";
import * as api from "./api/tager";
import type { FileEntry, ManagerStatus, TagEntry } from "./types/tager";

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
  isSyncing: boolean;
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
  removeTag: (filePath: string, tagName: string) => Promise<void>;
  disconnect: () => Promise<void>;
  sync: () => Promise<void>;
};

export const useTagerStore = create<TagerState>((set) => ({
  files: [],
  tags: [],
  status: null,

  selectedFileId: null,

  loading: false,
  error: null,
  rootPath: null,
  isSyncing: false,

  init: async (path) => {
    set({ loading: true, error: null });

    try {
      const files = await api.initTagerManager(path);
      const status = await api.getManagerStatus();
      const tags = await api.getAllTags();

      set({
        files,
        tags,
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

  sync: async () => {
    set({ isSyncing: true });
    try {
      const files = await api.syncAndGetFiles();
      const tags = await api.getAllTags();

      set({ files, tags });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isSyncing: false });
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

  removeTag: async (filePath, tagName) => {
    try {
      await api.removeTagFromFile(filePath, tagName);

      // szybki refresh
      const newFiles = await api.syncAndGetFiles();
      set((state) => {
        const oldFilesMap = new Map(
          state.files.map(f => [f.id, f])
        );

        const mergedFiles = newFiles.map(file => {
          const old = oldFilesMap.get(file.id);
          return {
            ...file,
            thumbnail:
              file.thumbnail != null
                ? file.thumbnail
                : old?.thumbnail,
          };
        });

        return {
          files: mergedFiles,
        }
      }
      );
    } catch (e) {
      set({ error: String(e) });
    }

  },

  assignTag: async (filePath, tagName) => {
    try {
      await api.assignTagToFile(filePath, tagName);

      // szybki refresh
      const newFiles = await api.syncAndGetFiles();
      set((state) => {
        const oldFilesMap = new Map(
          state.files.map(f => [f.id, f])
        );

        const mergedFiles = newFiles.map(file => {
          const old = oldFilesMap.get(file.id);
          return {
            ...file,
            thumbnail:
              file.thumbnail != null
                ? file.thumbnail
                : old?.thumbnail,
          };
        });

        return {
          files: mergedFiles
        }
      }
      );

      const tags = await api.getAllTags();
      set({ tags });
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