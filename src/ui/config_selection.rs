use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::templates::ConfigTemplate;

pub struct ConfigSelectionView<'a> {
    pub templates: &'a [ConfigTemplate],
    pub selected_index: usize,
}

pub fn render_config_selection(frame: &mut Frame, view: &ConfigSelectionView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(5),
        ])
        .split(area);

    let title = Paragraph::new("🧩 Choose a configuration template")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .centered();
    frame.render_widget(title, chunks[0]);

    let highlight_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::Cyan);
    let description_style = Style::default().fg(Color::Gray);

    let mut list_lines = Vec::new();

    if view.templates.is_empty() {
        list_lines.push(Line::from(Span::styled(
            "No templates available",
            Style::default().fg(Color::Red),
        )));
    } else {
        for (index, template) in view.templates.iter().enumerate() {
            let is_selected = index == view.selected_index;
            let selector = if is_selected { "❯" } else { " " };
            let name_style = if is_selected {
                highlight_style
            } else {
                normal_style
            };

            list_lines.push(Line::from(vec![
                Span::styled(selector, name_style),
                Span::raw(" "),
                Span::styled(template.name, name_style),
                Span::raw("  "),
                Span::styled(format!("({})", template.key), description_style),
            ]));

            list_lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(template.description, description_style),
            ]));

            list_lines.push(Line::from(""));
        }
    }

    let list = Paragraph::new(list_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Model providers"),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(list, chunks[1]);

    let detail_lines = if let Some(template) = view.templates.get(
        view.selected_index
            .min(view.templates.len().saturating_sub(1)),
    ) {
        vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(Color::Yellow)),
                Span::styled(template.name, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Key: ", Style::default().fg(Color::Yellow)),
                Span::raw(template.key),
            ]),
            Line::from(""),
            Line::from("Use ↑ ↓ to navigate, Enter to generate config.yaml"),
            Line::from("Press Esc to go back, Ctrl+C to exit"),
        ]
    } else {
        vec![
            Line::from("Use ↑ ↓ to navigate, Enter to generate config.yaml"),
            Line::from("Press Esc to go back, Ctrl+C to exit"),
        ]
    };

    let details = Paragraph::new(detail_lines)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });
    frame.render_widget(details, chunks[2]);
}
