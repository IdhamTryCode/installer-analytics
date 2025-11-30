use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct SuccessView<'a> {
    pub logs: &'a [String],
}

pub fn render_success(frame: &mut Frame, view: &SuccessView<'_>) {
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

    let title = Paragraph::new("✅ Installation Complete!")
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .centered();
    frame.render_widget(title, chunks[0]);

    let message = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Analytics has been successfully installed!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("All services are now running. You can access Analytics UI at:"),
        Line::from(Span::styled(
            "http://localhost:3000",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(""),
    ];

    let message_widget = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL).title("Success"))
        .centered();
    frame.render_widget(message_widget, chunks[1]);

    let log_lines: Vec<Line> = view
        .logs
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|log| Line::from(Span::styled(log.clone(), Style::default().fg(Color::White))))
        .collect();

    let logs_widget = Paragraph::new(log_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Installation Summary"),
    );
    frame.render_widget(logs_widget, chunks[2]);

    let help = Paragraph::new("Press Ctrl+C to exit")
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[3]);
}
