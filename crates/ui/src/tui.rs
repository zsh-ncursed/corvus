use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{self, stdout, Stdout};
use corvus_core::app_state::{AppState, InputMode, CreateFileType, RightPaneView};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn exit(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

/// Handles key presses and returns `false` if the app should quit.
pub fn handle_key_press(key: KeyEvent, app_state: &mut AppState) -> bool {
    let active_tab_view = app_state.get_active_tab().right_pane_view.clone();

    if active_tab_view == RightPaneView::Terminal {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
            app_state.get_active_tab_mut().right_pane_view = RightPaneView::Preview;
            app_state.focus = FocusBlock::Middle;
            return true;
        }

        if let Some(terminal) = &mut app_state.terminal {
            let mut bytes = Vec::new();
            match key.code {
                KeyCode::Char(c) => {
                    bytes.extend_from_slice(c.to_string().as_bytes());
                }
                KeyCode::Enter => {
                    bytes.push(b'\r');
                }
                KeyCode::Backspace => {
                    bytes.push(8); // Backspace character
                }
                KeyCode::Left => {
                    bytes.extend_from_slice(b"\x1b[D");
                }
                KeyCode::Right => {
                    bytes.extend_from_slice(b"\x1b[C");
                }
                KeyCode::Up => {
                    bytes.extend_from_slice(b"\x1b[A");
                }
                KeyCode::Down => {
                    bytes.extend_from_slice(b"\x1b[B");
                }
                KeyCode::Tab => {
                    bytes.push(b'\t');
                }
                _ => {}
            }
            if !bytes.is_empty() {
                use std::io::Write;
                if let Ok(mut writer) = terminal.pty_writer.as_mut().take_writer() {
                    if let Err(e) = writer.write_all(&bytes) {
                        log::error!("Failed to write to pty: {}", e);
                    }
                }
            }
        }
        return true;
    }

    // Global keybindings
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('n') => {
                log::info!("Ctrl+N key press detected");
                app_state.new_tab();
                return true;
            }
            KeyCode::Char('w') => {
                app_state.close_tab();
                return true;
            }
            KeyCode::Tab => {
                app_state.next_tab();
                return true;
            }
            KeyCode::Char('t') => {
                let active_tab = app_state.get_active_tab_mut();
                if active_tab.right_pane_view == RightPaneView::Preview {
                    active_tab.right_pane_view = RightPaneView::Terminal;
                    app_state.focus = FocusBlock::Terminal;
                } else {
                    active_tab.right_pane_view = RightPaneView::Preview;
                    app_state.focus = FocusBlock::Middle;
                }
                return true;
            }
            KeyCode::Char('j') => {
                let active_tab = app_state.get_active_tab_mut();
                active_tab.preview_scroll.0 = active_tab.preview_scroll.0.saturating_add(1);
                return true;
            }
            KeyCode::Char('k') => {
                let active_tab = app_state.get_active_tab_mut();
                active_tab.preview_scroll.0 = active_tab.preview_scroll.0.saturating_sub(1);
                return true;
            }
            _ => {}
        }
    }

    // Ctrl-Shift-Tab for previous tab
    if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) && key.code == KeyCode::Tab {
         app_state.previous_tab();
         return true;
    }

    // Alt-number for tab switching
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::Char(c @ '1'..='9') => {
                let tab_index = c.to_digit(10).unwrap_or(0) as usize;
                if tab_index > 0 && tab_index <= app_state.tabs.len() {
                    app_state.active_tab_index = tab_index - 1;
                }
                return true;
            }
            KeyCode::Char('t') => {
                app_state.toggle_tabs();
                return true;
            }
            _ => {}
        }
    }
    // crossterm might send BackTab for Shift-Tab
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::BackTab {
        app_state.previous_tab();
        return true;
    }


    if app_state.show_confirmation {
        match key.code {
            KeyCode::Char('y') => {
                app_state.confirm();
                return true;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app_state.cancel();
                return true;
            }
            _ => {}
        }
    }

    if app_state.show_input_dialog {
        match key.code {
            KeyCode::Char(c) => {
                app_state.input_buffer.push(c);
                return true;
            }
            KeyCode::Backspace => {
                app_state.input_buffer.pop();
                return true;
            }
            KeyCode::Tab => {
                // Handle tab key for format selection in archive mode
                if app_state.input_mode == InputMode::Archive {
                    app_state.select_archive_format();
                }
                return true;
            }
            KeyCode::Enter => {
                match app_state.input_mode {
                    InputMode::Rename => app_state.rename_item(),
                    InputMode::Chmod => app_state.chmod_item(),
                    InputMode::Chown => app_state.chown_item(),
                    InputMode::Archive => app_state.archive_item(),
                    _ => app_state.create_item(),
                }
                app_state.show_input_dialog = false;
                return true;
            }
            KeyCode::Esc => {
                app_state.show_input_dialog = false;
                app_state.input_buffer.clear();
                app_state.input_mode = InputMode::Normal;
                return true;
            }
            _ => {}
        }
    }
    
    // Handle search dialog
    if app_state.show_search_dialog {
        match key.code {
            KeyCode::Char(c) => {
                app_state.search_query.push(c);
                app_state.update_search();
                return true;
            }
            KeyCode::Backspace => {
                app_state.search_query.pop();
                app_state.update_search();
                return true;
            }
            KeyCode::Enter => {
                if !app_state.search_results.is_empty() {
                    app_state.select_search_result();
                    app_state.cancel_search();
                }
                return true;
            }
            KeyCode::Esc => {
                app_state.cancel_search();
                return true;
            }
            KeyCode::Down => {
                app_state.move_search_cursor_down();
                return true;
            }
            KeyCode::Up => {
                app_state.move_search_cursor_up();
                return true;
            }
            _ => {}
        }
    }

    // Normal mode keybindings
    use corvus_core::app_state::FocusBlock;
    match app_state.input_mode {
        InputMode::Normal => {
            match key.code {
                KeyCode::Char('c') => {
                    app_state.input_mode = InputMode::Command;
                    return true;
                }
                KeyCode::Char('n') => {
                    app_state.input_mode = InputMode::Create;
                    return true;
                }
                KeyCode::Char('q') => return false, // Signal to quit
                KeyCode::Tab => app_state.cycle_focus(),
                KeyCode::Char('.') => app_state.toggle_hidden_files(),
                KeyCode::Char('j') => {
                    match app_state.focus {
                        FocusBlock::Middle => {
                            let show_hidden = app_state.show_hidden_files;
                            app_state.get_active_tab_mut().move_cursor_down(show_hidden);
                            app_state.show_info_panel = false;
                        },
                        _ => { // Covers Xdg, Bookmarks, Disks
                            app_state.move_left_pane_cursor_down();
                        }
                    }
                },
                KeyCode::Char('J') => {
                    if app_state.focus == FocusBlock::Middle {
                        let show_hidden = app_state.show_hidden_files;
                        // Перемещаем курсор вниз
                        app_state.get_active_tab_mut().move_cursor_down(show_hidden);
                        // И выделяем новый элемент
                        app_state.get_active_tab_mut().select_current();
                    }
                },
                KeyCode::Char('k') => {
                    match app_state.focus {
                        FocusBlock::Middle => {
                            let show_hidden = app_state.show_hidden_files;
                            app_state.get_active_tab_mut().move_cursor_up(show_hidden);
                            app_state.show_info_panel = false;
                        },
                        _ => { // Covers Xdg, Bookmarks, Disks
                            app_state.move_left_pane_cursor_up();
                        }
                    }
                },
                KeyCode::Char('K') => {
                    if app_state.focus == FocusBlock::Middle {
                        let show_hidden = app_state.show_hidden_files;
                        // Перемещаем курсор вверх
                        app_state.get_active_tab_mut().move_cursor_up(show_hidden);
                        // И выделяем новый элемент
                        app_state.get_active_tab_mut().select_current();
                    }
                },
                KeyCode::Char('h') | KeyCode::Left => {
                    if app_state.focus == FocusBlock::Middle {
                        let show_hidden = app_state.show_hidden_files;
                        app_state.get_active_tab_mut().leave_directory(show_hidden);
                        app_state.show_info_panel = false;
                    }
                },
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    match app_state.focus {
                        FocusBlock::Middle => {
                            let show_hidden = app_state.show_hidden_files;
                            app_state.get_active_tab_mut().enter_directory(show_hidden);
                            app_state.show_info_panel = false;
                        },
                        _ => { // Covers Xdg, Bookmarks, Disks
                            app_state.update_middle_pane_from_left_pane_selection();
                            app_state.focus = FocusBlock::Middle;
                        }
                    }
                },
                KeyCode::Down => {
                    match app_state.focus {
                        FocusBlock::Middle => {
                            let show_hidden = app_state.show_hidden_files;
                            app_state.get_active_tab_mut().move_cursor_down(show_hidden);
                            app_state.show_info_panel = false;
                        },
                        _ => { // Covers Xdg, Bookmarks, Disks
                            app_state.move_left_pane_cursor_down();
                        }
                    }
                },
                KeyCode::Up => {
                    match app_state.focus {
                        FocusBlock::Middle => {
                            let show_hidden = app_state.show_hidden_files;
                            app_state.get_active_tab_mut().move_cursor_up(show_hidden);
                            app_state.show_info_panel = false;
                        },
                        _ => { // Covers Xdg, Bookmarks, Disks
                            app_state.move_left_pane_cursor_up();
                        }
                    }
                },
                KeyCode::Char(' ') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) && app_state.focus == FocusBlock::Middle {
                        // Получаем путь к текущему элементу
                        let current_path = {
                            let active_tab = app_state.get_active_tab();
                            active_tab.filtered_entries.get(active_tab.cursor).map(|entry| entry.path.clone())
                        };
                        
                        // Убираем выделение с текущего элемента
                        if let Some(path) = current_path {
                            let active_tab = app_state.get_active_tab_mut();
                            active_tab.selected_entries.remove(&path);
                        }
                    }
                },
                KeyCode::Char('y') => app_state.yank_selection(),
                KeyCode::Char('x') => app_state.cut_selection(),
                KeyCode::Char('d') => app_state.delete_selection(),
                KeyCode::Char('p') => app_state.paste(),
                KeyCode::Char('m') => app_state.add_bookmark(),
                KeyCode::Char('r') => app_state.rename_selection(),
                KeyCode::Char('u') => {
                    match app_state.focus {
                        FocusBlock::Disks => app_state.unmount_selection(),
                        FocusBlock::Bookmarks => app_state.remove_bookmark(),
                        _ => {} // Do nothing in other panes
                    }
                },
                KeyCode::Char('i') => app_state.show_info_panel(),
                KeyCode::Char('/') => app_state.start_search(), // Add search activation
                KeyCode::F(2) => {
                    app_state.input_mode = InputMode::Settings;
                    return true;
                },
                KeyCode::Esc => {
                    // Отмена выделения
                    app_state.get_active_tab_mut().selected_entries.clear();
                },
                _ => {}
            }
        },
        InputMode::Create => match key.code {
            KeyCode::Char('f') => {
                app_state.create_file_type = Some(CreateFileType::File);
                app_state.show_input_dialog = true;
                app_state.input_mode = InputMode::Normal;
                app_state.input_dialog_error = None;
                return true;
            }
            KeyCode::Char('d') => {
                app_state.create_file_type = Some(CreateFileType::Directory);
                app_state.show_input_dialog = true;
                app_state.input_mode = InputMode::Normal;
                app_state.input_dialog_error = None;
                return true;
            }
            _ => {
                app_state.input_mode = InputMode::Normal;
                return true;
            }
        },
        InputMode::Rename => {
            // Handled by the `show_input_dialog` block
        }
        InputMode::Command => {
            match key.code {
                KeyCode::Char('m') => {
                    let active_tab = app_state.get_active_tab();
                    if active_tab.selected_entries.is_empty() {
                        // Single item chmod: pre-fill with current permissions
                        if let Some(path) = active_tab.get_selected_entry_path() {
                            if let Ok(metadata) = std::fs::metadata(&path) {
                                let perms = metadata.permissions();
                                let mode = std::os::unix::fs::PermissionsExt::mode(&perms);
                                let permissions_str = format!("{:o}", mode & 0o777);
                                app_state.chmod_permissions = permissions_str.clone();
                                app_state.input_buffer = permissions_str;
                            }
                        }
                    } else {
                        // Bulk chmod: start with an empty input buffer
                        app_state.input_buffer.clear();
                    }
                    app_state.input_mode = InputMode::Chmod;
                    app_state.show_input_dialog = true;
                    return true;
                }
                KeyCode::Char('a') => {
                    app_state.archive_selection();
                    return true;
                }
                KeyCode::Char('o') => {
                    if let Some(path) = app_state.get_active_tab().get_selected_entry_path() {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let uid = std::os::unix::fs::MetadataExt::uid(&metadata);
                            let owner = users::get_user_by_uid(uid)
                                .map(|u| u.name().to_string_lossy().into_owned())
                                .unwrap_or_else(|| uid.to_string());
                            app_state.chown_owner = owner.clone();
                            app_state.input_buffer = owner;
                            app_state.input_mode = InputMode::Chown;
                            app_state.show_input_dialog = true;
                        }
                    }
                    return true;
                }
                _ => {}
            }
            app_state.input_mode = InputMode::Normal;
        }
        InputMode::Chmod | InputMode::Chown | InputMode::Archive => {
            // Handled by the `show_input_dialog` block
        }
        InputMode::Settings => {
            // Обработка клавиш в режиме настроек
            crate::settings::handle_settings_key(key.code, app_state);
            return true;
        }
    }
    true
}
