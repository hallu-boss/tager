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
import { useTagerStore } from "../store";

interface MainViewProps {
  directoryPath: string;
}

export default function MainView({ directoryPath }: MainViewProps) {
  const [query, setQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [allTags, setAllTags] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const {files, init: tager_init, loading: files_loading, select} = useTagerStore();

  async function loadFiles() {
    try {
      setError(null);

      console.log("ładuję pliki z be");

      try {
        await tager_init(directoryPath);
        console.log(files);
      } catch (err) {
        console.error('Błąd inicjalizacji:', err);
      }

      setAllTags([]);
    } catch (err) {
      const errMsg = `Błąd przy pobieraniu plików ${err}`
      setError(errMsg)
      console.error(error)
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
      file.file_name.toLowerCase().includes(query.toLowerCase()) ||
      file.tags.some((tag) => tag.name.toLowerCase().includes(query.toLowerCase()));

    // const matchesTags =
    //   selectedTags.length === 0 ||
    //   selectedTags.every((tag) => file.tags.includes(tag));

    return matchesSearch;
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
          label="Wyszukaj nazwę pliku"
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
      {files_loading && (
        <Box sx={{ display: "flex", justifyContent: "center", p: 4 }}>
          <CircularProgress />
        </Box>
      )}

      {/* Komunikat o braku wyników */}
      {!files_loading && filteredFiles.length === 0 && (
        <Alert severity="info">
          {query || selectedTags.length > 0
            ? "Nie znaleziono plików spełniających kryteria wyszukiwania."
            : "Brak plików w katalogu. Dodaj pliki lub wybierz inny katalog."}
        </Alert>
      )}

      {/* Siatka plików */}
      {!files_loading && filteredFiles.length > 0 && (
        <>
          <Typography variant="body2" color="text.secondary">
            Znaleziono {filteredFiles.length} plików
          </Typography>
          <Grid container spacing={2}>
            {filteredFiles.map((file) => (
              <Grid
                key={file.id}
                size={{ xs: 12, md: "auto" }}
                sx={{
                  minWidth: "250px"
                }}
              >
                <FileCard
                  id={file.id}
                  name={file.file_name}
                  path={file.abs_path}
                  tags={file.tags.map(t => t.name)}
                  onCardClick={() => select(file.id)}
                  size={formatFileSize(file.size)}
                  modified={file.last_modified}
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

