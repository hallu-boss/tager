import { useState } from 'react';
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  List,
  ListItemIcon,
  ListItemText,
  Divider,
  Container,
  Chip,
  ListItemButton,
} from '@mui/material';
import {
  Menu as MenuIcon,
  Folder as FolderIcon,
  Search as SearchIcon,
  Tag as TagIcon,
  Settings as SettingsIcon,
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
      <List>
        <ListItemButton>
          <ListItemIcon>
            <SearchIcon />
          </ListItemIcon>
          <ListItemText primary="Wyszukiwanie" />
        </ListItemButton>
        <ListItemButton>
          <ListItemIcon>
            <TagIcon />
          </ListItemIcon>
          <ListItemText primary="Zarządzanie tagami" />
          <Chip label="24" size="small" />
        </ListItemButton>
        <ListItemButton>
          <ListItemIcon>
            <SettingsIcon />
          </ListItemIcon>
          <ListItemText primary="Ustawienia" />
        </ListItemButton>
      </List>
      <Divider />
      <Box sx={{ p: 2 }}>
        <Typography variant="caption" color="text.secondary">
          Statystyki
        </Typography>
        <Box sx={{ mt: 1 }}>
          <Chip 
            icon={<TagIcon />} 
            label="42 pliki" 
            variant="outlined" 
            size="small" 
            sx={{ mr: 1, mb: 1 }}
          />
          <Chip 
            icon={<TagIcon />} 
            label="15 tagów" 
            variant="outlined" 
            size="small"
          />
        </Box>
      </Box>
    </div>
  );

  return (
    <Box sx={{ display: 'flex' }}>
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
            Tag Manager
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
        <Container maxWidth="xl" sx={{ mt: 2 }}>
          <MainView directoryPath={directoryPath} />
        </Container>
      </Box>
    </Box>
  );
}

export default App;