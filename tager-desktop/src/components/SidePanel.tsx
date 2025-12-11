import { Box, IconButton, Typography, Divider, Stack, Chip } from "@mui/material";
import {
  Folder as FolderIcon,
  InsertDriveFile as InsertDriveFileIcon,
  Tag as TagIcon,
} from '@mui/icons-material';
import { open } from '@tauri-apps/plugin-dialog';

interface SidePanelProps {
  directoryPath: string;
  onDirectoryChange: (path: string) => void;
  filesCount: number;
  tagsCount: number;
}

export default function SidePanel({
  directoryPath,
  onDirectoryChange,
  filesCount,
  tagsCount,
}: SidePanelProps) {

  const handleFolderPicker = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Wybierz katalog bazowy',
      });

      if (selected !== null) {
        const path = Array.isArray(selected) ? selected[0] : selected;
        onDirectoryChange(path);
      }
    } catch (error) {
      console.error('Błąd przy wyborze folderu:', error);
    }
  };

  return (
    <div>
      <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 2 }}>
        <IconButton onClick={handleFolderPicker}>
          <FolderIcon color="primary" />
        </IconButton>
        <Box>
          <Typography variant="subtitle2" color="text.secondary">
            Aktywny katalog
          </Typography>
          <Typography variant="body2" noWrap sx={{ maxWidth: 200 }}>
            {directoryPath}
          </Typography>
        </Box>
      </Box>
      <Divider />
      <Box sx={{ p: 2 }}>
        <Typography variant="caption" color="text.secondary">
          Statystyki
        </Typography>
        <Stack direction="row" spacing={1}>
          <Chip 
            icon={<InsertDriveFileIcon />} 
            label={`${filesCount} pliki`} 
            variant="outlined" 
            size="small" 
          />
          <Chip 
            icon={<TagIcon />} 
            label={`${tagsCount} tagów`}
            variant="outlined" 
            size="small" 
          />
        </Stack>
      </Box>
    </div>
  )
}
