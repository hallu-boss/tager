import { useState } from 'react';
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  Divider,
  Chip,
  Stack,
} from '@mui/material';
import {
  Menu as MenuIcon,
  Folder as FolderIcon,
  InsertDriveFile as InsertDriveFileIcon,
  Tag as TagIcon,
  Refresh as RefreshIcon,
} from '@mui/icons-material';
import MainView from './components/MainView';
import { MOCK_DIRECTORY_PATH } from './types';

const drawerWidth = 280;

function App() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [directoryPath] = useState(MOCK_DIRECTORY_PATH);

  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  const drawer = (
    <div>
      <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 2 }}>
        <FolderIcon color="primary" />
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
            label="42 pliki" 
            variant="outlined" 
            size="small" 
          />
          <Chip 
            icon={<TagIcon />} 
            label="15 tagów" 
            variant="outlined" 
            size="small" 
          />
        </Stack>
      </Box>
    </div>
  );

  return (
    <Box sx={{ display: 'flex', height: '100vh', width: '100vw' }}>
      <AppBar
        position="fixed"
        sx={{
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          ml: { sm: `${drawerWidth}px` },
        }}
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            edge="start"
            onClick={handleDrawerToggle}
            sx={{ mr: 2, display: { sm: 'none' } }}
          >
            <MenuIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div" sx={{ flexGrow: 1 }}>
            Tager
          </Typography>
          <IconButton color="inherit">
            <RefreshIcon />
          </IconButton>
        </Toolbar>
      </AppBar>
      <Box
        component="nav"
        sx={{ width: { sm: drawerWidth }, flexShrink: { sm: 0 } }}
      >
        <Drawer
          variant="temporary"
          open={mobileOpen}
          onClose={handleDrawerToggle}
          ModalProps={{
            keepMounted: true, // Better open performance on mobile.
          }}
          sx={{
            display: { xs: 'block', sm: 'none' },
            '& .MuiDrawer-paper': { boxSizing: 'border-box', width: drawerWidth },
          }}
        >
          {drawer}
        </Drawer>
        <Drawer
          variant="permanent"
          sx={{
            display: { xs: 'none', sm: 'block' },
            '& .MuiDrawer-paper': { boxSizing: 'border-box', width: drawerWidth },
          }}
          open
        >
          {drawer}
        </Drawer>
      </Box>
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          p: 3,
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          height: '100vh',
          overflow: 'auto',
        }}
      >
        <Toolbar /> {/* Spacing for AppBar */}
        <Box
          sx={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'auto',
            p: 2,
          }}
        >
          <MainView directoryPath={directoryPath} />
        </Box>
      </Box>
    </Box>
  );
}

export default App;