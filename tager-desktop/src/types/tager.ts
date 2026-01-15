
export type TagEntry = {
  id: number;
  name: string;
};

export type EntryType = 'image' | 'document' | 'video' | 'other' | 'directory'

export type FileEntry = {
  id: number;
  abs_path: string;
  rel_path: string;
  file_name: string;
  size: number;
  thumbnail?: string;
  tags: TagEntry[];
  type: EntryType;
  last_modified: string;
  created: string;
};

export type ManagerStatus = {
  initialized: boolean;
  root_path: string;
  total_files: number;
  total_tags: number;
};
