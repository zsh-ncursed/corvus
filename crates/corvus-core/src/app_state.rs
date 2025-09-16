use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::task_manager::{TaskManager, TaskKind};
use humansize::{format_size, BINARY};
use crate::clipboard::{Clipboard, ClipboardMode};
use utils::fs::get_directory_size;
use directories::UserDirs;
use config::Config;
use log;
use crate::search::{SearchEngine, SearchMode};
#[cfg(feature = "mounts")]
use proc_mounts::MountIter;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum FocusBlock {
    #[default]
    Middle,
    Xdg,
    Bookmarks,
    Disks,
}

use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Create,
    Rename,
    Chmod,
    Chown,
    Command,
    Settings,
    Archive,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub owner: String,
    pub permissions: String,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub enum CreateFileType {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewContent {
    File(String),
    Directory(Vec<PreviewEntry>),
    TooLarge(String),
    Error(String),
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: usize,
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub filtered_entries: Vec<DirEntry>, // For search filtering
    pub cursor: usize,
    pub preview_content: Option<PreviewContent>,
    pub preview_scroll: (u16, u16),
    pub selected_entries: HashSet<PathBuf>,
}

impl TabState {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            cursor: 0,
            preview_content: None,
            preview_scroll: (0, 0),
            selected_entries: HashSet::new(),
        }
    }

    pub fn set_current_dir(&mut self, new_path: PathBuf, show_hidden: bool) {
        self.current_dir = new_path;
        self.selected_entries.clear();
        self.update_entries(show_hidden);
    }

    pub fn update_entries(&mut self, show_hidden: bool) {
        self.entries = match fs::read_dir(&self.current_dir) {
            Ok(entries) => entries
                .filter_map(|res| res.ok())
                .filter(|entry| {
                    if show_hidden {
                        true
                    } else {
                        !entry.file_name().to_string_lossy().starts_with('.')
                    }
                })
                .map(|entry| {
                    let path = entry.path();
                    let is_dir = path.is_dir();
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    DirEntry { name, path, is_dir }
                })
                .collect(),
            Err(e) => {
                log::error!("Failed to read directory {:?}: {}", self.current_dir, e);
                vec![]
            }
        };
        self.entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
        self.filtered_entries = self.entries.clone(); // Initially, filtered entries are the same as all entries
        self.cursor = 0;
        self.update_preview();
    }

    pub fn move_cursor_down(&mut self, _show_hidden: bool) {
        let max = self.filtered_entries.len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
        }
        self.update_preview();
    }

    pub fn move_cursor_up(&mut self, _show_hidden: bool) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.update_preview();
    }

    pub fn enter_directory(&mut self, show_hidden: bool) {
        if let Some(entry) = self.filtered_entries.get(self.cursor) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.selected_entries.clear();
                self.update_entries(show_hidden);
            }
        }
        self.update_preview();
    }

    pub fn leave_directory(&mut self, show_hidden: bool) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.selected_entries.clear();
            self.update_entries(show_hidden);
        }
        self.update_preview();
    }

    pub fn update_preview(&mut self) {
        self.preview_scroll = (0, 0);
        let Some(selected_entry) = self.entries.get(self.cursor) else {
            self.preview_content = None;
            return;
        };

        if selected_entry.is_dir {
            self.preview_content = Some(match fs::read_dir(&selected_entry.path) {
                Ok(entries) => {
                    let mut preview_entries = entries
                        .filter_map(|res| res.ok())
                        .map(|entry| {
                            let path = entry.path();
                            PreviewEntry {
                                name: entry.file_name().to_string_lossy().to_string(),
                                is_dir: path.is_dir(),
                            }
                        })
                        .collect::<Vec<_>>();
                    preview_entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
                    PreviewContent::Directory(preview_entries)
                }
                Err(e) => {
                    PreviewContent::Error(format!("Error reading directory:\n{}", e))
                }
            });
        } else {
            const MAX_PREVIEW_SIZE: u64 = 1024 * 1024; // 1MB
            let file_content = match fs::File::open(&selected_entry.path) {
                Ok(mut file) => {
                    let metadata = file.metadata().ok();
                    let file_size = metadata.as_ref().map_or(0, |m| m.len());

                    if file_size > MAX_PREVIEW_SIZE {
                        PreviewContent::TooLarge(format_size(file_size, BINARY))
                    } else {
                        let mut buffer = Vec::new();
                        match file.read_to_end(&mut buffer) {
                            Ok(_) => match String::from_utf8(buffer) {
                                Ok(s) => PreviewContent::File(s),
                                Err(_) => PreviewContent::Binary,
                            },
                            Err(e) => PreviewContent::Error(format!("Error reading file:\n{}", e)),
                        }
                    }
                }
                Err(e) => PreviewContent::Error(format!("Error opening file:\n{}", e)),
            };
            self.preview_content = Some(file_content);
        }
    }

    pub fn update_filtered_entries(&mut self, query: &str) {
        if query.is_empty() {
            self.filtered_entries = self.entries.clone();
        } else {
            self.filtered_entries = self.entries.iter()
                .filter(|entry| {
                    entry.name.to_lowercase().contains(&query.to_lowercase())
                })
                .cloned()
                .collect();
        }
    }

    pub fn get_selected_entry_path(&self) -> Option<PathBuf> {
        self.filtered_entries.get(self.cursor).map(|e| e.path.clone())
    }

    pub fn toggle_selection(&mut self) {
        if let Some(entry) = self.filtered_entries.get(self.cursor) {
            if self.selected_entries.contains(&entry.path) {
                self.selected_entries.remove(&entry.path);
            } else {
                self.selected_entries.insert(entry.path.clone());
            }
        }
    }
    
    /// Выделяет или снимает выделение с текущего элемента без перемещения курсора
    pub fn select_current(&mut self) {
        if let Some(entry) = self.filtered_entries.get(self.cursor) {
            if self.selected_entries.contains(&entry.path) {
                self.selected_entries.remove(&entry.path);
            } else {
                self.selected_entries.insert(entry.path.clone());
            }
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub tabs: Vec<TabState>,
    pub active_tab_index: usize,
    pub show_tabs: bool,
    #[serde(skip)]
    pub task_manager: TaskManager,
    pub clipboard: Clipboard,
    pub show_terminal: bool,
    pub show_hidden_files: bool, // Re-add this
    #[serde(skip)]
    pub focus: FocusBlock,
    pub xdg_dirs: Vec<(String, PathBuf)> ,
    pub xdg_cursor: usize,
    pub bookmarks: Vec<(String, PathBuf)>,
    pub bookmarks_cursor: usize,
    #[cfg(feature = "mounts")]
    #[serde(skip)]
    pub mounts: Vec<proc_mounts::MountInfo>,
    #[cfg(feature = "mounts")]
    pub disks_cursor: usize,
    #[serde(skip)]
    pub config: Config,
    pub show_confirmation: bool,
    pub confirmation_message: String,
    #[serde(skip)]
    pub path_to_delete: Option<PathBuf>,
    #[serde(skip)]
    pub action_to_confirm: Option<ActionToConfirm>,
    #[serde(skip)]
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_dialog_prompt: String,
    pub show_input_dialog: bool,
    #[serde(skip)]
    pub create_file_type: Option<CreateFileType>,
    #[serde(skip)]
    pub path_to_rename: Option<PathBuf>,
    #[serde(skip)]
    pub pending_paste: Option<(Clipboard, PathBuf)> ,
    pub notification: Option<String>,
    #[serde(skip)]
    pub notification_timer: Option<std::time::Instant>,
    pub input_dialog_error: Option<String>,
    #[serde(skip)]
    pub file_info: Option<FileInfo>,
    pub show_info_panel: bool,
    pub chmod_permissions: String,
    pub chown_owner: String,
    
    // Search functionality
    pub show_search_dialog: bool,
    pub search_query: String,
    #[serde(skip)]
    pub search_mode: SearchMode,
    pub search_results: Vec<DirEntry>,
    pub search_cursor: usize,
    pub archive_format: String,
}

#[derive(Debug)]
pub enum ActionToConfirm {
    Delete,
    Paste,
    Unmount,
    RemoveBookmark,
}

impl AppState {
    pub fn new() -> Self {
        let config = config::load_config().unwrap_or_else(|err| {
            log::error!("Failed to load config: {}", err);
            Config::default()
        });

        let mut xdg_dirs = Vec::new();
        if let Some(user_dirs) = UserDirs::new() {
            if let Some(path) = user_dirs.document_dir() { xdg_dirs.push(("Documents".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.download_dir() { xdg_dirs.push(("Downloads".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.picture_dir() { xdg_dirs.push(("Pictures".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.video_dir() { xdg_dirs.push(("Videos".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.audio_dir() { xdg_dirs.push(("Music".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.desktop_dir() { xdg_dirs.push(("Desktop".to_string(), path.to_path_buf())); }
            xdg_dirs.push(("Home".to_string(), user_dirs.home_dir().to_path_buf()));
        }

        let bookmarks = config.bookmarks.clone().into_iter().collect();

        // Попытка загрузить сессию
        let mut app_state = Self {
            tabs: vec![],
            active_tab_index: 0,
            show_tabs: false, // Hidden by default with one tab
            task_manager: TaskManager::new(),
            clipboard: Clipboard::new(),
            show_terminal: false,
            show_hidden_files: false,
            focus: FocusBlock::Middle,
            xdg_dirs,
            xdg_cursor: 0,
            bookmarks,
            bookmarks_cursor: 0,
            #[cfg(feature = "mounts")]
            mounts: Vec::new(), // Initially empty, will be populated by update_mounts
            #[cfg(feature = "mounts")]
            disks_cursor: 0,
            config,
            show_confirmation: false,
            confirmation_message: String::new(),
            path_to_delete: None,
            action_to_confirm: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_dialog_prompt: String::new(),
            show_input_dialog: false,
            create_file_type: None,
            path_to_rename: None,
            pending_paste: None,
            notification: None,
            notification_timer: None,
            input_dialog_error: None,
            file_info: None,
            show_info_panel: false,
            chmod_permissions: String::new(),
            chown_owner: String::new(),
            
            // Search functionality
            show_search_dialog: false,
            search_query: String::new(),
            search_mode: SearchMode::Name,
            search_results: Vec::new(),
            search_cursor: 0,
            archive_format: "zip".to_string(),
        };

        // Попытка загрузить сохраненную сессию
        match crate::session::load_session() {
            Ok(Some(session_state)) => {
                log::info!("Загружена сохраненная сессия");
                session_state.apply_to_app_state(&mut app_state);
            }
            Ok(None) => {
                log::info!("Нет сохраненной сессии, создается новая");
                // Создаем начальную вкладку, если сессия не была загружена
                let show_hidden_files = app_state.show_hidden_files;
                let mut initial_tab = TabState::new(0);
                initial_tab.update_entries(show_hidden_files);
                app_state.tabs = vec![initial_tab];
            }
            Err(e) => {
                log::error!("Ошибка при загрузке сессии: {}", e);
                // Создаем начальную вкладку в случае ошибки
                let show_hidden_files = app_state.show_hidden_files;
                let mut initial_tab = TabState::new(0);
                initial_tab.update_entries(show_hidden_files);
                app_state.tabs = vec![initial_tab];
            }
        }
        
        app_state.update_mounts();

        app_state
    }

    pub fn update_mounts(&mut self) {
        #[cfg(feature = "mounts")]
        {
            const ALLOWED_FS_TYPES: &[&str] = &["davfs", "davfs2", "fuse.sshfs", "sshfs"];
            self.mounts = match MountIter::new() {
                Ok(iter) => iter
                    .filter_map(|res| res.ok())
                    .filter(|mount| {
                        let fstype = mount.fstype.as_str();
                        let dest_path = mount.dest.to_string_lossy();

                        // Keep if it's an allowed FS type or if it's mounted in a common media path
                        ALLOWED_FS_TYPES.contains(&fstype)
                            || dest_path.starts_with("/run/media/")
                            || dest_path.starts_with("/media/")
                    })
                    .collect(),
                Err(e) => {
                    log::error!("Failed to get mounts: {}", e);
                    Vec::new()
                }
            };
            // Ensure cursor is not out of bounds
            if self.disks_cursor >= self.mounts.len() {
                self.disks_cursor = self.mounts.len().saturating_sub(1);
            }
        }
    }

    pub fn toggle_tabs(&mut self) {
        self.show_tabs = !self.show_tabs;
    }

    pub fn get_active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn get_active_tab(&self) -> &TabState {
        &self.tabs[self.active_tab_index]
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusBlock::Xdg => FocusBlock::Bookmarks,
            FocusBlock::Bookmarks => FocusBlock::Disks,
            FocusBlock::Disks => FocusBlock::Middle,
            FocusBlock::Middle => FocusBlock::Xdg,
        };
    }

    pub fn move_left_pane_cursor_down(&mut self) {
        match self.focus {
            FocusBlock::Xdg => {
                let max = self.xdg_dirs.len().saturating_sub(1);
                if self.xdg_cursor < max { self.xdg_cursor += 1; }
            },
            FocusBlock::Bookmarks => {
                let max = self.bookmarks.len().saturating_sub(1);
                if self.bookmarks_cursor < max { self.bookmarks_cursor += 1; }
            },
            FocusBlock::Disks => {
                #[cfg(feature = "mounts")]
                {
                    let max = self.mounts.len().saturating_sub(1);
                    if self.disks_cursor < max { self.disks_cursor += 1; }
                }
            },
            FocusBlock::Middle => {}, // Should not happen
        }
        self.update_middle_pane_from_left_pane_selection();
    }

    pub fn move_left_pane_cursor_up(&mut self) {
        match self.focus {
            FocusBlock::Xdg => {
                if self.xdg_cursor > 0 { self.xdg_cursor -= 1; }
            },
            FocusBlock::Bookmarks => {
                if self.bookmarks_cursor > 0 { self.bookmarks_cursor -= 1; }
            },
            FocusBlock::Disks => {
                #[cfg(feature = "mounts")]
                {
                    if self.disks_cursor > 0 { self.disks_cursor -= 1; }
                }
            },
            FocusBlock::Middle => {}, // Should not happen
        }
        self.update_middle_pane_from_left_pane_selection();
    }

    pub fn update_middle_pane_from_left_pane_selection(&mut self) {
        let path = match self.focus {
            FocusBlock::Xdg => self.xdg_dirs.get(self.xdg_cursor).map(|(_, path)| path.clone()),
            FocusBlock::Bookmarks => self.bookmarks.get(self.bookmarks_cursor).map(|(_, path)| path.clone()),
            FocusBlock::Disks => {
                #[cfg(feature = "mounts")]
                {
                    self.mounts.get(self.disks_cursor).map(|mount| mount.dest.clone())
                }
                #[cfg(not(feature = "mounts"))]
                {
                    None
                }
            },
            FocusBlock::Middle => None, // No-op
        };

        if let Some(path) = path {
            let show_hidden = self.show_hidden_files;
            let active_tab = self.get_active_tab_mut();
            active_tab.set_current_dir(path, show_hidden);
            active_tab.update_preview();
            self.show_info_panel = false;
            
            // Cancel any active search when changing directories
            self.cancel_search();
        }
    }

    pub fn toggle_hidden_files(&mut self) {
        self.show_hidden_files = !self.show_hidden_files;
        for tab in &mut self.tabs {
            tab.update_entries(self.show_hidden_files);
        }
    }

    pub fn yank_selection(&mut self) {
        let active_tab = self.get_active_tab();
        let paths_to_yank = if !active_tab.selected_entries.is_empty() {
            active_tab.selected_entries.iter().cloned().collect()
        } else if let Some(path) = active_tab.get_selected_entry_path() {
            vec![path]
        } else {
            Vec::new()
        };

        if !paths_to_yank.is_empty() {
            self.clipboard.yank(paths_to_yank);
        }
    }

    pub fn cut_selection(&mut self) {
        let active_tab = self.get_active_tab();
        let paths_to_cut = if !active_tab.selected_entries.is_empty() {
            active_tab.selected_entries.iter().cloned().collect()
        } else if let Some(path) = active_tab.get_selected_entry_path() {
            vec![path]
        } else {
            Vec::new()
        };

        if !paths_to_cut.is_empty() {
            self.clipboard.cut(paths_to_cut);
        }
    }

    pub fn paste(&mut self) {
        if self.clipboard.paths.is_empty() {
            return;
        }

        let destination = self.get_active_tab().current_dir.clone();
        let conflict = self.clipboard.paths.iter().any(|src_path| {
            let dest_path = destination.join(src_path.file_name().unwrap());
            dest_path.exists()
        });

        if conflict {
            self.confirmation_message = "A file with the same name already exists. Overwrite? (y/n)".to_string();
            self.show_confirmation = true;
            self.action_to_confirm = Some(ActionToConfirm::Paste);
            self.pending_paste = Some((self.clipboard.clone(), destination));
        } else {
            self.execute_paste(self.clipboard.clone(), destination);
        }
    }

    fn execute_paste(&mut self, clipboard: Clipboard, destination: PathBuf) {
        let mode = clipboard.mode.clone().unwrap();
        for src_path in &clipboard.paths {
            let dest_path = destination.join(src_path.file_name().unwrap());
            let description = format!("{:?} {:?} -> {:?}", mode, src_path.file_name().unwrap(), destination);
            let task_kind = match mode {
                ClipboardMode::Copy => TaskKind::Copy { src: src_path.clone(), dest: dest_path },
                ClipboardMode::Move => TaskKind::Move { src: src_path.clone(), dest: dest_path },
            };
            self.task_manager.add_task(task_kind, description);
        }

        if mode == ClipboardMode::Move {
            self.clipboard.clear();
        }
    }

    pub fn confirm_paste(&mut self) {
        if let Some((clipboard, destination)) = self.pending_paste.take() {
            self.execute_paste(clipboard, destination);
        }
        self.show_confirmation = false;
        self.action_to_confirm = None;
    }

    pub fn cancel_paste(&mut self) {
        self.pending_paste = None;
        self.show_confirmation = false;
        self.action_to_confirm = None;
    }

    pub fn next_tab(&mut self) {
        self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        if self.active_tab_index > 0 {
            self.active_tab_index -= 1;
        } else {
            self.active_tab_index = self.tabs.len() - 1;
        }
    }

    pub fn new_tab(&mut self) {
        if self.tabs.len() >= 10 {
            return;
        }
        log::info!("new_tab called. Current tab count: {}", self.tabs.len());
        let new_id = self.tabs.len();
        let mut new_tab = TabState::new(new_id);
        new_tab.update_entries(self.show_hidden_files);
        self.tabs.push(new_tab);
        self.active_tab_index = new_id;
        self.show_tabs = true; // Show tabs when a new one is created
        log::info!("new_tab finished. New tab count: {}. Active index: {}", self.tabs.len(), self.active_tab_index);
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab_index);
            if self.active_tab_index >= self.tabs.len() {
                self.active_tab_index = self.tabs.len() - 1;
            }
            if self.tabs.len() == 1 {
                self.show_tabs = false; // Hide tabs when only one is left
            }
        }
    }

    pub fn toggle_terminal(&mut self) {
        self.show_terminal = !self.show_terminal;
    }

    pub fn add_bookmark(&mut self) {
        let path = self.get_active_tab().current_dir.clone();
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if !name.is_empty() {
            self.config.bookmarks.insert(name.clone(), path.clone());
            self.bookmarks.push((name, path));
            if let Err(e) = config::save_config(&self.config) {
                log::error!("Failed to save config: {}", e);
            }
        }
    }

    pub fn remove_bookmark(&mut self) {
        if self.focus == FocusBlock::Bookmarks {
            if let Some((name, _path)) = self.bookmarks.get(self.bookmarks_cursor) {
                self.confirmation_message = format!("Remove bookmark '{}'? (y/n)", name);
                self.show_confirmation = true;
                self.action_to_confirm = Some(ActionToConfirm::RemoveBookmark);
            }
        }
    }

    fn confirm_remove_bookmark(&mut self) {
        if let Some((name, _path)) = self.bookmarks.get(self.bookmarks_cursor).cloned() {
            self.bookmarks.remove(self.bookmarks_cursor);
            self.config.bookmarks.remove(&name);

            if let Err(e) = config::save_config(&self.config) {
                log::error!("Failed to save config after removing bookmark: {}", e);
            }

            if self.bookmarks_cursor >= self.bookmarks.len() {
                self.bookmarks_cursor = self.bookmarks.len().saturating_sub(1);
            }
        }
    }

    pub fn delete_selection(&mut self) {
        let active_tab = self.get_active_tab();
        let paths_to_delete: Vec<PathBuf> = if !active_tab.selected_entries.is_empty() {
            active_tab.selected_entries.iter().cloned().collect()
        } else if let Some(path) = active_tab.get_selected_entry_path() {
            vec![path]
        } else {
            Vec::new()
        };

        if !paths_to_delete.is_empty() {
            self.path_to_delete = Some(paths_to_delete[0].clone()); // A bit of a hack for now
            self.confirmation_message = if paths_to_delete.len() > 1 {
                format!("Are you sure you want to delete {} items? (y/n)", paths_to_delete.len())
            } else {
                format!("Are you sure you want to delete {:?}? (y/n)", paths_to_delete[0].file_name().unwrap())
            };
            self.show_confirmation = true;
            self.action_to_confirm = Some(ActionToConfirm::Delete);
        }
    }

    fn confirm_delete(&mut self) {
        // Сначала собираем пути для удаления
        let paths_to_delete: Vec<PathBuf> = {
            let active_tab = self.get_active_tab();
            if !active_tab.selected_entries.is_empty() {
                active_tab.selected_entries.iter().cloned().collect()
            } else if let Some(path) = self.path_to_delete.take() {
                vec![path]
            } else {
                Vec::new()
            }
        };

        // Затем добавляем задачи на удаление
        for path in paths_to_delete {
            let description = format!("Delete {:?}", path.file_name().unwrap());
            let task_kind = TaskKind::Delete { path };
            self.task_manager.add_task(task_kind, description);
        }
        
        // И наконец очищаем выделение
        self.get_active_tab_mut().selected_entries.clear();
    }

    pub fn unmount_selection(&mut self) {
        #[cfg(feature = "mounts")]
        if self.focus == FocusBlock::Disks {
            if let Some(mount_info) = self.mounts.get(self.disks_cursor) {
                self.path_to_delete = Some(mount_info.dest.clone()); // Re-use path_to_delete for unmount path
                self.confirmation_message = format!("Are you sure you want to unmount {:?}? (y/n)", mount_info.dest);
                self.show_confirmation = true;
                self.action_to_confirm = Some(ActionToConfirm::Unmount);
            }
        }
    }

    fn confirm_unmount(&mut self) {
        if let Some(path) = self.path_to_delete.take() {
            let description = format!("Unmount {:?}", path);
            let task_kind = TaskKind::Unmount { path };
            self.task_manager.add_task(task_kind, description);
        }
    }

    pub fn confirm(&mut self) {
        if let Some(action) = self.action_to_confirm.take() {
            match action {
                ActionToConfirm::Delete => self.confirm_delete(),
                ActionToConfirm::Paste => self.confirm_paste(),
                ActionToConfirm::Unmount => self.confirm_unmount(),
                ActionToConfirm::RemoveBookmark => self.confirm_remove_bookmark(),
            }
        }
        self.show_confirmation = false;
    }

    pub fn cancel(&mut self) {
        self.show_confirmation = false;
        self.action_to_confirm = None;
        self.path_to_delete = None;
        self.pending_paste = None;
    }

    pub fn create_item(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let new_item_name = self.input_buffer.clone();
        self.input_buffer.clear();

        let current_dir = self.get_active_tab().current_dir.clone();
        let new_item_path = current_dir.join(&new_item_name);

        if new_item_path.exists() {
            self.input_dialog_error = Some("A file with this name already exists.".to_string());
            self.show_input_dialog = true;
            self.input_buffer = new_item_name;
            return;
        }

        let task_kind = match self.create_file_type.as_ref().unwrap() {
            CreateFileType::File => TaskKind::CreateFile {
                path: new_item_path.clone(),
            },
            CreateFileType::Directory => TaskKind::CreateDirectory {
                path: new_item_path.clone(),
            },
        };

        let description = format!("Create {:?}", new_item_path);
        self.task_manager.add_task(task_kind, description);

        self.create_file_type = None;
    }

    pub fn rename_selection(&mut self) {
        if let Some(path) = self.get_active_tab().get_selected_entry_path() {
            self.path_to_rename = Some(path.clone());
            self.input_buffer = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            self.input_mode = InputMode::Rename;
            self.show_input_dialog = true;
            self.input_dialog_error = None;
        }
    }

    pub fn rename_item(&mut self) {
        if let Some(path_to_rename) = self.path_to_rename.clone() {
            let new_name = self.input_buffer.clone();
            if new_name.is_empty() {
                self.input_mode = InputMode::Normal;
                self.show_input_dialog = false;
                self.input_buffer.clear();
                self.path_to_rename = None;
                return;
            }
            let new_path = path_to_rename.with_file_name(&new_name);
            if new_path.exists() {
                self.input_dialog_error = Some("A file with this name already exists.".to_string());
                self.show_input_dialog = true;
                return;
            }

            self.path_to_rename = None;
            self.input_buffer.clear();

            let description = format!("Rename {:?} to {:?}", path_to_rename, new_path);
            let task_kind = TaskKind::Move {
                src: path_to_rename,
                dest: new_path,
            };
            self.task_manager.add_task(task_kind, description);
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn chmod_selection(&mut self) {
        let active_tab = self.get_active_tab();
        let selected_count = active_tab.selected_entries.len();

        let prompt = if selected_count > 1 {
            format!("What permissions do you want to set for {} selected files?", selected_count)
        } else if let Some(path) = active_tab.get_selected_entry_path() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            format!("Enter new permissions for \"{}\" (octal):", filename)
        } else {
            return; // No item selected
        };

        self.input_dialog_prompt = prompt;
        self.input_mode = InputMode::Chmod;
        self.show_input_dialog = true;
        self.input_buffer.clear();
        self.input_dialog_error = None;
    }

    pub fn chmod_item(&mut self) {
        if let Ok(mode) = u32::from_str_radix(&self.input_buffer, 8) {
            let active_tab = self.get_active_tab();
            let paths_to_chmod: Vec<PathBuf> = if !active_tab.selected_entries.is_empty() {
                active_tab.selected_entries.iter().cloned().collect()
            } else if let Some(path) = active_tab.get_selected_entry_path() {
                vec![path]
            } else {
                Vec::new()
            };

            for path in paths_to_chmod {
                let description = format!("Chmod {:?} to {:o}", path.file_name().unwrap(), mode);
                let task_kind = TaskKind::Chmod { path, mode };
                self.task_manager.add_task(task_kind, description);
            }
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn chown_item(&mut self) {
        if let Some(path) = self.get_active_tab().get_selected_entry_path() {
            let owner = self.input_buffer.clone();
            let description = format!("Chown {:?} to {}", path.file_name().unwrap(), owner);
            let task_kind = TaskKind::Chown { path, owner };
            self.task_manager.add_task(task_kind, description);
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn archive_selection(&mut self) {
        let active_tab = self.get_active_tab();
        let selected_count = active_tab.selected_entries.len();

        let prompt = if selected_count > 1 {
            format!("Archive {} selected files. Enter archive name:", selected_count)
        } else if let Some(path) = active_tab.get_selected_entry_path() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            format!("Archive \"{}\". Enter archive name:", filename)
        } else {
            return; // No item selected
        };

        self.input_dialog_prompt = prompt;
        self.input_mode = InputMode::Archive;
        self.show_input_dialog = true;
        self.input_buffer.clear();
        self.input_dialog_error = None;
    }
    
    pub fn select_archive_format(&mut self) {
        // For now, we'll just cycle through formats
        self.archive_format = match self.archive_format.as_str() {
            "zip" => "tar".to_string(),
            "tar" => "tar.gz".to_string(),
            "tar.gz" => "zip".to_string(),
            _ => "zip".to_string(),
        };
    }
    
    pub fn archive_item(&mut self) {
        if self.input_buffer.is_empty() {
            self.input_dialog_error = Some("Archive name cannot be empty".to_string());
            self.show_input_dialog = true;
            return;
        }

        let archive_name = self.input_buffer.clone();
        let format = self.archive_format.clone();
        
        let active_tab = self.get_active_tab();
        let paths_to_archive: Vec<PathBuf> = if !active_tab.selected_entries.is_empty() {
            active_tab.selected_entries.iter().cloned().collect()
        } else if let Some(path) = active_tab.get_selected_entry_path() {
            vec![path]
        } else {
            Vec::new()
        };

        if !paths_to_archive.is_empty() {
            let current_dir = active_tab.current_dir.clone();
            let extension = match format.as_str() {
                "zip" => ".zip",
                "tar" => ".tar",
                "tar.gz" => ".tar.gz",
                _ => ".zip", // Default to zip
            };
            
            let archive_path = current_dir.join(format!("{}{}", archive_name, extension));
            
            let description = format!("Archive {} items to {:?}", paths_to_archive.len(), archive_path);
            let task_kind = TaskKind::Archive { 
                paths: paths_to_archive, 
                dest: archive_path, 
                format 
            };
            self.task_manager.add_task(task_kind, description);
        }
        
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn show_info_panel(&mut self) {
        self.show_info_panel = true;
        if let Some(path) = self.get_active_tab().get_selected_entry_path() {
            if let Ok(metadata) = fs::metadata(&path) {
                let uid = metadata.uid();
                let gid = metadata.gid();
                let owner = users::get_user_by_uid(uid)
                    .map(|u| u.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| uid.to_string());
                let _group = users::get_group_by_gid(gid)
                    .map(|g| g.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| gid.to_string());

                let perms = metadata.permissions();
                let mode = perms.mode();
                let permissions = format!(
                    "{}{} {}{} {}{}{}{}{}",
                    if mode & 0o400 != 0 { "r" } else { "-" },
                    if mode & 0o200 != 0 { "w" } else { "-" },
                    if mode & 0o100 != 0 { "x" } else { "-" },
                    if mode & 0o040 != 0 { "r" } else { "-" },
                    if mode & 0o020 != 0 { "w" } else { "-" },
                    if mode & 0o010 != 0 { "x" } else { "-" },
                    if mode & 0o004 != 0 { "r" } else { "-" },
                    if mode & 0o002 != 0 { "w" } else { "-" },
                    if mode & 0o001 != 0 { "x" } else { "-" }
                );

                let size = if metadata.is_dir() {
                    get_directory_size(&path)
                } else {
                    metadata.len()
                };

                self.file_info = Some(FileInfo {
                    path,
                    size,
                    owner,
                    permissions,
                    created: metadata.created().ok(),
                    modified: metadata.modified().ok(),
                });
            }
        }
    }
    
    // Search functionality methods
    pub fn start_search(&mut self) {
        self.show_search_dialog = true;
        self.search_query = String::new();
        self.input_mode = InputMode::Normal;
    }
    
    pub fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            return;
        }
        
        // Clone the search query to avoid borrowing issues
        let search_query = self.search_query.clone();
        
        let active_tab = self.get_active_tab();
        self.search_results = SearchEngine::search_entries(
            &active_tab.entries, 
            &search_query, 
            &self.search_mode
        );
            
        self.search_cursor = 0;
        
        // Update filtered entries in the active tab
        let active_tab = self.get_active_tab_mut();
        active_tab.update_filtered_entries(&search_query);
    }
    
    pub fn move_search_cursor_down(&mut self) {
        let max = self.search_results.len().saturating_sub(1);
        if self.search_cursor < max {
            self.search_cursor += 1;
        }
    }
    
    pub fn move_search_cursor_up(&mut self) {
        if self.search_cursor > 0 {
            self.search_cursor -= 1;
        }
    }
    
    pub fn select_search_result(&mut self) {
        // Clone the selected entry path to avoid borrowing issues
        let selected_path = if let Some(selected_entry) = self.search_results.get(self.search_cursor) {
            Some(selected_entry.path.clone())
        } else {
            None
        };
        
        if let Some(selected_path) = selected_path {
            // Find the corresponding entry in the active tab's filtered entries
            let active_tab = self.get_active_tab_mut();
            if let Some(index) = active_tab.filtered_entries.iter().position(|e| e.path == selected_path) {
                active_tab.cursor = index;
                active_tab.update_preview();
            }
        }
    }
    
    pub fn cancel_search(&mut self) {
        self.show_search_dialog = false;
        self.search_query = String::new();
        self.search_results.clear();
        self.search_cursor = 0;
    }
    
    pub fn get_current_color_scheme(&self) -> crate::settings::ColorScheme {
        // Получаем имя текущей схемы из конфигурации
        if let Some(scheme_name) = &self.config.theme.color_scheme {
            // Пытаемся создать ColorScheme из имени
            if let Some(scheme) = crate::settings::ColorScheme::from_name(scheme_name) {
                return scheme;
            }
        }
        // Если не удалось получить схему из конфигурации, используем схему по умолчанию
        crate::settings::ColorScheme::Dracula
    }
    
    pub fn set_color_scheme(&mut self, color_scheme: crate::settings::ColorScheme) {
        // Обновляем тему в конфигурации
        self.config.theme.color_scheme = Some(color_scheme.name().to_string());
        println!("Color scheme updated to: {}", color_scheme.name());
    }
}