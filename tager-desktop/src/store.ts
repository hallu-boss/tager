import { create } from "zustand";
import type { FileItem } from "./types";

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