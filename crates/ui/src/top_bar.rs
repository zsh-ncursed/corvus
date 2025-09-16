use ratatui::{
    prelude::{Rect, Style},
    style::{Modifier},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use corvus_core::app_state::AppState;
use corvus_core::settings::ColorScheme;

pub fn render_top_bar(frame: &mut Frame, area: Rect, app_state: &AppState, color_scheme: &ColorScheme) {
    let titles: Vec<String> = app_state
        .tabs
        .iter()
        .map(|tab| {
            format!(
                "{} {}",
                tab.id + 1,
                tab.current_dir
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            )
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::BOTTOM)
            .style(Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.background())))
        .select(app_state.active_tab_index)
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()))
        .highlight_style(
            Style::default()
                .fg(color_scheme.text_color())
                .bg(color_scheme.highlight_bg())
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}