use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct InstallingView<'a> {
    pub progress: f64,
    pub current_service: &'a str,
    pub completed_services: usize,
    pub total_services: usize,
    pub logs: &'a [String],
}

pub fn render_installing(frame: &mut Frame, view: &InstallingView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new("🔄 Installing Analytics... Please wait")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .centered();
    frame.render_widget(title, chunks[0]);

    let progress_width = (chunks[1].width as f64 - 10.0).max(0.0) * (view.progress / 100.0);
    let filled = "█".repeat(progress_width as usize);
    let empty = "░".repeat((chunks[1].width as usize - 10 - progress_width as usize).max(0));

    let progress_text = format!("[{}{}] {:.0}%", filled, empty, view.progress);
    let progress_widget = Paragraph::new(progress_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .centered();
    frame.render_widget(progress_widget, chunks[1]);

    let current = if !view.current_service.is_empty() {
        format!(
            "Current: {} ({}/{})",
            view.current_service, view.completed_services, view.total_services
        )
    } else {
        "Initializing...".to_string()
    };

    let current_widget = Paragraph::new(current)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .centered();
    frame.render_widget(current_widget, chunks[2]);

    let log_lines: Vec<Line> = view
        .logs
        .iter()
        .map(|log| {
            let style = if log.contains("❌") || log.to_lowercase().contains("error") {
                Style::default().fg(Color::Red)
            } else if log.contains("✅") || log.contains("started") {
                Style::default().fg(Color::Green)
            } else if log.contains("⬇️") {
                Style::default().fg(Color::Blue)
            } else if log.contains("🔨") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(Span::styled(log.clone(), style))
        })
        .collect();

    let logs_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📋 Installation Logs"),
        )
        .wrap(Wrap { trim: false })
        .scroll((
            view.logs
                .len()
                .saturating_sub(chunks[3].height as usize - 2) as u16,
            0,
        ));
    frame.render_widget(logs_widget, chunks[3]);

    let help = Paragraph::new("Press Ctrl+C to cancel")
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[4]);
}
