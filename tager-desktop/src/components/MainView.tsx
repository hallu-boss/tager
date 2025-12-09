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
import type { FileItem } from "../types";
import { readDir } from "@tauri-apps/plugin-fs";

// Symulowane dane z różnych typów plików
const mockFilesData: FileItem[] = [
  {
    id: 1,
    name: "raport_q1.pdf",
    path: "/documents/raport_q1.pdf",
    thumbnail: "https://via.placeholder.com/300x200/FF6B6B/FFFFFF?text=PDF",
    tags: ["finanse", "kwartalny", "ważne"],
    size: 2457600,
    modified: "2024-01-15",
    type: "document",
  },
  {
    id: 2,
    name: "wakacje_2023.jpg",
    path: "/photos/wakacje_2023.jpg",
    thumbnail:
      "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=300&h=200&fit=crop",
    tags: ["wakacje", "rodzina", "lato"],
    size: 4194304,
    modified: "2023-08-20",
    type: "image",
  },
  {
    id: 3,
    name: "prezentacja.mp4",
    path: "/videos/prezentacja.mp4",
    thumbnail: "https://via.placeholder.com/300x200/4ECDC4/FFFFFF?text=VIDEO",
    tags: ["praca", "prezentacja", "ważne"],
    size: 10485760,
    modified: "2024-01-10",
    type: "video",
  },
  {
    id: 4,
    name: "umowa.docx",
    path: "/documents/umowa.docx",
    tags: ["praca", "kontrakt", "prawne"],
    size: 512000,
    modified: "2024-01-12",
    type: "document",
  },
  {
    id: 5,
    name: "logo.png",
    path: "/design/logo.png",
    thumbnail: "https://via.placeholder.com/300x200/45B7D1/FFFFFF?text=LOGO",
    tags: ["design", "branding", "ważne"],
    size: 102400,
    modified: "2024-01-05",
    type: "image",
  },
  {
    id: 6,
    name: "notatki.txt",
    path: "/notes/notatki.txt",
    tags: ["notatki", "tymczasowe"],
    size: 10240,
    modified: "2024-01-14",
    type: "other",
  },
  {
    id: 7,
    name: "zdjecie_profilowe.jpg",
    path: "/photos/zdjecie_profilowe.jpg",
    thumbnail:
      "https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=300&h=200&fit=crop",
    tags: ["profil", "osobiste"],
    size: 2097152,
    modified: "2024-01-08",
    type: "image",
  },
  {
    id: 8,
    name: "instrukcja.pdf",
    path: "/documents/instrukcja.pdf",
    thumbnail: "https://via.placeholder.com/300x200/96CEB4/FFFFFF?text=INSTR",
    tags: ["dokumentacja", "techniczne"],
    size: 1572864,
    modified: "2024-01-03",
    type: "document",
  },
];

interface MainViewProps {
  directoryPath: string;
}

export default function MainView({ directoryPath }: MainViewProps) {
  const [query, setQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [files, setFiles] = useState<FileItem[]>([]);
  const [allTags, setAllTags] = useState<string[]>([]);

  useEffect(() => {
    async function getFiles() {
      const entries = await readDir("/home/hallu/Documents");
      console.log(entries);
    }
    getFiles()
    const timer = setTimeout(() => {
      setFiles(mockFilesData);

      const tags = Array.from(
        new Set(mockFilesData.flatMap((file) => file.tags))
      );
      setAllTags(tags);

      setIsLoading(false);
    }, 500);

    return () => clearTimeout(timer);
  }, []);

  const handleTagClick = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag]
    );
  };

  const handleAddTag = (fileId: number) => {
    // Tutaj będzie logika dodawania tagu do pliku
    console.log("Dodaj tag do pliku:", fileId);
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
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
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
            {filteredFiles.map((file) => (
              <Grid size={5} key={file.id}>
                <FileCard
                  name={file.name}
                  thumbnail={file.thumbnail}
                  tags={file.tags}
                  onAddTag={() => handleAddTag(file.id)}
                  size={formatFileSize(file.size)}
                  modified={file.modified}
                />
              </Grid>
            ))}
          </Grid>
        </>
      )}
    </Box>
  );
}
