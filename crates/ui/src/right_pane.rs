use ratatui::{
    prelude::{Line, Rect, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use corvus_core::app_state::{PreviewContent, TabState};
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

pub fn render_right_pane(frame: &mut Frame, area: Rect, tab_state: &TabState, color_scheme: &ColorScheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Preview")
        .style(Style::default()
            .fg(color_scheme.text_color())
            .bg(color_scheme.background()));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    match &tab_state.preview_content {
        Some(content) => match content {
            PreviewContent::Directory(entries) => {
                let items: Vec<ListItem> = entries
                    .iter()
                    .map(|entry| {
                        let icon = get_icon_for_file(&entry.name, entry.is_dir);
                        let color = to_ratatui_color(get_color_for_file(&entry.name, entry.is_dir));
                        let icon_span = Span::styled(icon, Style::default().fg(color));

                        let mut name = entry.name.clone();
                        if entry.is_dir {
                            name.push('/');
                        }
                        let name_span = Span::raw(format!(" {}", name));

                        let line = Line::from(vec![icon_span, name_span]);
                        ListItem::new(line)
                            .style(Style::default()
                                .fg(color_scheme.text_color())
                                .bg(color_scheme.background()))
                    })
                    .collect();

                let list = List::new(items).highlight_style(
                    Style::default()
                        .bg(color_scheme.highlight_bg())
                        .fg(color_scheme.text_color())
                );
                frame.render_widget(list, inner_area);
            }
            PreviewContent::File(text) => {
                let paragraph = Paragraph::new(text.as_str())
                    .style(Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background()))
                    .scroll(tab_state.preview_scroll);
                frame.render_widget(paragraph, inner_area);
            }
            PreviewContent::TooLarge(size) => {
                let text = format!("File is too large to preview ({} > 1MB)", size);
                let paragraph = Paragraph::new(text)
                    .style(Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background()));
                frame.render_widget(paragraph, inner_area);
            }
            PreviewContent::Error(error) => {
                let paragraph = Paragraph::new(error.as_str())
                    .style(Style::default()
                        .fg(ratatui::prelude::Color::Red)
                        .bg(color_scheme.background()));
                frame.render_widget(paragraph, inner_area);
            }
            PreviewContent::Binary => {
                let paragraph = Paragraph::new("[Binary File]")
                    .style(Style::default()
                        .fg(color_scheme.text_color())
                        .bg(color_scheme.background()));
                frame.render_widget(paragraph, inner_area);
            }
        },
        None => {
            let paragraph = Paragraph::new("No item selected")
                .style(Style::default()
                    .fg(color_scheme.text_color())
                    .bg(color_scheme.background()));
            frame.render_widget(paragraph, inner_area);
        }
    }
}