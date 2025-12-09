import React, { useState } from "react";
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

const iconSize = 60;

// ikony zależne od rozszerzenia
const extensionIcons: Record<string, React.ReactNode> = {
  pdf: <PictureAsPdfIcon sx={{ fontSize: iconSize }} />,
  jpg: <ImageIcon sx={{ fontSize: iconSize }} />,
  jpeg: <ImageIcon sx={{ fontSize: iconSize }} />,
  png: <ImageIcon sx={{ fontSize: iconSize }} />,
  mp4: <MovieIcon sx={{ fontSize: iconSize }} />,
  docx: <DescriptionIcon sx={{ fontSize: iconSize }} />,
  default: <InsertDriveFileIcon sx={{ fontSize: iconSize }} />,
};

interface FileCardProps {
  name: string;
  thumbnail?: string;
  tags: string[];
  size?: string;
  modified?: string;
  onAddTag?: () => void;
}

export default function FileCard({ name, thumbnail, tags }: FileCardProps) {
  const [loadError, setLoadError] = useState(false);

  const ext = name.split(".").pop()?.toLowerCase() || "";
  const icon = extensionIcons[ext] ?? extensionIcons.default;

  const hasThumbnail = thumbnail && !loadError;

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
          {hasThumbnail ? (
            <CardMedia
              component="img"
              image={thumbnail}
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