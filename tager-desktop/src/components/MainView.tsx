import { useState, useEffect } from "react";
import {
  TextField,
  Grid,
  Box,
  Typography,
  Chip,
  Button,
  Stack,
  CircularProgress,
  Alert,
  Paper,
} from "@mui/material";
import { FilterList as FilterListIcon } from "@mui/icons-material";
import FileCard from "./FileCard";
import type { EntryType, FileInfo, FileItem } from "../types";
import { invoke } from "@tauri-apps/api/core";
import { useFileStore } from "../store";

interface MainViewProps {
  directoryPath: string;
}

export default function MainView({ directoryPath }: MainViewProps) {
  const [query, setQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [allTags, setAllTags] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const files = useFileStore(s => s.files);
  const setFiles = useFileStore(s => s.setFiles)
  const select = useFileStore(s => s.select);
  const selectedFile = useFileStore(s => s.selectedIndex)

  console.log(selectedFile)

  async function loadFiles() {
    try {
      setIsLoading(true);
      setError(null);

      console.log("ładuję pliki z be");

      const fileInfos: FileInfo[] = await invoke("read_directory_with_metadata", {
        path: directoryPath
      })

      console.log(fileInfos)

      const fileItems: FileItem[] = fileInfos.map((info, index) => {

        let type: EntryType = "other";
        if (info.is_dir) {
          type = "directory";
        } else if (info.extension) {
          const ext = info.extension.toLowerCase();
          if (["jpg", "jpeg", "png", "gif", "bmp", "webp"].includes(ext)) {
            type = "image";
          } else if (["pdf", "docx", "doc", "txt", "rtf"].includes(ext)) {
            type = "document";
          } else if (["mp4", "avi", "mov", "mkv", "wmv"].includes(ext)) {
            type = "video";
          }
        }

        return {
          id: index,
          name: info.name,
          path: info.path,
          thumbnail: undefined,
          tags: [],
          size: info.size,
          modified: new Date(info.modified * 1000).toISOString(),
          type,
          isDir: info.is_dir,
        }
      })

      setFiles(fileItems)
      setAllTags([]);
    } catch (err) {
      const errMsg = `Błąd przy pobieraniu plików ${err}`
      setError(errMsg)
      console.error(error)
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    loadFiles();
  }, [directoryPath])

  const handleTagClick = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag]
    );
  };

  const filteredFiles = files.filter((file) => {
    const matchesSearch =
      file.name.toLowerCase().includes(query.toLowerCase()) ||
      file.tags.some((tag) => tag.toLowerCase().includes(query.toLowerCase()));

    const matchesTags =
      selectedTags.length === 0 ||
      selectedTags.every((tag) => file.tags.includes(tag));

    return matchesSearch && matchesTags;
  });

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3, width: "100%", height: "100%" }}>
      {/* Nagłówek z informacjami o katalogu */}
      <Paper elevation={0} sx={{ p: 2, bgcolor: "background.default" }}>
        <Typography variant="h6" gutterBottom>
          Katalog: {directoryPath.split("/").pop()}
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Pełna ścieżka: {directoryPath}
        </Typography>
        <Stack direction="row" spacing={1}>
          <Chip
            label={`${files.length} plików`}
            size="small"
            variant="outlined"
          />
          <Chip
            label={`${allTags.length} tagów`}
            size="small"
            variant="outlined"
          />
          <Chip
            label={`${formatFileSize(
              files.reduce((acc, file) => acc + file.size, 0)
            )}`}
            size="small"
            variant="outlined"
          />
        </Stack>
      </Paper>

      {/* Pasek wyszukiwania i filtrów */}
      <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
        <TextField
          label="Wyszukaj pliki lub tagi"
          variant="outlined"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          fullWidth
        />

        <Box
          sx={{
            display: "flex",
            alignItems: "center",
            gap: 2,
            flexWrap: "wrap",
          }}
        >
          <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
            <FilterListIcon fontSize="small" />
            <Typography variant="body2">Filtry tagów:</Typography>
          </Box>

          {allTags.map((tag) => (
            <Chip
              key={tag}
              label={tag}
              clickable
              color={selectedTags.includes(tag) ? "primary" : "default"}
              variant={selectedTags.includes(tag) ? "filled" : "outlined"}
              onClick={() => handleTagClick(tag)}
              size="small"
            />
          ))}

          {selectedTags.length > 0 && (
            <Button
              size="small"
              onClick={() => setSelectedTags([])}
              sx={{ ml: "auto" }}
            >
              Wyczyść filtry
            </Button>
          )}
        </Box>
      </Box>

      {/* Komunikat ładowania */}
      {isLoading && (
        <Box sx={{ display: "flex", justifyContent: "center", p: 4 }}>
          <CircularProgress />
        </Box>
      )}

      {/* Komunikat o braku wyników */}
      {!isLoading && filteredFiles.length === 0 && (
        <Alert severity="info">
          {query || selectedTags.length > 0
            ? "Nie znaleziono plików spełniających kryteria wyszukiwania."
            : "Brak plików w katalogu. Dodaj pliki lub wybierz inny katalog."}
        </Alert>
      )}

      {/* Siatka plików */}
      {!isLoading && filteredFiles.length > 0 && (
        <>
          <Typography variant="body2" color="text.secondary">
            Znaleziono {filteredFiles.length} plików
          </Typography>
          <Grid container spacing={2}>
            {filteredFiles.map((file, index) => (
              <Grid 
                key={file.id}
                size={{ xs: 12, md: "auto"}}
                sx={{
                  minWidth: "250px"
                }}
              >
                <FileCard
                  index={index}
                  name={file.name}
                  path={file.path}
                  tags={file.tags}
                  onCardClick={() => select(index)}
                  size={formatFileSize(file.size)}
                  modified={file.modified}
                  type={file.type}
                />
              </Grid>
            ))}
          </Grid>
        </>
      )}
    </Box>
  );
}
