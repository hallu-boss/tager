import { invoke } from "@tauri-apps/api/core";
import type { FileEntry, TagEntry, ManagerStatus } from "../types/tager";

export const initTagerManager = (path: string) =>
    invoke<FileEntry[]>("init_tager_manager", { path });

export const getFilteredFiles = (
    nameFilter?: string,
    tagFilters?: string[]
) =>
    invoke<FileEntry[]>("get_filtered_files", {
        nameFilter,
        tagFilters,
    });

export const getFilesWithoutTags = () =>
    invoke<FileEntry[]>("get_files_without_tags");

export const assignTagToFile = (filePath: string, tagName: string) => 
{
    invoke<void>("assign_tag_to_file", {
        filePath,
        tagName,
    });
    console.log("wysłano do be")
}

export const removeTagFromFile = (filePath: string, tagName: string) =>
    invoke<void>("remove_tag_from_file", {
        filePath,
        tagName,
    });

export const getAllTags = () =>
    invoke<TagEntry[]>("get_all_tags");

export const syncAndGetFiles = () =>
    invoke<FileEntry[]>("sync_and_get_files");

export const getManagerStatus = () =>
    invoke<ManagerStatus>("get_manager_status");

export const disconnectManager = () =>
    invoke<void>("disconnect_manager");
