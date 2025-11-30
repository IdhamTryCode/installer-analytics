use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct ErrorView<'a> {
    pub logs: &'a [String],
}

pub fn render_error(frame: &mut Frame, error: &str, view: &ErrorView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new("❌ Installation Failed")
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL))
        .centered();
    frame.render_widget(title, chunks[0]);

    let message = vec![
        Line::from(""),
        Line::from(Span::styled(
            "An error occurred:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(error, Style::default().fg(Color::White))),
        Line::from(""),
    ];

    let message_widget = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error Details"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(message_widget, chunks[1]);

    let log_lines: Vec<Line> = view
        .logs
        .iter()
        .map(|log| Line::from(Span::styled(log.clone(), Style::default().fg(Color::White))))
        .collect();

    let logs_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Installation Logs"),
        )
        .wrap(Wrap { trim: false })
        .scroll((
            view.logs
                .len()
                .saturating_sub(chunks[2].height as usize - 2) as u16,
            0,
        ));
    frame.render_widget(logs_widget, chunks[2]);

    let help = Paragraph::new("Press Ctrl+C to exit")
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[3]);
}
