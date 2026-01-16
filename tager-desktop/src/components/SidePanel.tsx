import { 
  Box, 
  IconButton, 
  Typography, 
  Divider, 
  Stack, 
  Chip, 
  Tooltip,
  Badge,
  TextField,
  Autocomplete,
  Popover,
} from "@mui/material";
import {
  Folder as FolderIcon,
  Movie as MovieIcon,
  InsertDriveFile as InsertDriveFileIcon,
  Tag as TagIcon,
  Storage as StorageIcon,
  Image as ImageIcon,
  Description as DescriptionIcon,
  FolderOpen as FolderOpenIcon,
  Add as AddIcon,
} from '@mui/icons-material';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { useTagerStore } from "../store";
import type { EntryType } from "../types";
import { useState, useRef } from 'react';

interface SidePanelProps {
  directoryPath: string;
  onDirectoryChange: (path: string) => void;
  filesCount: number;
  tagsCount: number;
}

// Funkcja do pobierania ikony wg typu pliku
const getFileTypeIcon = (type: EntryType) => {
  const iconSize = 120
  switch (type) {
    case 'image':
      return <ImageIcon sx={{ fontSize: iconSize }} color="success" />;
    case 'document':
      return <DescriptionIcon sx={{ fontSize: iconSize }} color="info"/>;
    case 'video':
      return <MovieIcon sx={{ fontSize: iconSize }} color="warning" />;
    case 'directory':
      return <FolderOpenIcon />;
    default:
      return <InsertDriveFileIcon />;
  }
};

// Funkcja do formatowania rozmiaru pliku
const formatFileSize = (bytes: number) => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

// Funkcja do formatowania daty
const formatDate = (dateString: string) => {
  const date = new Date(dateString);
  return date.toLocaleDateString('pl-PL', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
};

export default function SidePanel({
  directoryPath,
  onDirectoryChange,
}: SidePanelProps) {
  const { files, selectedFileId, tags, assignTag, removeTag } = useTagerStore();
  const [isAddingTag, setIsAddingTag] = useState(false);
  const [newTagName, setNewTagName] = useState('');
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const addButtonRef = useRef<HTMLDivElement>(null);
  
  const selectedFile = selectedFileId !== null && files.find(e => e.id === selectedFileId);

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

  const handleOpenFile = async (filePath: string) => {
    try {
      await openPath(filePath);
    } catch (err) {
      console.log('open file error: ', err);
    }
  };

  const handleAddTagClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
    setIsAddingTag(true);
  };

  const handleAddTag = () => {
    if (selectedFile && newTagName.trim()) {
      assignTag(selectedFile.rel_path, newTagName.trim());
      setNewTagName('');
      setIsAddingTag(false);
      setAnchorEl(null);
    }
  };

  const handleTagInputKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter') {
      handleAddTag();
    } else if (event.key === 'Escape') {
      setIsAddingTag(false);
      setNewTagName('');
      setAnchorEl(null);
    }
  };

  const handleTagInputClose = () => {
    setIsAddingTag(false);
    setNewTagName('');
    setAnchorEl(null);
  };

  function handleDeleteTag(name: string) {
    if (!selectedFile)
      return
    removeTag(selectedFile.rel_path, name);
  }

  return (
    <Box sx={{ 
      height: '100%', 
      display: 'flex', 
      flexDirection: 'column',
      overflow: 'hidden',
    }}>
      {/* Główna zawartość */}
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        <Box sx={{ p: 2, display: "flex", alignItems: "center", gap: 2 }}>
          <Tooltip title="Zmień katalog">
            <IconButton onClick={handleFolderPicker}>
              <FolderIcon color="primary" />
            </IconButton>
          </Tooltip>
          <Box sx={{ flex: 1, minWidth: 0 }}>
            <Typography variant="subtitle2" color="text.secondary">
              Aktywny katalog
            </Typography>
            <Typography 
              variant="body2" 
              noWrap 
              sx={{ 
                maxWidth: 200,
                textOverflow: 'ellipsis',
                overflow: 'hidden',
              }}
              title={directoryPath}
            >
              {directoryPath}
            </Typography>
          </Box>
        </Box>
        <Divider />
        
        {/* Sekcja szczegółów pliku lub statystyk */}
        <Box sx={{ p: 2 }}>
          {selectedFile ? (
            // Widok szczegółów pliku
            <>
              {/* Miniaturka i podstawowe informacje */}
              <Stack >
                <Box
                  sx={{
                    width: 250,
                    height: 200,
                    borderRadius: 1,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    bgcolor: 'action.selected',
                    flexShrink: 0,
                    overflow: 'hidden',
                    position: 'relative',
                  }}
                >
                  {selectedFile.thumbnail ? (
                    <Box
                      component="img"
                      src={selectedFile.thumbnail}
                      alt={selectedFile.file_name}
                      sx={{
                        width: '100%',
                        height: '100%',
                        objectFit: 'cover',
                      }}
                    />
                  ) : (getFileTypeIcon(selectedFile.type))}
                </Box>
                
                <Box sx={{ flex: 1, minWidth: 0 }}>
                  <Typography 
                    variant="body2" 
                    sx={{ 
                      fontWeight: 'medium',
                      mb: 0.5,
                    }}
                  >
                    {selectedFile.file_name}
                  </Typography>
                  
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, flexWrap: 'wrap' }}>
                    <Typography 
                      variant="caption" 
                      color="text.secondary"
                      sx={{ 
                        display: 'flex',
                        alignItems: 'center',
                        gap: 0.5,
                      }}
                    >
                      <StorageIcon fontSize="inherit" />
                      {formatFileSize(selectedFile.size)}
                    </Typography>
                    
                    {selectedFile.tags.length > 0 && (
                      <Badge
                        badgeContent={selectedFile.tags.length}
                        color="primary"
                        sx={{
                          '& .MuiBadge-badge': {
                            fontSize: '0.6rem',
                            height: 16,
                            minWidth: 16,
                          },
                        }}
                      />
                    )}
                  </Box>
                  
                  <Typography 
                    variant="caption" 
                    color="text.secondary"
                    sx={{ 
                      display: 'block',
                      mt: 0.5,
                      fontSize: '0.75rem',
                    }}
                  >
                    Zmodyfikowano: {formatDate(selectedFile.last_modified)}
                  </Typography>
                </Box>
              </Stack>

              {/* Szczegółowe informacje */}
              <Box sx={{ mb: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                  Szczegóły pliku
                </Typography>
                
                <Stack spacing={1}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">
                      Ścieżka:
                    </Typography>
                    <Typography 
                      variant="body2" 
                      sx={{ 
                        maxWidth: 150,
                        textOverflow: 'ellipsis',
                        overflow: 'hidden',
                        whiteSpace: 'nowrap',
                        fontFamily: 'monospace',
                        fontSize: '0.8rem',
                        '&:hover': {
                          textDecoration: 'underline',
                          cursor: 'pointer',
                        }
                      }}
                      title={`Kliknij, aby otworzyć: ${selectedFile.abs_path}`}
                      onClick={() => handleOpenFile(selectedFile.abs_path)}
                    >
                      {selectedFile.abs_path}
                    </Typography>
                  </Box>
                  
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">
                      Typ:
                    </Typography>
                    <Chip 
                      label={selectedFile.type} 
                      size="small" 
                      sx={{ textTransform: 'capitalize' }}
                    />
                  </Box>
                  
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">
                      ID:
                    </Typography>
                    <Typography variant="body2">
                      #{selectedFile.id}
                    </Typography>
                  </Box>
                </Stack>
              </Box>

              {/* Tagi */}
              <Box sx={{ mb: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                  Tagi ({selectedFile.tags.length})
                </Typography>
                <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, alignItems: 'center' }}>
                  {selectedFile.tags.map((tag, index) => (
                    <Chip 
                      key={index} 
                      label={tag.name} 
                      size="small" 
                      color="primary"
                      variant="outlined"
                      onDelete={() => handleDeleteTag(tag.name)}
                    />
                  ))}
                  
                  <div ref={addButtonRef}>
                    <Chip
                      icon={<AddIcon />}
                      label="Dodaj tag"
                      size="small"
                      variant="outlined"
                      onClick={handleAddTagClick}
                      sx={{ cursor: 'pointer' }}
                    />
                  </div>
                </Box>
              </Box>
            </>
          ) : (
            // Widok domyślny (statystyki katalogu)
            <>
              <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 1 }}>
                Statystyki katalogu
              </Typography>
              <Stack direction="row" spacing={1} sx={{ mb: 3 }}>
                <Chip
                  icon={<InsertDriveFileIcon />}
                  label={`${files.length} files`}
                  variant="outlined"
                  size="small"
                />
                <Chip
                  icon={<TagIcon />}
                  label={`${tags.length} tags`}
                  variant="outlined"
                  size="small"
                />
              </Stack>
              
              <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 1 }}>
                Informacja
              </Typography>
              <Typography 
                variant="body2" 
                color="text.secondary" 
                sx={{ 
                  fontSize: '0.8rem',
                  lineHeight: 1.5,
                }}
              >
                Kliknij na dowolny plik, aby wyświetlić jego szczegóły w tym panelu.
              </Typography>
            </>
          )}
        </Box>
      </Box>

      {/* Popover dla dodawania nowego tagu */}
      <Popover
        open={isAddingTag}
        anchorEl={anchorEl}
        onClose={handleTagInputClose}
        anchorOrigin={{
          vertical: 'center',
          horizontal: 'center',
        }}
        transformOrigin={{
          vertical: 'center',
          horizontal: 'center',
        }}
        PaperProps={{
          sx: {
            p: 1,
            width: 250,
          }
        }}
      >
        <Autocomplete
          freeSolo
          options={ selectedFile ?
            tags.filter(tag => !selectedFile.tags.some(t => t.id === tag.id)).map(t => t.name) :
            tags.map(t => t.name)
          }
          inputValue={newTagName}
          onInputChange={(_, newValue) => {
            setNewTagName(newValue);
          }}
          onKeyDown={handleTagInputKeyDown}
          renderInput={(params) => (
            <TextField
              {...params}
              autoFocus
              size="small"
              fullWidth
              placeholder="Wpisz nazwę tagu"
              variant="outlined"
            />
          )}
          renderOption={(props, option) => (
            <li {...props}>
              {option}
            </li>
          )}
          slotProps={{
            listbox: {
              style: {
                maxHeight: 200,
              }
            }
          }}
        />
      </Popover>
    </Box>
  );
}