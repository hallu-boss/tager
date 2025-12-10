import React, { useEffect, useState } from "react";
import {
  Card,
  CardMedia,
  CardContent,
  Typography,
  Chip,
  Box,
  Stack,
  CardActionArea,
  Avatar,
} from "@mui/material";

import PictureAsPdfIcon from "@mui/icons-material/PictureAsPdf";
import ImageIcon from "@mui/icons-material/Image";
import MovieIcon from "@mui/icons-material/Movie";
import DescriptionIcon from "@mui/icons-material/Description";
import InsertDriveFileIcon from "@mui/icons-material/InsertDriveFile";
import type { EntryType } from "../types";
import { invoke } from "@tauri-apps/api/core";

const iconSize = 60;

// ikony zależne od rozszerzenia
const extensionIcons: Record<string, React.ReactNode> = {
  pdf: <PictureAsPdfIcon sx={{ fontSize: iconSize }} color="error" />,
  jpg: <ImageIcon sx={{ fontSize: iconSize }} color="success" />,
  jpeg: <ImageIcon sx={{ fontSize: iconSize }} color="success" />,
  png: <ImageIcon sx={{ fontSize: iconSize }} color="success" />,
  gif: <ImageIcon sx={{ fontSize: iconSize }} color="success" />,
  bmp: <ImageIcon sx={{ fontSize: iconSize }} color="success" />,
  webp: <ImageIcon sx={{ fontSize: iconSize }} color="success" />,
  mp4: <MovieIcon sx={{ fontSize: iconSize }} color="warning" />,
  avi: <MovieIcon sx={{ fontSize: iconSize }} color="warning" />,
  mov: <MovieIcon sx={{ fontSize: iconSize }} color="warning" />,
  mkv: <MovieIcon sx={{ fontSize: iconSize }} color="warning" />,
  docx: <DescriptionIcon sx={{ fontSize: iconSize }} color="info" />,
  doc: <DescriptionIcon sx={{ fontSize: iconSize }} color="info" />,
  txt: <DescriptionIcon sx={{ fontSize: iconSize }} color="action" />,
  default: <InsertDriveFileIcon sx={{ fontSize: iconSize }} color="action" />,
};

interface FileCardProps {
  name: string;
  path: string
  tags: string[];
  size?: string;
  modified?: string;
  onAddTag?: () => void;
  type: EntryType;
}

export default function FileCard({ name, path, tags, type }: FileCardProps) {
  const [loadError, setLoadError] = useState(false);
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isImage, setIsImage] = useState(false);

  const ext = name.split(".").pop()?.toLowerCase() || "";

  useEffect(() => {
    if (type === "image") {
      setIsImage(true);
    }
  }, [type, ext])

   useEffect(() => {
    const loadThumbnail = async () => {
      if (!isImage || loadError) return;
      
      setIsLoading(true);
      try {
        const thumbnail = await invoke<string>("get_thumbnail", {
          path,
          width: 300,
          height: 200,
        });
        
        if (thumbnail && thumbnail.length > 0) {
          console.log(thumbnail);
          setImageUrl(thumbnail);
        } else {
          console.log("thumbnail");
          setLoadError(true);
        }
      } catch (error) {
        console.error("Błąd ładowania miniaturki:", error);
        setLoadError(true);
      } finally {
        setIsLoading(false);
      }
    };

    loadThumbnail();
  }, [path, isImage, loadError]);

  const icon = extensionIcons[ext] ?? extensionIcons.default;

  const shouldShowThumbnail = isImage && imageUrl && !loadError && !isLoading;
  console.log(shouldShowThumbnail)

  return (
    <Card sx={{ borderRadius: 2, boxShadow: 1, '&:hover': { boxShadow: 4 } }}>
      <CardActionArea>
        {/* --- Miniatura lub ikona --- */}
        <Box
          sx={{
            height: 150,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'grey.100',
            overflow: 'hidden',
            position: 'relative',
          }}
        >
          {shouldShowThumbnail ? (
            <CardMedia
              component="img"
              image={imageUrl}
              alt={name}
              onError={() => setLoadError(true)}
              sx={{
                width: '100%',
                height: '100%',
                objectFit: 'cover',
              }}
            />
          ) : (
            <Avatar
              sx={{
                width: iconSize,
                height: iconSize,
                bgcolor: 'transparent',
                color: 'grey.500',
                '& .MuiSvgIcon-root': {
                  fontSize: iconSize
                }
              }}
            >
              {icon}
            </Avatar>
          )}
        </Box>

        {/* --- Nazwa + tagi --- */}
        <CardContent>
          <Typography 
            variant="body2" 
            noWrap
            sx={{ 
              mb: tags.length > 0 ? 1 : 0, 
              textAlign: "center"
            }}
          >
            {name}
          </Typography>

          {tags.length > 0 && (
            <Stack direction="row" spacing={0.5} flexWrap="wrap" useFlexGap>
              {tags.map((tag) => (
                <Chip 
                  key={tag} 
                  label={tag} 
                  color="primary" 
                  variant="outlined" 
                  size="small" 
                  sx={{ mb: 0.5 }}
                />
              ))}
            </Stack>
          )}
        </CardContent>
      </CardActionArea>
    </Card>
  );
}