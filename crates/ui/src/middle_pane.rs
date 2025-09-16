use ratatui::{
    prelude::{Rect, Style, Line},
    text::Span,
    widgets::{List, ListItem, ListState},
    Frame,
};
use corvus_core::app_state::TabState;
use corvus_core::settings::ColorScheme;
use utils::icons::{get_color_for_file, get_icon_for_file, IconColor};

fn to_ratatui_color(icon_color: IconColor) -> ratatui::prelude::Color {
    match icon_color {
        IconColor::Blue => ratatui::prelude::Color::Blue,
        IconColor::Rgb(r, g, b) => ratatui::prelude::Color::Rgb(r, g, b),
        IconColor::Magenta => ratatui::prelude::Color::Magenta,
        IconColor::Yellow => ratatui::prelude::Color::Yellow,
        IconColor::Cyan => ratatui::prelude::Color::Cyan,
        IconColor::Red => ratatui::prelude::Color::Red,
        IconColor::White => ratatui::prelude::Color::White,
        IconColor::Gray => ratatui::prelude::Color::Gray,
    }
}

pub fn render_middle_pane(frame: &mut Frame, area: Rect, tab_state: &TabState, color_scheme: &ColorScheme) {
    let items: Vec<ListItem> = tab_state
        .filtered_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = tab_state.selected_entries.contains(&entry.path);
            let is_focused = i == tab_state.cursor;

            let mut style = if entry.name.starts_with('.') {
                Style::default().fg(ratatui::prelude::Color::DarkGray)
            } else {
                Style::default().fg(color_scheme.text_color())
            };

            // Apply background color based on state
            if !is_focused && is_selected {
                style = style.bg(color_scheme.selection_bg());
            } else {
                style = style.bg(color_scheme.background());
            }

            let icon = get_icon_for_file(&entry.name, entry.is_dir);
            let color = to_ratatui_color(get_color_for_file(&entry.name, entry.is_dir));
            let icon_span = Span::styled(icon, Style::default().fg(color));

            let mut name = entry.name.clone();
            if entry.is_dir {
                name.push('/');
            }
            
            let selection_indicator = if is_selected { "* " } else { "  " };
            let name_span = Span::raw(format!("{}{}", selection_indicator, name));

            let line = Line::from(vec![icon_span, name_span]);
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(color_scheme.highlight_bg())
            .fg(color_scheme.text_color())
    );

    let mut list_state = ListState::default();
    list_state.select(Some(tab_state.cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}