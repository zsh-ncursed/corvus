use ratatui::{
    prelude::{Rect, Frame, Style, Span, Constraint, Direction, Layout, Modifier},
    widgets::{block::Title, Block, Borders, List, ListItem, ListState},
};
use corvus_core::app_state::{AppState, FocusBlock};
use corvus_core::settings::ColorScheme;

pub fn render_xdg_block(frame: &mut Frame, area: Rect, app_state: &AppState, color_scheme: &ColorScheme) {
    let items: Vec<ListItem> = app_state
        .xdg_dirs
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone())
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Xdg;
    let title_style = if is_focused { 
        Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background())
    } else { 
        Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background())
    };
    let highlight_style = if is_focused { 
        Style::default()
            .bg(color_scheme.highlight_bg())
            .fg(color_scheme.text_color())
    } else { 
        Style::default()
            .bg(color_scheme.background())
            .fg(color_scheme.text_color())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Title::from(Span::styled("XDG Dirs", title_style)))
                .borders(Borders::BOTTOM)
                .style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background()))
        )
        .highlight_style(highlight_style);

    let mut list_state = ListState::default();
    list_state.select(Some(app_state.xdg_cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}

pub fn render_bookmarks_block(frame: &mut Frame, area: Rect, app_state: &AppState, color_scheme: &ColorScheme) {
    let items: Vec<ListItem> = app_state
        .bookmarks
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone())
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Bookmarks;
    let title_style = if is_focused { 
        Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background())
    } else { 
        Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background())
    };
    let highlight_style = if is_focused { 
        Style::default()
            .bg(color_scheme.highlight_bg())
            .fg(color_scheme.text_color())
    } else { 
        Style::default()
            .bg(color_scheme.background())
            .fg(color_scheme.text_color())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Title::from(Span::styled("Bookmarks", title_style)))
                .borders(Borders::BOTTOM)
                .style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background()))
        )
        .highlight_style(highlight_style);

    let mut list_state = ListState::default();
    list_state.select(Some(app_state.bookmarks_cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}

#[cfg(feature = "mounts")]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, app_state: &AppState, color_scheme: &ColorScheme) {
    let mount_items: Vec<ListItem> = app_state.mounts.iter().map(|mount| {
        // Extract the last component of the path for display
        let display_name = mount.dest.file_name().unwrap_or_default().to_string_lossy();
        ListItem::new(format!("💾 {}", display_name))
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background()))
    }).collect();

    let is_focused = app_state.focus == FocusBlock::Disks;
    let title_style = if is_focused {
        Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background())
    } else {
        Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background())
    };
    let highlight_style = if is_focused {
        Style::default()
            .bg(color_scheme.highlight_bg())
            .fg(color_scheme.text_color())
    } else {
        Style::default()
            .bg(color_scheme.background())
            .fg(color_scheme.text_color())
    };

    let mounts_list = List::new(mount_items)
        .block(
            Block::default()
                .title(Title::from(Span::styled("Disks", title_style)))
                .borders(Borders::BOTTOM)
                .style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background()))
        )
        .highlight_style(highlight_style);

    let mut mounts_state = ListState::default();
    mounts_state.select(Some(app_state.disks_cursor));
    frame.render_stateful_widget(mounts_list, area, &mut mounts_state);
}

#[cfg(not(feature = "mounts"))]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, _app_state: &AppState, color_scheme: &ColorScheme) {
    let block = Block::new()
        .borders(Borders::ALL)
        .title("Mounts (unsupported)")
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));
    frame.render_widget(block, area);
}

/// Render the entire left pane with all its blocks
pub fn render_left_pane(frame: &mut Frame, area: Rect, app_state: &AppState, color_scheme: &ColorScheme) {
    // Split the area into three parts for XDG, Bookmarks, and Mounts
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33), // XDG
            Constraint::Percentage(33), // Bookmarks
            Constraint::Percentage(34), // Mounts
        ])
        .split(area);
    
    render_xdg_block(frame, chunks[0], app_state, color_scheme);
    render_bookmarks_block(frame, chunks[1], app_state, color_scheme);
    render_mounts_block(frame, chunks[2], app_state, color_scheme);
}