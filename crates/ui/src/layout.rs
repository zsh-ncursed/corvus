use crate::{left_pane, middle_pane, top_bar, right_pane, settings};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use corvus_core::app_state::{AppState, CreateFileType, InputMode};
use corvus_core::clipboard::ClipboardMode;
use corvus_core::settings::ColorScheme;
use humansize;
use chrono;

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_main_layout(frame: &mut Frame, app_state: &mut AppState) {
    // Получаем текущую цветовую схему
    let color_scheme = app_state.get_current_color_scheme();
    
    match app_state.input_mode {
        InputMode::Settings => {
            // Отображаем экран настроек
            settings::render_settings_screen(frame, frame.size(), app_state);
            return;
        }
        _ => {
            // Отображаем обычный интерфейс
            render_normal_layout(frame, app_state, &color_scheme);
        }
    }
}

fn render_normal_layout(frame: &mut Frame, app_state: &AppState, color_scheme: &ColorScheme) {
    let top_bar_height = if app_state.show_tabs { 2 } else { 0 };
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_bar_height), // Top bar
            Constraint::Min(0),    // Main content
            Constraint::Length(6), // Footer
        ])
        .split(frame.size());

    let top_bar_area = main_chunks[0];
    let main_area = main_chunks[1];
    let footer_area = main_chunks[2];

    // --- Top Bar (Tabs) ---
    top_bar::render_top_bar(frame, top_bar_area, app_state, color_scheme);

    // --- Main Area (Left, Middle, Right) ---
    let main_horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left
            Constraint::Percentage(40), // Middle
            Constraint::Percentage(40), // Right
        ])
        .split(main_area);

    let left_pane_area = main_horizontal_chunks[0];
    let middle_pane_area = main_horizontal_chunks[1];
    let right_pane_area = main_horizontal_chunks[2];

    // --- Render Panes with Borders ---
    let active_tab = app_state.get_active_tab();

    // Left Pane
    let left_pane_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));
    let left_pane_inner_area = left_pane_block.inner(left_pane_area);
    frame.render_widget(left_pane_block, left_pane_area);
    left_pane::render_left_pane(frame, left_pane_inner_area, app_state, color_scheme);

    // Middle Pane
    let middle_pane_block = Block::default()
        .title(format!("Current: {}", active_tab.current_dir.display()))
        .borders(Borders::ALL)
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));
    let middle_pane_inner_area = middle_pane_block.inner(middle_pane_area);
    frame.render_widget(middle_pane_block, middle_pane_area);
    middle_pane::render_middle_pane(frame, middle_pane_inner_area, active_tab, color_scheme);

    // Right Pane
    right_pane::render_right_pane(frame, right_pane_area, active_tab, color_scheme);

    // --- Footer (Tasks, Info) ---
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Tasks
            Constraint::Percentage(50), // Info
        ])
        .split(footer_area);

    render_tasks_footer(frame, footer_chunks[0], app_state, color_scheme);
    render_info_panel(frame, footer_chunks[1], app_state, color_scheme);

    if app_state.show_confirmation {
        render_confirmation_dialog(frame, app_state, color_scheme);
    }
    if app_state.show_input_dialog {
        render_input_dialog(frame, app_state, color_scheme);
    }
    if app_state.show_search_dialog {
        render_search_dialog(frame, app_state, color_scheme);
    }
}

fn render_input_dialog(frame: &mut Frame, app_state: &AppState, color_scheme: &ColorScheme) {
    let title = match app_state.input_mode {
        InputMode::Create => {
            let file_type = match app_state.create_file_type {
                Some(CreateFileType::File) => "file",
                Some(CreateFileType::Directory) => "directory",
                None => "",
            };
            format!("Create new {}", file_type)
        }
        InputMode::Rename => "Rename".to_string(),
        InputMode::Chmod => "Chmod (e.g. 755)".to_string(),
        InputMode::Chown => "Chown (e.g. user:group)".to_string(),
        InputMode::Archive => format!("Archive (Format: {})", app_state.archive_format),
        _ => "Input".to_string(),
    };

    let mut text = app_state.input_buffer.clone();
    if let Some(error) = &app_state.input_dialog_error {
        text.push_str(&format!("\n\n{}", error));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));

    // Center the dialog
    let area = centered_rect(50, 20, frame.size());
    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(paragraph, area);
}

fn render_search_dialog(frame: &mut Frame, app_state: &AppState, color_scheme: &ColorScheme) {
    let title = "Search (Press Esc to cancel, Enter to select)".to_string();
    
    // Create the text content
    let mut text = format!("Search query: {}\n\n", app_state.search_query);
    
    // Add search results
    if app_state.search_results.is_empty() {
        text.push_str("No results found");
    } else {
        text.push_str("Search results:\n");
        for (i, result) in app_state.search_results.iter().enumerate() {
            let prefix = if i == app_state.search_cursor { "> " } else { "  " };
            let icon = if result.is_dir { "📁" } else { "📄" };
            text.push_str(&format!("{}{} {}\n", prefix, icon, result.name));
        }
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));

    // Center the dialog
    let area = centered_rect(60, 40, frame.size());
    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(paragraph, area);
}

fn render_confirmation_dialog(frame: &mut Frame, app_state: &AppState, color_scheme: &ColorScheme) {
    let message = &app_state.confirmation_message;
    let text = Paragraph::new(message.as_str())
        .block(Block::default()
            .title("Confirmation")
            .borders(Borders::ALL)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));

    // Center the dialog
    let area = centered_rect(50, 20, frame.size());
    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(text, area);
}

fn render_tasks_footer(frame: &mut Frame, area: Rect, app_state: &AppState, color_scheme: &ColorScheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Tasks")
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let tasks = app_state.task_manager.get_tasks();
    let task_items: Vec<ListItem> = tasks
        .iter()
        .map(|task| ListItem::new(task.description.clone())
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .collect();

    let task_list = List::new(task_items);

    frame.render_widget(task_list, inner_area);
}

fn render_info_panel(frame: &mut Frame, area: Rect, app_state: &AppState, color_scheme: &ColorScheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Info")
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));
    let inner_area = block.inner(area);

    let mut info_text = String::new();

    // Always display clipboard info
    let clipboard = &app_state.clipboard;
    let clipboard_info = if !clipboard.paths.is_empty() {
        let mode = match clipboard.mode {
            Some(ClipboardMode::Copy) => "Copy",
            Some(ClipboardMode::Move) => "Move",
            None => "None",
        };
        format!("Buffer: {} files ({})", clipboard.paths.len(), mode)
    } else {
        "Buffer: Empty".to_string()
    };
    info_text.push_str(&clipboard_info);

    // Display notification if there is one
    if let Some(notification) = &app_state.notification {
        info_text.push_str("\n\n");
        info_text.push_str(notification);
    }

    if app_state.show_info_panel {
        if let Some(info) = &app_state.file_info {
            info_text.push_str("\n\n");
            info_text.push_str(&format!("Path: {}\n", info.path.display()));
            info_text.push_str(&format!("Size: {}\n", humansize::format_size(info.size, humansize::BINARY)));
            info_text.push_str(&format!("Owner: {}\n", info.owner));
            info_text.push_str(&format!("Permissions: {}\n", info.permissions));
            if let Some(created) = info.created {
                let datetime: chrono::DateTime<chrono::Local> = created.into();
                info_text.push_str(&format!("Created: {}\n", datetime.format("%Y-%m-%d %H:%M:%S")));
            }
            if let Some(modified) = info.modified {
                let datetime: chrono::DateTime<chrono::Local> = modified.into();
                info_text.push_str(&format!("Modified: {}", datetime.format("%Y-%m-%d %H:%M:%S")));
            }
        }
    }


    let paragraph = Paragraph::new(info_text)
        .wrap(Wrap { trim: true })
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));

    frame.render_widget(paragraph, inner_area);
    frame.render_widget(block, area);
}